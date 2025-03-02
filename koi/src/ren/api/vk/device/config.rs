use std::{collections::HashSet, ffi::{c_char, CStr}, u32};
use ash::{khr, vk, Instance};

use crate::ren::api::vk::surface::Surface;

pub struct DeviceConfig<'a> {
    pub extensions: Vec<&'a CStr>,
    pub features: vk::PhysicalDeviceFeatures,
    pub vk_13_features: vk::PhysicalDeviceVulkan13Features<'a>,
    pub vk_12_features: vk::PhysicalDeviceVulkan12Features<'a>,
    pub queue_create_infos: Vec<vk::DeviceQueueCreateInfo<'a>>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum DeviceConfigError<'a> {
    FeatureNotSupported(&'a CStr),
    ExtensionNotSupported(&'a CStr),
    PropertyNotFulfilled(&'a CStr),
    QueueFamilyNotSupported(&'a CStr),
}

#[derive(PartialEq, Eq, PartialOrd)]
pub struct PhysicalDeviceProperties {
    pub max_image_dimension_2d: u32
}

impl Ord for PhysicalDeviceProperties {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.max_image_dimension_2d.cmp(&self.max_image_dimension_2d)
    }
}

impl PhysicalDeviceProperties {
    pub fn new(properties: &vk::PhysicalDeviceProperties) -> Self {
        Self { max_image_dimension_2d: properties.limits.max_image_dimension2_d }
    }
}

#[allow(unused)]
pub enum QueueFamilyType {
    Graphics,
    Present
}

#[derive(Clone, PartialEq, Eq, PartialOrd)]
pub struct PhysicalDeviceQueueFamilies {
    pub graphics_family_index: Option<u32>,
    pub present_family_index: Option<u32>,
}

// NB! Hack; hardcoded as we only need one queue from each family.
const QUEUE_PRIORITIES: [f32; 1] = [1.0];

impl PhysicalDeviceQueueFamilies {
    pub fn new() -> Self {
        Self {
            graphics_family_index: None,
            present_family_index: None,
        }
    }

    pub fn get_family_index(&self, family_type: QueueFamilyType) -> u32 {
        match family_type {
            QueueFamilyType::Graphics => self.graphics_family_index.unwrap_or(u32::MAX),
            QueueFamilyType::Present => self.present_family_index.unwrap_or(u32::MAX)
        }
    }

    pub fn get_unique_indices(&self) -> Vec<u32> {
        let mut unique_indices = HashSet::new();
        if self.graphics_family_index.is_some() { unique_indices.insert(self.graphics_family_index.unwrap()); }
        if self.present_family_index.is_some() { unique_indices.insert(self.present_family_index.unwrap()); }
        unique_indices.into_iter().collect()
    }
}

#[derive(PartialEq, Eq, PartialOrd)]
pub struct ValidPhysicalDevice {
    pub handle: vk::PhysicalDevice,
    pub properties: PhysicalDeviceProperties,
    pub queue_families: PhysicalDeviceQueueFamilies,
}

impl Ord for ValidPhysicalDevice {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.properties.cmp(&self.properties)
    }
}

impl ValidPhysicalDevice {
    pub fn new(handle: vk::PhysicalDevice, properties: &vk::PhysicalDeviceProperties, queue_families: PhysicalDeviceQueueFamilies, ) -> Self {
        Self {
            handle,
            properties: PhysicalDeviceProperties::new(properties),
            queue_families,
        }
    }
}

impl DeviceConfig<'_> {
    pub fn new<'a>(instance: &Instance, valid_physical_device: &ValidPhysicalDevice) -> Result<Self, DeviceConfigError<'a>> {
        let extensions = vec![
            khr::swapchain::NAME,
            khr::dynamic_rendering::NAME,
        ];
        
        validate_extensions(instance, valid_physical_device.handle, &extensions)?;

        let mut features: vk::PhysicalDeviceFeatures = Default::default();
        features = features.geometry_shader(true);

        let mut vk_13_features: vk::PhysicalDeviceVulkan13Features = Default::default();
        vk_13_features.dynamic_rendering = vk::TRUE;
        vk_13_features.synchronization2 = vk::TRUE;

        let mut vk_12_features: vk::PhysicalDeviceVulkan12Features = Default::default();
        vk_12_features.buffer_device_address = vk::TRUE;
        vk_12_features.descriptor_indexing = vk::TRUE;

        let queue_create_infos = valid_physical_device.queue_families.get_unique_indices()
            .into_iter()
            .map(|index| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(index)
                    .queue_priorities(&QUEUE_PRIORITIES)
            })
            .collect();

        Ok(Self { extensions, features, vk_13_features, vk_12_features, queue_create_infos })
    }

    pub fn get_extensions(&self) -> Vec<*const c_char> {
        self.extensions.iter().map(|extension| {extension.as_ptr()}).collect()
    }
}

