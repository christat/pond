use std::collections::VecDeque;

use ash::{vk, Device as DeviceHandle, Instance as InstanceHandle};
use gpu_allocator::vulkan as vka;

pub struct AllocatedResources {
    images: VecDeque<(vk::Image, vk::ImageView, vka::Allocation)>
}

impl AllocatedResources {
    pub fn new() -> Self {
        Self {
            images: VecDeque::new(),
        }
    }

    pub fn add_image(&mut self, image: vk::Image, view: vk::ImageView, allocation: vka::Allocation) {
        self.images.push_back((image, view, allocation));
    }

    pub fn drop(&mut self, device: &DeviceHandle, allocator: &mut vka::Allocator) {
        while !self.images.is_empty() {
            let (image, view, allocation) = self.images.pop_front().unwrap();
            unsafe {
                device.destroy_image_view(view, None);
                allocator.free(allocation).expect("koi::vk::allocator - failed to free Image Allocation");
                device.destroy_image(image, None);
            }
        }
    }
}

pub struct Allocator {
    pub handle: vka::Allocator,
    pub frame_resources: Vec<AllocatedResources>,
    pub global_resources: AllocatedResources,
}

#[allow(unused)]
impl Allocator {
    pub fn new(instance: InstanceHandle, device: DeviceHandle, physical_device: vk::PhysicalDevice, buffering: u32) -> Self {
        let handle = vka::Allocator::new(&vka::AllocatorCreateDesc {
            instance,
            device,
            physical_device,
            debug_settings: Default::default(),
            buffer_device_address: true,
            allocation_sizes: Default::default()
        }).expect("koi::ren::vk - failed to create Allocator");

        let frame_resources = (0..buffering).into_iter().map(|_| AllocatedResources::new()).collect();

        Self { handle, frame_resources, global_resources: AllocatedResources::new() }
    }

    pub fn add_image(&mut self, frame: Option<usize>, image: vk::Image, view: vk::ImageView, allocation: vka::Allocation) {
        match frame {
            Some(index) => self.frame_resources[index].add_image(image, view, allocation),
            None => self.global_resources.add_image(image, view, allocation),
        }
    }

    pub fn drop_frame(&mut self, device: &DeviceHandle, frame: usize) {
        self.frame_resources[frame].drop(device, &mut self.handle);
    }

    pub fn drop(&mut self, device: &DeviceHandle) {
        self.frame_resources.iter_mut().for_each(|handle| handle.drop(device, &mut self.handle));
        self.global_resources.drop(device, &mut self.handle);
    }
}