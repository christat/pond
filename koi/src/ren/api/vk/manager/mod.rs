use crate::{
    ren::{
        Allocator, ComputePipeline, ComputePushConstants, DescriptorSetAllocator,
        DescriptorSetLayoutBuilder, Device, Frame, Image, Mesh, QueueFamilyType, Resolution,
        Settings, pipeline,
    },
    scene::Scene,
};

use ash::{Device as DeviceHandle, vk};
use koi_gpu::{PUSH_CONSTANTS_SIZE, PushConstants};
use spirv_std::glam::{Mat4, Vec4};

pub struct RenderManager {
    pub buffering: u32,
    pub frames: Vec<Frame>,
    pub color_image: Image,
    pub depth_image: Image,
    pub color_image_descriptor_set_layout: vk::DescriptorSetLayout,
    pub color_image_descriptor: vk::DescriptorSet,
    pub frame_count: u32,

    pub compute_pipelines: [ComputePipeline; 2],
    pub compute_pipeline_index: usize,

    pub graphics_pipeline_layout: vk::PipelineLayout,
    pub graphics_pipeline: vk::Pipeline,
    pub vertex_shader_module: vk::ShaderModule,
    pub fragment_shader_module: vk::ShaderModule,
    pub meshes: Vec<Mesh>,
}

