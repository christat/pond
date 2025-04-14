use crate::ren::Settings;

use ash::{Device as DeviceHandle, Instance as InstanceHandle, vk};
use gpu_allocator::vulkan as vka;
use std::collections::VecDeque;

pub struct ResourceManager {
    pub images: VecDeque<(vk::Image, vk::ImageView, vka::Allocation)>,
    pub buffers: VecDeque<(vk::Buffer, vka::Allocation)>,
}

impl ResourceManager {
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

    pub fn drop_image(
        &mut self,
        device: &DeviceHandle,
        allocator: &mut vka::Allocator,
        image: vk::Image,
    ) {
        let search = self.images.iter().enumerate().find_map(|(i, (img, _, _))| {
            if *img == image {
                return Some(i);
            } else {
                return None;
            }
        });
        match search {
            None => {}
            Some(index) => {
                let (image, image_view, allocation) = self.images.remove(index).unwrap();
                unsafe {
                    device.destroy_image_view(image_view, None);
                    device.destroy_image(image, None);
                    allocator
                        .free(allocation)
                        .expect("koi::vk::allocator - failed to free Image Allocation");
                }
            }
        }
    }

    pub fn add_buffer(&mut self, buffer: vk::Buffer, allocation: vka::Allocation) {
        self.buffers.push_back((buffer, allocation));
    }

    pub fn drop_buffer(
        &mut self,
        device: &DeviceHandle,
        allocator: &mut vka::Allocator,
        buffer: vk::Buffer,
    ) {
        let search = self.buffers.iter().enumerate().find_map(|(i, (buf, _))| {
            if *buf == buffer {
                return Some(i);
            } else {
                return None;
            }
        });
        match search {
            None => {}
            Some(index) => {
                let (buffer, allocation) = self.buffers.remove(index).unwrap();
                unsafe {
                    device.destroy_buffer(buffer, None);
                    allocator
                        .free(allocation)
                        .expect("koi::vk::allocator - failed to free Buffer Allocation");
                }
            }
        }
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

pub struct Allocator {
    pub handle: vka::Allocator,
    pub frame_resources: Vec<ResourceManager>,
    pub global_resources: ResourceManager,
}

#[allow(unused)]
impl Allocator {
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
            .map(|_| ResourceManager::new())
            .collect();

        Self {
            handle,
            frame_resources,
            global_resources: ResourceManager::new(),
        }
    }

    pub fn add_buffer(
        &mut self,
        frame: Option<usize>,
        buffer: vk::Buffer,
        allocation: vka::Allocation,
    ) {
        match frame {
            Some(index) => self.frame_resources[index].add_buffer(buffer, allocation),
            None => self.global_resources.add_buffer(buffer, allocation),
        }
    }

    pub fn drop_buffer(
        &mut self,
        device: &DeviceHandle,
        allocator: &mut vka::Allocator,
        frame: Option<usize>,
        buffer: vk::Buffer,
    ) {
        match frame {
            Some(index) => self.frame_resources[index].drop_buffer(device, allocator, buffer),
            None => self.global_resources.drop_buffer(device, allocator, buffer),
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

    pub fn drop_image(
        &mut self,
        device: &DeviceHandle,
        allocator: &mut vka::Allocator,
        frame: Option<usize>,
        image: vk::Image,
    ) {
        match frame {
            Some(index) => self.frame_resources[index].drop_image(device, allocator, image),
            None => self.global_resources.drop_image(device, allocator, image),
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
