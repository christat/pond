pub mod buffer;
pub mod descriptor;
pub mod device;
pub mod frame;
pub mod image;
pub mod imgui;
pub mod instance;
pub mod pipeline;
pub mod resource_allocator;
pub mod surface;
pub mod swapchain;

use std::u64;

use crate::{
    imgui::ImGui,
    ren::{Info, Renderer as RendererTrait, Settings, Window, settings::Resolution},
    traits::Drop,
};
use descriptor::{DescriptorSetAllocator, DescriptorSetLayoutBuilder, DescriptorSetPoolSizeRatio};
use device::{Device, config::QueueFamilyType};
use frame::Frame;
use image::Image;
use instance::Instance;
use resource_allocator::ResourceAllocator;
use surface::Surface;
use swapchain::{SurfaceSupport, Swapchain};

use ash::{Device as DeviceHandle, Entry, vk};

pub struct DrawManager {
    buffering: u32,
    pub frames: Vec<Frame>,
    pub image: Image,
    pub image_descriptor_set_layout: vk::DescriptorSetLayout,
    pub image_descriptor: vk::DescriptorSet,
    pub compute_shader: vk::ShaderModule,
    pub compute_pipeline_layout: vk::PipelineLayout,
    pub compute_pipeline: vk::Pipeline,
    pub frame_count: u32,
}

impl<'a> DrawManager {
    pub fn new(
        device: &Device,
        resource_allocator: &mut ResourceAllocator,
        descriptor_set_allocator: &mut DescriptorSetAllocator,
        settings: &Settings,
    ) -> Self {
        let frames = Frame::generator(&device, settings.buffering);
        let Resolution { width, height } = settings.resolution;
        let image = Image::new(
            &device.handle,
            &mut resource_allocator.handle,
            &mut resource_allocator.global_resources,
            vk::Format::R16G16B16A16_SFLOAT,
            vk::Extent3D::default().width(width).height(height).depth(1),
            vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::STORAGE
                | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            vk::ImageAspectFlags::COLOR,
        );

        let mut descriptor_set_layout_builder =
            DescriptorSetLayoutBuilder::default().add_binding(0, vk::DescriptorType::STORAGE_IMAGE);
        let image_descriptor_set_layout = descriptor_set_layout_builder
            .build::<vk::DescriptorSetLayoutBindingFlagsCreateInfo>(
            &device.handle,
            vk::ShaderStageFlags::COMPUTE,
            None,
            None,
        );
        let image_descriptor_set_layouts = vec![image_descriptor_set_layout];
        let image_descriptor =
            descriptor_set_allocator.allocate(&device.handle, &image_descriptor_set_layouts);
        Self::update_sets(&device.handle, image.view, image_descriptor);

        let compute_shader = pipeline::load_shader_module(
            &device.handle,
            include_bytes!(env!("gradient.spv")),
            None,
        );
        let compute_pipeline_layout =
            pipeline::create_pipeline_layout(&device.handle, &image_descriptor_set_layouts, None);
        let compute_pipeline = pipeline::create_compute_pipeline(
            &device.handle,
            compute_shader,
            compute_pipeline_layout,
        );

        Self {
            buffering: settings.buffering,
            frames,
            image,
            image_descriptor_set_layout,
            image_descriptor,
            compute_shader,
            compute_pipeline_layout,
            compute_pipeline,
            frame_count: 0,
        }
    }

    fn update_sets(
        device_handle: &DeviceHandle,
        image_view: vk::ImageView,
        dst_set: vk::DescriptorSet,
    ) {
        let image_info = [vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::GENERAL)
            .image_view(image_view)];

        let descriptor_writes = [vk::WriteDescriptorSet::default()
            .dst_binding(0)
            .dst_set(dst_set)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .image_info(&image_info)];

        let descriptor_copies = [];

        unsafe { device_handle.update_descriptor_sets(&descriptor_writes, &descriptor_copies) };
    }

    pub fn get_current_frame_index(&self) -> usize {
        (self.frame_count % self.buffering) as usize
    }

    pub fn get_current_frame(&'a mut self) -> &'a mut Frame {
        let index = self.get_current_frame_index();
        &mut self.frames[index]
    }

    pub fn done(&mut self) {
        self.frame_count += 1;
    }

    pub fn write_command_buffer(
        &mut self,
        device_handle: &DeviceHandle,
        command_buffer: vk::CommandBuffer,
    ) {
        unsafe {
            device_handle.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.compute_pipeline,
            );
            device_handle.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.compute_pipeline_layout,
                0,
                &[self.image_descriptor],
                &[],
            );
            device_handle.cmd_dispatch(
                command_buffer,
                (self.image.extent_2d.width as f32 / 16.0).ceil() as u32,
                (self.image.extent_2d.height as f32 / 16.0).ceil() as u32,
                1,
            );
        };
    }

    pub fn drop(&mut self, device_handle: &DeviceHandle) {
        unsafe {
            device_handle.destroy_shader_module(self.compute_shader, None);
            device_handle.destroy_pipeline_layout(self.compute_pipeline_layout, None);
            device_handle.destroy_pipeline(self.compute_pipeline, None);
            self.frames
                .iter_mut()
                .for_each(|frame| frame.drop(device_handle));
            device_handle.destroy_descriptor_set_layout(self.image_descriptor_set_layout, None);
        };
    }
}

