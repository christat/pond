use super::device::{config::QueueFamilyType, Device};

use ash::vk;

pub struct Frame {
    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,

    pub swapchain_semaphore: vk::Semaphore,
    pub render_semaphore: vk::Semaphore,
    pub render_fence: vk::Fence,
}

impl Frame {
    pub fn new(device: &Device) -> Self {
        let pool_create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(device.queue_families.get_family_index(QueueFamilyType::Graphics));

        let command_pool = unsafe { device.handle.create_command_pool(&pool_create_info, None).expect("koi::ren::vk::Frame - failed to create Command Pool") };

        let buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .command_buffer_count(1)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe { device.handle.allocate_command_buffers(&buffer_allocate_info).expect("koi::ren::vk::Frame - failed to allocate Command Buffer") };

        Self { 
            command_pool,
            command_buffer: command_buffers.first().unwrap().clone(),
            swapchain_semaphore: create_semaphore(device, None),
            render_semaphore: create_semaphore(device, None),
            render_fence: create_fence(device, Some(vk::FenceCreateFlags::SIGNALED)),
        }
    }

    pub fn drop(&mut self, device: &Device) {
        unsafe { 
            device.handle.device_wait_idle().expect("koi::ren::vk::Frame - failed to Wait for Device Idle");
            device.handle.destroy_command_pool(self.command_pool, None);
            device.handle.destroy_fence(self.render_fence, None);
            device.handle.destroy_semaphore(self.render_semaphore, None);
            device.handle.destroy_semaphore(self.swapchain_semaphore, None);
        }

    }
}

fn create_semaphore(device: &Device, flags: Option<vk::SemaphoreCreateFlags>) -> vk::Semaphore {
    let mut create_info = vk::SemaphoreCreateInfo::default();
    if flags.is_some() { create_info = create_info.flags(flags.unwrap()); };

    unsafe{ device.handle.create_semaphore(&create_info, None).expect("koi::ren::vk::Frame - failed to reate Semaphore") }
}

fn create_fence(device: &Device, flags: Option<vk::FenceCreateFlags>) -> vk::Fence {
    let mut create_info = vk::FenceCreateInfo::default();
    if flags.is_some() { create_info = create_info.flags(flags.unwrap()); };

    unsafe{ device.handle.create_fence(&create_info, None).expect("koi::ren::vk::Frame - failed to reate Fence") }
}

pub fn transition_image(device: &Device, command_buffer: vk::CommandBuffer, image: vk::Image, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout) {
    let subresource_range = get_image_subresource_range(if new_layout == vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL { vk::ImageAspectFlags::DEPTH } else { vk::ImageAspectFlags::COLOR });
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

    unsafe{ device.handle.cmd_pipeline_barrier2(command_buffer, &dependency_info) };
}

pub fn get_image_subresource_range(aspect_mask: vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange::default()
        .aspect_mask(aspect_mask)
        .base_mip_level(0)
        .level_count(vk::REMAINING_MIP_LEVELS)
        .base_array_layer(0)
        .layer_count(vk::REMAINING_ARRAY_LAYERS)
}

pub fn get_submit_info<'a>(command_buffer_infos: &'a [vk::CommandBufferSubmitInfo<'a>], wait_semaphore_infos: Option<&'a [vk::SemaphoreSubmitInfo]>, signal_semaphore_infos: Option<&'a [vk::SemaphoreSubmitInfo]>) -> vk::SubmitInfo2<'a> {
    let mut submit_info = vk::SubmitInfo2::default()
        .command_buffer_infos(command_buffer_infos);

    if wait_semaphore_infos.is_some() { submit_info = submit_info.wait_semaphore_infos(wait_semaphore_infos.unwrap()); };
    if signal_semaphore_infos.is_some() { submit_info = submit_info.signal_semaphore_infos(signal_semaphore_infos.unwrap()); };

    submit_info
}