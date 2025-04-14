use crate::ren::{Buffer, DescriptorSetAllocator, DescriptorSetPoolSizeRatio, Image, QueueFamilyType, pipeline};

use ash::{
    Device as DeviceHandle,
    vk::{self, Handle},
};
use bytemuck::cast_slice;
use glam::Vec2;
use gpu_allocator::{MemoryLocation, vulkan as vka};
use imgui::{DrawData, internal::RawWrapper};
use std::collections::VecDeque;

// NB! Reference implementation shamelessly stolen from:
// https://github.com/ocornut/imgui/blob/master/examples/example_win32_vulkan/main.cpp

const IMGUI_DEFAULT_DESCRIPTOR_COUNT: u32 = 1000;
const IMGUI_DEFAULT_ALLOCATION_SIZE: vk::DeviceSize = 1024 * 1024;
const F32_SIZE: usize = size_of::<f32>();
const IDX_SIZE: usize = size_of::<imgui::DrawIdx>();
const VTX_SIZE: usize = size_of::<imgui::DrawVert>();

#[allow(unused)]
pub struct BackendRendererUserData {
    command_buffer: vk::CommandBuffer,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
}

type RenderBuffers = (Buffer, vka::Allocation, Buffer, vka::Allocation);

fn create_render_buffers(device_handle: &DeviceHandle, allocator: &mut vka::Allocator, index: u32) -> RenderBuffers {
    let (index_buffer, index_buffer_allocation) = Buffer::create(
        device_handle,
        allocator,
        IMGUI_DEFAULT_ALLOCATION_SIZE,
        vk::BufferUsageFlags::INDEX_BUFFER,
        &format!("imgui_index_buffer_{}", index),
        MemoryLocation::CpuToGpu,
    );
    let (vertex_buffer, vertex_buffer_allocation) = Buffer::create(
        device_handle,
        allocator,
        IMGUI_DEFAULT_ALLOCATION_SIZE,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        &format!("imgui_vertex_buffer_{}", index),
        MemoryLocation::CpuToGpu,
    );
    (index_buffer, index_buffer_allocation, vertex_buffer, vertex_buffer_allocation)
}

struct ViewportResources {
    pub index: u32,
    pub count: u32,
    pub buffers: VecDeque<RenderBuffers>,
}

impl<'a> ViewportResources {
    pub fn new(device_handle: &DeviceHandle, allocator: &mut vka::Allocator, count: u32) -> Self {
        Self { index: count, count, buffers: (0..count).map(|index| create_render_buffers(device_handle, allocator, index)).collect() }
    }

    pub fn pop(&mut self) -> RenderBuffers {
        self.index = self.index + 1 % self.count;
        self.buffers.pop_front().unwrap()
    }

    pub fn push(&mut self, buffers: RenderBuffers) {
        self.buffers.push_back(buffers);
    }
}

pub struct Renderer {
    // Base resources
    texture_sampler: vk::Sampler,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_set_allocator: DescriptorSetAllocator,
    descriptor_set: vk::DescriptorSet,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    shader_module: vk::ShaderModule,

    // Font Atlas resources
    atlas: Image,
    atlas_allocation: VecDeque<vka::Allocation>,

    // Viewport resources
    viewport_resources: ViewportResources,
}

impl Renderer {
    pub fn new(context: &mut imgui::Context, api: &mut super::Renderer, image_count: u32) -> Self {
        let (texture_sampler, descriptor_set_layout, descriptor_set, descriptor_set_allocator, pipeline_layout, pipeline, shader_module) =
            initialize_vulkan_structures(api);

        let (atlas, atlas_allocation) = create_font_atlas(context, api, descriptor_set, texture_sampler);

        let viewport_resources = ViewportResources::new(&api.device.handle, &mut api.allocator.handle, image_count);

        Self {
            texture_sampler,
            descriptor_set_layout,
            descriptor_set_allocator,
            descriptor_set,
            pipeline_layout,
            pipeline,
            shader_module,
            atlas,
            atlas_allocation: VecDeque::from([atlas_allocation]),
            viewport_resources,
        }
    }

