pub mod info;

use crate::ren;

use std::ffi::CStr;
use winit::{application::ApplicationHandler, dpi::PhysicalSize, error::EventLoopError, event::WindowEvent, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, window::{Window, WindowId}};

pub struct Runtime {
    window: Window,
    ren: ren::Handle
}

impl Runtime {
    pub fn new(window: Window, ren: ren::Handle) -> Self {
        Self { window, ren }
    }

    fn update(&mut self) {
        self.ren.draw();
    }

    fn redraw(&mut self) {
        self.window.request_redraw();
    }
}

pub struct App<'a> {
    info: info::Info<'a>,
    runtime: Option<Runtime>,
}

impl App<'_> {
    pub fn run(&mut self) -> Result<(), EventLoopError> {
        let event_loop =  EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self)
    }
}

pub fn new(name: &CStr) -> App {
    App {
        info: info::new(name, info::make_version(0, 1, 0, 0)),
        runtime: None,
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        
        let window_attributes = Window::default_attributes()
            .with_resizable(false)
            .with_inner_size(PhysicalSize::new(1920, 1080))
            .with_title(self.info.app_name.to_string_lossy().into_owned());

        let window = event_loop.create_window(window_attributes).expect("koi::App - Failed to create window");
        let ren = ren::new(&self.info,&window);

        self.runtime = Some(Runtime::new(window, ren));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let runtime = self.runtime.as_mut().unwrap();
                runtime.update();
                runtime.redraw();
            }
            _ => (),
        }
    }
}