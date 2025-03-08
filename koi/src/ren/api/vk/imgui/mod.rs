use crate::{imgui::ImGuiRendering, ren::api::vk::{pipeline, image::Image, device::config::QueueFamilyType}};

use ash::{vk::{self, Handle}, Device as DeviceHandle};
use gpu_allocator::{vulkan as vka, MemoryLocation};

// NB! Reference implementation shamelessly stolen from:
// https://github.com/ocornut/imgui/blob/master/examples/example_win32_vulkan/main.cpp
pub struct Renderer {
    pub queue: vk::Queue,

    pub texture_sampler: vk::Sampler,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub shader_module: vk::ShaderModule,

    pub image_descriptor_pool: vk::DescriptorPool,
    pub image_descriptor_set: vk::DescriptorSet,
    pub image: Image,
}

const IMGUI_DEFAULT_DESCRIPTOR_COUNT: u32 = 1000;

impl Renderer {
    pub fn new(context: &mut imgui::Context, api: &mut super::Renderer) -> Self {
        let queue= api.graphics_queue;

        let (
            texture_sampler,
            descriptor_set_layout,
            descriptor_pool,
            pipeline_layout,
            pipeline,
            shader_module,
        ) = initialize_vulkan_structures(api);

        let set_layouts = [descriptor_set_layout];
        let ( image_descriptor_pool, image_descriptor_set, image ) = create_fonts_texture(context, api, descriptor_pool, &set_layouts, texture_sampler);
       
        Self {
            queue,
            texture_sampler,
            descriptor_set_layout,
            descriptor_pool,
            pipeline_layout,
            pipeline,
            shader_module,
            image_descriptor_pool,
            image_descriptor_set,
            image,
        }
    }

    fn drop(&mut self, device_handle: &DeviceHandle) {
        unsafe {
            device_handle.destroy_descriptor_pool(self.image_descriptor_pool, None);
            device_handle.destroy_pipeline(self.pipeline, None);
            device_handle.destroy_shader_module(self.shader_module, None);
            device_handle.destroy_pipeline_layout(self.pipeline_layout, None);
            device_handle.destroy_descriptor_pool(self.descriptor_pool, None);
            device_handle.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            device_handle.destroy_sampler(self.texture_sampler, None);
        }
    }
}

