use crate::ren::settings::Settings;

use ash::{Device as DeviceHandle, Instance as InstanceHandle, vk};
use gpu_allocator::vulkan as vka;
use std::collections::VecDeque;

pub struct AllocatedResources {
    pub images: VecDeque<(vk::Image, vk::ImageView, vka::Allocation)>,
    pub buffers: VecDeque<(vk::Buffer, vka::Allocation)>,
}

impl AllocatedResources {
    pub fn new() -> Self {
        Self {
            images: VecDeque::new(),
            buffers: VecDeque::new(),
        }
    }

    pub fn add_image(
        &mut self,
        image: vk::Image,
        view: vk::ImageView,
        allocation: vka::Allocation,
    ) {
        self.images.push_back((image, view, allocation));
    }

    pub fn add_buffer(&mut self, buffer: vk::Buffer, allocation: vka::Allocation) {
        self.buffers.push_back((buffer, allocation));
    }

    pub fn drop(&mut self, device: &DeviceHandle, allocator: &mut vka::Allocator) {
        while !self.images.is_empty() {
            let (image, view, allocation) = self.images.pop_front().unwrap();
            unsafe {
                device.destroy_image_view(view, None);
                device.destroy_image(image, None);
                allocator
                    .free(allocation)
                    .expect("koi::vk::allocator - failed to free Image Allocation");
            }
        }
        while !self.buffers.is_empty() {
            let (buffer, allocation) = self.buffers.pop_front().unwrap();
            unsafe { device.destroy_buffer(buffer, None) };
            allocator
                .free(allocation)
                .expect("koi::vk::allocator - failed to free Buffer Allocation");
        }
    }
}

pub struct ResourceAllocator {
    pub handle: vka::Allocator,
    pub frame_resources: Vec<AllocatedResources>,
    pub global_resources: AllocatedResources,
    pub min_alignment: usize,
}

#[allow(unused)]
impl ResourceAllocator {
    pub fn new(
        instance: InstanceHandle,
        device: DeviceHandle,
        physical_device: vk::PhysicalDevice,
        settings: &Settings,
        min_alignment: usize,
    ) -> Self {
        let handle = vka::Allocator::new(&vka::AllocatorCreateDesc {
            instance,
            device,
            physical_device,
            debug_settings: Default::default(),
            buffer_device_address: true,
            allocation_sizes: Default::default(),
        })
        .expect("koi::ren::vk::allocator - failed to create Allocator");

        let frame_resources = (0..settings.buffering)
            .into_iter()
            .map(|_| AllocatedResources::new())
            .collect();

        Self {
            handle,
            frame_resources,
            global_resources: AllocatedResources::new(),
            min_alignment,
        }
    }

    pub fn add_image(
        &mut self,
        frame: Option<usize>,
        image: vk::Image,
        view: vk::ImageView,
        allocation: vka::Allocation,
    ) {
        match frame {
            Some(index) => self.frame_resources[index].add_image(image, view, allocation),
            None => self.global_resources.add_image(image, view, allocation),
        }
    }

    pub fn drop_frame_resources(&mut self, device: &DeviceHandle, frame: usize) {
        self.frame_resources[frame].drop(device, &mut self.handle);
    }

    pub fn drop(&mut self, device: &DeviceHandle) {
        self.frame_resources
            .iter_mut()
            .for_each(|handle| handle.drop(device, &mut self.handle));
        self.global_resources.drop(device, &mut self.handle);
        #[cfg(feature = "debug")]
        self.handle.report_memory_leaks(log::Level::Error);
    }
}
