use drm_sys;
use control::{self, ResourceHandle, ResourceInfo};
use result::*;
use ffi;

use control::framebuffer::Id as FbId;
use control::connector::Id as ConId;

#[derive(Clone, Copy, PartialEq, Eq)]
/// A `ResourceHandle` to a crtc.
pub struct Id(control::RawId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The `ResourceInfo` on a crtc.
pub struct Info {
    handle: Id,
    position: (u32, u32),
    // TODO: mode
    fb: control::framebuffer::Id,
    gamma_length: u32
}

impl ResourceHandle for Id {
    type RawHandle = control::RawId;

    fn from_raw(raw: Self::RawHandle) -> Self {
        Id(raw)
    }

    fn as_raw(&self) -> Self::RawHandle {
        self.0
    }
}

impl control::property::LoadProperties for Id {
    const TYPE: u32 = ffi::DRM_MODE_OBJECT_CRTC;
}

impl ResourceInfo for Info {
    type Handle = Id;

    fn load_from_device<T>(device: &T, handle: Id) -> Result<Self>
        where T: control::Device {

        let mut raw: drm_sys::drm_mode_crtc = Default::default();
        raw.crtc_id = handle.0;
        unsafe {
            try!(ffi::ioctl_mode_getcrtc(device.as_raw_fd(), &mut raw));
        }

        let crtc = Self {
            handle: handle,
            position: (raw.x, raw.y),
            fb: control::framebuffer::Id::from_raw(raw.fb_id),
            gamma_length: raw.gamma_size
        };

        Ok(crtc)
    }

    fn handle(&self) -> Self::Handle { self.handle }
}

impl Id {
    pub fn set_on_device<T>(&self, device: &T, fb: FbId, cons: &[ConId],
                            pos: (u32, u32), mode: Option<control::Mode>) -> Result<()>
        where T: control::Device {

        let (x, y) = pos;

        let mut raw: ffi::drm_mode_crtc = Default::default();
        raw.x = x;
        raw.y = y;
        raw.crtc_id = self.as_raw();
        raw.fb_id = fb.as_raw();
        raw.set_connectors_ptr = cons.as_ptr() as u64;
        raw.count_connectors = cons.len() as u32;

        match mode {
            Some(x) => {
                raw.mode = x.mode;
                raw.mode_valid = 1;
            },
            _ => ()
        };

        unsafe {
            try!(ffi::ioctl_mode_setcrtc(device.as_raw_fd(), &mut raw));
        }

        Ok(())
    }
}

impl ::std::fmt::Debug for Id {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "crtc::Id({})", self.0)
    }
}