impl<'a> RenderManager {
    pub fn new(
        device: &Device,
        allocator: &mut Allocator,
        descriptor_set_allocator: &mut DescriptorSetAllocator,
        settings: &Settings,
    ) -> Self {
        let frames = Frame::generator(&device, settings.buffering);
        let Resolution { width, height } = settings.resolution;
        let color_image = Image::new(
            &device.handle,
            allocator,
            vk::Format::R16G16B16A16_SFLOAT,
            vk::Extent3D::default().width(width).height(height).depth(1),
            vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::STORAGE
                | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            vk::ImageAspectFlags::COLOR,
            vk::ImageLayout::UNDEFINED,
        );
        let depth_image = Image::new(
            &device.handle,
            allocator,
            vk::Format::D32_SFLOAT,
            vk::Extent3D::default().width(width).height(height).depth(1),
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::ImageAspectFlags::DEPTH,
            vk::ImageLayout::UNDEFINED,
        );

        let mut descriptor_set_layout_builder =
            DescriptorSetLayoutBuilder::default().add_binding(0, vk::DescriptorType::STORAGE_IMAGE);
        let color_image_descriptor_set_layout = descriptor_set_layout_builder
            .build::<vk::DescriptorSetLayoutBindingFlagsCreateInfo>(
            &device.handle,
            vk::ShaderStageFlags::COMPUTE,
            None,
            None,
        );
        let color_image_descriptor_set_layouts = vec![color_image_descriptor_set_layout];
        let color_image_descriptor =
            descriptor_set_allocator.allocate(&device.handle, &color_image_descriptor_set_layouts);
        Self::update_sets(&device.handle, color_image.view, color_image_descriptor);

        let gradient_shader = include_bytes!(env!("gradient.spv"));
        let gradient_shader_module =
            pipeline::load_shader_module(&device.handle, gradient_shader, None);
        let sky_shader = include_bytes!(env!("sky.spv"));
        let sky_shader_module = pipeline::load_shader_module(&device.handle, sky_shader, None);

        let push_constant_ranges = [vk::PushConstantRange::default()
            .offset(0)
            .size(size_of::<ComputePushConstants>() as u32)
            .stage_flags(vk::ShaderStageFlags::COMPUTE)];
        let compute_pipeline_layout = pipeline::create_pipeline_layout(
            &device.handle,
            &color_image_descriptor_set_layouts,
            Some(&push_constant_ranges),
        );

        let gradient_pipeline = ComputePipeline {
            name: String::from("gradient"),
            shader: gradient_shader_module,
            handle: pipeline::create_compute_pipeline(
                &device.handle,
                gradient_shader_module,
                compute_pipeline_layout,
            ),
            pipeline_layout: compute_pipeline_layout,
            push_constants: ComputePushConstants::default()
                .data_0(Vec4::new(0.14, 0.44, 0.86, 1.0))
                .data_1(Vec4::new(0.5, 0.54, 0.38, 1.0)),
        };

        let sky_pipeline = ComputePipeline {
            name: String::from("sky"),
            shader: sky_shader_module,
            handle: pipeline::create_compute_pipeline(
                &device.handle,
                sky_shader_module,
                compute_pipeline_layout,
            ),
            pipeline_layout: compute_pipeline_layout,
            push_constants: pipeline::ComputePushConstants::default()
                .data_0(Vec4::new(0.14, 0.17, 0.36, 1.0))
                .data_1(Vec4::new(0.0, 0.0, 0.0, 0.98)),
        };

        let vertex_shader = include_bytes!("../../../../../../shaders/glsl/vertex.spv");
        let fragment_shader = include_bytes!(env!("fragment.spv"));
        let vertex_shader_module =
            pipeline::load_shader_module(&device.handle, vertex_shader, None);
        let fragment_shader_module =
            pipeline::load_shader_module(&device.handle, fragment_shader, None);

        let push_constant_ranges = [vk::PushConstantRange::default()
            .offset(0)
            .size(PUSH_CONSTANTS_SIZE as u32)
            .stage_flags(vk::ShaderStageFlags::VERTEX)];
        let graphics_pipeline_layout =
            pipeline::create_pipeline_layout(&device.handle, &[], Some(&push_constant_ranges));
        let graphics_pipeline = pipeline::PipelineBuilder::default()
            .pipeline_layout(graphics_pipeline_layout)
            .shaders(vertex_shader_module, Some(fragment_shader_module))
            .input_topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE, vk::FrontFace::CLOCKWISE)
            .multisampling()
            .blending_alpha_blend()
            .depth_stencil_state(true, vk::CompareOp::GREATER_OR_EQUAL)
            .color_attachment_formats(&[color_image.format])
            .depth_attachment_format(depth_image.format)
            .build(&device.handle);

        Self {
            buffering: settings.buffering,
            frames,
            color_image,
            depth_image,
            color_image_descriptor_set_layout,
            color_image_descriptor,
            frame_count: 0,

            compute_pipelines: [sky_pipeline, gradient_pipeline],
            compute_pipeline_index: 0,

            graphics_pipeline,
            graphics_pipeline_layout,
            vertex_shader_module,
            fragment_shader_module,
            meshes: vec![],
        }
    }

    pub fn load_scene(
        &mut self,
        device_handle: &DeviceHandle,
        allocator: &mut Allocator,
        submit: &mut SubmitManager,
        scene: &Scene,
    ) {
        for mesh in &scene.meshes {
            self.meshes.push(Mesh::new(
                device_handle,
                allocator,
                submit,
                &mesh.indices,
                &mesh.vertices,
                mesh.surfaces.clone(),
            ));
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

        unsafe { device_handle.update_descriptor_sets(&descriptor_writes, &[]) };
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

    pub fn draw_compute(
        &mut self,
        device_handle: &DeviceHandle,
        command_buffer: vk::CommandBuffer,
    ) {
        unsafe {
            let compute_pipeline = &self.compute_pipelines[self.compute_pipeline_index];
            device_handle.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                compute_pipeline.handle,
            );
            device_handle.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                compute_pipeline.pipeline_layout,
                0,
                &[self.color_image_descriptor],
                &[],
            );
            device_handle.cmd_push_constants(
                command_buffer,
                compute_pipeline.pipeline_layout,
                vk::ShaderStageFlags::COMPUTE,
                0,
                &compute_pipeline.push_constants.as_buffer(),
            );
            device_handle.cmd_dispatch(
                command_buffer,
                (self.color_image.extent.width as f32 / 16.0).ceil() as u32,
                (self.color_image.extent.height as f32 / 16.0).ceil() as u32,
                1,
            );
        };
    }

    pub fn draw_graphics(
        &mut self,
        device_handle: &DeviceHandle,
        command_buffer: vk::CommandBuffer,
    ) {
        let color_attachments = [pipeline::get_attachment_info(
            self.color_image.view,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            None,
        )];
        let mut depth_clear_value = vk::ClearValue::default();
        depth_clear_value.depth_stencil = vk::ClearDepthStencilValue::default().depth(0.0);
        let depth_attachment = pipeline::get_attachment_info(
            self.depth_image.view,
            vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            Some(depth_clear_value),
        );
        let rendering_info = pipeline::get_rendering_info(
            vk::Extent2D::default()
                .width(self.color_image.extent.width)
                .height(self.color_image.extent.height),
            &color_attachments,
            Some(&depth_attachment),
        );

        unsafe {
            device_handle.cmd_begin_rendering(command_buffer, &rendering_info);
            device_handle.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline,
            )
        };

        let aspect_ratio =
            self.color_image.extent.width as f32 / self.color_image.extent.height as f32;
        let mut view = Mat4::IDENTITY;
        *view.col_mut(3) = Vec4::new(0.0, 0.0, -5.0, 1.0);
        let projection = Mat4::perspective_rh(70.0, aspect_ratio, 10000.0, 0.1);
        let world_transform = projection * view;

        let test_mesh = &self.meshes[2];
        unsafe {
            device_handle.cmd_push_constants(
                command_buffer,
                self.graphics_pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                &PushConstants::default()
                    .vertex_buffer_address(test_mesh.vertex_buffer_address)
                    .world_transform(world_transform)
                    .as_buffer(),
            );
            device_handle.cmd_bind_index_buffer(
                command_buffer,
                test_mesh.index_buffer.handle,
                0,
                vk::IndexType::UINT32,
            );
        };

        let image_extent_height = self.color_image.extent.height as f32;
        let viewports = [vk::Viewport::default()
            .x(0.0)
            .y(image_extent_height)
            .width(self.color_image.extent.width as f32)
            .height(-image_extent_height)
            .min_depth(0.0)
            .max_depth(1.0)];

        unsafe { device_handle.cmd_set_viewport(command_buffer, 0, &viewports) };

        let scissors = [vk::Rect2D::default()
            .offset(vk::Offset2D::default().x(0).y(0))
            .extent(
                vk::Extent2D::default()
                    .width(self.color_image.extent.width)
                    .height(self.color_image.extent.height),
            )];

        unsafe {
            device_handle.cmd_set_scissor(command_buffer, 0, &scissors);
            device_handle.cmd_draw_indexed(
                command_buffer,
                test_mesh.surfaces[0].count,
                1,
                test_mesh.surfaces[0].start_index,
                0,
                0,
            );
            device_handle.cmd_end_rendering(command_buffer);
        }
    }

    pub fn drop(&mut self, device_handle: &DeviceHandle) {
        unsafe {
            device_handle.destroy_shader_module(self.fragment_shader_module, None);
            device_handle.destroy_shader_module(self.vertex_shader_module, None);
            device_handle.destroy_pipeline_layout(self.graphics_pipeline_layout, None);
            device_handle.destroy_pipeline(self.graphics_pipeline, None);
            self.compute_pipelines.iter().for_each(|effect| {
                device_handle.destroy_shader_module(effect.shader, None);
                device_handle.destroy_pipeline_layout(effect.pipeline_layout, None);
                device_handle.destroy_pipeline(effect.handle, None);
            });
            self.frames
                .iter_mut()
                .for_each(|frame| frame.drop(device_handle));
            device_handle
                .destroy_descriptor_set_layout(self.color_image_descriptor_set_layout, None);
        };
    }
}

