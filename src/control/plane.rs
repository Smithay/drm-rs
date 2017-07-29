//! # Plane
//!
//! A plane is an object you can attach framebuffers to for use in displays.

use control::{self, ResourceHandle, ResourceInfo, crtc, framebuffer};
use result::*;
use ffi;
use ::{iRect, uRect};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PresentFlag {
    TopField = (0 << 1),
    BottomField = (1 << 1),
}

pub fn set<T>(plane: Handle, device: &T, crtc: crtc::Handle, framebuffer: framebuffer::Handle, flags: PresentFlag, crtc_rect: iRect, src_rect: uRect) -> Result<()>
    where T: control::Device
{
    let mut raw : ffi::drm_mode_set_plane = Default::default();

    raw.plane_id = plane.as_raw();
    raw.crtc_id = crtc.as_raw();
    raw.fb_id = framebuffer.as_raw();
    raw.flags = flags as u32;
    raw.crtc_x = (crtc_rect.0).0;
    raw.crtc_y = (crtc_rect.0).1;
    raw.crtc_w = (crtc_rect.1).0;
    raw.crtc_h = (crtc_rect.1).1;
    raw.src_x = (src_rect.0).0;
    raw.src_y = (src_rect.0).1;
    raw.src_w = (src_rect.1).0;
    raw.src_h = (src_rect.1).1;

    unsafe { ffi::ioctl_mode_setplane(device.as_raw_fd(), &mut raw)?; }

    Ok(())
}

impl ::std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "plane::Handle({})", self.0)
    }
}
