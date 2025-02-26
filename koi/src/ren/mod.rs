#[cfg(feature = "directx")]
mod dx;
#[cfg(feature = "vulkan")]
mod vk;

mod info;

use crate::info::Info;

trait Renderer {
    fn new(info: &Info) -> Self;
}

pub struct Handle {
    #[cfg(feature = "directx")]
    api: dx::Renderer,
    #[cfg(feature = "vulkan")]
    api: vk::Renderer,
}

pub fn new(info: &Info) -> Handle {
    #[cfg(feature = "directx")]
    return Handle { api: dx::Renderer::new(info) };
    #[cfg(feature = "vulkan")]
    return Handle { api: vk::Renderer::new(info) };
}