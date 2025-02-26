
use crate::t;

use ash::{Entry, Instance, vk, khr};

pub struct Surface {
    pub instance: khr::surface::Instance,
    pub khr: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(entry: &Entry, instance: &Instance) -> Self {
        let surface_instance = khr::surface::Instance::new(&entry, &instance);
        let surface_khr = vk::SurfaceKHR::default();
        Self { instance: surface_instance, khr: surface_khr }
    }
}

impl t::Drop for Surface {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_surface(self.khr, None) };
    }
}