    pub fn draw(&mut self, context: &mut imgui::Context, api: &mut super::Renderer, command_buffer: vk::CommandBuffer) {
        let draw_data = context.render();

        let [display_width, display_height] = draw_data.display_size;
        let [framebuffer_scale_x, framebuffer_scale_y] = draw_data.framebuffer_scale;
        let framebuffer_width = framebuffer_scale_x * display_width;
        let framebuffer_height = framebuffer_scale_y * display_height;
        if framebuffer_width <= 0.0 || framebuffer_height <= 0.0 {
            return;
        }

        let (mut index_buffer, mut index_buffer_allocation, mut vertex_bufer, mut vertex_buffer_allocation) = self.viewport_resources.pop();

        // copy index/vertex buffers
        if draw_data.total_vtx_count > 0 {
            let required_index_buffer_size = (draw_data.total_idx_count as usize * IDX_SIZE) as vk::DeviceSize;
            if index_buffer.size < required_index_buffer_size {
                index_buffer_allocation = index_buffer.resize(
                    &api.device.handle,
                    &mut api.allocator.handle,
                    index_buffer_allocation,
                    required_index_buffer_size,
                    &format!("imgui_index_buffer_{}", self.viewport_resources.index),
                );
            }

            let required_vertex_buffer_size = (draw_data.total_vtx_count as usize * VTX_SIZE) as vk::DeviceSize;
            if vertex_bufer.size < required_vertex_buffer_size {
                vertex_buffer_allocation = vertex_bufer.resize(
                    &api.device.handle,
                    &mut api.allocator.handle,
                    vertex_buffer_allocation,
                    required_vertex_buffer_size,
                    &format!("imgui_vertex_buffer_{}", self.viewport_resources.index),
                );
            }

            let mut indices = Vec::with_capacity(draw_data.total_idx_count as usize);
            let mut vertices = Vec::with_capacity(draw_data.total_vtx_count as usize);
            for draw_list in draw_data.draw_lists() {
                indices.extend_from_slice(draw_list.idx_buffer());
                vertices.extend_from_slice(draw_list.vtx_buffer());
            }

            index_buffer.upload(indices.as_slice(), &mut index_buffer_allocation, 0);
            vertex_bufer.upload(vertices.as_slice(), &mut vertex_buffer_allocation, 0);
        }

        // setup render state
        setup_render_state(
            api,
            draw_data,
            index_buffer.handle,
            vertex_bufer.handle,
            self.pipeline_layout,
            self.pipeline,
            command_buffer,
            framebuffer_width,
            framebuffer_height,
        );

        // TODO
        // {
        //     let io: &mut imgui::sys::ImGuiIO = unsafe{ context.io_mut().raw_mut() };
        //     let mut renderer_user_data = BackendRendererUserData{ command_buffer, pipeline: self.pipeline, pipeline_layout: self.pipeline_layout };
        //     let ptr: *mut BackendRendererUserData = &mut renderer_user_data;
        //     io.BackendRendererUserData = ptr as *mut c_void;
        // }

        let clip_off = Vec2::from(draw_data.display_pos);
        let clip_scale = Vec2::from(draw_data.framebuffer_scale);

        // render command lists
        if draw_data.draw_lists_count() > 0 {
            let mut idx_offset = 0;
            let mut vtx_offset = 0;

            for draw_list in draw_data.draw_lists() {
                let draw_list_raw = unsafe { draw_list.raw() };

                for command in draw_list.commands() {
                    match command {
                        imgui::DrawCmd::Elements { count, cmd_params } => {
                            let clip_min = Vec2::new(
                                f32::max(0.0, (cmd_params.clip_rect[0] - clip_off.x) * clip_scale.x),
                                f32::max(0.0, (cmd_params.clip_rect[1] - clip_off.y) * clip_scale.y),
                            );
                            let clip_max = Vec2::new(
                                f32::min(framebuffer_width, (cmd_params.clip_rect[2] - clip_off.x) * clip_scale.x),
                                f32::min(framebuffer_height, (cmd_params.clip_rect[3] - clip_off.y) * clip_scale.y),
                            );

                            if clip_max.x <= clip_min.x || clip_max.y <= clip_min.y {
                                continue;
                            }

                            let scissors =
                                [vk::Rect2D::default().offset(vk::Offset2D::default().x(clip_min.x as i32).y(clip_min.y as i32)).extent(
                                    vk::Extent2D::default()
                                        .width((clip_max.x - clip_min.x) as u32)
                                        .height((clip_max.y - clip_min.y) as u32),
                                )];
                            let descriptor_sets = [vk::DescriptorSet::from_raw(cmd_params.texture_id.id() as u64)];
                            unsafe {
                                api.device.handle.cmd_set_scissor(command_buffer, 0, &scissors);
                                api.device.handle.cmd_bind_descriptor_sets(
                                    command_buffer,
                                    vk::PipelineBindPoint::GRAPHICS,
                                    self.pipeline_layout,
                                    0,
                                    &descriptor_sets,
                                    &[],
                                );
                                api.device.handle.cmd_draw_indexed(
                                    command_buffer,
                                    count as u32,
                                    1,
                                    (cmd_params.idx_offset + idx_offset) as u32,
                                    (cmd_params.vtx_offset + vtx_offset) as i32,
                                    0,
                                );
                            };
                        }
                        imgui::DrawCmd::ResetRenderState => {
                            setup_render_state(
                                api,
                                draw_data,
                                index_buffer.handle,
                                vertex_bufer.handle,
                                self.pipeline_layout,
                                self.pipeline,
                                command_buffer,
                                framebuffer_width,
                                framebuffer_height,
                            );
                        }
                        imgui::DrawCmd::RawCallback { callback, raw_cmd } => {
                            unsafe { callback(draw_list_raw, raw_cmd) };
                        }
                    }
                }

                idx_offset = idx_offset + draw_list_raw.IdxBuffer.Size as usize;
                vtx_offset = vtx_offset + draw_list_raw.VtxBuffer.Size as usize;
            }
        }

        self.viewport_resources.push((index_buffer, index_buffer_allocation, vertex_bufer, vertex_buffer_allocation));
    }

