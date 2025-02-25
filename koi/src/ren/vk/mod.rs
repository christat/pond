use ash::Entry;
use config::VkConfig;

mod config;
mod instance;

use crate::ren::Renderer;
use crate::ren::vk::instance::VkInstance;
use crate::t;

pub struct VkRenderer {
    config: VkConfig,
    entry: Entry,
    instance: VkInstance,
    // physical_device: vk::PhysicalDevice,
    // device: vk::Device,
    // surface: vk::SurfaceKHR
}

impl Renderer for VkRenderer {
    fn new(app_name: &str) -> Self {
        let entry = unsafe { Entry::load().expect("VkRenderer::new - Failed to create Vulkan Instance") };

        let config = VkConfig::new(&entry, app_name, "koi");
        let instance = VkInstance::new(&entry, &config);

        Self {
            config: config,
            entry: entry,
            instance: instance,
        }
    }
}

impl t::Drop for VkRenderer {
    fn drop(&mut self) {
        self.instance.drop();
    }
}