//! # Plane
//!
//! Attachment point for a Framebuffer.
//!
//! A Plane is a resource that can have a framebuffer attached to it, either for
//! hardware compositing or displaying directly to a screen. There are three
//! types of planes available for use:
//!
//! * Primary - A CRTC's built-in plane. When attaching a framebuffer to a CRTC,
//! it is actually being attached to this kind of plane.
//!
//! * Overlay - Can be overlayed on top of a primary plane, utilizing extremely
//! fast hardware compositing.
//!
//! * Cursor - Similar to an overlay plane, these are typically used to display
//! cursor type objects.

use ffi::{self, mode::RawHandle};
use control::{ResourceHandle, ResourceInfo, Device};
use control::crtc;
use control::framebuffer;
use result::Result;

#[derive(Copy, Clone, Hash, PartialEq, Eq, From, Into)]
/// A [ResourceHandle](../ResourceHandle.t.html) representing a plane.
pub struct Handle(RawHandle);

impl ResourceHandle for Handle {
    type Info = Info;

    fn get_info<T: Device>(device: &T, handle: Self) -> Result<Info> {
        let mut t = ffi::mode::GetPlane::default();
        t.as_mut().plane_id = handle.into();
        t.cmd(device.as_raw_fd())?;
        Ok(Info(t))
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
/// A [ResourceInfo](../ResourceInfo.t.html) object about a plane.
pub struct Info(ffi::mode::GetPlane);

impl ResourceInfo for Info {
    type Handle = Handle;

    fn handle(&self) -> Handle {
        Handle::from(self.0.as_ref().plane_id)
    }
}

impl Info {
    /// Returns the current CRTC this plane is attached to.
    pub fn current_crtc(&self) -> Option<crtc::Handle> {
        crtc::Handle::from_checked(self.0.as_ref().crtc_id)
    }

    /// Returns the current framebuffer attached to this plane.
    pub fn current_framebuffer(&self) -> Option<framebuffer::Handle> {
        framebuffer::Handle::from_checked(self.0.as_ref().fb_id)
    }

    /// Returns a filter that can be used to determine which CRTC resources
    /// are compatible with this plane.
    pub fn possible_crtcs(&self) -> u32 {
        self.0.as_ref().possible_crtcs
    }

    /// Returns the size of the gamma buffers.
    pub fn gamma_size(&self) -> u32 {
        self.0.as_ref().gamma_size
    }

    /// Returns a list of supported formats.
    pub fn formats(&self) -> &[u32] {
        self.formats()
    }
}

/*
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum PresentFlag {
    TopField = (0 << 1),
    BottomField = (1 << 1),
}

/// Attaches a framebuffer to a CRTC's plane for hardware-composing
pub fn set<T>(
    plane: Handle,
    device: &T,
    crtc: crtc::Handle,
    framebuffer: framebuffer::Handle,
    flags: PresentFlag,
    crtc_rect: iRect,
    src_rect: uRect,
) -> Result<()>
where
    T: control::Device,
{
    let mut raw: ffi::drm_mode_set_plane = Default::default();

    raw.plane_id = plane.into();
    raw.crtc_id = crtc.into();
    raw.fb_id = framebuffer.into();
    raw.flags = flags as u32;
    raw.crtc_x = (crtc_rect.0).0;
    raw.crtc_y = (crtc_rect.0).1;
    raw.crtc_w = (crtc_rect.1).0;
    raw.crtc_h = (crtc_rect.1).1;
    raw.src_x = (src_rect.0).0;
    raw.src_y = (src_rect.0).1;
    raw.src_w = (src_rect.1).0;
    raw.src_h = (src_rect.1).1;

    unsafe {
        ffi::ioctl_mode_setplane(device.as_raw_fd(), &mut raw)?;
    }

    Ok(())
}
*/
