use crate::imgui::ImGuiRendering;

pub struct Renderer {

}

impl Renderer {
    pub fn new(api: &mut super::Renderer) -> Self {

        


        Self {}
    }
}

impl ImGuiRendering for Renderer {
    fn draw(&mut self, draw_data: &imgui::DrawData) {
        
    }
}