use crate::ren::{self, Handle as Renderer, api::vk::Renderer as vkRenderer};

use imgui::{Context, FontSource, StyleColor};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;
use winit::window::Window as WindowHandle;

pub const IMGUI_DEFAULT_IMAGE_COUNT: u32 = 3;
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

        Self::load_fonts(&mut context);
        Self::apply_styles(&mut context);

        let renderer = ImGuiRenderer::new(&mut context, ren, IMGUI_DEFAULT_IMAGE_COUNT);

        Self {
            context,
            platform,
            renderer,
            open: false,
            now: Instant::now(),
        }
    }

    fn load_fonts(context: &mut Context) {
        let fonts = context.fonts();
        fonts.clear();
        let size_pixels: f32 = 16.0;
        fonts.add_font(&[
            FontSource::TtfData {
                data: include_bytes!("../../../assets/fonts/Roboto-Light.ttf"),
                size_pixels,
                config: None,
            },
            FontSource::TtfData {
                data: include_bytes!("../../../assets/fonts/Hack-Regular.ttf"),
                size_pixels,
                config: None,
            },
        ]);
    }

    fn apply_styles(context: &mut Context) {
        let style = context.style_mut();

        // styles from https://github.com/ocornut/imgui/issues/707#issuecomment-678611331
        style[StyleColor::Text] = [1.0, 10.0, 1.0, 1.0];
        style[StyleColor::TextDisabled] = [0.5, 0.5, 0.5, 1.0];
        style[StyleColor::WindowBg] = [0.13, 0.14, 0.14, 1.0];
        style[StyleColor::ChildBg] = [0.13, 0.14, 0.14, 1.0];
        style[StyleColor::PopupBg] = [0.13, 0.14, 0.14, 1.0];
        style[StyleColor::Border] = [0.43, 0.43, 0.5, 0.5];
        style[StyleColor::BorderShadow] = [0.0, 0.0, 0.0, 1.0];
        style[StyleColor::FrameBg] = [0.25, 0.25, 0.25, 1.0];
        style[StyleColor::FrameBgHovered] = [0.38, 0.38, 0.38, 1.0];
        style[StyleColor::FrameBgActive] = [0.67, 0.67, 0.67, 0.39];
        style[StyleColor::TitleBg] = [0.08, 0.08, 0.09, 1.0];
        style[StyleColor::TitleBgActive] = [0.08, 0.08, 0.09, 1.0];
        style[StyleColor::TitleBgCollapsed] = [0.0, 0.0, 0.0, 0.51];
        style[StyleColor::MenuBarBg] = [0.14, 0.14, 0.14, 1.0];
        style[StyleColor::ScrollbarBg] = [0.02, 0.02, 0.02, 0.53];
        style[StyleColor::ScrollbarGrab] = [0.31, 0.31, 0.31, 1.0];
        style[StyleColor::ScrollbarGrabHovered] = [0.41, 0.41, 0.41, 1.0];
        style[StyleColor::ScrollbarGrabActive] = [0.51, 0.51, 0.51, 1.0];
        style[StyleColor::CheckMark] = [0.11, 0.64, 0.92, 1.0];
        style[StyleColor::SliderGrab] = [0.11, 0.64, 0.92, 1.0];
        style[StyleColor::SliderGrabActive] = [0.08, 0.5, 0.72, 1.0];
        style[StyleColor::Button] = [0.25, 0.25, 0.25, 1.0];
        style[StyleColor::ButtonHovered] = [0.38, 0.38, 0.38, 1.0];
        style[StyleColor::ButtonActive] = [0.67, 0.67, 0.67, 0.39];
        style[StyleColor::Header] = [0.22, 0.22, 0.2, 1.0];
        style[StyleColor::HeaderHovered] = [0.25, 0.25, 0.5, 1.0];
        style[StyleColor::HeaderActive] = [0.67, 0.67, 0.67, 0.39];
        style[StyleColor::Separator] = [0.43, 0.43, 0.5, 0.5];
        style[StyleColor::SeparatorHovered] = [0.41, 0.42, 0.44, 1.0];
        style[StyleColor::SeparatorActive] = [0.26, 0.59, 0.98, 0.95];
        style[StyleColor::ResizeGrip] = [0.0, 0.0, 0.0, 0.0];
        style[StyleColor::ResizeGripHovered] = [0.29, 0.30, 0.31, 0.67];
        style[StyleColor::ResizeGripActive] = [0.26, 0.59, 0.98, 0.95];
        style[StyleColor::Tab] = [0.08, 0.08, 0.09, 0.83];
        style[StyleColor::TabHovered] = [0.33, 0.34, 0.36, 0.83];
        style[StyleColor::TabActive] = [0.23, 0.23, 0.24, 1.0];
        style[StyleColor::TabUnfocused] = [0.08, 0.08, 0.09, 1.0];
        style[StyleColor::TabUnfocusedActive] = [0.13, 0.14, 0.15, 1.0];
        style[StyleColor::PlotLines] = [0.61, 0.61, 0.61, 1.0];
        style[StyleColor::PlotLinesHovered] = [1.0, 0.43, 0.35, 1.0];
        style[StyleColor::PlotHistogram] = [0.9, 0.7, 0.0, 1.0];
        style[StyleColor::PlotHistogramHovered] = [1.0, 0.6, 0.0, 1.0];
        style[StyleColor::TextSelectedBg] = [0.26, 0.59, 0.98, 0.35];
        style[StyleColor::DragDropTarget] = [0.11, 0.64, 0.92, 1.0];
        style[StyleColor::NavHighlight] = [0.26, 0.59, 0.98, 1.0];
        style[StyleColor::NavWindowingHighlight] = [1.0, 1.0, 1.0, 0.7];
        style[StyleColor::NavWindowingDimBg] = [0.8, 0.8, 0.8, 0.2];
        style[StyleColor::ModalWindowDimBg] = [0.8, 0.8, 0.8, 0.35];
        style.grab_rounding = 2.3;
        style.frame_rounding = 2.3;
    }

    pub fn handle_window_event(
        &mut self,
        window_handle: &WindowHandle,
        event: &winit::event::WindowEvent,
    ) {
        self.platform
            .handle_window_event(self.context.io_mut(), window_handle, event);
    }

    pub fn update(&mut self, window_handle: &WindowHandle, ren: &mut Renderer) {
        self.tick();
        self.platform
            .prepare_frame(self.context.io_mut(), &window_handle)
            .expect("koi::imgui - failed to prepare ImGui frame");

        let ui = self.context.frame();
        // ui.show_demo_window(&mut self.open);

        ui.window("Compute Effect")
            .size([300.0, 50.0], imgui::Condition::FirstUseEver)
            .build(|| {
                #[cfg(feature = "vulkan")]
                {
                    let draw_manager = &mut ren.api.draw_manager;
                    let compute_effects_len = draw_manager.compute_effects.len();
                    let compute_effect =
                        &mut draw_manager.compute_effects[draw_manager.compute_effect_index];

                    ui.text(format!("Compute Shader: {}\n", compute_effect.name));

                    if ui.button("Toggle Compute Shader") {
                        draw_manager.compute_effect_index = draw_manager.compute_effect_index + 1;
                        if draw_manager.compute_effect_index >= compute_effects_len {
                            draw_manager.compute_effect_index = 0;
                        }
                    }

                    ui.color_picker4(
                        "Push Constant 0",
                        compute_effect.push_constants.data_0.as_mut(),
                    );
                    if compute_effect.name == "gradient" {
                        ui.color_picker4(
                            "Push Constant 1",
                            compute_effect.push_constants.data_1.as_mut(),
                        );
                    }
                }
            });

        self.platform.prepare_render(ui, window_handle);
    }

    #[cfg(feature = "vulkan")]
    pub fn draw(&mut self, api: &mut vkRenderer, command_buffer: ash::vk::CommandBuffer) {
        self.renderer.draw(&mut self.context, api, command_buffer);
    }

    fn tick(&mut self) {
        let now = Instant::now();
        self.context.io_mut().update_delta_time(now - self.now);
        self.now = now;
    }

    pub fn drop(&mut self, ren: &mut Renderer) {
        self.renderer.drop(ren);
    }
}

pub struct ImGuiRenderer {
    #[cfg(feature = "directx")]
    api: ren::api::dx::imgui::Renderer,
    #[cfg(feature = "vulkan")]
    api: ren::api::vk::imgui::Renderer,
}

impl ImGuiRenderer {
    pub fn new(context: &mut imgui::Context, ren: &mut Renderer, image_count: u32) -> Self {
        #[cfg(feature = "directx")]
        let api = ren::api::dx::imgui::Renderer::new(context, &mut ren.api, image_count);
        #[cfg(feature = "vulkan")]
        let api = ren::api::vk::imgui::Renderer::new(context, &mut ren.api, image_count);

        Self { api }
    }

    #[cfg(feature = "vulkan")]
    pub fn draw(
        &mut self,
        context: &mut imgui::Context,
        api: &mut vkRenderer,
        command_buffer: ash::vk::CommandBuffer,
    ) {
        self.api.draw(context, api, command_buffer);
    }

    pub fn drop(&mut self, ren: &mut Renderer) {
        self.api.drop(&mut ren.api);
    }
}
