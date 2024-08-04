use std::sync::{Arc, Mutex};

use crabgrab::{
    platform::windows::WindowsCaptureConfigExt,
    prelude::{
        CapturableContent, CapturableContentFilter, CaptureConfig, CapturePixelFormat,
        CaptureStream, StreamEvent, WgpuCaptureConfigExt, WgpuVideoFrameExt,
        WgpuVideoFramePlaneTexture,
    },
};
use pollster::block_on;

pub struct Capturer {
    stream: CaptureStream,
    current_texture: Arc<Mutex<Option<Arc<wgpu::Texture>>>>,
}

impl Capturer {
    pub fn new<D: AsRef<wgpu::Device> + Send + Sync + 'static>(wgpu_device_ref: D) -> Self {
        let token = block_on(CaptureStream::request_access(true)).expect("Capture not allowed");

        let content = block_on(CapturableContent::new(CapturableContentFilter::DISPLAYS)).unwrap();
        // TODO
        let display = content.displays().next().unwrap();

        let config = CaptureConfig::with_display(display, CapturePixelFormat::Bgra8888)
            .with_borderless(true)
            .with_wgpu_device(Arc::new(wgpu_device_ref))
            .unwrap();

        let current_texture = Arc::new(Mutex::new(None));

        let stream = {
            let current_texture = Arc::clone(&current_texture);
            CaptureStream::new(token, config, move |event| match event.unwrap() {
                StreamEvent::Video(frame) => {
                    let mut current_texture = current_texture.lock().unwrap();
                    *current_texture = Some(Arc::new(
                        frame
                            .get_wgpu_texture(WgpuVideoFramePlaneTexture::Rgba, None)
                            .unwrap(),
                    ));
                }
                _ => (),
            })
            .unwrap()
        };

        Self {
            stream,
            current_texture: Arc::clone(&current_texture),
        }
    }

    pub fn get_current_texture(&self) -> Arc<wgpu::Texture> {
        Arc::clone(
            self
                .current_texture
                .lock()
                .unwrap()
                .as_ref()
                .expect("No frame captured yet"),
        )
    }
}

impl Drop for Capturer {
    fn drop(&mut self) {
        self.stream.stop().unwrap();
    }
}
