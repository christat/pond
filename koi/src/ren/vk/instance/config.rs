use ash::{ext, khr, vk, Entry};
use std::ffi::{c_char, CString};

pub struct Instance {
    layers: Vec<CString>,
    extensions: Vec<CString>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum InstanceConfigError {
    LayerNotSupported(CString),
    ExtensionNotSupported(CString)
}

impl Instance {
    pub fn new(entry: &Entry) -> Result<Self, InstanceConfigError> {
        let layers = vec![
            #[cfg(feature = "debug")]
            CString::new("VK_LAYER_KHRONOS_validation").unwrap(),
        ];
        
        validate_layers(&entry, &layers)?;

        let extensions = vec![
            khr::get_physical_device_properties2::NAME.to_owned(),
            khr::surface::NAME.to_owned(),
            
            #[cfg(target_os = "windows")]
            khr::win32_surface::NAME.to_owned(),
            #[cfg(target_os = "linux")]
            khr::xcb_surface::NAME.to_owned(),

            #[cfg(feature = "debug")]
            ext::debug_utils::NAME.to_owned(),
        ];

        validate_extensions(&entry, &extensions)?;

        Ok(Self {
            layers: layers,
            extensions: extensions
        })
    }

    pub fn get_layers(&self) -> Vec<*const c_char> {
        self.layers.iter().map(|layer| {layer.as_ptr()}).collect()
    }

    pub fn get_extensions(&self) -> Vec<*const c_char> {
        self.extensions.iter().map(|layer| {layer.as_ptr()}).collect()
    }
}

fn validate_layers(entry: &Entry, layers: &[CString]) -> Result<(), InstanceConfigError> {
    let instance_layer_properties = unsafe{ entry.enumerate_instance_layer_properties().expect("ren::vk::instance::Config - failed to enumerate instance layer properties") };

    fn validate_layer(layer: &CString, available_layers: &[vk::LayerProperties]) -> Result<(), InstanceConfigError> {
        match available_layers.iter().any(|layer_property| layer_property.layer_name_as_c_str().unwrap() == layer.as_c_str()) {
            true => Ok(()),
            false => Err(InstanceConfigError::LayerNotSupported(layer.to_owned()))
        }
    }

    layers.iter().map(|layer| validate_layer(layer, &instance_layer_properties)).collect()
}

fn validate_extensions(entry: &Entry, extensions: &[CString]) -> Result<(), InstanceConfigError> {
    let instance_extension_properties = unsafe{ entry.enumerate_instance_extension_properties(None).expect("ren::vk::instance::Config - failed to enumerate instance extensions properties") };

    fn validate_extension(extension: &CString, available_extensions: &[vk::ExtensionProperties]) -> Result<(), InstanceConfigError> {
        match available_extensions.iter().any(|extensions_property| extensions_property.extension_name_as_c_str().unwrap() == extension.as_c_str()) {
            true => Ok(()),
            false => Err(InstanceConfigError::ExtensionNotSupported(extension.to_owned()))
        }
    }

    extensions.iter().map(|extension| validate_extension(extension, &instance_extension_properties)).collect()
}
