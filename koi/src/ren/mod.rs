use std::ffi::CStr;

#[cfg(feature = "directx")]
mod dx;

#[cfg(feature = "vulkan")]
mod vk;

trait Renderer {
    fn new(app_name: &str) -> Self;
}

pub struct Handle {
    #[cfg(feature = "directx")]
    api: dx::DxRenderer,
    #[cfg(feature = "vulkan")]
    api: vk::VkRenderer,
}

pub fn new(app_name: &str) -> Handle {
    #[cfg(feature = "directx")]
    return Handle { api: dx::DxRenderer::new(app_name) };
    #[cfg(feature = "vulkan")]
    return Handle { api: vk::VkRenderer::new(app_name) };
}