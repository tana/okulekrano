use drm_fourcc::DrmFourcc;
use glium::{glutin::surface::WindowSurface, Display, Texture2d};
use std::{
    collections::HashMap,
    sync::Arc,
};
use texture::DmabufTexture;
use wayland_client::{
    protocol::{
        wl_buffer::{self, WlBuffer},
        wl_output::{self, WlOutput},
        wl_registry::{self, WlRegistry},
    },
    Connection, Dispatch, EventQueue, Proxy,
};
use wayland_protocols::wp::linux_dmabuf::zv1::client::{
    zwp_linux_buffer_params_v1::{self, ZwpLinuxBufferParamsV1},
    zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1,
};
use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1::{self, ZwlrScreencopyFrameV1},
    zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
};

mod texture;

pub struct Capturer {
    queue: EventQueue<State>,
    state: State,
    glium_display: Arc<Display<WindowSurface>>,
    texture: Option<DmabufTexture>,
    wl_buffer: Option<WlBuffer>,
}

#[derive(Default)]
struct State {
    all_outputs: Vec<WlOutput>,
    output_from_name: HashMap<String, WlOutput>,
    output: Option<WlOutput>,
    manager: Option<ZwlrScreencopyManagerV1>,
    dmabuf_factory: Option<ZwpLinuxDmabufV1>,
    buf_width: u32,
    buf_height: u32,
    buf_format: u32,
    ready: bool,
    released: bool,
}

impl Capturer {
    pub fn new(
        glium_display: Arc<Display<WindowSurface>>,
        output_to_capture: Option<&str>,
    ) -> Self {
        let conn = Connection::connect_to_env().unwrap();
        let display = conn.display();

        let mut queue = conn.new_event_queue();

        let mut state = State::default();

        // (1) Retrieve global objects such as wl_display.
        // It will fire `wl_registry::global` event.
        let _registry = display.get_registry(&queue.handle(), ());
        queue.roundtrip(&mut state).unwrap();

        // Receive names (and other properties) of each output
        while state.output_from_name.len() < state.all_outputs.len() {
            queue.blocking_dispatch(&mut state).unwrap();
        }

        for output_name in state.output_from_name.keys() {
            println!("Output: {}", output_name);
        }

        // (3) Select output.
        let output = if let Some(output_to_capture) = output_to_capture {
            let output = state
                .output_from_name
                .iter()
                .find(|(name, _)| *name == output_to_capture)
                .unwrap();
            println!("Using {}", output.0);

            output.1.clone()
        } else {
            state.all_outputs[0].clone()
        };
        state.output = Some(output);

        Self {
            queue,
            state,
            glium_display,
            texture: None,
            wl_buffer: None,
        }
    }

    pub fn get_current_texture(&mut self) -> Arc<Texture2d> {
        self.state.ready = false;
        self.state.released = false;

        // (4) Request the compositor to capture the screen.
        // It will fire several events such as `zwlr_screencopy_frame_v1::buffer_done`.
        let frame = self.state.manager.as_ref().unwrap().capture_output(
            1, // include mouse cursor
            self.state.output.as_ref().unwrap(),
            &self.queue.handle(),
            (),
        );
        self.queue.roundtrip(&mut self.state).unwrap();

        if self.texture.is_none() {
            let width = self.state.buf_width;
            let height = self.state.buf_height;

            // (5) Query size and format of the buffer.
            let dmabuf_params = self
                .state
                .dmabuf_factory
                .as_ref()
                .unwrap()
                .create_params(&self.queue.handle(), ());
            self.queue.roundtrip(&mut self.state).unwrap();

            println!(
                "{:?} {} {}",
                DrmFourcc::try_from(self.state.buf_format),
                width,
                height
            );

            // (6) Create a buffer on GPU.
            self.texture = Some(DmabufTexture::new(
                Texture2d::empty(
                    self.glium_display.as_ref(),
                    self.state.buf_width,
                    self.state.buf_height,
                )
                .unwrap(),
            ));

            // (7) Create Wayland buffer from the buffer.
            let texture = self.texture.as_ref().unwrap();
            dmabuf_params.add(texture.fd(), 0, texture.offset(), texture.stride(), 0, 0);
            self.wl_buffer = Some(dmabuf_params.create_immed(
                width as i32,
                height as i32,
                self.state.buf_format,
                zwp_linux_buffer_params_v1::Flags::empty(),
                &self.queue.handle(),
                (),
            ));
            self.queue.roundtrip(&mut self.state).unwrap();
            println!("{:?}", self.wl_buffer);
        }

        // (8) Copy the captured frame into the buffer.
        frame.copy(self.wl_buffer.as_ref().unwrap());
        self.queue.flush().unwrap();
        while !self.state.ready {
            self.queue.blocking_dispatch(&mut self.state).unwrap();
        }

        frame.destroy();
        self.queue.flush().unwrap();
        while !self.state.released {
            self.queue.blocking_dispatch(&mut self.state).unwrap();
        }

        self.texture.as_ref().unwrap().texture()
    }
}

impl Dispatch<WlRegistry, (), Self> for State {
    fn event(
        state: &mut Self,
        proxy: &WlRegistry,
        event: <WlRegistry as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        // (2) Store received global objects.
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => {
                if interface == WlOutput::interface().name {
                    state
                        .all_outputs
                        .push(proxy.bind(name, version, qhandle, ()));
                } else if interface == ZwlrScreencopyManagerV1::interface().name {
                    state.manager = Some(proxy.bind(name, version, qhandle, ()));
                } else if interface == ZwpLinuxDmabufV1::interface().name {
                    state.dmabuf_factory = Some(proxy.bind(name, version, qhandle, ()));
                }
            }
            _ => (),
        }
    }
}

impl Dispatch<WlOutput, (), Self> for State {
    fn event(
        state: &mut Self,
        proxy: &WlOutput,
        event: <WlOutput as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            wl_output::Event::Name { name } => {
                state.output_from_name.insert(name, proxy.clone());
            }
            _ => (),
        }
    }
}

impl Dispatch<ZwlrScreencopyManagerV1, (), Self> for State {
    fn event(
        _state: &mut Self,
        _proxy: &ZwlrScreencopyManagerV1,
        _event: <ZwlrScreencopyManagerV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrScreencopyFrameV1, (), Self> for State {
    fn event(
        state: &mut Self,
        _proxy: &ZwlrScreencopyFrameV1,
        event: <ZwlrScreencopyFrameV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        // (4) Store information about capturing frame.
        match event {
            zwlr_screencopy_frame_v1::Event::LinuxDmabuf {
                format,
                width,
                height,
            } => {
                state.buf_format = format;
                state.buf_width = width;
                state.buf_height = height;
            }
            zwlr_screencopy_frame_v1::Event::Failed => {
                panic!("Capture failed");
            }
            zwlr_screencopy_frame_v1::Event::Ready { .. } => {
                state.ready = true;
            }
            _ => (),
        }
    }
}

impl Dispatch<ZwpLinuxDmabufV1, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpLinuxDmabufV1,
        _event: <ZwpLinuxDmabufV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpLinuxBufferParamsV1, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpLinuxBufferParamsV1,
        _event: <ZwpLinuxBufferParamsV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlBuffer, ()> for State {
    fn event(
        state: &mut Self,
        _proxy: &WlBuffer,
        event: <WlBuffer as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            wl_buffer::Event::Release => {
                state.released = true;
            }
            _ => (),
        }
    }
}
