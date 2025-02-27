#[cfg(feature = "directx")]
mod dx;
#[cfg(feature = "vulkan")]
mod vk;

mod info;
mod window;

use crate::info::Info;
use window::Window;
use winit::window::Window as WindowHandle;

trait Renderer {
    fn new(info: &Info, window: Window) -> Self;
}

#[allow(unused)]
pub struct Handle {
    #[cfg(feature = "directx")]
    api: dx::Renderer,
    #[cfg(feature = "vulkan")]
    api: vk::Renderer,
}

pub fn new(info: &Info, window_handle: &WindowHandle) -> Handle {
    let window = Window::new(window_handle).expect("koi::ren::new - failed to create window handle");
    
    #[cfg(feature = "vulkan")]
    let api = vk::Renderer::new(info, window);
    #[cfg(feature = "directx")]
    let api = dx::Renderer::new(info, window);

    Handle { api: api }
}