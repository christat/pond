use crate::ren::{self, Handle as Renderer, api::vk::Renderer as vkRenderer};

use imgui::Context;
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

        let renderer = ImGuiRenderer::new(&mut context, ren, IMGUI_DEFAULT_IMAGE_COUNT);

        Self {
            context,
            platform,
            renderer,
            open: false,
            now: Instant::now(),
        }
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
            .size([300.0, 100.0], imgui::Condition::FirstUseEver)
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

                    ui.slider(
                        "Push Constant 0 x",
                        0.0,
                        1.0,
                        &mut compute_effect.push_constants.data_0.x,
                    );
                    ui.slider(
                        "Push Constant 0 y",
                        0.0,
                        1.0,
                        &mut compute_effect.push_constants.data_0.y,
                    );
                    ui.slider(
                        "Push Constant 0 z",
                        0.0,
                        1.0,
                        &mut compute_effect.push_constants.data_0.z,
                    );
                    ui.slider(
                        "Push Constant 0 w",
                        0.0,
                        1.0,
                        &mut compute_effect.push_constants.data_0.w,
                    );

                    ui.slider(
                        "Push Constant 1 x",
                        0.0,
                        1.0,
                        &mut compute_effect.push_constants.data_1.x,
                    );
                    ui.slider(
                        "Push Constant 1 y",
                        0.0,
                        1.0,
                        &mut compute_effect.push_constants.data_1.y,
                    );
                    ui.slider(
                        "Push Constant 1 z",
                        0.0,
                        1.0,
                        &mut compute_effect.push_constants.data_1.z,
                    );
                    ui.slider(
                        "Push Constant 1 w",
                        0.0,
                        1.0,
                        &mut compute_effect.push_constants.data_1.w,
                    );
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
