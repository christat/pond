pub mod api;
pub mod settings;
pub mod window;

use crate::{app::info::Info, imgui::ImGui, scene::Scene};
use settings::{Resolution, Settings};
use window::Window;
use winit::window::Window as WindowHandle;

pub trait Renderer {
    fn new(info: &Info, settings: Settings, window: Window) -> Self;
    fn load_scene(&mut self, scene: &Scene);
    fn handle_resize(&mut self, resolution: &Resolution);
    fn draw(&mut self, imgui: &mut ImGui);
}

#[allow(unused)]
pub struct Handle {
    #[cfg(feature = "directx")]
    pub api: api::dx::Renderer,
    #[cfg(feature = "vulkan")]
    pub api: api::vk::Renderer,
}

pub fn new(info: &Info, window_handle: &WindowHandle) -> Handle {
    let window =
        Window::new(window_handle).expect("koi::ren::new - failed to create window handle");
    let settings = Settings::default()
        .resolution(Resolution::new(1920, 1080))
        .buffering(2);

    #[cfg(feature = "directx")]
    let api = api::dx::Renderer::new(info, settings, window);
    #[cfg(feature = "vulkan")]
    let api = api::vk::Renderer::new(info, settings, window);

    Handle { api }
}

impl Handle {
    pub fn load_scene(&mut self, scene: &Scene) {
        self.api.load_scene(scene);
    }

    pub fn handle_resize(&mut self, width: u32, height: u32) {
        self.api.handle_resize(&Resolution::new(width, height));
    }

    pub fn draw(&mut self, imgui: &mut ImGui) {
        self.api.draw(imgui);
    }
}
