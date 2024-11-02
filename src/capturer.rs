use drm_fourcc::DrmFourcc;
use gbm::{AsRaw as _, BufferObject, BufferObjectFlags};
use glium::{glutin::surface::WindowSurface, texture::ExternalTexture, Display};
use khronos_egl as egl;
use std::{ffi::c_void, os::fd::{AsFd, OwnedFd}, sync::Arc};
use wayland_client::{
    protocol::{
        wl_buffer::WlBuffer,
        wl_output::WlOutput,
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

pub struct Capturer<T: AsFd> {
    queue: EventQueue<State>,
    state: State,
    glium_display: Arc<Display<WindowSurface>>,
    gbm: gbm::Device<T>,
    egl: Arc<egl::Instance<egl::Static>>,
    bo: Option<BufferObject<()>>,
    bo_fd: Option<OwnedFd>,
    texture: Option<Arc<ExternalTexture>>,
    wl_buffer: Option<WlBuffer>,
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

impl<T: AsFd> Capturer<T> {
    pub fn new(
        glium_display: Arc<Display<WindowSurface>>,
        gbm: gbm::Device<T>,
        egl: Arc<egl::Instance<egl::Static>>,
    ) -> Self {
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

        Self {
            queue,
            state,
            glium_display,
            gbm,
            egl,
            bo: None,
            bo_fd: None,
            texture: None,
            wl_buffer: None,
        }
    }

    pub fn get_current_texture(&mut self) -> Arc<ExternalTexture> {
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
            let bo = self
                .gbm
                .create_buffer_object::<()>(
                    width,
                    height,
                    DrmFourcc::try_from(self.state.buf_format).unwrap(),
                    BufferObjectFlags::empty(),
                )
                .unwrap();
            let modifier: u64 = bo.modifier().unwrap().into();
            let offset = bo.offset(0).unwrap();
            let stride = bo.stride().unwrap();

            self.bo_fd = Some(bo.fd_for_plane(0).unwrap());
            self.bo = Some(bo);
            let fd = self.bo_fd.as_ref().unwrap().as_fd();

            // Create GL texture from GBM buffer
            self.texture = Some(Arc::new(texture::texture_from_dmabuf(
                self.glium_display.as_ref(),
                &self.egl,
                self.gbm.as_raw_mut() as *mut c_void,
                &fd,
                width,
                height,
                stride,
                offset,
                self.state.buf_format,
                modifier,
            )));

            // (7) Create Wayland buffer from the buffer.
            dmabuf_params.add(
                fd,
                0,
                offset,
                stride,
                ((modifier & 0xFFFFFFFF00000000) >> 32) as u32,
                (modifier & 0xFFFFFFFF) as u32,
            );
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
        self.queue.roundtrip(&mut self.state).unwrap();

        Arc::clone(self.texture.as_ref().unwrap())
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
        _state: &mut Self,
        _proxy: &WlBuffer,
        _event: <WlBuffer as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}
