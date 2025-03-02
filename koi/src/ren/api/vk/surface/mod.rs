
use crate::{ren::window::Window, traits};

use ash::{khr, vk, Entry, Instance};

#[allow(unused)]
pub struct Surface {
    pub instance: khr::surface::Instance,
    pub khr: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(entry: &Entry, instance: &Instance, handle: &Window) -> Self {
        let surface_instance = ash::khr::surface::Instance::new(entry, instance);

        #[cfg(target_os = "windows")]
        {
            let khr_instance= khr::win32_surface::Instance::new(entry, instance);

            let create_info = vk::Win32SurfaceCreateInfoKHR::default()
                .hwnd(handle.window.hwnd.into())
                .hinstance(handle.window.hinstance.expect("koi::ren::vk::Surface - failed to obtain window hinstance").into());

            let khr = unsafe { khr_instance.create_win32_surface(&create_info, None).expect("koi::ren::vk::Surface - Failed to create Win32 Surface") };
            
            Self { instance: surface_instance, khr }
        }
        #[cfg(target_os = "linux")]
        {
            let khr_instance = khr::xcb_surface::Instance::new(entry, instance);

            let create_info = vk::XcbSurfaceCreateInfoKHR::default()
                .connection(handle.display.connection.expect("koi::ren::vk::Surface - Failed to obtain display connection").as_ptr() as *mut _)
                .window(handle.window.window.into());

            let khr = unsafe { khr_instance.create_xcb_surface(&create_info, None).expect("koi::ren::vk::Surface - Failed to create Win32 Surface") };
            
            Self { instance: surface_instance, khr }
        }
    }
}

impl traits::Drop for Surface {
    fn drop(&mut self) {
        unsafe{ self.instance.destroy_surface(self.khr, None) };
    }
}