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

use control;
use ffi;

use util::*;

/// A handle to a plane
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Handle(u32);

impl control::Handle for Handle {
    const OBJ_TYPE: u32 = ffi::DRM_MODE_OBJECT_PLANE;

    fn from_raw(raw: u32) -> Self {
        Handle(raw)
    }

    fn into_raw(self) -> u32 {
        let Handle(raw) = self;
        raw
    }
}

/// Information about a plane
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Info {
    pub(crate) handle: Handle,
    pub(crate) crtc: Option<control::crtc::Handle>,
    pub(crate) fb: Option<control::framebuffer::Handle>,
    pub(crate) pos_crtcs: u32,
    pub(crate) formats: Buffer4x32<u32>,
}

impl Info {
    /// Returns the handle to this plane.
    pub fn handle(&self) -> Handle {
        self.handle
    }

    /// Returns the CRTC this plane is attached to.
    pub fn crtc(&self) -> Option<control::crtc::Handle> {
        self.crtc
    }

    /// Returns the framebuffer this plane is attached to.
    pub fn framebuffer(&self) -> Option<control::framebuffer::Handle> {
        self.fb
    }

    /// Returns the formats this plane supports.
    pub fn formats(&self) -> &[u32] {
        unsafe { self.formats.as_slice() }
    }
}
