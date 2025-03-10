use ash::{Entry, ext, khr, vk};
use std::ffi::{CStr, c_char};

pub struct InstanceConfig<'a> {
    layers: Vec<&'a CStr>,
    extensions: Vec<&'a CStr>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum InstanceConfigError<'a> {
    LayerNotSupported(&'a CStr),
    ExtensionNotSupported(&'a CStr),
}

impl InstanceConfig<'_> {
    pub fn new(entry: &Entry) -> Result<Self, InstanceConfigError> {
        let layers = vec![
            #[cfg(feature = "debug")]
            c"VK_LAYER_KHRONOS_validation",
        ];

        validate_layers(&entry, &layers)?;

        let extensions = vec![
            khr::get_physical_device_properties2::NAME,
            khr::surface::NAME,
            #[cfg(target_os = "windows")]
            khr::win32_surface::NAME,
            #[cfg(target_os = "linux")]
            khr::xcb_surface::NAME,
            #[cfg(feature = "debug")]
            ext::debug_utils::NAME,
        ];

        validate_extensions(&entry, &extensions)?;

        Ok(Self { layers, extensions })
    }

    pub fn get_layers(&self) -> Vec<*const c_char> {
        self.layers.iter().map(|layer| layer.as_ptr()).collect()
    }

    pub fn get_extensions(&self) -> Vec<*const c_char> {
        self.extensions.iter().map(|layer| layer.as_ptr()).collect()
    }
}

fn validate_layers<'a>(entry: &Entry, layers: &[&'a CStr]) -> Result<(), InstanceConfigError<'a>> {
    let instance_layer_properties = unsafe {
        entry.enumerate_instance_layer_properties().expect(
            "koi::ren::vk::instance::Config - failed to enumerate instance layer properties",
        )
    };

    fn validate_layer<'b>(
        layer: &'b CStr,
        available_layers: &[vk::LayerProperties],
    ) -> Result<(), InstanceConfigError<'b>> {
        match available_layers
            .iter()
            .any(|layer_property| layer_property.layer_name_as_c_str().unwrap() == layer)
        {
            true => Ok(()),
            false => Err(InstanceConfigError::LayerNotSupported::<'b>(layer)),
        }
    }

    layers
        .iter()
        .map(|layer| validate_layer(layer, &instance_layer_properties))
        .collect()
}

fn validate_extensions<'a>(
    entry: &Entry,
    extensions: &[&'a CStr],
) -> Result<(), InstanceConfigError<'a>> {
    let instance_extension_properties = unsafe {
        entry.enumerate_instance_extension_properties(None).expect(
            "koi::ren::vk::instance::Config - failed to enumerate instance extensions properties",
        )
    };

    fn validate_extension<'b>(
        extension: &'b CStr,
        available_extensions: &[vk::ExtensionProperties],
    ) -> Result<(), InstanceConfigError<'b>> {
        match available_extensions.iter().any(|extensions_property| {
            extensions_property.extension_name_as_c_str().unwrap() == extension
        }) {
            true => Ok(()),
            false => Err(InstanceConfigError::ExtensionNotSupported::<'b>(extension)),
        }
    }

    extensions
        .iter()
        .map(|extension| validate_extension(extension, &instance_extension_properties))
        .collect()
}
