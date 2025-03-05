use ash::{Device as DeviceHandle, vk};

pub const GRADIENT_SHADER: &[u8] = include_bytes!(env!("gradient.spv"));

pub fn load_shader_module(device_handle: &DeviceHandle, shader: &[u8], flags: Option<vk::ShaderModuleCreateFlags>) -> vk::ShaderModule {
    let create_info = vk::ShaderModuleCreateInfo::default()
        .code(shader)
        .flags(flags.unwrap_or_default());

    unsafe { device_handle.create_shader_module(&create_info, None).expect("koi::ren::vk::pipeline - failed to create shader module") }
}

pub fn create_pipeline_layout(device_handle: &DeviceHandle, set_layouts: &[vk::DescriptorSetLayout]) -> vk::PipelineLayout {
    let create_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(set_layouts);

    unsafe { device_handle.create_pipeline_layout(&create_info, None).expect("koi::ren::vk::pipeline - failed to create pipeline layout") }
}

pub fn create_compute_pipeline(device_handle: &DeviceHandle, shader_module: vk::ShaderModule, layout: vk::PipelineLayout, cache: vk::PipelineCache) -> Vec<vk::Pipeline> {
    let stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::COMPUTE)
        .module(shader_module);

    let create_infos = [
        vk::ComputePipelineCreateInfo::default()
            .layout(layout)
            .stage(stage_info)
    ];

    unsafe{ device_handle.create_compute_pipelines(cache, &create_infos, None).expect("koi::ren::vk::pipeline - failed to create compute pipeline") }
}