    pub fn drop(&mut self, api: &mut super::Renderer) {
        let device_handle = &mut api.device.handle;
        unsafe {
            device_handle.device_wait_idle().expect("koi::ren::vk::imgui - failed to Wait for Device Idle");
            device_handle.destroy_image(self.atlas.handle, None);
            let atlas_allocation = self.atlas_allocation.pop_front().unwrap();
            api.allocator.handle.free(atlas_allocation).expect("imgui::ren::vk::imgui - failed to Free ImGui Font Atlas Image");
            self.descriptor_set_allocator.drop(&device_handle);
            device_handle.destroy_pipeline(self.pipeline, None);
            device_handle.destroy_shader_module(self.shader_module, None);
            device_handle.destroy_pipeline_layout(self.pipeline_layout, None);
            device_handle.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            device_handle.destroy_sampler(self.texture_sampler, None);
        }
    }
}

fn initialize_vulkan_structures(
    api: &mut super::Renderer,
) -> (vk::Sampler, vk::DescriptorSetLayout, vk::DescriptorSet, DescriptorSetAllocator, vk::PipelineLayout, vk::Pipeline, vk::ShaderModule) {
    // Texture Sampler
    let create_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .min_lod(-1000f32)
        .max_lod(1000f32)
        .max_anisotropy(-1f32);

    let texture_sampler = unsafe {
        api.device.handle.create_sampler(&create_info, None).expect("koi::ren::vk::imgui - failed to create ImGui Texture Sampler")
    };

    // Descriptor Set Layout
    let descriptor_set_layout = unsafe {
        api.device
            .handle
            .create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default().bindings(&[vk::DescriptorSetLayoutBinding::default()
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .binding(0)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT)]),
                None,
            )
            .expect("koi::ren::vk::imgui - failed to create ImGui Descriptor Set Layout")
    };

    let pool_ratios = [
        DescriptorSetPoolSizeRatio::new(vk::DescriptorType::SAMPLER),
        DescriptorSetPoolSizeRatio::new(vk::DescriptorType::COMBINED_IMAGE_SAMPLER),
        DescriptorSetPoolSizeRatio::new(vk::DescriptorType::SAMPLED_IMAGE),
        DescriptorSetPoolSizeRatio::new(vk::DescriptorType::STORAGE_IMAGE),
        DescriptorSetPoolSizeRatio::new(vk::DescriptorType::UNIFORM_TEXEL_BUFFER),
        DescriptorSetPoolSizeRatio::new(vk::DescriptorType::STORAGE_TEXEL_BUFFER),
        DescriptorSetPoolSizeRatio::new(vk::DescriptorType::UNIFORM_BUFFER),
        DescriptorSetPoolSizeRatio::new(vk::DescriptorType::STORAGE_BUFFER),
        DescriptorSetPoolSizeRatio::new(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC),
        DescriptorSetPoolSizeRatio::new(vk::DescriptorType::STORAGE_BUFFER_DYNAMIC),
        DescriptorSetPoolSizeRatio::new(vk::DescriptorType::INPUT_ATTACHMENT),
    ];

    let mut descriptor_set_allocator = DescriptorSetAllocator::new(
        &api.device.handle,
        IMGUI_DEFAULT_DESCRIPTOR_COUNT,
        &pool_ratios,
        Some(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET),
    );
    let descriptor_set = descriptor_set_allocator.allocate(&api.device.handle, &[descriptor_set_layout]);

    // Pipeline Layout
    let set_layouts = [descriptor_set_layout];
    let push_constant_ranges =
        [vk::PushConstantRange::default().stage_flags(vk::ShaderStageFlags::VERTEX).offset(0).size(F32_SIZE as u32 * 4)];
    let pipeline_layout = pipeline::create_pipeline_layout(&api.device.handle, &set_layouts, Some(&push_constant_ranges));

    // Pipeline
    let shader_module = pipeline::load_shader_module(&api.device.handle, include_bytes!(env!("imgui.spv")), None);
    let pipeline = pipeline::PipelineBuilder::default()
        .pipeline_layout(pipeline_layout)
        .shaders(shader_module, None)
        .input_topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .vertex_input_state(
            &[vk::VertexInputBindingDescription::default()
                .stride(size_of::<imgui::DrawVert>() as u32)
                .input_rate(vk::VertexInputRate::VERTEX)],
            &[
                vk::VertexInputAttributeDescription::default()
                    .location(0)
                    .binding(0)
                    .format(vk::Format::R32G32_SFLOAT)
                    .offset(std::mem::offset_of!(imgui::DrawVert, pos) as u32),
                vk::VertexInputAttributeDescription::default()
                    .location(1)
                    .binding(0)
                    .format(vk::Format::R32G32_SFLOAT)
                    .offset(std::mem::offset_of!(imgui::DrawVert, uv) as u32),
                vk::VertexInputAttributeDescription::default()
                    .location(2)
                    .binding(0)
                    .format(vk::Format::R8G8B8A8_UNORM)
                    .offset(std::mem::offset_of!(imgui::DrawVert, col) as u32),
            ],
        )
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE, vk::FrontFace::COUNTER_CLOCKWISE)
        .multisampling()
        .color_attachment_formats(&[api.swapchain.format])
        .blending(
            true,
            vk::BlendFactor::SRC_ALPHA,
            vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            vk::BlendOp::ADD,
            vk::BlendFactor::ONE,
            vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            vk::BlendOp::ADD,
            vk::ColorComponentFlags::RGBA,
        )
        .build(&api.device.handle);

    (texture_sampler, descriptor_set_layout, descriptor_set, descriptor_set_allocator, pipeline_layout, pipeline, shader_module)
}