fn initialize_vulkan_structures(api: &mut super::Renderer) -> 
(
    vk::Sampler,
    vk::DescriptorSetLayout,
    vk::DescriptorPool,
    vk::PipelineLayout,
    vk::Pipeline,
    vk::ShaderModule,
) {
     // Texture Sampler
     let create_info= vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .min_lod(-1000f32)
        .max_lod(1000f32)
        .max_anisotropy(-1f32);

    let texture_sampler = unsafe { api.device.handle.create_sampler(&create_info, None).expect("koi::ren::vk::imgui - failed to create ImGui Texture Sampler") };
    
    // Descriptor Set Layout
    let bindings = [
        vk::DescriptorSetLayoutBinding::default()
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
    ];
    let create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);

    let descriptor_set_layout = unsafe { api.device.handle.create_descriptor_set_layout(&create_info, None).expect("koi::ren::vk::imgui - failed to create ImGui Descriptor Set Layout") };
    
    // Descriptor Pool
    let pool_sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::SAMPLER)
            .descriptor_count(IMGUI_DEFAULT_DESCRIPTOR_COUNT),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(IMGUI_DEFAULT_DESCRIPTOR_COUNT),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::SAMPLED_IMAGE)
            .descriptor_count(IMGUI_DEFAULT_DESCRIPTOR_COUNT),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_IMAGE)
            .descriptor_count(IMGUI_DEFAULT_DESCRIPTOR_COUNT),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_TEXEL_BUFFER)
            .descriptor_count(IMGUI_DEFAULT_DESCRIPTOR_COUNT),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_TEXEL_BUFFER)
            .descriptor_count(IMGUI_DEFAULT_DESCRIPTOR_COUNT),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(IMGUI_DEFAULT_DESCRIPTOR_COUNT),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(IMGUI_DEFAULT_DESCRIPTOR_COUNT),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
            .descriptor_count(IMGUI_DEFAULT_DESCRIPTOR_COUNT),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER_DYNAMIC)
            .descriptor_count(IMGUI_DEFAULT_DESCRIPTOR_COUNT),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::INPUT_ATTACHMENT)
            .descriptor_count(IMGUI_DEFAULT_DESCRIPTOR_COUNT),
    ];

    let create_info = vk::DescriptorPoolCreateInfo::default()
        .max_sets(IMGUI_DEFAULT_DESCRIPTOR_COUNT)
        .pool_sizes(&pool_sizes);

    let descriptor_pool = unsafe { api.device.handle.create_descriptor_pool(&create_info, None).expect("koi::ren::vk::imgui - failed to create ImGui Descriptor Pool") };

    // Pipeline Layout
    let set_layouts = [descriptor_set_layout];
    let push_constant_ranges = [
        vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(size_of::<f32>() as u32 * 4)
    ];
    let pipeline_layout = pipeline::create_pipeline_layout(&api.device.handle, &set_layouts, Some(&push_constant_ranges));

    // Pipeline
    let shader_module = pipeline::load_shader_module(&api.device.handle, include_bytes!(env!("imgui.spv")), None);
    
    let stages= [
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .name(c"main_vs")
            .module(shader_module),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .name(c"main_fs")
            .module(shader_module)
    ];

    let vertex_binding_descriptions = [
        vk::VertexInputBindingDescription::default()
            .stride(size_of::<imgui::DrawVert>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    ];

    let vertex_attributes_descriptions = [
        vk::VertexInputAttributeDescription::default()
            .location(0)
            .binding(vertex_binding_descriptions[0].binding)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(std::mem::offset_of!(imgui::DrawVert, pos) as u32),
        vk::VertexInputAttributeDescription::default()
            .location(1)
            .binding(vertex_binding_descriptions[0].binding)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(std::mem::offset_of!(imgui::DrawVert, uv) as u32),
        vk::VertexInputAttributeDescription::default()
            .location(1)
            .binding(vertex_binding_descriptions[0].binding)
            .format(vk::Format::R8G8B8A8_UNORM)
            .offset(std::mem::offset_of!(imgui::DrawVert, uv) as u32),
    ];

    let vertex_input_state: vk::PipelineVertexInputStateCreateInfo<'_> = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(&vertex_binding_descriptions)
        .vertex_attribute_descriptions(&vertex_attributes_descriptions);

    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);

    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .line_width(1f32);

    let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);

    let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::default();

    let color_blend_attachments = [
        vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::RGBA)
    ];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default().attachments(&color_blend_attachments);

    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

    let formats = [api.swapchain.format];
    let mut dynamic_rendering = vk::PipelineRenderingCreateInfo::default()
        .color_attachment_formats(&formats);

    let create_infos = [
        vk::GraphicsPipelineCreateInfo::default()
            .flags(vk::PipelineCreateFlags::default())
            .stages(&stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .subpass(0)
            .push_next(&mut dynamic_rendering)
    ];

    let pipeline = unsafe { api.device.handle.create_graphics_pipelines(vk::PipelineCache::null(), &create_infos, None).expect("koi::ren::vk::imgui - failed to create ImGui Graphics Pipeline")[0] };

    (
        texture_sampler,
        descriptor_set_layout,
        descriptor_pool,
        pipeline_layout,
        pipeline,
        shader_module,
    )
}

