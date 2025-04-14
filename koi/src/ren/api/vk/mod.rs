mod allocator;
mod buffer;
mod descriptor;
mod device;
mod frame;
mod image;
pub mod imgui;
mod instance;
mod manager;
mod mesh;
pub mod pipeline;
mod surface;
mod swapchain;
pub use allocator::*;
pub use buffer::*;
pub use descriptor::*;
pub use device::*;
pub use frame::*;
pub use image::*;
pub use imgui::*;
pub use instance::*;
pub use manager::*;
pub use mesh::*;
pub use pipeline::*;
pub use surface::*;
pub use swapchain::*;

use crate::{
    imgui::ImGui,
    ren::{Info, Renderer as RendererTrait, Settings, Window, settings::Resolution},
    scene::Scene,
    traits::Drop,
};

use ash::{Entry, vk};

#[allow(unused)]
pub struct Renderer {
    pub settings: Settings,
    pub window: Window,

    pub entry: Entry,
    pub instance: Instance,
    pub surface: Surface,
    pub device: Device,
    pub swapchain: Swapchain,
    pub surface_support: SurfaceSupport,
    pub graphics_queue: vk::Queue,

    pub allocator: Allocator,
    pub descriptor_set_allocator: DescriptorSetAllocator,

    pub submit_manager: SubmitManager,
    pub render_manager: RenderManager,
}

impl Renderer {
    fn draw_imgui(&mut self, imgui: &mut ImGui, command_buffer: vk::CommandBuffer, target: vk::ImageView) {
        unsafe {
            self.device.handle.cmd_begin_rendering(
                command_buffer,
                &pipeline::get_rendering_info(
                    self.swapchain.extent,
                    &[pipeline::get_attachment_info(target, vk::ImageLayout::ATTACHMENT_OPTIMAL, None)],
                    None,
                ),
            )
        };

        imgui.draw(self, command_buffer);

        unsafe { self.device.handle.cmd_end_rendering(command_buffer) };
    }
}

impl RendererTrait for Renderer {
    fn new(info: &Info, settings: Settings, window: Window) -> Self {
        let entry = unsafe { Entry::load().expect("koi::ren::vk - Failed to load Vulkan Instance") };

        let instance = Instance::new(&entry, &info);
        let surface = Surface::new(&entry, &instance.handle, &window);
        let device = Device::new(&instance.handle, &surface);
        let (swapchain, surface_support) =
            Swapchain::new(&instance, &device, &surface, &settings.resolution).expect("koi::ren::vk - failed to create Swapchain");

        let mut allocator = Allocator::new(
            instance.handle.clone(),
            device.handle.clone(),
            device.physical_device,
            &settings,
            device.get_min_memory_map_alignment(),
        );

        let pool_sizes = vec![DescriptorSetPoolSizeRatio::new(vk::DescriptorType::STORAGE_IMAGE)];
        let mut descriptor_set_allocator = DescriptorSetAllocator::new(&device.handle, 10, &pool_sizes, None);
        let graphics_queue = device.get_queue(QueueFamilyType::Graphics);

        let submit_manager = SubmitManager::new(&device, graphics_queue);
        let render_manager = RenderManager::new(&device, &mut allocator, &mut descriptor_set_allocator, &settings);

        Self {
            settings,
            window,

            entry,
            instance,
            surface,
            device,
            swapchain,
            surface_support,
            graphics_queue,

            allocator,
            descriptor_set_allocator,

            submit_manager,
            render_manager,
        }
    }

    fn load_scene(&mut self, scene: &Scene) {
        self.render_manager.load_scene(&self.device.handle, &mut self.allocator, &mut self.submit_manager, scene);
    }

    fn handle_resize(&mut self, resolution: &Resolution) {
        self.swapchain.resize(&self.instance, &self.device, &self.surface, &self.surface_support, resolution);
    }

