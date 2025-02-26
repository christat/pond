mod device;
mod instance;
mod surface;

use device::Device;
use instance::Instance;
use surface::Surface;
use crate::{info::Info, t};
use super::{info::Info as RenInfo, Renderer as RendererTrait};

use ash::Entry;

pub struct Renderer {
    ren_info: RenInfo,
    entry: Entry,
    instance: Instance,
    device: Device,
    surface: Surface,
    // swapchain: Swapchain,
}

impl RendererTrait for Renderer {
    fn new(info: &Info) -> Self {
        let entry = unsafe { Entry::load().expect("ren::vk::new - Failed to create Vulkan Instance") };

        let ren_info = RenInfo::new();
        let instance = Instance::new(&entry, &info, &ren_info);
        let device = Device::new(&instance.handle);
        let surface = Surface::new(&entry, &instance.handle);

        Self {
            ren_info: ren_info,
            entry: entry,
            instance: instance,
            device: device,
            surface: surface
        }
    }
}

impl t::Drop for Renderer {
    fn drop(&mut self) {
        self.instance.drop();
    }
}