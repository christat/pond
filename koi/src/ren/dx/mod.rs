use std::ffi::CStr;

use crate::ren::Renderer;

pub struct DxRenderer {
}

impl Renderer for VkRenderer {
    fn new(name: &CStr) -> Self {
        Self {
        }
    }
}