    fn draw(&mut self, imgui: &mut ImGui) {
        const SECOND_IN_NS: u64 = 10e9 as u64;

        let device_handle: ash::Device = self.device.handle.clone();

        // clone frame data handles
        let Frame { command_buffer, render_fence, render_semaphore, swapchain_semaphore, .. } = self.render_manager.get_current_frame();
        let command_buffer = command_buffer.clone();
        let render_fence = render_fence.clone();
        let render_semaphore = render_semaphore.clone();
        let swapchain_semaphore = swapchain_semaphore.clone();

        // wait until GPU is done rendering the last frame; 1s timeout
        let fences: [vk::Fence; 1] = [render_fence];
        unsafe {
            device_handle.wait_for_fences(&fences, true, SECOND_IN_NS).expect("koi::ren::vk - failed to wait for Render Fence");
            device_handle.reset_fences(&fences).expect("koi::ren::vk - failed to reset Render Fence");
        }

        // drop frame-specific resources
        let frame_index = self.render_manager.get_current_frame_index();
        self.allocator.drop_frame_resources(&device_handle, frame_index);

        // request swapchain image
        let mut swapchain_image_index = 0;
        unsafe {
            match self.swapchain.device.acquire_next_image(self.swapchain.khr, SECOND_IN_NS, swapchain_semaphore, vk::Fence::null()) {
                Ok((index, _suboptimal)) => {
                    swapchain_image_index = index;
                }
                Err(e) => {
                    if e == vk::Result::ERROR_OUT_OF_DATE_KHR {
                        imgui.context.render(); // discard imgui draw
                        return;
                    }
                }
            };
        };
        let swapchain_image = self.swapchain.images[swapchain_image_index as usize];

        // reset/begin frame command buffer
        unsafe {
            device_handle
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .expect("koi::ren::vk - failed to Reset current frame Command Buffer")
        };
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            device_handle
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("koi::ren::vk - failed to Begin current frame Command Buffer")
        };

        // transition draw image to write
        self.render_manager.color_image.transition(&device_handle, command_buffer, vk::ImageLayout::GENERAL);

        self.render_manager.draw_compute(&device_handle, command_buffer);

        // transition draw image for graphics pipeline
        self.render_manager.color_image.transition(&device_handle, command_buffer, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        // transition depth image for graphics pipeline
        self.render_manager.depth_image.transition(&device_handle, command_buffer, vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL);

        self.render_manager.draw_graphics(&device_handle, command_buffer);

        // transition draw image for copy src and swaphain for copy dst; perform ccopy
        self.render_manager.color_image.transition(&device_handle, command_buffer, vk::ImageLayout::TRANSFER_SRC_OPTIMAL);
        image::transition(
            &device_handle,
            command_buffer,
            swapchain_image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        );
        self.render_manager.color_image.copy(&device_handle, command_buffer, swapchain_image, self.swapchain.extent);

        // transition swapchain to draw imgui; draw on swapchain
        image::transition(
            &device_handle,
            command_buffer,
            swapchain_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        );
        self.draw_imgui(imgui, command_buffer, self.swapchain.image_views[swapchain_image_index as usize]);

        // transition swapchain to present
        image::transition(
            &device_handle,
            command_buffer,
            swapchain_image,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
        );

        // end command buffer
        unsafe { device_handle.end_command_buffer(command_buffer).expect("koi::ren::vk - failed to End current frame Command Buffer") };

        // submit command buffer to queue
        let command_buffer_infos = [vk::CommandBufferSubmitInfo::default().command_buffer(command_buffer)];
        let wait_semaphore_infos = [vk::SemaphoreSubmitInfo::default()
            .semaphore(swapchain_semaphore)
            .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)];
        let signal_semaphore_infos =
            [vk::SemaphoreSubmitInfo::default().semaphore(render_semaphore).stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS)];
        let submit_info = [manager::get_submit_info(&command_buffer_infos, Some(&wait_semaphore_infos), Some(&signal_semaphore_infos))];
        unsafe {
            device_handle
                .queue_submit2(self.graphics_queue, &submit_info, render_fence)
                .expect("koi::ren::vk - failed to Submit command buffer to Queue")
        };

        // present swapchain image
        let swapchains = [self.swapchain.khr];
        let wait_semaphores = [render_semaphore];
        let image_indices = [swapchain_image_index];
        let present_info =
            vk::PresentInfoKHR::default().swapchains(&swapchains).wait_semaphores(&wait_semaphores).image_indices(&image_indices);

        unsafe {
            if let Err(e) = self.swapchain.device.queue_present(self.graphics_queue, &present_info) {
                if e == vk::Result::ERROR_OUT_OF_DATE_KHR {
                    return;
                }
            }
        };

        // frame done.
        self.render_manager.done();
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe { self.device.handle.device_wait_idle().expect("koi::ren::vk - failed to Wait for Device Idle") };
        self.submit_manager.drop(&self.device.handle);
        self.render_manager.drop(&self.device.handle);
        self.descriptor_set_allocator.drop(&self.device.handle);
        self.allocator.drop(&self.device.handle);
        self.swapchain.drop(&self.device.handle);
        self.device.drop();
        self.surface.drop();
        self.instance.drop();
    }
}
