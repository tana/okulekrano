use std::sync::Arc;

use glium::{backend::Facade, Texture2d};

use super::Capturer;

pub struct FakeCapturer {
    texture: Arc<Texture2d>,
}

impl FakeCapturer {
    pub fn new(facade: &impl Facade) -> Self {
        log::warn!("Fake capture is used. This is only for debugging.");

        let tex = Texture2d::new(
            facade,
            include_bytes!("fake_desktop.bin")
                .chunks(4 * 640)
                .map(|row| {
                    row.chunks(4)
                        .map(|pixel| (pixel[0], pixel[1], pixel[2], pixel[3]))
                        .collect()
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        Self {
            texture: Arc::new(tex),
        }
    }
}

impl Capturer for FakeCapturer {
    fn capture(&mut self) -> Arc<Texture2d> {
        Arc::clone(&self.texture)
    }
}
