pub mod allocator;
pub mod device;
pub mod frame;
pub mod image;
pub mod instance;
pub mod surface;
pub mod swapchain;

use crate::{ren::{settings::Resolution, Info, Renderer as RendererTrait, Settings, Window}, traits::Drop};
use allocator::Allocator;
use device::{Device, config::QueueFamilyType};
use frame::Frame;
use image::Image;
use instance::Instance;
use surface::Surface;
use swapchain::{Swapchain, SurfaceSupport};

use ash::{Entry, vk};

#[allow(unused)]
pub struct Renderer {
    settings: Settings,
    window: Window,

    // Vulkan structures
    entry: Entry,
    instance: Instance,
    surface: Surface,
    device: Device,
    swapchain: Swapchain,
    surface_support: SurfaceSupport,

    // Render loop structures
    frames: Vec<Frame>,
    graphics_queue: vk::Queue,
    allocator: Allocator,

    // Render loop resources
    draw_image: Image,
    frame_count: u32,
}

impl RendererTrait for Renderer {
    fn new(info: &Info, settings: Settings, window: Window) -> Self {
        let entry = unsafe { Entry::load().expect("koi::ren::vk - Failed to load Vulkan Instance") };

        let instance = Instance::new(&entry, &info);
        let surface = Surface::new(&entry, &instance.handle, &window);
        let device = Device::new(&instance.handle, &surface);
        let (swapchain, surface_support) = Swapchain::new(&instance, &device, &surface, &settings.resolution).expect("koi::ren::vk - failed to create Swapchain");

        let frames = Frame::generator(&device, settings.buffering);
        let graphics_queue = device.get_queue(QueueFamilyType::Graphics);
        let mut allocator = allocator::Allocator::new(instance.handle.clone(), device.handle.clone(), device.physical_device.clone(), settings.buffering);
        
        let Resolution { width, height } = settings.resolution;
        let draw_image = Image::new(
            &device.handle,
            &mut allocator.handle,
            &mut allocator.global_resources,
            vk::Format::R16G16B16A16_SFLOAT,
            vk::Extent3D::default()
                .width(width)
                .height(height)
                .depth(1),
            vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            vk::ImageAspectFlags::COLOR
        );

        Self { settings, window, entry, instance, surface, device, swapchain, surface_support, frames, graphics_queue, allocator, draw_image, frame_count: 0 }
    }

    fn draw(&mut self) {
        const SECOND_AS_NS: u64 = 10e9 as u64;

        // clone frame data handles
        let Frame{ command_buffer, render_fence, render_semaphore, swapchain_semaphore, .. } = self.get_current_frame();
        let command_buffer = command_buffer.clone();
        let render_fence = render_fence.clone();
        let render_semaphore = render_semaphore.clone();
        let swapchain_semaphore = swapchain_semaphore.clone();

        // wait until GPU is done rendering the last frame; 1s timeout
        let fences: [vk::Fence; 1] = [render_fence];
        unsafe { self.device.handle.wait_for_fences(&fences, true, SECOND_AS_NS).expect("koi::ren::vk - failed to wait for Render Fence") };
        unsafe { self.device.handle.reset_fences(&fences).expect("koi::ren::vk - failed to reset Render Fence") };

        // drop frame-specific resources
        let device_handle = self.device.handle.clone();
        let frame_index = self.get_current_frame_index();
        self.allocator.drop_frame(&device_handle, frame_index);

        // request swapchain image
        let (swapchain_image_index, _suboptimal) = unsafe { self.swapchain.device.acquire_next_image(self.swapchain.khr, SECOND_AS_NS, swapchain_semaphore, vk::Fence::null()).expect("koi::ren::vk - Failed to acquire next Swapchain Image") };
        let swapchain_image = self.swapchain.images[swapchain_image_index as usize];

        // reset/begin frame command buffer
        unsafe { self.device.handle.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty()).expect("koi::ren::vk - failed to Reset current frame Command Buffer") };
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe { self.device.handle.begin_command_buffer(command_buffer, &command_buffer_begin_info).expect("koi::ren::vk - failed to Begin current frame Command Buffer") };

        // transition draw image to write
        image::transition(&device_handle, command_buffer, self.draw_image.image, vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL);

        self.write_command_buffer(command_buffer);

        // transition draw image for copy src and swaphain for copy dst; perform ccopy
        image::transition(&device_handle, command_buffer, self.draw_image.image, vk::ImageLayout::GENERAL, vk::ImageLayout::TRANSFER_SRC_OPTIMAL);
        image::transition(&device_handle, command_buffer, swapchain_image, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        image::copy(&device_handle, command_buffer, self.draw_image.image, swapchain_image, self.draw_image.extent_2d, self.swapchain.extent);

        // transition swapchain to presetn
        image::transition(&device_handle, command_buffer, swapchain_image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::PRESENT_SRC_KHR);
        
        // end command buffer
        unsafe { self.device.handle.end_command_buffer(command_buffer).expect("koi::ren::vk - failed to End current frame Command Buffer") };

        // submit command buffer to queue
        let command_buffer_infos = [
            vk::CommandBufferSubmitInfo::default()
                .command_buffer(command_buffer)
        ];
        let wait_semaphore_infos = [
            vk::SemaphoreSubmitInfo::default()
                .semaphore(swapchain_semaphore)
                .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
        ];
        let signal_semaphore_infos = [
            vk::SemaphoreSubmitInfo::default()
                .semaphore(render_semaphore)
                .stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS)
        ];
        let submit_info = [frame::get_submit_info(&command_buffer_infos, Some(&wait_semaphore_infos), Some(&signal_semaphore_infos))];
        unsafe { self.device.handle.queue_submit2(self.graphics_queue, &submit_info, render_fence).expect("koi::ren::vk - failed to Submit command buffer to Queue") };

        // present swapchain image
        let swapchains = [self.swapchain.khr];
        let wait_semaphores = [render_semaphore];
        let image_indices = [swapchain_image_index];
        let present_info = vk::PresentInfoKHR::default()
            .swapchains(&swapchains)
            .wait_semaphores(&wait_semaphores)
            .image_indices(&image_indices);

        unsafe{ self.swapchain.device.queue_present(self.graphics_queue, &present_info).expect("koi::vk::ren - failed to Present swapchain image") };

        // frame done.
        self.frame_count += 1;
    }
}

impl<'a> Renderer {
    fn get_current_frame_index(&self) -> usize {
        (self.frame_count % self.settings.buffering) as usize
    }

    fn get_current_frame(&'a mut self) -> &'a mut Frame {
        let index = self.get_current_frame_index();
        &mut self.frames[index]
    }

    fn write_command_buffer(&mut self, command_buffer: vk::CommandBuffer) {
        let mut clear_color_value = vk::ClearColorValue::default();
        let color_channel_wave = ((self.frame_count as f32 / 120.0).sin()).abs();
        clear_color_value.float32 = [ 0.0, 0.0, color_channel_wave, 0.0 ];

        let clear_ranges = [image::get_subresource_range(vk::ImageAspectFlags::COLOR)];

        unsafe { self.device.handle.cmd_clear_color_image(command_buffer, self.draw_image.image, vk::ImageLayout::GENERAL, &clear_color_value, &clear_ranges) } ;
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.frames.iter_mut().for_each(|frame| frame.drop(&self.device.handle));
        self.allocator.drop(&self.device.handle);
        self.swapchain.drop(&self.device.handle);
        self.device.drop();
        self.surface.drop();
        self.instance.drop();
    }
}