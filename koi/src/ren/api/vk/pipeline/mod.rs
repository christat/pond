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

pub fn create_pipeline_layout(device_handle: &DeviceHandle, set_layouts: &[vk::DescriptorSetLayout]) -> vk::PipelineLayout {
    let create_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(set_layouts);

    unsafe { device_handle.create_pipeline_layout(&create_info, None).expect("koi::ren::vk::pipeline - failed to create pipeline layout") }
}

pub fn create_compute_pipeline(device_handle: &DeviceHandle, shader_module: vk::ShaderModule, layout: vk::PipelineLayout) -> Vec<vk::Pipeline> {
    let stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::COMPUTE)
        .name(c"main")
        .module(shader_module);

    let create_infos = [
        vk::ComputePipelineCreateInfo::default()
            .layout(layout)
            .stage(stage)
    ];

    unsafe{ device_handle.create_compute_pipelines(vk::PipelineCache::null(), &create_infos, None).expect("koi::ren::vk::pipeline - failed to create compute pipeline") }
}