pub fn validate_physical_device<'a>(instance: &'a Instance, physical_device: vk::PhysicalDevice, surface: &Surface) -> Result<ValidPhysicalDevice, DeviceConfigError<'a>> {
    let properties = unsafe { instance.get_physical_device_properties(physical_device) };
    
    if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU { return Err(DeviceConfigError::PropertyNotFulfilled(c"discrete_gpu")) }

    validate_physical_device_feature_requirements(instance, physical_device)?;
    let queue_families = validate_physical_device_queue_families(instance, physical_device, surface)?;

    Ok(ValidPhysicalDevice::new(physical_device, &properties, queue_families))
}

fn validate_extensions<'a>(instance: &Instance, physical_device: vk::PhysicalDevice, extensions: &[&'a CStr]) -> Result<(), DeviceConfigError<'a>> {
    let device_extension_properties = unsafe { instance.enumerate_device_extension_properties(physical_device).expect("koi::ren::vk::device::Config - failed to enumerate device extension properties") };
    
    fn validate_extension<'b>(extension: &'b CStr, available_extensions: &[vk::ExtensionProperties]) -> Result<(), DeviceConfigError<'b>> {
        match available_extensions.iter().any(|extensions_property| extensions_property.extension_name_as_c_str().unwrap() == extension) {
            true => Ok(()),
            false => Err(DeviceConfigError::ExtensionNotSupported(extension))
        }
    }

    extensions.iter().map(|extension| validate_extension(extension, &device_extension_properties)).collect()
}

fn validate_physical_device_feature_requirements(instance: &Instance, physical_device: vk::PhysicalDevice) -> Result<(), DeviceConfigError> {
    // NB! C-style pattern of feeding-in blank struct refs
    let mut vk_13_features: vk::PhysicalDeviceVulkan13Features = Default::default();
    let mut vk_12_features: vk::PhysicalDeviceVulkan12Features = Default::default();
    let mut features_2: vk::PhysicalDeviceFeatures2 = Default::default();
    features_2 = features_2.push_next(&mut vk_13_features);
    features_2 = features_2.push_next(&mut vk_12_features);

    unsafe { instance.get_physical_device_features2(physical_device, &mut features_2) };

    if features_2.features.geometry_shader == vk::FALSE { return Err(DeviceConfigError::FeatureNotSupported(c"geometry_shader")) }
    if vk_13_features.dynamic_rendering == vk::FALSE { return Err(DeviceConfigError::FeatureNotSupported(c"vk_13_dynamic_rendering")) }
    if vk_13_features.synchronization2 == vk::FALSE { return Err(DeviceConfigError::FeatureNotSupported(c"vk_13_synchronization2")) }
    if vk_12_features.buffer_device_address == vk::FALSE { return Err(DeviceConfigError::FeatureNotSupported(c"vk_12_buffer_device_address")) }
    if vk_12_features.descriptor_indexing == vk::FALSE { return Err(DeviceConfigError::FeatureNotSupported(c"vk_12_descriptor_indexing")) }
    Ok(())
}

fn validate_physical_device_queue_families<'a>(instance: &Instance, physical_device: vk::PhysicalDevice, surface: &Surface) -> Result<PhysicalDeviceQueueFamilies, DeviceConfigError<'a>> {
    let queue_family_properties = unsafe{ instance.get_physical_device_queue_family_properties(physical_device) } ;

    let physical_device_queue_families = queue_family_properties.iter().enumerate()
        .fold(PhysicalDeviceQueueFamilies::new(), |mut fold, (queue_family_index, family)| {
            let qfi = queue_family_index as u32;
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) && fold.graphics_family_index.is_none() {
                fold.graphics_family_index = Some(qfi);
                
            }
            if fold.present_family_index.is_none() {
                if unsafe { surface.instance.get_physical_device_surface_support(physical_device, qfi, surface.khr).expect("koi::ren::vk::device::Config - failed to get physical device surface support check") } {
                    fold.present_family_index = Some(qfi);
                }
            }

            fold
        });

    if physical_device_queue_families.graphics_family_index.is_none() { return Err(DeviceConfigError::QueueFamilyNotSupported(c"graphics")); }
    if physical_device_queue_families.present_family_index.is_none() { return Err(DeviceConfigError::QueueFamilyNotSupported(c"transfer")); }

    Ok(physical_device_queue_families)
}