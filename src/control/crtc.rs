//! # CRTC
//!
//! A CRTC is a display controller provided by your device. It's primary job is
//! to take pixel data and send it to a connector with the proper resolution and
//! frequencies.
//!
//! Specific CRTCs can only be attached to connectors that have an encoder it
//! supports. For example, you can have a CRTC that can not output to analog
//! connectors. These are built in hardware limitations.
//!
//! Each CRTC has a built in plane, which can be attached to a framebuffer. It
//! can also use pixel data from other planes to perform hardware compositing.

use control::{self, ResourceHandle, ResourceInfo};
use result::*;
use ffi;

use control::framebuffer::Handle as FBHandle;
use control::connector::Handle as ConHandle;

/// A [`ResourceHandle`] for a CRTC.
///
/// Like all control resources, every CRTC has a unique `Handle` associated with
/// it. This `Handle` can be used to acquire information about the CRTC (see
/// [`crtc::Info`]) or change the CRTC's state.
///
/// These can be retrieved by using [`ResourceIds::crtcs`].
///
/// [`ResourceHandle`]: ResourceHandle.t.html
/// [`crtc::Info`]: Info.t.html
/// [`ResourceIds::crtcs`]: ResourceIds.t.html#method.crtcs
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Handle(control::RawHandle);

/// A [`ResourceInfo`] for a CRTC.
///
/// [`ResourceInfo`]: ResourceInfo.t.html
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Info {
    handle: Handle,
    position: (u32, u32),
    // TODO: mode
    fb: control::framebuffer::Handle,
    gamma_length: u32
}

impl ResourceHandle for Handle {
    fn from_raw(raw: control::RawHandle) -> Self {
        Handle(raw)
    }

    fn as_raw(&self) -> control::RawHandle {
        self.as_raw()
    }
}

impl control::property::LoadProperties for Handle {
    const TYPE: u32 = ffi::DRM_MODE_OBJECT_CRTC;
}

impl ResourceInfo for Info {
    type Handle = Handle;

    fn load_from_device<T>(device: &T, handle: Handle) -> Result<Self>
        where T: control::Device {

        let crtc = {
            let mut raw: ffi::drm_mode_crtc = Default::default();
            raw.crtc_id = handle.0;
            unsafe {
                try!(ffi::ioctl_mode_getcrtc(device.as_raw_fd(), &mut raw));
            }

            Self {
                handle: handle,
                position: (raw.x, raw.y),
                fb: control::framebuffer::Handle::from_raw(raw.fb_id),
                gamma_length: raw.gamma_size
            }
        };

        Ok(crtc)
    }

    fn handle(&self) -> Self::Handle { self.handle }
}

impl ::std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "crtc::Handle({})", self.0)
    }
}
