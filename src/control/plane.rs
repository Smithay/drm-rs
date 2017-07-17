//! # Plane
//!
//! A plane is an object you can attach framebuffers to for use in displays.

use control::{self, ResourceHandle, ResourceInfo};
use result::*;
use ffi;

/// A [`ResourceHandle`] for a plane.
///
/// Like all control resources, every planehas a unique `Handle` associated with
/// it. This `Handle` can be used to acquire information about the plane
/// (see [`plane::Info`]) or change the plane's state.
///
/// These can be retrieved by using [`PlaneResourceHandles::planes`].
///
/// [`ResourceHandle`]: ResourceHandle.t.html
/// [`plane::Info`]: Info.t.html
/// [`PlaneResourceHandles::planes`]: PlaneResourceHandles.t.html#method.planes
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Handle(control::RawHandle);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The `ResourceInfo` on a plane.
pub struct Info {
    handle: Handle,
    crtc: control::crtc::Handle,
    fb: control::framebuffer::Handle,
    // TODO: count_formats,
    // TODO: possible_crtcs
    gamma_length: u32,
    // TODO: formats
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
    const TYPE: u32 = ffi::DRM_MODE_OBJECT_PLANE;
}

impl ResourceInfo for Info {
    type Handle = Handle;

    fn load_from_device<T>(device: &T, handle: Self::Handle) -> Result<Self>
        where T: control::Device {

        let plane = {
            let mut raw: ffi::drm_mode_get_plane = Default::default();
            raw.plane_id = handle.0;
            unsafe {
                try!(ffi::ioctl_mode_getplane(device.as_raw_fd(), &mut raw));
            }

            Self {
                handle: handle,
                crtc: control::crtc::Handle::from_raw(raw.crtc_id),
                fb: control::framebuffer::Handle::from_raw(raw.fb_id),
                gamma_length: raw.gamma_size,
            }
        };

        Ok(plane)
    }

    fn handle(&self) -> Self::Handle { self.handle }
}

impl ::std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "plane::Handle({})", self.0)
    }
}
