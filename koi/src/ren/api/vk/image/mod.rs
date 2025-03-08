use super::resource_allocator::AllocatedResources;

use ash::{vk, Device as DeviceHandle};
use gpu_allocator::{vulkan as vka, MemoryLocation};

#[allow(unused)]
pub struct Image {
    pub handle: vk::Image,
    pub view: vk::ImageView,
    pub extent_3d: vk::Extent3D,
    pub extent_2d: vk::Extent2D,
    pub format: vk::Format,
}

impl Image {
    pub fn new(
        device_handle: &DeviceHandle,
        allocator: &mut vka::Allocator,
        resources: &mut AllocatedResources,
        format: vk::Format,
        extent: vk::Extent3D,
        usage: vk::ImageUsageFlags,
        aspect_mask: vk::ImageAspectFlags
    ) -> Self {
        let image_create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(extent)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage);

        let image = unsafe { device_handle.create_image(&image_create_info, None).expect("koi::vk::Image - failed to create Image") };
        let requirements = unsafe { device_handle.get_image_memory_requirements(image) };

        let allocation = allocator.allocate(&vka::AllocationCreateDesc {
            name: "image",
            requirements,
            location: MemoryLocation::GpuOnly,
            linear: false,
            allocation_scheme: vka::AllocationScheme::GpuAllocatorManaged
        }).expect("koi::vk::Image - failed to allocate Image");

        unsafe { device_handle.bind_image_memory(image, allocation.memory(), allocation.offset()).expect("koi::vk::Image - failed to bind Image Memory") }

        let view_create_info = vk::ImageViewCreateInfo::default()
            .view_type(vk::ImageViewType::TYPE_2D)
            .image(image)
            .format(format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .aspect_mask(aspect_mask)
            );
        
        let view = unsafe { device_handle.create_image_view(&view_create_info, None).expect("koi::vk::Image - failed to create Image View") };

        resources.add_image(image, view, allocation);

        let extent_2d = vk::Extent2D::default()
            .width(extent.width)
            .height(extent.height);

        Self { handle: image, view, extent_3d: extent, extent_2d, format }
    }
}

pub fn get_subresource_range(aspect_mask: vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange::default()
        .aspect_mask(aspect_mask)
        .base_mip_level(0)
        .level_count(vk::REMAINING_MIP_LEVELS)
        .base_array_layer(0)
        .layer_count(vk::REMAINING_ARRAY_LAYERS)
}

pub fn transition(device_handle: &DeviceHandle, command_buffer: vk::CommandBuffer, image: vk::Image, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout) {
    let subresource_range = get_subresource_range(if new_layout == vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL { vk::ImageAspectFlags::DEPTH } else { vk::ImageAspectFlags::COLOR });
    let image_barriers = [
        vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
            .src_access_mask(vk::AccessFlags2::MEMORY_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
            .dst_access_mask(vk::AccessFlags2::MEMORY_WRITE | vk::AccessFlags2::MEMORY_READ)
            .old_layout(old_layout)
            .new_layout(new_layout)
            .subresource_range(subresource_range)
            .image(image)
    ];

    let dependency_info = vk::DependencyInfo::default()
        .image_memory_barriers(&image_barriers);

    unsafe{ device_handle.cmd_pipeline_barrier2(command_buffer, &dependency_info) };
}

pub fn copy(device_handle: &DeviceHandle, cmd: vk::CommandBuffer, src_image: vk::Image, dst_image: vk::Image, src_extent: vk::Extent2D, dst_extent: vk::Extent2D) {
    let regions = [
        vk::ImageBlit2::default()
            .src_offsets([vk::Offset3D::default(), vk::Offset3D::default().x(src_extent.width as i32).y(src_extent.height as i32).z(1)])
            .dst_offsets([vk::Offset3D::default(), vk::Offset3D::default().x(dst_extent.width as i32).y(dst_extent.height as i32).z(1)])
            .src_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_array_layer(0)
                    .layer_count(1)
                    .mip_level(0)
            )
            .dst_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_array_layer(0)
                    .layer_count(1)
                    .mip_level(0)
        )
    ];

    let blit_image_info = vk::BlitImageInfo2::default()
        .src_image(src_image)
        .src_image_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
        .dst_image(dst_image)
        .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .filter(vk::Filter::LINEAR)
        .regions(&regions);

    unsafe { device_handle.cmd_blit_image2(cmd, &blit_image_info) };
}