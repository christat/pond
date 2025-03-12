pub mod imgui;

use crate::{
    app::info::Info,
    ren::{Renderer as RendererTrait, settings::Settings, window::Window},
    scene::Scene,
};
pub struct Renderer {}

impl RendererTrait for Renderer {
    fn new(info: &Info, settings: Settings, window: Window) -> Self {
        todo!()
    }

    fn load_scene(&mut self, scene: &Scene) {
        todo!()
    }

    fn draw(&mut self, imgui: &mut crate::imgui::ImGui) {
        todo!()
    }
}
