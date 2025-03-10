pub mod config;

use super::surface::Surface;
use crate::traits;
use config::{PhysicalDeviceProperties, PhysicalDeviceQueueFamilies, QueueFamilyType};

use ash::{Device as DeviceHandle, Instance, vk};

#[allow(unused)]
pub struct Device {
    pub physical_device: vk::PhysicalDevice,
    pub physical_device_properties: PhysicalDeviceProperties,
    pub queue_families: PhysicalDeviceQueueFamilies,
    pub handle: DeviceHandle,
}

impl Device {
    pub fn new(instance: &Instance, surface: &Surface) -> Self {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("koi::ren::vk::Device - failed to enumerate physical devices")
        };

        let mut suitable_physical_devices: Vec<_> = physical_devices
            .iter()
            .filter_map(|&physical_device| {
                match config::validate_physical_device(instance, physical_device, surface) {
                    Ok(device) => Some(device),
                    Err(_e) => None,
                }
            })
            .collect();

        suitable_physical_devices.sort();

        let selected_physical_device = suitable_physical_devices
            .first()
            .expect("koi::ren::vk::Device - failed to find suitable physical device");

        let mut device_config = config::DeviceConfig::new(&instance, selected_physical_device)
            .expect("koi::ren::vk::Device - failed to create device config");
        let extensions = device_config.get_extensions();

        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&device_config.queue_create_infos)
            .enabled_extension_names(&extensions)
            .enabled_features(&device_config.features)
            .push_next(&mut device_config.vk_13_features)
            .push_next(&mut device_config.vk_12_features);

        let device = unsafe {
            instance
                .create_device(selected_physical_device.handle, &create_info, None)
                .expect("koi::ren::vk::Device - failed to create device")
        };

        Self {
            physical_device: selected_physical_device.handle,
            physical_device_properties: selected_physical_device.properties.clone(),
            queue_families: selected_physical_device.queue_families.clone(),
            handle: device,
        }
    }

    pub fn get_queue(&self, queue_family_type: QueueFamilyType) -> vk::Queue {
        let queue_family_index = match queue_family_type {
            QueueFamilyType::Graphics => self.queue_families.graphics_family_index.unwrap(),
            QueueFamilyType::Present => self.queue_families.present_family_index.unwrap(),
        };
        unsafe { self.handle.get_device_queue(queue_family_index, 0) }
    }

    pub fn get_min_memory_map_alignment(&self) -> usize {
        self.physical_device_properties.min_memory_map_alignment
    }
}

impl traits::Drop for Device {
    fn drop(&mut self) {
        unsafe { self.handle.destroy_device(None) };
    }
}
