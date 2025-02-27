pub mod info;

use crate::ren;

use winit::{application::ApplicationHandler, dpi::PhysicalSize, error::EventLoopError, event::WindowEvent, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, window::{Window, WindowId}};

pub struct App {
    info: info::Info,
    window: Option<Window>,
    ren: Option<ren::Handle>,
}

impl App {
    pub fn run(&mut self) -> Result<(), EventLoopError> {
        let event_loop =  EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self)
    }
}

pub fn new(name: String) -> App {
    App {
        info: info::new(name, info::make_version(0, 1, 0, 0)),
        window: None,
        ren: None,
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        
        let window_attributes = Window::default_attributes()
            .with_resizable(false)
            .with_inner_size(PhysicalSize::new(1920, 1080))
            .with_title(self.info.app_name.to_string_lossy().into_owned());

        let window = event_loop.create_window(window_attributes).expect(&format!("koi::App - Failed to create window"));
        self.ren = Some(ren::new(&self.info,&window));
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                // Draw.
            }
            _ => (),
        }
    }
}