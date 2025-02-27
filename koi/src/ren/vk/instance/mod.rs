mod config;

use crate::{app::info::Info, ren::info::Info as RenInfo, t};

use ash::{vk, Entry, Instance as VkInstance, ext::debug_utils};

#[cfg(feature = "debug")]
struct InstanceDebugUtils {
    instance: debug_utils::Instance,
    messenger: vk::DebugUtilsMessengerEXT,
}

pub struct Instance {
    pub handle: VkInstance,
    #[cfg(feature = "debug")]
    debug_utils: InstanceDebugUtils
}

impl Instance {
    pub fn new(entry: &Entry, info: &Info, ren_info: &RenInfo) -> Self {
        let app_info = vk::ApplicationInfo::default()
                .application_name(&info.app_name)
                .application_version(info.app_version)
                .engine_name(&info.engine_name)
                .engine_version(info.engine_version)
                .api_version(ren_info.api_version);
    
        let instance_config = config::Instance::new(entry).expect("ren::vk::Instance - failed to create Config");
        let extensions = instance_config.get_extensions();
        let layers = instance_config.get_layers();

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layers);
    
        let instance = unsafe { entry.create_instance(&create_info, None).expect("ren::vk::Instance - failed to create Instance") };

        #[cfg(feature = "debug")]
        {
            let debug_utils_instance = { debug_utils::Instance::new(&entry, &instance) };
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

            let debug_utils_messenger: vk::DebugUtilsMessengerEXT = unsafe { debug_utils_instance.create_debug_utils_messenger(&messenger_create_info, None).expect("ren::vk::Instance - failed to create debug utils messenger") };
            
            Self { handle: instance, debug_utils: InstanceDebugUtils{ instance: debug_utils_instance, messenger: debug_utils_messenger } }
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
    message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    p_user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    use log::{debug, error, info, warn};

    let msg_type = match message_types {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "GENERAL",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "VALIDATION",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "PERFORMANCE",
        _ => "OTHER",
    };

    let msg = format!(
        "[{}] - {:?}",
        msg_type,
        unsafe { (*p_callback_data).p_message }
    );

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => debug!("{}", msg),
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => info!("{}", msg),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => warn!("{}", msg),
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => error!("{}", msg),
        _ => {}
    }

    vk::FALSE
}


impl t::Drop for Instance {
    fn drop(&mut self) {
        #[cfg(feature = "debug")]
        unsafe { self.debug_utils.instance.destroy_debug_utils_messenger(self.debug_utils.messenger, None) };
        unsafe { self.handle.destroy_instance(None) };
    }
}