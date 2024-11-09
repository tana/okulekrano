mod capturer;
mod config;
mod glasses;
mod renderer;
mod winit_app;

extern crate nalgebra as na;

fn main() {
    env_logger::init();

    winit_app::run();
}
