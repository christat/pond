pub mod info;

use crate::imgui;
use crate::ren;
use crate::scene;
use crate::scene::Scene;

use std::ffi::CStr;
use std::path::Path;
use winit::dpi::PhysicalSize;
use winit::{application, dpi, error, event, event_loop, window};

pub struct Runtime {
    pub window: window::Window,
    pub ren: ren::Handle,
    pub imgui: imgui::ImGui,
    pub scene: Option<Scene>,
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.imgui.drop(&mut self.ren);
    }
}

impl Runtime {
    pub fn new(window: window::Window, ren: ren::Handle, imgui: imgui::ImGui) -> Self {
        Self {
            window,
            ren,
            imgui,
            scene: None,
        }
    }

    pub fn load_scene(&mut self, path: &Path) {
        let scene = scene::load(path);
        self.ren.load_scene(&scene);
        self.scene = Some(scene);
    }

    pub fn handle_resize(&mut self, width: u32, height: u32) {
        self.ren.handle_resize(width, height);
    }

    fn update(&mut self) {
        self.imgui.update(&self.window, &mut self.ren);
        self.ren.draw(&mut self.imgui);
        self.window.request_redraw();
    }
}

pub struct App<'a> {
    pub info: info::Info<'a>,
    pub runtime: Option<Runtime>,
}

impl App<'_> {
    pub fn run(&mut self) -> Result<(), error::EventLoopError> {
        let event_loop = event_loop::EventLoop::new()?;
        event_loop.set_control_flow(event_loop::ControlFlow::Poll);
        event_loop.run_app(self)
    }

    pub fn exit(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        event_loop.exit();
    }
}

pub fn new(name: &CStr) -> App {
    App {
        info: info::new(name, info::make_version(0, 1, 0, 0)),
        runtime: None,
    }
}

impl application::ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        let window_attributes = window::Window::default_attributes()
            .with_resizable(false)
            .with_inner_size(dpi::PhysicalSize::new(1920, 1080))
            .with_title(self.info.app_name.to_string_lossy().into_owned())
            .with_window_icon(Some(load_icon(include_bytes!(
                "../../../assets/window/icon.png"
            ))));

        let window: window::Window = event_loop
            .create_window(window_attributes)
            .expect("koi::App - Failed to create window");

        let mut ren = ren::new(&self.info, &window);
        let imgui = imgui::ImGui::new(&window, &mut ren);

        self.runtime = Some(Runtime::new(window, ren, imgui));
    }

    fn window_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        _id: window::WindowId,
        event: event::WindowEvent,
    ) {
        let runtime = self.runtime.as_mut().unwrap();
        runtime.imgui.handle_window_event(&runtime.window, &event);

        match event {
            event::WindowEvent::Resized(PhysicalSize { width, height }) => {
                runtime.handle_resize(width, height);
            }
            event::WindowEvent::CloseRequested => {
                self.exit(event_loop);
            }
            event::WindowEvent::RedrawRequested => {
                if runtime.scene.is_none() {
                    runtime.load_scene(Path::new("assets/models/test.glb"));
                }
                runtime.update();
            }
            _ => {}
        }
    }
}

fn load_icon(bytes: &[u8]) -> window::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(bytes).unwrap().into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    window::Icon::from_rgba(icon_rgba, icon_width, icon_height)
        .expect("koi::window - failed to open icon")
}
