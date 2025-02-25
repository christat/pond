use ash::{vk::{self, make_api_version, ApplicationInfo, InstanceCreateInfo}, Entry, Instance, ext::debug_utils};

use crate::{ren::vk::config::VkConfig, t};

#[cfg(feature = "debug")]
struct VkInstanceDebugUtils {
    instance: debug_utils::Instance,
    messenger: vk::DebugUtilsMessengerEXT,
}

pub struct VkInstance {
    instance: Instance,
    #[cfg(feature = "debug")]
    debug_utils: VkInstanceDebugUtils
}

impl VkInstance {
    pub fn new(entry: &Entry, config: &VkConfig) -> Self {
        let app_info = ApplicationInfo::default()
                .application_name(&config.app_name)
                .application_version(vk::make_api_version(0, 0, 1, 0))
                .engine_name(&config.engine_name)
                .engine_version(vk::make_api_version(0, 0, 1, 0))
                .api_version(make_api_version(0, 1, 3, 0));
    
        let extensions = config.instance.get_extensions();
        let layers = config.instance.get_layers();
        let create_info = InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layers);
    
        let instance = unsafe { entry.create_instance(&create_info, None).expect("vkRenderer::Instance - failed to create Instance") };

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

            let debug_utils_messenger: vk::DebugUtilsMessengerEXT = unsafe { debug_utils_instance.create_debug_utils_messenger(&messenger_create_info, None).expect("VkRenderer::Instance - failed to create debug utils messenger") };
            
            Self { instance: instance, debug_utils: VkInstanceDebugUtils{ instance: debug_utils_instance, messenger: debug_utils_messenger } }
        }
        #[cfg(not(feature = "debug"))]
        {
            Self { instance: instance }
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


impl t::Drop for VkInstance {
    fn drop(&mut self) {
        #[cfg(feature = "debug")]
        unsafe { self.debug_utils.instance.destroy_debug_utils_messenger(self.debug_utils.messenger, None) };
        unsafe { self.instance.destroy_instance(None) };
    }
}