fn create_font_atlas(
    context: &mut imgui::Context,
    api: &mut super::Renderer,
    descriptor_set: vk::DescriptorSet,
    sampler: vk::Sampler,
) -> (Image, vka::Allocation) {
    // Upload command buffer
    let create_info =
        vk::CommandPoolCreateInfo::default().queue_family_index(api.device.queue_families.get_family_index(QueueFamilyType::Graphics));

    let command_pool = unsafe {
        api.device.handle.create_command_pool(&create_info, None).expect("koi::ren::vk::imgui - failed to allocate ImGui Font Command Pool")
    };

    let allocate_info =
        vk::CommandBufferAllocateInfo::default().command_pool(command_pool).command_buffer_count(1).level(vk::CommandBufferLevel::PRIMARY);
    let command_buffer = unsafe {
        api.device
            .handle
            .allocate_command_buffers(&allocate_info)
            .expect("koi::ren::vk::imgui - failed to allocate ImGui Font Command Buffer")[0]
    };

    let font_atlas = context.fonts().build_rgba32_texture();

    // dst Image
    let (image, image_allocation) = super::image::Image::create(
        &api.device.handle,
        &mut api.allocator.handle,
        vk::Format::R8G8B8A8_UNORM,
        vk::Extent3D::default().width(font_atlas.width).height(font_atlas.height).depth(1),
        vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
        vk::ImageAspectFlags::COLOR,
        vk::ImageLayout::UNDEFINED,
    );

    // update descriptor set
    let image_info =
        [vk::DescriptorImageInfo::default().sampler(sampler).image_layout(vk::ImageLayout::READ_ONLY_OPTIMAL).image_view(image.view)];
    let descriptor_writes = [vk::WriteDescriptorSet::default()
        .dst_binding(0)
        .dst_set(descriptor_set)
        .descriptor_count(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(&image_info)];
    unsafe { api.device.handle.update_descriptor_sets(&descriptor_writes, &[]) };

    // src buffer
    let upload_size = (font_atlas.width * font_atlas.height * 4) as u64 * size_of::<u8>() as u64;
    let (mut upload_buffer, mut upload_buffer_allocation) = Buffer::create(
        &api.device.handle,
        &mut api.allocator.handle,
        upload_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        "imgui_font_atlas_upload_buffer",
        MemoryLocation::CpuToGpu,
    );

    upload_buffer.upload(font_atlas.data, &mut upload_buffer_allocation, 0);

    let begin_info = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe {
        api.device.handle.begin_command_buffer(command_buffer, &begin_info).expect("koi::ren::vk::imgui - failed to Begin Command Buffer")
    };

    // transition image for dst
    let image_memory_barriers = [vk::ImageMemoryBarrier::default()
        .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .old_layout(vk::ImageLayout::UNDEFINED)
        .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image.handle)
        .subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).level_count(1).layer_count(1))];
    unsafe {
        api.device.handle.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::HOST,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &image_memory_barriers,
        )
    };

    // perform buffer copy to image
    let regions = [vk::BufferImageCopy::default()
        .image_subresource(vk::ImageSubresourceLayers::default().aspect_mask(vk::ImageAspectFlags::COLOR).layer_count(1))
        .image_extent(vk::Extent3D::default().width(font_atlas.width).height(font_atlas.height).depth(1))];
    unsafe {
        api.device.handle.cmd_copy_buffer_to_image(
            command_buffer,
            upload_buffer.handle,
            image.handle,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &regions,
        )
    };

    let image_memory_barriers = [vk::ImageMemoryBarrier::default()
        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .dst_access_mask(vk::AccessFlags::SHADER_READ)
        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image.handle)
        .subresource_range(vk::ImageSubresourceRange::default().aspect_mask(vk::ImageAspectFlags::COLOR).level_count(1).layer_count(1))];

    unsafe {
        api.device.handle.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &image_memory_barriers,
        )
    };

    let command_buffers = [command_buffer];
    let submits = [vk::SubmitInfo::default().command_buffers(&command_buffers)];

    // set ImGui font atlas ID
    let tex_id = descriptor_set.as_raw() as usize;
    context.fonts().tex_id = tex_id.into();

    // done; free oneshot resources
    let device_handle = &api.device.handle;
    let allocator = &mut api.allocator.handle;

    unsafe {
        device_handle.end_command_buffer(command_buffer).expect("koi::ren::vk::imgui - failed to End Command Buffer");

        device_handle
            .queue_submit(api.graphics_queue, &submits, vk::Fence::null())
            .expect("koi::ren::vk::imgui - failed to Submit commands to Queue");

        device_handle.queue_wait_idle(api.graphics_queue).expect("koi::ren::vk::imgui - failed to Wait for Queue Idle");

        device_handle.destroy_buffer(upload_buffer.handle, None);

        allocator.free(upload_buffer_allocation).expect("koi::ren::vk::imgui - failed to Free ImGui Font Upload Buffer");
    };

    (image, image_allocation)
}

