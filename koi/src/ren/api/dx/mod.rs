use crate::ren::Renderer as RendererTrait;

use std::ffi::CStr;

pub struct Renderer {}

impl RendererTrait for Renderer {
    fn new(info: &Info, window: Window) -> Self {
        Self {}
    }

    fn draw(&mut self) {}
}
