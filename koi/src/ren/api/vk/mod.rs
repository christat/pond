pub mod device;
pub mod frame;
pub mod instance;
pub mod surface;
pub mod swapchain;

use crate::{ren::{Info, Renderer as RendererTrait, Settings, Window}, traits::Drop};
use device::{Device, config::QueueFamilyType};
use frame::Frame;
use instance::Instance;
use surface::Surface;
use swapchain::{Swapchain, SurfaceSupport};

use ash::{Entry, vk};

#[allow(unused)]
pub struct Renderer {
    settings: Settings,
    window: Window,

    entry: Entry,
    instance: Instance,
    surface: Surface,
    device: Device,
    swapchain: Swapchain,
    surface_support: SurfaceSupport,

    frames: Vec<Frame>,
    frame_count: u32,
    graphics_queue: vk::Queue,
}

impl RendererTrait for Renderer {
    fn new(info: &Info, settings: Settings, window: Window) -> Self {
        let entry = unsafe { Entry::load().expect("koi::ren::vk - Failed to load Vulkan Instance") };

        let instance = Instance::new(&entry, &info);
        let surface = Surface::new(&entry, &instance.handle, &window);
        let device = Device::new(&instance.handle, &surface);
        let (swapchain, surface_support) = Swapchain::new(&instance, &device, &surface, &settings.resolution).expect("koi::ren::vk - failed to create Swapchain");

        let frames: Vec<_> = (0..settings.buffering).into_iter().map(|_index| Frame::new(&device)).collect();
        let graphics_queue = device.get_queue(QueueFamilyType::Graphics);

        Self { settings, window, entry, instance, surface, device, swapchain, surface_support, frames, frame_count: 0, graphics_queue }
    }

    fn draw(&mut self) {
        const SECOND_AS_NS: u64 = 10e9 as u64;

        let current_frame = self.get_current_frame();

        // wait until GPU is done rendering the last frame; 1s timeout
        let fences: [vk::Fence; 1] = [current_frame.render_fence];
        unsafe { self.device.handle.wait_for_fences(&fences, true, SECOND_AS_NS).expect("koi::ren::vk - failed to wait for Render Fence") };
        unsafe { self.device.handle.reset_fences(&fences).expect("koi::ren::vk - failed to reset Render Fence") };

        // request swapchain image
        let (swapchain_image_index, _suboptimal) = unsafe { self.swapchain.device.acquire_next_image(self.swapchain.khr, SECOND_AS_NS, current_frame.swapchain_semaphore, vk::Fence::null()).expect("koi::ren::vk - Failed to acquire next Swapchain Image") };
        let swapchain_image = self.swapchain.images[swapchain_image_index as usize];

        // reset/begin frame command buffer
        let cmd = current_frame.command_buffer;
        unsafe { self.device.handle.reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty()).expect("koi::ren::vk - failed to Reset current frame Command Buffer") };
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe { self.device.handle.begin_command_buffer(cmd, &command_buffer_begin_info).expect("koi::ren::vk - failed to Begin current frame Command Buffer") };

        // transition swapchain to write
        frame::transition_image(&self.device, cmd, swapchain_image, vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL);

        // fill image
        let mut clear_color_value = vk::ClearColorValue::default();
        let color_channel_wave = ((self.frame_count as f32 / 120.0).sin()).abs();
        clear_color_value.float32 = [ 0.0, 0.0, color_channel_wave, 0.0 ];
        let clear_ranges = [frame::get_image_subresource_range(vk::ImageAspectFlags::COLOR)];
        unsafe { self.device.handle.cmd_clear_color_image(cmd, swapchain_image, vk::ImageLayout::GENERAL, &clear_color_value, &clear_ranges) } ;

        // transition swapchain to present
        frame::transition_image(&self.device, cmd, swapchain_image, vk::ImageLayout::GENERAL, vk::ImageLayout::PRESENT_SRC_KHR);


        // end command buffer
        unsafe { self.device.handle.end_command_buffer(cmd).expect("koi::ren::vk - failed to End current frame Command Buffer") };

        // submit command buffer to queue
        let command_buffer_infos = [
            vk::CommandBufferSubmitInfo::default()
                .command_buffer(cmd)
        ];
        let wait_semaphore_infos = [
            vk::SemaphoreSubmitInfo::default()
                .semaphore(current_frame.swapchain_semaphore)
                .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
        ];
        let signal_semaphore_infos = [
            vk::SemaphoreSubmitInfo::default()
                .semaphore(current_frame.render_semaphore)
                .stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS)
        ];
        let submit_info = [frame::get_submit_info(&command_buffer_infos, Some(&wait_semaphore_infos), Some(&signal_semaphore_infos))];
        unsafe { self.device.handle.queue_submit2(self.graphics_queue, &submit_info, current_frame.render_fence).expect("koi::ren::vk - failed to Submit command buffer to Queue") };

        // present swapchain image
        let swapchains = [self.swapchain.khr];
        let wait_semaphores = [current_frame.render_semaphore];
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

impl Renderer {
    pub fn get_current_frame<'a>(&'a self) -> &'a Frame {
        &self.frames[(self.frame_count % self.settings.buffering) as usize]
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.frames.iter_mut().for_each(|frame| frame.drop(&self.device));
        self.swapchain.drop(&self.device);
        self.device.drop();
        self.surface.drop();
        self.instance.drop();
    }
}