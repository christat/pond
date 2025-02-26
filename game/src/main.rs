use winit::{application::ApplicationHandler, dpi::PhysicalSize, error::EventLoopError, event::WindowEvent, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, window::{Window, WindowId}};

use koi;
use koi::info::Info;

struct App {
    info: Info,
    window: Option<Window>,
    ren: Option<koi::ren::Handle>,
}

impl App {
    pub fn new(info: Info) -> Self {
        Self {
            info: info,
            window: None,
            ren: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_resizable(false)
            .with_inner_size(PhysicalSize::new(1920, 1080))
            .with_title(self.info.app_name.to_string_lossy().into_owned());

        let window = event_loop.create_window(window_attributes).expect(&format!("{:?} - Failed to create window", &self.info.app_name));

        self.window = Some(window);
        self.ren = Some(koi::ren::new(&self.info))
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

fn main() -> Result<(), EventLoopError> {
    let event_loop = match EventLoop::new() {
        Ok(event_loop) => event_loop,
        Err(e) => return Err(e),
    };
    event_loop.set_control_flow(ControlFlow::Poll);

    let info = koi::info::new(String::from("Pond"), koi::info::make_version(0, 1, 0, 0));
    let mut app = App::new(info);

    match event_loop.run_app(&mut app) {
        Ok(()) => (),
        Err(e) => return Err(e),
    };

    Ok(())
}
