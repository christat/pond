pub mod imgui;

use crate::ren::Renderer as RendererTrait;
pub struct Renderer {}

impl RendererTrait for Renderer {
    fn new(
        info: &crate::app::info::Info,
        settings: crate::ren::settings::Settings,
        window: crate::ren::window::Window,
    ) -> Self {
        todo!()
    }

    fn draw(&mut self, imgui: &mut crate::imgui::ImGui) {
        todo!()
    }
}
