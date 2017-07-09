use drm_sys;
use control::{self, ResourceHandle, ResourceInfo};
use result::*;
use ffi;

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

impl ::std::fmt::Debug for Id {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "crtc::Id({})", self.0)
    }
}
