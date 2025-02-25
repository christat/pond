use std::ffi::{c_char, CString};

use ash::{Entry, ext, khr, vk::{ExtensionProperties, LayerProperties}};

pub struct VkInstanceConfig {
    layers: Vec<CString>,
    extensions: Vec<CString>,
}

#[derive(Debug)]
pub enum VkInstanceConfigError {
    LayerNotSupported(CString),
    ExtensionNotSupported(CString)
}

impl VkInstanceConfig {
    pub fn new(entry: &Entry) -> Result<Self, VkInstanceConfigError> {
        let layers = vec![
            #[cfg(feature = "debug")]
            CString::new("VK_LAYER_KHRONOS_validation").unwrap(),
        ];
        
        validate_layers(&entry, &layers)?;

        let extensions = vec![
            khr::surface::NAME.to_owned(),
            
            #[cfg(target_os = "windows")]
            khr::win32_surface::NAME.to_owned(),
            #[cfg(target_os = "linux")]
            khr::xcb_surface::NAME.to_owned(),
            #[cfg(target_os = "macos")]
            khr::macos::NAME.to_owned(),

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

fn validate_layers(entry: &Entry, layers: &[CString]) -> Result<(), VkInstanceConfigError> {
    let instance_layer_properties = unsafe{ entry.enumerate_instance_layer_properties().expect("vkRenderer::Instance - failed to enumerate instance layer properties") };

    fn validate_layer(layer: &CString, available_layers: &[LayerProperties]) -> Result<(), VkInstanceConfigError> {
        match available_layers.iter().any(|layer_property| layer_property.layer_name_as_c_str().unwrap() == layer.as_c_str()) {
            true => Ok(()),
            false => Err(VkInstanceConfigError::LayerNotSupported(layer.to_owned()))
        }
    }

    let _validations: Result<Vec<_>, _> = layers.iter().map(|layer| validate_layer(layer, &instance_layer_properties)).collect();
    Ok(())
}

fn validate_extensions(entry: &Entry, extensions: &[CString]) -> Result<(), VkInstanceConfigError> {
    let instance_extension_properties = unsafe{ entry.enumerate_instance_extension_properties(None).expect("vkRenderer::Instance - failed to enumerate instance extensions properties") };

    fn validate_extension(extension: &CString, available_extensions: &[ExtensionProperties]) -> Result<(), VkInstanceConfigError> {
        match available_extensions.iter().any(|extensions_property| extensions_property.extension_name_as_c_str().unwrap() == extension.as_c_str()) {
            true => Ok(()),
            false => Err(VkInstanceConfigError::ExtensionNotSupported(extension.to_owned()))
        }
    }

    let _validations: Result<Vec<_>, _> = extensions.iter().map(|extension| validate_extension(extension, &instance_extension_properties)).collect();
    Ok(())
}

pub struct VkConfig {
    pub app_name: CString,
    pub engine_name: CString,
    pub instance: VkInstanceConfig
}

impl VkConfig {
    pub fn new(entry: &Entry, app_name: &str, engine_name: &str) -> Self {
        let instance = VkInstanceConfig::new(entry).expect("vkRenderer::Config - failed to create instance config");

        Self {
            app_name: CString::new(app_name).unwrap(),
            engine_name: CString::new(engine_name).unwrap(),
            instance: instance,
        }
    }
}