pub struct ImmediateManager {
    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,
    pub fence: vk::Fence,
}

impl<'a> ImmediateManager {
    pub fn new(device: &Device) -> Self {
        let pool_create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(
                device
                    .queue_families
                    .get_family_index(QueueFamilyType::Graphics),
            );

        let command_pool = unsafe {
            device
                .handle
                .create_command_pool(&pool_create_info, None)
                .expect("koi::ren::vk - failed to create Command Pool")
        };

        let buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .command_buffer_count(1)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffer = unsafe {
            device
                .handle
                .allocate_command_buffers(&buffer_allocate_info)
                .expect("koi::ren::vk - failed to allocate Command Buffer")[0]
        };

        let fence_create_info =
            vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let fence = unsafe {
            device
                .handle
                .create_fence(&fence_create_info, None)
                .expect("koi::ren::vk - failed to create Fence")
        };

        Self {
            command_pool,
            command_buffer,
            fence,
        }
    }

    pub fn submit(
        &mut self,
        device_handle: &DeviceHandle,
        queue: vk::Queue,
        command_recorder: fn(vk::CommandBuffer),
    ) {
        let fences = [self.fence];
        unsafe {
            device_handle
                .reset_fences(&fences)
                .expect("koi::ren::vk - failed to reset ImmediateManager Fence");
            device_handle
                .reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::empty())
                .expect("koi::ren::vk - failed to reset ImmediateManager Command Buffer");
        }

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            device_handle
                .begin_command_buffer(self.command_buffer, &command_buffer_begin_info)
                .expect("koi::ren::vk - failed to begin ImmediateManager Command Buffer")
        };

        command_recorder(self.command_buffer);

        unsafe {
            device_handle
                .end_command_buffer(self.command_buffer)
                .expect("koi::ren::vk - failed to end ImmediateManager Command Buffer")
        };

        let command_buffer_infos =
            [vk::CommandBufferSubmitInfo::default().command_buffer(self.command_buffer)];
        let submit_info = [frame::get_submit_info(&command_buffer_infos, None, None)];
        unsafe {
            device_handle
                .queue_submit2(queue, &submit_info, self.fence)
                .expect("koi::ren::vk - failed to Submit ImmediateManager command buffer to Queue");
            device_handle
                .wait_for_fences(&fences, true, u64::MAX)
                .expect("koi::ren::vk - failed to wait for ImmediateManager Fence");
        }
    }

    pub fn drop(&mut self, device_handle: &DeviceHandle) {
        unsafe {
            device_handle.destroy_command_pool(self.command_pool, None);
            device_handle.destroy_fence(self.fence, None);
        }
    }
}

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
    graphics_queue: vk::Queue,

    resource_allocator: ResourceAllocator,
    descriptor_set_allocator: DescriptorSetAllocator,

    draw_manager: DrawManager,
    // immediate_manager: ImmediateManager,
}

impl Renderer {
    fn draw_imgui(
        &mut self,
        imgui: &mut ImGui,
        command_buffer: vk::CommandBuffer,
        target: vk::ImageView,
    ) {
        let color_attachments = [pipeline::get_attachment_info(
            target,
            vk::ImageLayout::ATTACHMENT_OPTIMAL,
            None,
        )];
        let rendering_info =
            pipeline::get_rendering_info(self.swapchain.extent, &color_attachments, None);

        unsafe {
            self.device
                .handle
                .cmd_begin_rendering(command_buffer, &rendering_info)
        };

        imgui.draw(self, command_buffer);

        unsafe { self.device.handle.cmd_end_rendering(command_buffer) };
    }
}

