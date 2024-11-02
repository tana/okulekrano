use std::{ffi::c_void, os::fd::{AsFd, AsRawFd}, sync::OnceLock};

use glow::HasContext;
use khronos_egl as egl;

type EglImageTargetTexture2dOesType = unsafe extern "C" fn(u32, *mut c_void);

static EGL_IMAGE_TARGET_TEXTURE_2D_OES: OnceLock<EglImageTargetTexture2dOesType> = OnceLock::new();

// https://registry.khronos.org/EGL/extensions/EXT/EGL_EXT_image_dma_buf_import.txt
const EGL_LINUX_DRM_FOURCC_EXT: u32 = 0x3271;
const EGL_DMA_BUF_PLANE0_FD_EXT: u32 = 0x3272;
const EGL_DMA_BUF_PLANE0_OFFSET_EXT: u32 = 0x3273;
const EGL_DMA_BUF_PLANE0_PITCH_EXT: u32 = 0x3274;
// https://registry.khronos.org/OpenGL/extensions/OES/OES_EGL_image_external.txt
const TEXTURE_EXTERNAL_OES: u32 = 0x8D65;

pub struct ExternalTexture {
    texture: glow::NativeTexture,
}

impl ExternalTexture {
    pub fn from_dmabuf(egl: &egl::Instance<egl::Static>, glow: &glow::Context, fd: &impl AsFd, width: u32, height: u32, stride: u32, offset: u32, fourcc: u32) -> Self {
        let egl_image_target_texture_2d_oes = EGL_IMAGE_TARGET_TEXTURE_2D_OES.get_or_init(|| unsafe {
            std::mem::transmute(egl.get_proc_address("EGLImageTargetTexture2DOES"))
        });

        let display = egl.get_current_display().unwrap();

        let egl_image = unsafe {
            egl.create_image(
                display,
                egl::Context::from_ptr(egl::NO_CONTEXT),
                EGL_LINUX_DRM_FOURCC_EXT,
                egl::ClientBuffer::from_ptr(0 as *mut c_void),
                &[
                    egl::WIDTH as usize, width as usize,
                    egl::HEIGHT as usize, height as usize,
                    EGL_LINUX_DRM_FOURCC_EXT as usize, fourcc as usize,
                    EGL_DMA_BUF_PLANE0_FD_EXT as usize, fd.as_fd().as_raw_fd() as usize,
                    EGL_DMA_BUF_PLANE0_OFFSET_EXT as usize, offset as usize,
                    EGL_DMA_BUF_PLANE0_PITCH_EXT as usize, stride as usize,
                    egl::ATTRIB_NONE,
                ],
            )
            .unwrap()
        };

        let texture = unsafe {
            let texture = glow.create_texture().unwrap();
            glow.bind_texture(TEXTURE_EXTERNAL_OES, Some(texture));
            egl_image_target_texture_2d_oes(TEXTURE_EXTERNAL_OES, egl_image.as_ptr());
            glow.bind_texture(TEXTURE_EXTERNAL_OES, None);

            texture
        };

        Self {
            texture,
        }
    }

    pub fn with_bind<R>(&self, glow: &glow::Context, unit: u32, f: impl FnOnce() -> R) -> R {
        unsafe { glow.bind_texture_unit(unit, Some(self.texture)); }
        let ret = f();
        unsafe { glow.bind_texture_unit(unit, None); }

        ret
    }
}