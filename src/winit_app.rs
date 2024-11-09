use std::{num::NonZero, sync::Arc};

use crate::{config::Config, glasses::GlassesController, renderer::Renderer};
use glium::glutin::{
    self,
    config::ConfigTemplateBuilder,
    context::{ContextApi, ContextAttributesBuilder},
    prelude::{GlDisplay, NotCurrentGlContext},
    surface::{SurfaceAttributesBuilder, WindowSurface},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Window, WindowAttributes, WindowButtons},
};

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    config: Config,
    glasses: GlassesController,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window: Arc<Window> = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_resizable(false)
                    .with_enabled_buttons(WindowButtons::CLOSE | WindowButtons::MINIMIZE),
            )
            .unwrap()
            .into();
        let rwh = window.window_handle().unwrap().as_raw();
        let rdh = window.display_handle().unwrap().as_raw();
        let width = NonZero::new(window.inner_size().width).unwrap();
        let height = NonZero::new(window.inner_size().height).unwrap();

        let glutin_display = glutin::display::Display::Egl(unsafe {
            glutin::api::egl::display::Display::new(rdh).unwrap()
        });
        let config_template = ConfigTemplateBuilder::new().build();
        let config = unsafe { glutin_display.find_configs(config_template) }
            .unwrap()
            .next()
            .unwrap();
        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(Some(glutin::context::Version::new(3, 1))))
            .build(Some(rwh));
        let context = unsafe {
            glutin_display
                .create_context(&config, &context_attributes)
                .unwrap()
        };
        let surface_attributes =
            SurfaceAttributesBuilder::<WindowSurface>::new().build(rwh, width, height);
        let window_surface =
            unsafe { glutin_display.create_window_surface(&config, &surface_attributes) }.unwrap();

        let context = context.make_current(&window_surface).unwrap();

        let display = glium::Display::new(context, window_surface).unwrap();

        self.window = Some(window);

        self.renderer = Some(Renderer::new(Arc::new(display), &self.config));
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
                if let Some(ref mut renderer) = self.renderer {
                    self.glasses.update_pose();

                    renderer.render(&self.glasses.camera_mat_left());
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window.as_ref().unwrap().request_redraw();
    }
}

pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        renderer: None,
        config: confy::load("okulekrano", None).unwrap(),
        glasses: GlassesController::new(),
    };

    event_loop.run_app(&mut app).unwrap();
}
