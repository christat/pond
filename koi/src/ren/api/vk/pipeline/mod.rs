use ash::{Device as DeviceHandle, vk};

#[derive(Default)]
pub struct PipelineBuilder<'a> {
    pub shader_stages: Vec<vk::PipelineShaderStageCreateInfo<'a>>,
    pub input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo<'a>,
    pub rasterization_state: vk::PipelineRasterizationStateCreateInfo<'a>,
    pub color_blend_attachment: vk::PipelineColorBlendAttachmentState,
    pub multisample_state: vk::PipelineMultisampleStateCreateInfo<'a>,
    pub pipeline_layout: vk::PipelineLayout,
    pub depth_stencil_state: vk::PipelineDepthStencilStateCreateInfo<'a>,
    pub rendering: vk::PipelineRenderingCreateInfo<'a>,
    pub color_attachment_formats: Vec<vk::Format>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn clear(mut self) -> Self {
        self.shader_stages.clear();
        self.input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default();
        self.rasterization_state = vk::PipelineRasterizationStateCreateInfo::default();
        self.color_blend_attachment = vk::PipelineColorBlendAttachmentState::default();
        self.multisample_state = vk::PipelineMultisampleStateCreateInfo::default();
        self.pipeline_layout = vk::PipelineLayout::default();
        self.depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::default();
        self.rendering = vk::PipelineRenderingCreateInfo::default();
        self.color_attachment_formats = vec![vk::Format::UNDEFINED];
        self
    }

    pub fn pipeline_layout(mut self, layout: vk::PipelineLayout) -> Self {
        self.pipeline_layout = layout;
        self
    }

    pub fn shaders(mut self, vertex: vk::ShaderModule, fragment: Option<vk::ShaderModule>) -> Self {
        self.shader_stages.clear();
        self.shader_stages.push(
            vk::PipelineShaderStageCreateInfo::default()
                .module(vertex)
                .stage(vk::ShaderStageFlags::VERTEX)
                .name(c"main_vs"),
        );
        self.shader_stages.push(
            vk::PipelineShaderStageCreateInfo::default()
                .module(fragment.unwrap_or(vertex))
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .name(c"main_fs"),
        );
        self
    }

    pub fn input_topology(mut self, topology: vk::PrimitiveTopology) -> Self {
        self.input_assembly_state = self
            .input_assembly_state
            .primitive_restart_enable(false)
            .topology(topology);
        self
    }

    pub fn polygon_mode(mut self, polygon_mode: vk::PolygonMode) -> Self {
        self.rasterization_state = self
            .rasterization_state
            .line_width(1.0)
            .polygon_mode(polygon_mode);
        self
    }

    pub fn cull_mode(mut self, cull_mode: vk::CullModeFlags, front_face: vk::FrontFace) -> Self {
        self.rasterization_state = self
            .rasterization_state
            .cull_mode(cull_mode)
            .front_face(front_face);
        self
    }

    pub fn multisampling(mut self) -> Self {
        self.multisample_state = self
            .multisample_state
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0)
            .sample_mask(&[])
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);
        self
    }

    pub fn blending_disabled(mut self) -> Self {
        self.color_blend_attachment = self
            .color_blend_attachment
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false);
        self
    }

    pub fn blending_additive(mut self) -> Self {
        self.color_blend_attachment = self
            .color_blend_attachment
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);
        self
    }

    pub fn blending_alpha_blend(mut self) -> Self {
        self.color_blend_attachment = self
            .color_blend_attachment
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);
        self
    }

    pub fn color_attachment_formats(mut self, formats: &'a [vk::Format]) -> Self {
        self.color_attachment_formats = formats.to_owned();
        self.rendering = self.rendering.color_attachment_formats(formats);
        self
    }

    pub fn depth_attachment_format(mut self, format: vk::Format) -> Self {
        self.rendering = self.rendering.depth_attachment_format(format);
        self
    }

    pub fn depth_stencil_state(
        mut self,
        depth_write_enable: bool,
        depth_compare_op: vk::CompareOp,
    ) -> Self {
        self.depth_stencil_state = self
            .depth_stencil_state
            .depth_test_enable(true)
            .depth_write_enable(depth_write_enable)
            .depth_compare_op(depth_compare_op)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .front(vk::StencilOpState::default())
            .back(vk::StencilOpState::default())
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0);
        self
    }

    pub fn build(&mut self, device_handle: &DeviceHandle) -> vk::Pipeline {
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let color_blend_attachments = [self.color_blend_attachment];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachments);

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default();

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let create_infos = [vk::GraphicsPipelineCreateInfo::default()
            .push_next(&mut self.rendering)
            .stages(&self.shader_stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&self.input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&self.rasterization_state)
            .multisample_state(&self.multisample_state)
            .color_blend_state(&color_blend_state)
            .depth_stencil_state(&self.depth_stencil_state)
            .layout(self.pipeline_layout)
            .dynamic_state(&dynamic_state)];

        unsafe {
            device_handle
                .create_graphics_pipelines(vk::PipelineCache::null(), &create_infos, None)
                .expect("koi::ren::vk::pipeline - failed to Create Graphics Pipelines")[0]
        }
    }
}

