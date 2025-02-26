mod config;

use crate::t;

use ash::{vk, Device as VkDevice, Instance};
use std::cmp::Ord;

pub struct Device {
    physical_device: vk::PhysicalDevice,
    device: VkDevice
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]

struct Properties {
    max_image_dimension_2d: u32
}

#[derive(PartialEq, Eq, PartialOrd)]
struct SortableDevice {
    handle: vk::PhysicalDevice,
    properties: Properties,
}

impl Ord for SortableDevice {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.properties.cmp(&self.properties)
    }
}

impl SortableDevice {
    pub fn new(physical_device: vk::PhysicalDevice, properties: vk::PhysicalDeviceProperties) -> Self {
        Self {
            handle: physical_device,
            properties: Properties{ max_image_dimension_2d: properties.limits.max_image_dimension2_d }
        }
    }
}

impl Device {
    pub fn new(instance: &Instance) -> Self {
        let physical_devices = unsafe { instance.enumerate_physical_devices().expect("ren::vk::Device - failed to enumerate physical devices") };
        
        let mut suitable_physical_devices: Vec<_> = physical_devices.iter()
            .filter_map(|&physical_device| {
                match config::Device::validate_physical_device(instance, physical_device) {
                    Ok(properties) => Some(SortableDevice::new(physical_device, properties)),
                    Err(_e) => None
                }
            })
            .collect();

        suitable_physical_devices.sort();

        let selected_physical_device = suitable_physical_devices.first().expect("ren::vk::Device - failed to find suitable physical device");

        let mut device_config = config::Device::new(&instance, selected_physical_device.handle).expect("ren::vk::Device - failed to create device config");
        let extensions = device_config.get_extensions();
        let create_info = vk::DeviceCreateInfo::default()
            .enabled_extension_names(&extensions)
            .enabled_features(&device_config.features)
            .push_next(&mut device_config.vk_13_features)
            .push_next(&mut device_config.vk_12_features);

        let device = unsafe { instance.create_device(selected_physical_device.handle, &create_info, None).expect("ren::vk::Device - failed to create device") };
        
        Self {
            physical_device: selected_physical_device.handle,
            device: device,
        }
    }
}

impl t::Drop for Device {
    fn drop(&mut self) {
        unsafe{ self.device.destroy_device(None) };
    }
}