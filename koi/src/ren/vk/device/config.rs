use std::ffi::{c_char, CString};

use ash::{vk, Instance, khr};

pub struct Device<'a> {
    pub extensions: Vec<CString>,
    pub features: vk::PhysicalDeviceFeatures,
    pub vk_13_features: vk::PhysicalDeviceVulkan13Features<'a>,
    pub vk_12_features: vk::PhysicalDeviceVulkan12Features<'a>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum DeviceConfigError {
    FeatureNotSupported(CString),
    ExtensionNotSupported(CString),
    PropertyNotFulfilled(CString)
}

pub fn validate_extensions(instance: &Instance, physical_device: vk::PhysicalDevice, extensions: &[CString]) -> Result<(), DeviceConfigError> {
    let device_extension_properties = unsafe { instance.enumerate_device_extension_properties(physical_device).expect("ren::vk::device::Config - failed to enumerate device extension properties") };
    
    fn validate_extension(extension: &CString, available_extensions: &[vk::ExtensionProperties]) -> Result<(), DeviceConfigError> {
        match available_extensions.iter().any(|extensions_property| extensions_property.extension_name_as_c_str().unwrap() == extension.as_c_str()) {
            true => Ok(()),
            false => Err(DeviceConfigError::ExtensionNotSupported(extension.to_owned()))
        }
    }

    extensions.iter().map(|extension| validate_extension(extension, &device_extension_properties)).collect()
}

impl Device<'_> {
    pub fn new(instance: &Instance, physical_device: vk::PhysicalDevice) -> Result<Self, DeviceConfigError> {
        let extensions = vec![
            khr::swapchain::NAME.to_owned(),
            khr::dynamic_rendering::NAME.to_owned(),
        ];
        
        validate_extensions(instance, physical_device, &extensions)?;

        let mut features: vk::PhysicalDeviceFeatures = Default::default();
        features = features.geometry_shader(true);

        let mut vk_13_features: vk::PhysicalDeviceVulkan13Features = Default::default();
        vk_13_features.dynamic_rendering = vk::TRUE;
        vk_13_features.synchronization2 = vk::TRUE;

        let mut vk_12_features: vk::PhysicalDeviceVulkan12Features = Default::default();
        vk_12_features.buffer_device_address = vk::TRUE;
        vk_12_features.descriptor_indexing = vk::TRUE;

        Ok(Self {
            extensions: extensions,
            features: features,
            vk_13_features: vk_13_features,
            vk_12_features: vk_12_features,
        })
    }

    pub fn get_extensions(&self) -> Vec<*const c_char> {
        self.extensions.iter().map(|extension| {extension.as_ptr()}).collect()
    }

    pub fn validate_physical_device(instance: &Instance, physical_device: vk::PhysicalDevice) -> Result<vk::PhysicalDeviceProperties, DeviceConfigError> {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };

        validate_physical_device_property_requirements(&properties)?;
        validate_physical_device_feature_requirements(instance, physical_device)?;

        Ok(properties)
    }
}

fn validate_physical_device_property_requirements(properties: &vk::PhysicalDeviceProperties) -> Result<(), DeviceConfigError>  {
    if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU { return Err(DeviceConfigError::PropertyNotFulfilled(CString::new("discrete_gpu").unwrap())) }
    Ok(())
}

fn validate_physical_device_feature_requirements(instance: &Instance, physical_device: vk::PhysicalDevice) -> Result<(), DeviceConfigError> {
    // NB! C-style pattern of feeding-in blank struct refs
    let mut vk_13_features: vk::PhysicalDeviceVulkan13Features = Default::default();
    let mut vk_12_features: vk::PhysicalDeviceVulkan12Features = Default::default();
    let mut features_2: vk::PhysicalDeviceFeatures2 = Default::default();
    features_2 = features_2.push_next(&mut vk_13_features);
    features_2 = features_2.push_next(&mut vk_12_features);

    unsafe { instance.get_physical_device_features2(physical_device, &mut features_2) };

    if features_2.features.geometry_shader == vk::FALSE { return Err(DeviceConfigError::FeatureNotSupported(CString::new("geometry_shader").unwrap())) }
    if vk_13_features.dynamic_rendering == vk::FALSE { return Err(DeviceConfigError::FeatureNotSupported(CString::new("vk_13_dynamic_rendering").unwrap())) }
    if vk_13_features.synchronization2 == vk::FALSE { return Err(DeviceConfigError::FeatureNotSupported(CString::new("vk_13_synchronization2").unwrap())) }
    if vk_12_features.buffer_device_address == vk::FALSE { return Err(DeviceConfigError::FeatureNotSupported(CString::new("vk_12_buffer_device_address").unwrap())) }
    if vk_12_features.descriptor_indexing == vk::FALSE { return Err(DeviceConfigError::FeatureNotSupported(CString::new("vk_12_descriptor_indexing").unwrap())) }
    Ok(())
}