use super::device::{Device, config::QueueFamilyType};

use ash::{Device as DeviceHandle, vk};

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
            .queue_family_index(
                device
                    .queue_families
                    .get_family_index(QueueFamilyType::Graphics),
            );

        let command_pool = unsafe {
            device
                .handle
                .create_command_pool(&pool_create_info, None)
                .expect("koi::ren::vk::Frame - failed to create Command Pool")
        };

        let buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .command_buffer_count(1)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe {
            device
                .handle
                .allocate_command_buffers(&buffer_allocate_info)
                .expect("koi::ren::vk::Frame - failed to allocate Command Buffer")
        };

        Self {
            command_pool,
            command_buffer: command_buffers.first().unwrap().clone(),
            swapchain_semaphore: create_semaphore(&device.handle, None),
            render_semaphore: create_semaphore(&device.handle, None),
            render_fence: create_fence(&device.handle, Some(vk::FenceCreateFlags::SIGNALED)),
        }
    }

    pub fn generator(device: &Device, buffering: u32) -> Vec<Frame> {
        (0..buffering)
            .into_iter()
            .map(|_index| Frame::new(&device))
            .collect()
    }

    pub fn drop(&mut self, device: &DeviceHandle) {
        unsafe {
            device.destroy_command_pool(self.command_pool, None);
            device.destroy_fence(self.render_fence, None);
            device.destroy_semaphore(self.render_semaphore, None);
            device.destroy_semaphore(self.swapchain_semaphore, None);
        }
    }
}

fn create_semaphore(
    device_handle: &DeviceHandle,
    flags: Option<vk::SemaphoreCreateFlags>,
) -> vk::Semaphore {
    let create_info = vk::SemaphoreCreateInfo::default().flags(flags.unwrap_or_default());
    unsafe {
        device_handle
            .create_semaphore(&create_info, None)
            .expect("koi::ren::vk::Frame - failed to reate Semaphore")
    }
}

fn create_fence(device_handle: &DeviceHandle, flags: Option<vk::FenceCreateFlags>) -> vk::Fence {
    let mut create_info = vk::FenceCreateInfo::default();
    if flags.is_some() {
        create_info = create_info.flags(flags.unwrap());
    };

    unsafe {
        device_handle
            .create_fence(&create_info, None)
            .expect("koi::ren::vk::Frame - failed to reate Fence")
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
