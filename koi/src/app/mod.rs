pub mod info;

use crate::imgui;
use crate::ren;

use std::ffi::CStr;
use winit::{application, dpi, error, event, event_loop, window};

pub struct Runtime {
    pub window: window::Window,
    pub ren: ren::Handle,
    pub imgui: imgui::ImGui,
}

impl Runtime {
    pub fn new(window: window::Window, ren: ren::Handle, imgui: imgui::ImGui) -> Self {
        Self { window, ren, imgui }
    }

    fn update(&mut self) {
        self.imgui.update(&self.window);
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
        let event_loop =  event_loop::EventLoop::new()?;
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
            .with_window_icon(Some(load_icon(include_bytes!("../../../resources/assets/window/icon.png"))));

        let window: window::Window = event_loop.create_window(window_attributes).expect("koi::App - Failed to create window");
        let mut ren = ren::new(&self.info,&window);
        let imgui = imgui::ImGui::new(&window, &mut ren);

        self.runtime = Some(Runtime::new(window, ren, imgui));
    }


    fn window_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        _id: window::WindowId,
        event: event::WindowEvent
    ) {
        let runtime = self.runtime.as_mut().unwrap();
        runtime.imgui.handle_window_event(&runtime.window, &event);

        match event {
            event::WindowEvent::CloseRequested => {
                self.exit(event_loop);
            },
            event::WindowEvent::RedrawRequested => {
                runtime.update();
            },
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
    window::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("koi::window - failed to open icon")
}