pub struct SubmitManager {
    pub queue: vk::Queue,
    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,
    pub fence: vk::Fence,
}

impl<'a> SubmitManager {
    pub fn new(device: &Device, queue: vk::Queue) -> Self {
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
            queue,
            command_pool,
            command_buffer,
            fence,
        }
    }

    pub fn submit<T: Fn(vk::CommandBuffer)>(
        &mut self,
        device_handle: &DeviceHandle,
        command_recorder: &T,
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
        let submit_info = [get_submit_info(&command_buffer_infos, None, None)];
        unsafe {
            device_handle
                .queue_submit2(self.queue, &submit_info, self.fence)
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

pub fn get_submit_info<'a>(
    command_buffer_infos: &'a [vk::CommandBufferSubmitInfo<'a>],
    wait_semaphore_infos: Option<&'a [vk::SemaphoreSubmitInfo]>,
    signal_semaphore_infos: Option<&'a [vk::SemaphoreSubmitInfo]>,
) -> vk::SubmitInfo2<'a> {
    let mut submit_info = vk::SubmitInfo2::default().command_buffer_infos(command_buffer_infos);

    if wait_semaphore_infos.is_some() {
        submit_info = submit_info.wait_semaphore_infos(wait_semaphore_infos.unwrap());
    };
    if signal_semaphore_infos.is_some() {
        submit_info = submit_info.signal_semaphore_infos(signal_semaphore_infos.unwrap());
    };

    submit_info
}
