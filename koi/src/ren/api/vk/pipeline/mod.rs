use ash::{Device as DeviceHandle, vk};

//pub const GRADIENT_SHADER: &[u8] = include_bytes!(env!("gradient.spv"));
pub const GRADIENT_SHADER: &[u8] = include_bytes!("../../../../../../resources/shaders/gradient.spv");

pub fn load_shader_module(device_handle: &DeviceHandle, shader: &[u8], flags: Option<vk::ShaderModuleCreateFlags>) -> vk::ShaderModule {
    let (_, code, _) = unsafe {  shader.align_to::<u32>() };
    let create_info = vk::ShaderModuleCreateInfo::default()
        .code(code)
        .flags(flags.unwrap_or_default());

    unsafe { device_handle.create_shader_module(&create_info, None).expect("koi::ren::vk::pipeline - failed to create shader module") }
}

pub fn create_pipeline_layout(device_handle: &DeviceHandle, set_layouts: &[vk::DescriptorSetLayout], push_constant_ranges: Option<&[vk::PushConstantRange]>) -> vk::PipelineLayout {
    let create_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(set_layouts)
        .push_constant_ranges(push_constant_ranges.unwrap_or_default());

    unsafe { device_handle.create_pipeline_layout(&create_info, None).expect("koi::ren::vk::pipeline - failed to create pipeline layout") }
}

pub fn create_compute_pipeline(device_handle: &DeviceHandle, shader_module: vk::ShaderModule, layout: vk::PipelineLayout) -> vk::Pipeline {
    let stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::COMPUTE)
        .name(c"main")
        .module(shader_module);

    let create_infos = [
        vk::ComputePipelineCreateInfo::default()
            .layout(layout)
            .stage(stage)
    ];

    unsafe{ device_handle.create_compute_pipelines(vk::PipelineCache::null(), &create_infos, None).expect("koi::ren::vk::pipeline - failed to create compute pipeline")[0] }
}

pub fn get_attachment_info<'a>(image_view: vk::ImageView, image_layout: vk::ImageLayout, clear_value: Option<vk::ClearValue>) -> vk::RenderingAttachmentInfo<'a> {
    vk::RenderingAttachmentInfo::default()
        .image_view(image_view)
        .image_layout(image_layout)
        .load_op(if clear_value.is_some() { vk::AttachmentLoadOp::CLEAR } else { vk::AttachmentLoadOp::LOAD })
        .store_op(vk::AttachmentStoreOp::STORE)
        .clear_value(clear_value.unwrap_or_default())
}

pub fn get_rendering_info<'a>(extent: vk::Extent2D, color_attachments: &'a [vk::RenderingAttachmentInfo<'a>], depth_attachment: Option<&'a vk::RenderingAttachmentInfo<'a>>) -> vk::RenderingInfo<'a> {
    let mut info = vk::RenderingInfo::default()
        .render_area(
            vk::Rect2D::default()
                .extent(extent)
        )
        .layer_count(1)
        .color_attachments(color_attachments);
    if depth_attachment.is_some() { info = info.depth_attachment(depth_attachment.unwrap()) };

    info
}