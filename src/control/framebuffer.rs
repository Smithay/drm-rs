//! # CFramebuffer
//!
//! A framebuffer is pixel data that can be attached to a plane.

use control::{self, ResourceHandle, ResourceInfo};
use result::*;
use ffi;

/// A [`ResourceHandle`] for a framebuffer.
///
/// Like all control resources, every framebuffer has a unique `Handle`
/// associated with it. This `Handle` can be used to acquire information about
/// the framebuffer (see [`framebuffer::Info`]) or change the framebuffer's
/// state.
///
/// These can be retrieved by using [`ResourceIds::framebuffers`], but most
/// often you will create your own using [`Device::create_framebuffer`].
///
/// [`ResourceHandle`]: ResourceHandle.t.html
/// [`framebuffer::Info`]: Info.t.html
/// [`ResourceIds::framebuffers`]: ResourceIds.t.html#method.framebuffers
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Handle(control::RawHandle);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Info {
    handle: Handle,
    size: (u32, u32),
    pitch: u32,
    bpp: u8,
    // TODO: Gem handle?
    depth: u8
}

impl ResourceHandle for Handle {
    fn from_raw(raw: control::RawHandle) -> Self {
        Handle(raw)
    }

    fn as_raw(&self) -> control::RawHandle {
        self.0
    }
}

impl control::property::LoadProperties for Handle {
    const TYPE: u32 = ffi::DRM_MODE_OBJECT_FB;
}

impl ResourceInfo for Info {
    type Handle = Handle;

    fn load_from_device<T>(device: &T, handle: Self::Handle) -> Result<Self>
        where T: control::Device {

        let framebuffer = {
            let mut raw: ffi::drm_mode_fb_cmd = Default::default();
            raw.fb_id = handle.as_raw();
            unsafe {
                try!(ffi::ioctl_mode_getfb(device.as_raw_fd(), &mut raw));
            }

            Self {
                handle: handle,
                size: (raw.width, raw.height),
                pitch: raw.pitch,
                bpp: raw.bpp as u8,
                depth: raw.depth as u8
            }
        };

        Ok(framebuffer)
    }

    fn handle(&self) -> Self::Handle { self.handle }
}

/// Creates a framebuffer from a [`Buffer`], returning
/// [`framebuffer::Info`].
///
/// [`framebuffer::Info`]: framebuffer.Handle.html
pub fn create<T, U>(device: &T, buffer: &U) -> Result<Info>
    where T: control::Device, U: super::super::buffer::Buffer {

    let framebuffer = {
        let mut raw: ffi::drm_mode_fb_cmd = Default::default();
        let (w, h) = buffer.size();
        raw.width = w;
        raw.height = h;
        raw.pitch = buffer.pitch();
        raw.bpp = buffer.bpp() as u32;
        raw.depth = buffer.depth() as u32;
        raw.handle = buffer.handle().as_raw();

        unsafe {
            try!(ffi::ioctl_mode_addfb(device.as_raw_fd(), &mut raw));
        }

        Info {
            handle: Handle::from_raw(raw.fb_id),
            size: (raw.width, raw.height),
            pitch: raw.pitch,
            bpp: raw.bpp as u8,
            depth: raw.depth as u8
        }
    };

    Ok(framebuffer)
}

impl ::std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "framebuffer::Handle({})", self.0)
    }
}
