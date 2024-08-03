mod renderer;

use std::sync::Arc;

use renderer::Renderer;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowButtons},
};

struct App<'a> {
    window: Option<Arc<Window>>,
    renderer: Renderer<'a>,
}

impl<'a> App<'a> {
    fn new() -> Self {
        Self {
            window: None,
            renderer: Renderer::new(),
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window: Arc<Window> = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_resizable(false)
                    .with_enabled_buttons(WindowButtons::CLOSE | WindowButtons::MINIMIZE),
            )
            .unwrap()
            .into();

        self.window = Some(Arc::clone(&window));

        self.renderer.set_target(
            Arc::clone(&window),
            window.inner_size().width,
            window.inner_size().height,
        );
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                self.renderer.render();
            }
            _ => (),
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
