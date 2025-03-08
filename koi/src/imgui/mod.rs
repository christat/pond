use crate::ren::{self, Handle as Renderer};

use imgui::Context;
use imgui_winit_support::{WinitPlatform, HiDpiMode};
use std::time::Instant;
use winit::window::Window as WindowHandle;

pub struct ImGuiRenderer {
    #[cfg(feature = "directx")]
    api: ren::api::dx::imgui::Renderer,
    #[cfg(feature = "vulkan")]
    api: ren::api::vk::imgui::Renderer,
}

pub trait ImGuiRendering {
    fn draw(&mut self, draw_data: &imgui::DrawData);
}

impl ImGuiRenderer {
    pub fn new(context: &mut imgui::Context, ren: &mut Renderer) -> Self {
        #[cfg(feature = "directx")]
        let api = ren::api::dx::imgui::Renderer::new(context, &mut ren.api);
        #[cfg(feature = "vulkan")]
        let api = ren::api::vk::imgui::Renderer::new(context, &mut ren.api);

        Self { api }
    }

    pub fn draw(&mut self, context: &mut imgui::Context) {
        self.api.draw(context.render());
    }
}

pub struct ImGui {
    pub context: Context,
    pub platform: WinitPlatform,
    pub renderer: ImGuiRenderer,

    pub open: bool,
    pub now: Instant,
}

impl ImGui {
    pub fn new(window_handle: &WindowHandle, ren: &mut Renderer) -> Self {
        let mut context = Context::create();
        let mut platform = WinitPlatform::new(&mut context);

        platform.attach_window(context.io_mut(), window_handle, HiDpiMode::Default);

        let renderer = ImGuiRenderer::new(&mut context, ren);

        Self { 
            context,
            platform,
            renderer,
            open: false,
            now: Instant::now()
        }
    }

    pub fn handle_window_event(&mut self, window_handle: &WindowHandle, event: &winit::event::WindowEvent) {
        self.platform.handle_window_event(self.context.io_mut(), window_handle, event);
    }

    pub fn draw(&mut self, window_handle: &WindowHandle) {
        self.tick();
        self.platform.prepare_frame(self.context.io_mut(), &window_handle).expect("koi::imgui - failed to prepare ImGui frame");

        let ui = self.context.frame();
        ui.show_demo_window(&mut self.open);

        self.platform.prepare_render(ui, window_handle);
        self.renderer.draw(&mut self.context);
    }

    fn tick(&mut self) {
        let now = Instant::now();
        self.context.io_mut().update_delta_time(now - self.now);
        self.now = now;
    }
}