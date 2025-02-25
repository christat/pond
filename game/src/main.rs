use winit::{application::ApplicationHandler, error::EventLoopError, event::WindowEvent, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, window::{Window, WindowId}};

use koi;

struct App {
    name: String,
    window: Option<Window>,
    ren: Option<koi::ren::Handle>,
}

impl App {
    pub fn new(name: String) -> Self {
        Self {
            name: name,
            window: None,
            ren: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_resizable(false)
            .with_title(&self.name);

        let window = event_loop.create_window(window_attributes).expect(&format!("{} - Failed to create window", &self.name));

        self.window = Some(window);
        self.ren = Some(koi::ren::new(&self.name))
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

    let app_name = String::from("Pond");
    let mut app = App::new(app_name);

    match event_loop.run_app(&mut app) {
        Ok(()) => (),
        Err(e) => return Err(e),
    };

    Ok(())
}
