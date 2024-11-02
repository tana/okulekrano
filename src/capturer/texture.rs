use std::{
    ffi::c_void,
    os::fd::{AsFd, AsRawFd},
};

use khronos_egl as egl;
use glium::{backend::Facade, texture::ExternalTexture};

// https://registry.khronos.org/EGL/extensions/EXT/EGL_EXT_image_dma_buf_import.txt
const EGL_LINUX_DMA_BUF_EXT: u32 = 0x3270;
const EGL_LINUX_DRM_FOURCC_EXT: u32 = 0x3271;
const EGL_DMA_BUF_PLANE0_FD_EXT: u32 = 0x3272;
const EGL_DMA_BUF_PLANE0_OFFSET_EXT: u32 = 0x3273;
const EGL_DMA_BUF_PLANE0_PITCH_EXT: u32 = 0x3274;
// https://registry.khronos.org/EGL/extensions/EXT/EGL_EXT_image_dma_buf_import_modifiers.txt
const EGL_DMA_BUF_PLANE0_MODIFIER_LO_EXT: u32 = 0x3443;
const EGL_DMA_BUF_PLANE0_MODIFIER_HI_EXT: u32 = 0x3444;

pub fn texture_from_dmabuf<F: Facade>(
    facade: &F,
    egl: &egl::Instance<egl::Static>,
    display: *mut c_void,
    fd: &impl AsFd,
    width: u32,
    height: u32,
    stride: u32,
    offset: u32,
    fourcc: u32,
    modifier: u64,
) -> ExternalTexture {
    let egl_image = unsafe {
        let display = egl.get_display(display).unwrap();
        let (major, minor) = egl.initialize(display).unwrap();
        println!("EGL initialized {}.{}", major, minor);

        egl.create_image(
            display,
            egl::Context::from_ptr(egl::NO_CONTEXT),
            EGL_LINUX_DMA_BUF_EXT,
            egl::ClientBuffer::from_ptr(0 as *mut c_void),
            &[
                egl::WIDTH as usize,
                width as usize,
                egl::HEIGHT as usize,
                height as usize,
                EGL_LINUX_DRM_FOURCC_EXT as usize,
                fourcc as usize,
                EGL_DMA_BUF_PLANE0_FD_EXT as usize,
                fd.as_fd().as_raw_fd() as usize,
                EGL_DMA_BUF_PLANE0_OFFSET_EXT as usize,
                offset as usize,
                EGL_DMA_BUF_PLANE0_PITCH_EXT as usize,
                stride as usize,
                EGL_DMA_BUF_PLANE0_MODIFIER_HI_EXT as usize,
                ((modifier & 0xFFFFFFFF00000000) >> 32) as usize,
                EGL_DMA_BUF_PLANE0_MODIFIER_LO_EXT as usize,
                (modifier & 0xFFFFFFFF) as usize,
                egl::ATTRIB_NONE,
            ],
        )
        .unwrap()
    };

    ExternalTexture::new(facade, egl_image.as_ptr())
}
