pub mod descriptor;
pub mod device;
pub mod frame;
pub mod image;
pub mod instance;
pub mod pipeline;
pub mod resource_allocator;
pub mod surface;
pub mod swapchain;

use crate::{ren::{settings::Resolution, Info, Renderer as RendererTrait, Settings, Window}, traits::Drop};
use resource_allocator::ResourceAllocator;
use descriptor::{DescriptorSetAllocator, DescriptorSetLayoutBuilder, DescriptorSetPoolSizeRatio};
use device::{Device, config::QueueFamilyType};
use frame::Frame;
use image::Image;
use instance::Instance;
use surface::Surface;
use swapchain::{Swapchain, SurfaceSupport};

use ash::{Device as DeviceHandle, Entry, vk};

fn update_sets(device_handle: &DeviceHandle, image_view: vk::ImageView, dst_set: vk::DescriptorSet) {
    let image_info = [
        vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::GENERAL)
        .image_view(image_view)
    ];

    let descriptor_writes = [
        vk::WriteDescriptorSet::default()
            .dst_binding(0)
            .dst_set(dst_set)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .image_info(&image_info)
    ];

    let descriptor_copies = [];

    unsafe{ device_handle.update_descriptor_sets(&descriptor_writes, &descriptor_copies) };
}

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

    resource_allocator: ResourceAllocator,
    descriptor_set_allocator: DescriptorSetAllocator,

    frames: Vec<Frame>,
    graphics_queue: vk::Queue,
    image: Image,
    image_descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
    image_descriptors: Vec<vk::DescriptorSet>,
    compute_shader: vk::ShaderModule,
    compute_pipeline_layout: vk::PipelineLayout,
    compute_pipelines: Vec<vk::Pipeline>,

    frame_count: u32,
}

impl RendererTrait for Renderer {
    fn new(info: &Info, settings: Settings, window: Window) -> Self {
        let entry = unsafe { Entry::load().expect("koi::ren::vk - Failed to load Vulkan Instance") };

        let instance = Instance::new(&entry, &info);
        let surface = Surface::new(&entry, &instance.handle, &window);
        let device = Device::new(&instance.handle, &surface);
        let (swapchain, surface_support) = Swapchain::new(&instance, &device, &surface, &settings.resolution).expect("koi::ren::vk - failed to create Swapchain");

        let mut resource_allocator = ResourceAllocator::new(instance.handle.clone(), device.handle.clone(), device.physical_device.clone(), &settings);
        
        let pool_sizes = vec![DescriptorSetPoolSizeRatio::new(vk::DescriptorType::STORAGE_IMAGE, 1.0)];
        let mut descriptor_set_allocator = DescriptorSetAllocator::new(&device.handle, 10, &pool_sizes);

        let frames = Frame::generator(&device, settings.buffering);
        let graphics_queue = device.get_queue(QueueFamilyType::Graphics);
        let Resolution { width, height } = settings.resolution;
        let image = Image::new(
            &device.handle,
            &mut resource_allocator.handle,
            &mut resource_allocator.global_resources,
            vk::Format::R16G16B16A16_SFLOAT,
            vk::Extent3D::default()
                .width(width)
                .height(height)
                .depth(1),
            vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            vk::ImageAspectFlags::COLOR
        );

        let mut descriptor_set_layout_builder = DescriptorSetLayoutBuilder::default().add_binding(0, vk::DescriptorType::STORAGE_IMAGE);
        let descriptor_set_layout = descriptor_set_layout_builder.build::<vk::DescriptorSetLayoutBindingFlagsCreateInfo>(&device.handle, vk::ShaderStageFlags::COMPUTE, None, None);
        let image_descriptor_set_layouts = vec![descriptor_set_layout];
        let image_descriptors = descriptor_set_allocator.allocate(&device.handle, &image_descriptor_set_layouts);
        update_sets(&device.handle, image.view, image_descriptors.first().unwrap().clone());


        let compute_shader = pipeline::load_shader_module(&device.handle, pipeline::GRADIENT_SHADER, None);
        let compute_pipeline_layout = pipeline::create_pipeline_layout(&device.handle, &image_descriptor_set_layouts);
        let compute_pipelines = pipeline::create_compute_pipeline(&device.handle, compute_shader.clone(), compute_pipeline_layout);

        Self { 
            settings,
            window,

            entry,
            instance,
            surface,
            device,
            swapchain,
            surface_support,

            resource_allocator,
            descriptor_set_allocator,

            frames,
            graphics_queue,
            image,
            image_descriptor_set_layouts,
            image_descriptors,
            compute_shader,
            compute_pipeline_layout,
            compute_pipelines,

            frame_count: 0
        }
    }

