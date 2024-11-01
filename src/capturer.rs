use std::sync::Arc;
use wayland_client::{
    protocol::{
        wl_output::WlOutput,
        wl_registry::{self, WlRegistry},
    },
    Connection, Dispatch, EventQueue, Proxy,
};
use wayland_protocols::wp::linux_dmabuf::zv1::client::{
    zwp_linux_buffer_params_v1::ZwpLinuxBufferParamsV1, zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1,
};
use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1::{self, ZwlrScreencopyFrameV1},
    zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
};

pub struct Capturer {
    queue: EventQueue<State>,
    state: State,
}

#[derive(Default)]
struct State {
    all_outputs: Vec<WlOutput>,
    output: Option<WlOutput>,
    manager: Option<ZwlrScreencopyManagerV1>,
    dmabuf_factory: Option<ZwpLinuxDmabufV1>,
    buf_width: u32,
    buf_height: u32,
    buf_format: u32,
}

impl Capturer {
    pub fn new<D: AsRef<wgpu::Device> + Send + Sync + 'static>(wgpu_device_ref: D) -> Self {
        let conn = Connection::connect_to_env().unwrap();
        let display = conn.display();

        let mut queue = conn.new_event_queue();

        let mut state = State::default();

        // (1) Retrieve global objects such as wl_display.
        // It will fire `wl_registry::global` event.
        let _registry = display.get_registry(&queue.handle(), ());
        queue.roundtrip(&mut state).unwrap();

        // (3) Select output.
        state.output = Some(state.all_outputs[0].clone()); // TODO

        Self { queue, state }
    }

    pub fn get_current_texture(&mut self) -> Arc<wgpu::Texture> {
        // (4) Request the compositor to capture the screen.
        // It will fire several events such as `zwlr_screencopy_frame_v1::buffer_done`.
        self.state.manager.as_ref().unwrap().capture_output(
            1, // include mouse cursor
            self.state.output.as_ref().unwrap(),
            &self.queue.handle(),
            (),
        );
        self.queue.roundtrip(&mut self.state).unwrap();

        // (5) Create GPU buffer (first capture only).
        let dmabuf_params = self
            .state
            .dmabuf_factory
            .as_ref()
            .unwrap()
            .create_params(&self.queue.handle(), ());
        self.queue.roundtrip(&mut self.state).unwrap();

        println!("{:?}", dmabuf_params);

        todo!()
    }
}

impl Drop for Capturer {
    fn drop(&mut self) {
        // self.stream.stop().unwrap();
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
                println!("{}", interface);
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
        _state: &mut Self,
        _proxy: &WlOutput,
        _event: <WlOutput as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
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
        qhandle: &wayland_client::QueueHandle<Self>,
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
            zwlr_screencopy_frame_v1::Event::BufferDone => {
                println!("done")
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
    ) {}
}

impl Dispatch<ZwpLinuxBufferParamsV1, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpLinuxBufferParamsV1,
        _event: <ZwpLinuxBufferParamsV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {}
}