fn setup_render_state(
    api: &mut super::Renderer,
    draw_data: &DrawData,
    index_buffer: vk::Buffer,
    vertex_buffer: vk::Buffer,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    command_buffer: vk::CommandBuffer,
    frambuffer_width: f32,
    framebuffer_height: f32,
) {
    unsafe { api.device.handle.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline) };

    if draw_data.total_vtx_count > 0 {
        unsafe {
            api.device.handle.cmd_bind_index_buffer(
                command_buffer,
                index_buffer,
                0,
                if IDX_SIZE == 2 { vk::IndexType::UINT16 } else { vk::IndexType::UINT32 },
            );
            let buffers = [vertex_buffer];
            let offsets = [0];
            api.device.handle.cmd_bind_vertex_buffers(command_buffer, 0, &buffers, &offsets);
        }
    }

    let viewports =
        [vk::Viewport::default().x(0.0).y(0.0).width(frambuffer_width).height(framebuffer_height).min_depth(0.0).max_depth(1.0)];
    unsafe { api.device.handle.cmd_set_viewport(command_buffer, 0, &viewports) };

    let scale = Vec2::new(2.0 / draw_data.display_size[0], 2.0 / draw_data.display_size[1]);
    let translate = Vec2::new(-1.0 - draw_data.display_pos[0] * scale[0], -1.0 - draw_data.display_pos[1] * scale[1]);

    unsafe {
        api.device.handle.cmd_push_constants(
            command_buffer,
            pipeline_layout,
            vk::ShaderStageFlags::VERTEX,
            F32_SIZE as u32 * 0,
            cast_slice::<f32, u8>(&scale.to_array()),
        );
        api.device.handle.cmd_push_constants(
            command_buffer,
            pipeline_layout,
            vk::ShaderStageFlags::VERTEX,
            F32_SIZE as u32 * 2,
            cast_slice::<f32, u8>(&translate.to_array()),
        );
    }
}