fn create_fonts_texture(context: &mut imgui::Context, api: &mut super::Renderer, descriptor_pool: vk::DescriptorPool, set_layouts: &[vk::DescriptorSetLayout], sampler: vk::Sampler) -> (
    vk::DescriptorPool,
    vk::DescriptorSet,
    super::image::Image
) {
    let create_info = vk::CommandPoolCreateInfo::default()
        .queue_family_index(api.device.queue_families.get_family_index(QueueFamilyType::Graphics));

    let command_pool = unsafe { api.device.handle.create_command_pool(&create_info, None).expect("koi::ren::vk::imgui - failed to allocate ImGui Font Command Pool") };

    let allocate_info = vk::CommandBufferAllocateInfo::default().command_pool(command_pool);
    let command_buffer = unsafe { api.device.handle.allocate_command_buffers(&allocate_info).expect("koi::ren::vk::imgui - failed to allocate ImGui Font Command Buffer")[0] };

    let font_atlas = context.fonts().build_rgba32_texture();
    let image = super::image::Image::new(
        &api.device.handle, 
        &mut api.resource_allocator.handle, 
        &mut api.resource_allocator.global_resources,
        vk::Format::R8G8B8A8_UNORM, 
        vk::Extent3D::default().width(font_atlas.width).height(font_atlas.height).depth(1),
        vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
        vk::ImageAspectFlags::COLOR
    );

    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(descriptor_pool)
        .set_layouts(set_layouts);
    let descriptor_set = unsafe { api.device.handle.allocate_descriptor_sets(&allocate_info).expect("koi::ren::vk::imgui - failed to allocate ImGui Font Descriptor Set")[0] };

    let image_info = [
        vk::DescriptorImageInfo::default()
            .sampler(sampler)
            .image_layout(vk::ImageLayout::READ_ONLY_OPTIMAL)
            .image_view(image.view)
    ];
    let descriptor_writes = [
        vk::WriteDescriptorSet::default()
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_info)
    ];
    unsafe { api.device.handle.update_descriptor_sets(&descriptor_writes, &[]) };

    let upload_size = font_atlas.width as u64 * font_atlas.height as u64 * 4 * size_of::<char>() as u64; 
    let create_info = vk::BufferCreateInfo::default()
        .size(upload_size)
        .usage(vk::BufferUsageFlags::TRANSFER_SRC)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let upload_buffer = unsafe { api.device.handle.create_buffer(&create_info, None).expect("koi::ren::vk::imgui - failed to create ImGui Font Upload Buffer") };

    let requirements = unsafe { api.device.handle.get_buffer_memory_requirements(upload_buffer) };
    let buffer_allocation = api.resource_allocator.handle.allocate(&vka::AllocationCreateDesc {
        name: "ImGuiFont",
        requirements,
        location: MemoryLocation::CpuToGpu,
        linear: false,
        allocation_scheme: vka::AllocationScheme::GpuAllocatorManaged
    }).expect("koi::vk::Image - failed to allocate ImGui Font Upload Buffer");
    let memory = unsafe { buffer_allocation.memory() };

    // bind buffer memory, perform upload, unbind
    unsafe {
        api.device.handle.bind_buffer_memory(upload_buffer, memory, 0).expect("koi::ren::vk::imgui - failed to bind ImGui Font Upload Memory");
        let dst = api.device.handle.map_memory(memory, 0, upload_size, vk::MemoryMapFlags::empty()).expect("koi::ren::vk::imgui - failed to map ImGui Font Upload Memory");
        std::ptr::copy_nonoverlapping(font_atlas.data.as_ptr(), dst as *mut u8, upload_size as usize);
        let ranges = [
            vk::MappedMemoryRange::default()
                .memory(memory)
                .size(buffer_allocation.size())
        ];
        api.device.handle.flush_mapped_memory_ranges(&ranges).expect("koi::ren::vk::imgui - failed to flushed Mapped Memory Ranges for ImGui Font Upload Buffer");
        api.device.handle.unmap_memory(memory);
    }

    let begin_info = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe { api.device.handle.begin_command_buffer(command_buffer, &begin_info).expect("koi::ren::vk::imgui - failed to Begin Command Buffer") };

    let image_memory_barriers = [
        vk::ImageMemoryBarrier::default()
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .image(image.handle)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1)
            )
    ];
    unsafe { api.device.handle.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::HOST, vk::PipelineStageFlags::TRANSFER, vk::DependencyFlags::empty(), &[], &[], &image_memory_barriers) };

    let regions = [
            vk::BufferImageCopy::default()
                .image_subresource(
                    vk::ImageSubresourceLayers::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .layer_count(1)
                )
                .image_extent(
                    vk::Extent3D::default()
                        .width(font_atlas.width)
                        .height(font_atlas.height)
                        .depth(1)
                )
    ];
    unsafe { api.device.handle.cmd_copy_buffer_to_image(command_buffer, upload_buffer, image.handle, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &regions) };

    let image_memory_barriers = [
        vk::ImageMemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image(image.handle)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .level_count(1)
                    .layer_count(1)
            )
    ];
    unsafe { api.device.handle.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[], &[], &image_memory_barriers) };

    let command_buffers = [command_buffer];
    let submits = [vk::SubmitInfo::default().command_buffers(&command_buffers)];

    let tex_id = descriptor_set.as_raw() as usize;
    context.fonts().tex_id = tex_id.into();

    unsafe {
        api.device.handle.end_command_buffer(command_buffer).expect("koi::ren::vk::imgui - failed to End Command Buffer");
        api.device.handle.queue_submit(api.graphics_queue, &submits, vk::Fence::null()).expect("koi::ren::vk::imgui - failed to Submit commands to Queue");
        api.device.handle.destroy_buffer(upload_buffer, None);
        api.resource_allocator.handle.free(buffer_allocation).expect("koi::ren::vk::imgui - failed to Free ImGui Font Upload Buffer");
    };

    (
        descriptor_pool,
        descriptor_set,
        image
    )
}

impl ImGuiRendering for Renderer {
    fn draw(&mut self, draw_data: &imgui::DrawData) {
        
        
    }
}