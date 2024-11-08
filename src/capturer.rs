use std::sync::Arc;

use glium::Texture2d;

pub mod fake;
mod texture;
pub mod wayland;

pub trait Capturer {
    fn capture(&mut self) -> Arc<Texture2d>;
}
