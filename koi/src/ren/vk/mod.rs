mod device;
mod instance;
mod surface;

use device::Device;
use instance::Instance;
use surface::Surface;
use crate::{app::info::Info, t::Drop};
use super::{info::Info as RenInfo, window::Window, Renderer as RendererTrait};

use ash::Entry;

#[allow(unused)]
pub struct Renderer {
    ren_info: RenInfo,
    window: Window,
    entry: Entry,
    instance: Instance,
    surface: Surface,
    device: Device,
    // swapchain: Swapchain,
}

impl RendererTrait for Renderer {
    fn new(info: &Info, window: Window) -> Self {
        let entry = unsafe { Entry::load().expect("ren::vk::new - Failed to create Vulkan Instance") };

        let ren_info = RenInfo::new();
        let instance = Instance::new(&entry, &info, &ren_info);
        let surface = Surface::new(&entry, &instance.handle, &window);
        let device = Device::new(&instance.handle);

        Self {
            ren_info: ren_info,
            window: window,
            entry: entry,
            instance: instance,
            device: device,
            surface: surface
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        //self.swaphain.drop();
        self.instance.drop();
        self.device.drop();
        self.surface.drop();
    }
}