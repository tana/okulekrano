use std::{
    ffi::c_void,
    os::fd::{AsFd, AsRawFd},
};

use glium::{
    backend::Facade, texture::{Dimensions, MipmapsOption, UncompressedFloatFormat}, Texture2d
};
use khronos_egl as egl;

// https://registry.khronos.org/EGL/extensions/EXT/EGL_EXT_image_dma_buf_import.txt
const EGL_LINUX_DMA_BUF_EXT: u32 = 0x3270;
const EGL_LINUX_DRM_FOURCC_EXT: u32 = 0x3271;
const EGL_DMA_BUF_PLANE0_FD_EXT: u32 = 0x3272;
const EGL_DMA_BUF_PLANE0_OFFSET_EXT: u32 = 0x3273;
const EGL_DMA_BUF_PLANE0_PITCH_EXT: u32 = 0x3274;
// https://registry.khronos.org/EGL/extensions/EXT/EGL_EXT_image_dma_buf_import_modifiers.txt
const EGL_DMA_BUF_PLANE0_MODIFIER_LO_EXT: u32 = 0x3443;
const EGL_DMA_BUF_PLANE0_MODIFIER_HI_EXT: u32 = 0x3444;

type GlEglImageTargetTexture2dOesType = unsafe extern "C" fn(u32, *const c_void);

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
) -> Texture2d {
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

    unsafe {
        let gl_egl_image_target_texture_2d_oes: GlEglImageTargetTexture2dOesType =
            std::mem::transmute(
                egl.get_proc_address("glEGLImageTargetTexture2DOES")
                    .unwrap(),
            );
        gl::load_with(|s| egl.get_proc_address(s).unwrap() as *const _);

        let id = facade.get_context().exec_in_context(|| {
            let mut id = 0;
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl_egl_image_target_texture_2d_oes(gl::TEXTURE_2D, egl_image.as_ptr());
            gl::BindTexture(gl::TEXTURE_2D, 0);

            id
        });

        Texture2d::from_id(
            facade,
            UncompressedFloatFormat::U8U8U8U8,
            id,
            true,
            MipmapsOption::NoMipmap,
            Dimensions::Texture2d { width, height },
        )
    }
}
