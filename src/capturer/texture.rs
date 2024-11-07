use std::{os::{fd::{AsFd, BorrowedFd, FromRawFd, OwnedFd}, raw::c_void}, sync::Arc};

use glium::{GlObject, Texture2d};
use khronos_egl::{ClientBuffer, EGLDisplay, EGLImage, ATTRIB_NONE, GL_TEXTURE_2D};

type EglExportDmabufImageMesaFunc = unsafe extern "C" fn(EGLDisplay, EGLImage, *mut i32, *mut i32, *mut i32) -> bool;

// DMABUF-capable texture
pub struct DmabufTexture {
    texture: Arc<Texture2d>,
    fd: OwnedFd,
    offset: u32,
    stride: u32,
}

impl DmabufTexture {
    pub fn new(texture: Texture2d) -> Self {
        let egl = khronos_egl::Instance::new(khronos_egl::Static);

        let egl_ctx = egl.get_current_context().unwrap();
        let display = egl.get_current_display().unwrap();

        let egl_image = unsafe {
            egl.create_image(
                display,
                egl_ctx,
                GL_TEXTURE_2D as u32,
                ClientBuffer::from_ptr(texture.get_id() as *mut c_void),
                &[ATTRIB_NONE],
            )
            .unwrap()
        };

        let mut offset: i32 = 0;
        let mut stride: i32 = 0;

        let fd = unsafe {
            let export_dmabuf_image: EglExportDmabufImageMesaFunc = std::mem::transmute(egl.get_proc_address("eglExportDMABUFImageMESA").unwrap());
            let mut fd = 0;
            if !export_dmabuf_image(display.as_ptr(), egl_image.as_ptr(), &mut fd, &mut stride, &mut offset) {
                panic!("DMABUF export failed")
            }

            OwnedFd::from_raw_fd(fd)
        };

        Self { texture: Arc::new(texture), fd, offset: offset as u32, stride: stride as u32 }
    }

    pub fn texture(&self) -> Arc<Texture2d> {
        Arc::clone(&self.texture)
    }

    pub fn fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }

    pub fn offset(&self) -> u32 {
        self.offset
    }

    pub fn stride(&self) -> u32 {
        self.stride
    }
}
