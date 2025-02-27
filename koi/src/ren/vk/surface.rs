
use crate::ren::window::Window;

use ash::{Entry, Instance, vk, khr};

#[allow(unused)]
pub struct Surface {
    pub surface: vk::SurfaceKHR,
    #[cfg(target_os = "windows")]
    pub instance: khr::win32_surface::Instance,
    #[cfg(target_os = "linux")]
    pub instance: khr::xcb_surface::Instance,
}

impl Surface {
    pub fn new(entry: &Entry, instance: &Instance, handle: &Window) -> Self {
        #[cfg(target_os = "windows")]
        {
            let instance= khr::win32_surface::Instance::new(entry, instance);

            let create_info = vk::Win32SurfaceCreateInfoKHR::default()
                .hwnd(handle.window.hwnd.into())
                .hinstance(handle.window.hinstance.expect("koi::ren::vk::Surface - failed to obtain window hinstance").into());

            let surface = unsafe { instance.create_win32_surface(&create_info, None).expect("ren::vk::Surface - Failed to create Win32 Surface") };
            
           Self { instance: instance, surface: surface }
        }
        #[cfg(target_os = "linux")]
        {
            let instance = khr::xcb_surface::Instance::new(entry, instance);

            let create_info = vk::XcbSurfaceCreateInfoKHR::default()
                .connection(handle.display.connection.expect("ren::vk::Surface - Failed to obtain display connection").as_ptr() as *mut _)
                .window(handle.window.window.into());

            let surface = unsafe { instance.create_xcb_surface(&create_info, None).expect("ren::vk::Surface - Failed to create Win32 Surface") };
            
            return Self { instance: instance, surface: surface }
        }
    }
}