    fn draw(&mut self) {
        const SECOND_IN_NS: u64 = 10e9 as u64;

        let device_handle: ash::Device = self.device.handle.clone();

        // clone frame data handles
        let Frame{ command_buffer, render_fence, render_semaphore, swapchain_semaphore, .. } = self.get_current_frame();
        let command_buffer = command_buffer.clone();
        let render_fence = render_fence.clone();
        let render_semaphore = render_semaphore.clone();
        let swapchain_semaphore = swapchain_semaphore.clone();

        // wait until GPU is done rendering the last frame; 1s timeout
        let fences: [vk::Fence; 1] = [render_fence.clone()];
        unsafe { device_handle.wait_for_fences(&fences, true, SECOND_IN_NS).expect("koi::ren::vk - failed to wait for Render Fence") };
        unsafe { device_handle.reset_fences(&fences).expect("koi::ren::vk - failed to reset Render Fence") };

        // drop frame-specific resources
        let frame_index = self.get_current_frame_index();
        self.resource_allocator.drop_frame_resources(&device_handle, frame_index);

        // request swapchain image
        let (swapchain_image_index, _suboptimal) = unsafe { self.swapchain.device.acquire_next_image(self.swapchain.khr, SECOND_IN_NS, swapchain_semaphore, vk::Fence::null()).expect("koi::ren::vk - Failed to acquire next Swapchain Image") };
        let swapchain_image = self.swapchain.images[swapchain_image_index as usize];

        // reset/begin frame command buffer
        unsafe { device_handle.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty()).expect("koi::ren::vk - failed to Reset current frame Command Buffer") };
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe { device_handle.begin_command_buffer(command_buffer, &command_buffer_begin_info).expect("koi::ren::vk - failed to Begin current frame Command Buffer") };

        // transition draw image to write
        let image = self.image.handle.clone();
        image::transition(&device_handle, command_buffer, image, vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL);

        self.write_command_buffer(command_buffer);

        // transition draw image for copy src and swaphain for copy dst; perform ccopy
        image::transition(&device_handle, command_buffer, image, vk::ImageLayout::GENERAL, vk::ImageLayout::TRANSFER_SRC_OPTIMAL);
        image::transition(&device_handle, command_buffer, swapchain_image, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        image::copy(&device_handle, command_buffer, image, swapchain_image, self.image.extent_2d, self.swapchain.extent);

        // transition swapchain to present
        image::transition(&device_handle, command_buffer, swapchain_image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::PRESENT_SRC_KHR);
        
        // end command buffer
        unsafe { device_handle.end_command_buffer(command_buffer).expect("koi::ren::vk - failed to End current frame Command Buffer") };

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
        unsafe { device_handle.queue_submit2(self.graphics_queue, &submit_info, render_fence).expect("koi::ren::vk - failed to Submit command buffer to Queue") };

        // present swapchain image
        let swapchains = [self.swapchain.khr];
        let wait_semaphores = [render_semaphore];
        let image_indices = [swapchain_image_index];
        let present_info = vk::PresentInfoKHR::default()
            .swapchains(&swapchains)
            .wait_semaphores(&wait_semaphores)
            .image_indices(&image_indices);

        unsafe{ self.swapchain.device.queue_present(self.graphics_queue, &present_info).expect("koi::ren::vk - failed to Present swapchain image") };

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
        unsafe {
            self.device.handle.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::COMPUTE, self.compute_pipelines[0]);
            let dynamic_offsets = [];
            self.device.handle.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::COMPUTE, self.compute_pipeline_layout, 0, &self.image_descriptors, &dynamic_offsets);
            self.device.handle.cmd_dispatch(command_buffer, (self.image.extent_2d.width as f32 / 16.0).ceil() as u32, (self.image.extent_2d.height as f32 / 16.0).ceil() as u32, 1);
        };
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe { self.device.handle.device_wait_idle().expect("koi::ren::vk - failed to Wait for Device Idle") };

        unsafe { 
            self.device.handle.destroy_shader_module(self.compute_shader, None);
            self.device.handle.destroy_pipeline_layout(self.compute_pipeline_layout, None);
            self.compute_pipelines.iter().for_each(|pipeline|  self.device.handle.destroy_pipeline(pipeline.clone(), None));
        };
        
        self.frames.iter_mut().for_each(|frame| frame.drop(&self.device.handle));
        self.descriptor_set_allocator.drop(&self.device.handle);
        unsafe { self.device.handle.destroy_descriptor_set_layout(self.image_descriptor_set_layouts.first().unwrap().clone(), None) };
        self.resource_allocator.drop(&self.device.handle);
        self.swapchain.drop(&self.device.handle);
        self.device.drop();
        self.surface.drop();
        self.instance.drop();
    }
}