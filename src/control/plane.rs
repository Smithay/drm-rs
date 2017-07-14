use drm_sys;
use control::{self, ResourceHandle, ResourceInfo};
use result::*;
use ffi;

#[derive(Clone, Copy, PartialEq, Eq)]
/// A `ResourceHandle` to a plane.
pub struct Id(control::RawId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The `ResourceInfo` on a plane.
pub struct Info {
    handle: Id,
    crtc: control::crtc::Id,
    fb: control::framebuffer::Id,
    // TODO: count_formats,
    // TODO: possible_crtcs
    gamma_length: u32,
    // TODO: formats
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
    const TYPE: u32 = ffi::DRM_MODE_OBJECT_PLANE;
}

impl ResourceInfo for Info {
    type Handle = Id;

    fn load_from_device<T>(device: &T, handle: Self::Handle) -> Result<Self>
        where T: control::Device {

        let mut raw: drm_sys::drm_mode_get_plane = Default::default();
        raw.plane_id = handle.0;
        unsafe {
            try!(ffi::ioctl_mode_getplane(device.as_raw_fd(), &mut raw));
        }

        let plane = Self {
            handle: handle,
            crtc: control::crtc::Id::from_raw(raw.crtc_id),
            fb: control::framebuffer::Id::from_raw(raw.fb_id),
            gamma_length: raw.gamma_size,
        };

        Ok(plane)
    }

    fn handle(&self) -> Self::Handle { self.handle }
}

impl ::std::fmt::Debug for Id {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "plane::Id({})", self.0)
    }
}
