use std::{
    num::NonZero,
    sync::{mpsc, Arc},
    time::Duration,
};

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
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Fullscreen, Window, WindowAttributes, WindowButtons},
};

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    config: Config,
    glasses: Option<GlassesController>,
    stop_receiver: mpsc::Receiver<()>,
}

impl App {
    fn new(config: Config) -> Self {
        let (stop_sender, stop_receiver) = mpsc::channel();
        ctrlc::set_handler(move || {
            log::info!("Closing");
            stop_sender.send(()).unwrap()
        })
        .unwrap();

        Self {
            window: None,
            renderer: None,
            config,
            glasses: None,
            stop_receiver,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.glasses = Some(GlassesController::new());
        log::info!("Waiting for mode change...");
        std::thread::sleep(Duration::from_secs_f32(
            self.config.glasses.delay_after_mode_switch,
        ));

        let monitor = if let Some(ref monitor_name) = self.config.glasses.monitor_name {
            let monitor = event_loop
                .available_monitors()
                .find(|monitor| match monitor.name() {
                    Some(name) => name == monitor_name.as_str(),
                    None => false,
                });
            log::info!(
                "Displaying to monitor {} ({}x{})",
                monitor.as_ref().unwrap().name().unwrap(),
                monitor.as_ref().unwrap().size().width,
                monitor.as_ref().unwrap().size().height,
            );
            monitor
        } else {
            None
        };

        let window_attrs = if self.config.glasses.window_mode {
            WindowAttributes::default()
                .with_inner_size(PhysicalSize::new(800, 450))
                .with_resizable(false)
                .with_enabled_buttons(WindowButtons::CLOSE | WindowButtons::MINIMIZE)
        } else {
            WindowAttributes::default()
                .with_fullscreen(Some(Fullscreen::Borderless(monitor.clone())))
                .with_inner_size(monitor.clone().unwrap().size()) // Note: this is necessary, even in fullscreen mode
        };

        let window: Arc<Window> = event_loop.create_window(window_attrs).unwrap().into();
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
                // Close window if Ctrl-C is pressed in the terminal
                if self.stop_receiver.try_recv().is_ok() {
                    event_loop.exit();
                }

                if let Some(ref mut renderer) = self.renderer {
                    let glasses = self.glasses.as_mut().unwrap();
                    glasses.update_pose();

                    renderer.render(&glasses);
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

    let mut app = App::new(confy::load("okulekrano", None).unwrap());

    event_loop.run_app(&mut app).unwrap();
}
