pub mod config;

use crate::{app::info::Info, traits};

use ash::{vk, Entry, Instance as VkInstance, ext::debug_utils};

#[cfg(feature = "debug")]
struct InstanceDebugUtils {
    instance: debug_utils::Instance,
    messenger: vk::DebugUtilsMessengerEXT,
}

#[cfg(feature = "debug")]
impl InstanceDebugUtils {
    pub fn new(instance: debug_utils::Instance, messenger: vk::DebugUtilsMessengerEXT) -> Self {
        Self{ instance, messenger }
    }
}

pub struct Instance {
    pub handle: VkInstance,
    #[cfg(feature = "debug")]
    debug_utils: InstanceDebugUtils
}

impl Instance {
    pub fn new(entry: &Entry, info: &Info) -> Self {
        let app_info = vk::ApplicationInfo::default()
                .application_name(&info.app_name)
                .application_version(info.app_version)
                .engine_name(&info.engine_name)
                .engine_version(info.engine_version)
                .api_version(vk::API_VERSION_1_3);
    
        let instance_config = config::InstanceConfig::new(entry).expect("koi::ren::vk::Instance - failed to create Config");
        let extensions = instance_config.get_extensions();
        let layers = instance_config.get_layers();

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layers);
    
        let instance = unsafe { entry.create_instance(&create_info, None).expect("koi::ren::vk::Instance - failed to create Instance") };

        #[cfg(feature = "debug")]
        {            
            let messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                )
                .pfn_user_callback(Some(pfn_user_callback));

            let debug_utils_instance = { debug_utils::Instance::new(&entry, &instance) };
            let debug_utils_messenger = unsafe { debug_utils_instance.create_debug_utils_messenger(&messenger_create_info, None).expect("koi::ren::vk::Instance - failed to create debug utils messenger") };
            
            Self { handle: instance, debug_utils: InstanceDebugUtils::new(debug_utils_instance, debug_utils_messenger) }
        }
        #[cfg(not(feature = "debug"))]
        {
            Self { handle: instance }
        }
    }
}

#[cfg(feature = "debug")]
#[allow(unused_variables)]
unsafe extern "system" fn pfn_user_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    p_user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    use std::{borrow::Cow, ffi};
    use log::{debug, error, info, warn};

    let callback_data = unsafe { *p_callback_data };
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        unsafe { ffi::CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy() }
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        unsafe { ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy() }
    };

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => debug!("{message_severity:?}: {message_type:?} [{message_id_name} ({message_id_number})] : {message}\n"),
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => info!("{message_severity:?}: {message_type:?} [{message_id_name} ({message_id_number})] : {message}\n"),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => warn!("{message_severity:?}: {message_type:?} [{message_id_name} ({message_id_number})] : {message}\n"),
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => error!("{message_severity:?}: {message_type:?} [{message_id_name} ({message_id_number})] : {message}\n"),
        _ => {},
    }

    vk::FALSE
}


impl traits::Drop for Instance {
    fn drop(&mut self) {
        #[cfg(feature = "debug")]
        unsafe { self.debug_utils.instance.destroy_debug_utils_messenger(self.debug_utils.messenger, None) };
        unsafe { self.handle.destroy_instance(None) };
    }
}