pub fn load_shader_module(
    device_handle: &DeviceHandle,
    shader: &[u8],
    flags: Option<vk::ShaderModuleCreateFlags>,
) -> vk::ShaderModule {
    let (_, code, _) = unsafe { shader.align_to::<u32>() };
    let create_info = vk::ShaderModuleCreateInfo::default()
        .code(code)
        .flags(flags.unwrap_or_default());

    unsafe {
        device_handle
            .create_shader_module(&create_info, None)
            .expect("koi::ren::vk::pipeline - failed to create shader module")
    }
}

pub fn create_pipeline_layout(
    device_handle: &DeviceHandle,
    set_layouts: &[vk::DescriptorSetLayout],
    push_constant_ranges: Option<&[vk::PushConstantRange]>,
) -> vk::PipelineLayout {
    let create_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(set_layouts)
        .push_constant_ranges(push_constant_ranges.unwrap_or_default());

    unsafe {
        device_handle
            .create_pipeline_layout(&create_info, None)
            .expect("koi::ren::vk::pipeline - failed to create pipeline layout")
    }
}

pub fn create_compute_pipeline(
    device_handle: &DeviceHandle,
    shader_module: vk::ShaderModule,
    layout: vk::PipelineLayout,
) -> vk::Pipeline {
    let stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::COMPUTE)
        .name(c"main_cs")
        .module(shader_module);

    let create_infos = [vk::ComputePipelineCreateInfo::default()
        .layout(layout)
        .stage(stage)];

    unsafe {
        device_handle
            .create_compute_pipelines(vk::PipelineCache::null(), &create_infos, None)
            .expect("koi::ren::vk::pipeline - failed to create compute pipeline")[0]
    }
}

pub fn get_attachment_info<'a>(
    image_view: vk::ImageView,
    image_layout: vk::ImageLayout,
    clear_value: Option<vk::ClearValue>,
) -> vk::RenderingAttachmentInfo<'a> {
    vk::RenderingAttachmentInfo::default()
        .image_view(image_view)
        .image_layout(image_layout)
        .load_op(if clear_value.is_some() {
            vk::AttachmentLoadOp::CLEAR
        } else {
            vk::AttachmentLoadOp::LOAD
        })
        .store_op(vk::AttachmentStoreOp::STORE)
        .clear_value(clear_value.unwrap_or_default())
}

pub fn get_rendering_info<'a>(
    extent: vk::Extent2D,
    color_attachments: &'a [vk::RenderingAttachmentInfo<'a>],
    depth_attachment: Option<&'a vk::RenderingAttachmentInfo<'a>>,
) -> vk::RenderingInfo<'a> {
    let mut info = vk::RenderingInfo::default()
        .render_area(vk::Rect2D::default().extent(extent))
        .layer_count(1)
        .color_attachments(color_attachments);
    if depth_attachment.is_some() {
        info = info.depth_attachment(depth_attachment.unwrap())
    };

    info
}
