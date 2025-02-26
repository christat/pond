use std::ffi::CStr;

use crate::ren::Renderer as RendererTrait;

pub struct Renderer {
}

impl RendererTrait for Renderer {
    fn new(app_name: &str) -> Self {
        Self {
        }
    }
}