impl RendererTrait for Renderer {
    fn new(info: &Info, settings: Settings, window: Window) -> Self {
        let entry =
            unsafe { Entry::load().expect("koi::ren::vk - Failed to load Vulkan Instance") };

        let instance = Instance::new(&entry, &info);
        let surface = Surface::new(&entry, &instance.handle, &window);
        let device = Device::new(&instance.handle, &surface);
        let (swapchain, surface_support) =
            Swapchain::new(&instance, &device, &surface, &settings.resolution)
                .expect("koi::ren::vk - failed to create Swapchain");

        let mut resource_allocator = ResourceAllocator::new(
            instance.handle.clone(),
            device.handle.clone(),
            device.physical_device,
            &settings,
        );

        let pool_sizes = vec![DescriptorSetPoolSizeRatio::new(
            vk::DescriptorType::STORAGE_IMAGE,
            1.0,
        )];
        let mut descriptor_set_allocator =
            DescriptorSetAllocator::new(&device.handle, 10, &pool_sizes);
        let graphics_queue = device.get_queue(QueueFamilyType::Graphics);

        let draw_manager = DrawManager::new(
            &device,
            &mut resource_allocator,
            &mut descriptor_set_allocator,
            &settings,
        );
        // let immediate_manager = ImmediateManager::new(&device);

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

            resource_allocator,
            descriptor_set_allocator,

            draw_manager,
            // immediate_manager,
        }
    }

    fn draw(&mut self, imgui: &mut ImGui) {
        const SECOND_IN_NS: u64 = 10e9 as u64;

        let device_handle: ash::Device = self.device.handle.clone();

        // clone frame data handles
        let Frame {
            command_buffer,
            render_fence,
            render_semaphore,
            swapchain_semaphore,
            ..
        } = self.draw_manager.get_current_frame();
        let command_buffer = command_buffer.clone();
        let render_fence = render_fence.clone();
        let render_semaphore = render_semaphore.clone();
        let swapchain_semaphore = swapchain_semaphore.clone();

        // wait until GPU is done rendering the last frame; 1s timeout
        let fences: [vk::Fence; 1] = [render_fence];
        unsafe {
            device_handle
                .wait_for_fences(&fences, true, SECOND_IN_NS)
                .expect("koi::ren::vk - failed to wait for Render Fence");
            device_handle
                .reset_fences(&fences)
                .expect("koi::ren::vk - failed to reset Render Fence");
        }

        // drop frame-specific resources
        let frame_index = self.draw_manager.get_current_frame_index();
        self.resource_allocator
            .drop_frame_resources(&device_handle, frame_index);

        // request swapchain image
        let (swapchain_image_index, _suboptimal) = unsafe {
            self.swapchain
                .device
                .acquire_next_image(
                    self.swapchain.khr,
                    SECOND_IN_NS,
                    swapchain_semaphore,
                    vk::Fence::null(),
                )
                .expect("koi::ren::vk - Failed to acquire next Swapchain Image")
        };
        let swapchain_image = self.swapchain.images[swapchain_image_index as usize];

        // reset/begin frame command buffer
        unsafe {
            device_handle
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .expect("koi::ren::vk - failed to Reset current frame Command Buffer")
        };
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            device_handle
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("koi::ren::vk - failed to Begin current frame Command Buffer")
        };

        // transition draw image to write
        let image = self.draw_manager.image.handle.clone();
        image::transition(
            &device_handle,
            command_buffer,
            image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
        );

        self.draw_manager
            .write_command_buffer(&device_handle, command_buffer);

        // transition draw image for copy src and swaphain for copy dst; perform ccopy
        image::transition(
            &device_handle,
            command_buffer,
            image,
            vk::ImageLayout::GENERAL,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        );
        image::transition(
            &device_handle,
            command_buffer,
            swapchain_image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        );
        image::copy(
            &device_handle,
            command_buffer,
            image,
            swapchain_image,
            self.draw_manager.image.extent_2d,
            self.swapchain.extent,
        );

        // transition swapchain to draw imgui; draw on swapchain
        image::transition(
            &device_handle,
            command_buffer,
            swapchain_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        );
        self.draw_imgui(
            imgui,
            command_buffer,
            self.swapchain.image_views[swapchain_image_index as usize],
        );

        // transition swapchain to present
        image::transition(
            &device_handle,
            command_buffer,
            swapchain_image,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
        );

        // end command buffer
        unsafe {
            device_handle
                .end_command_buffer(command_buffer)
                .expect("koi::ren::vk - failed to End current frame Command Buffer")
        };

        // submit command buffer to queue
        let command_buffer_infos =
            [vk::CommandBufferSubmitInfo::default().command_buffer(command_buffer)];
        let wait_semaphore_infos = [vk::SemaphoreSubmitInfo::default()
            .semaphore(swapchain_semaphore)
            .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)];
        let signal_semaphore_infos = [vk::SemaphoreSubmitInfo::default()
            .semaphore(render_semaphore)
            .stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS)];
        let submit_info = [frame::get_submit_info(
            &command_buffer_infos,
            Some(&wait_semaphore_infos),
            Some(&signal_semaphore_infos),
        )];
        unsafe {
            device_handle
                .queue_submit2(self.graphics_queue, &submit_info, render_fence)
                .expect("koi::ren::vk - failed to Submit command buffer to Queue")
        };

        // present swapchain image
        let swapchains = [self.swapchain.khr];
        let wait_semaphores = [render_semaphore];
        let image_indices = [swapchain_image_index];
        let present_info = vk::PresentInfoKHR::default()
            .swapchains(&swapchains)
            .wait_semaphores(&wait_semaphores)
            .image_indices(&image_indices);

        unsafe {
            self.swapchain
                .device
                .queue_present(self.graphics_queue, &present_info)
                .expect("koi::ren::vk - failed to Present swapchain image")
        };

        // frame done.
        self.draw_manager.done();
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .handle
                .device_wait_idle()
                .expect("koi::ren::vk - failed to Wait for Device Idle")
        };
        // self.immediate_manager.drop(&self.device.handle);
        self.draw_manager.drop(&self.device.handle);
        self.descriptor_set_allocator.drop(&self.device.handle);
        self.resource_allocator.drop(&self.device.handle);
        self.swapchain.drop(&self.device.handle);
        self.device.drop();
        self.surface.drop();
        self.instance.drop();
    }
}
