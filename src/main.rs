mod capturer;
mod renderer;
mod winit_app;

fn main() {
    env_logger::init();

    winit_app::run();
}
