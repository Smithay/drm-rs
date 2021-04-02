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
use drm_ffi as ffi;

/// A handle to a plane
#[repr(transparent)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Handle(control::RawResourceHandle);

impl Into<control::RawResourceHandle> for Handle {
    fn into(self) -> control::RawResourceHandle {
        self.0
    }
}

impl Into<u32> for Handle {
    fn into(self) -> u32 {
        self.0.into()
    }
}

impl From<control::RawResourceHandle> for Handle {
    fn from(handle: control::RawResourceHandle) -> Self {
        Handle(handle)
    }
}

impl control::ResourceHandle for Handle {
    const FFI_TYPE: u32 = ffi::DRM_MODE_OBJECT_PLANE;
}

impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("plane::Handle")
            .field(&self.0)
            .finish()
    }
}

/// Information about a plane
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Info {
    pub(crate) handle: Handle,
    pub(crate) crtc: Option<control::crtc::Handle>,
    pub(crate) fb: Option<control::framebuffer::Handle>,
    pub(crate) pos_crtcs: u32,
    pub(crate) formats: [u32; 8],
    pub(crate) fmt_len: usize
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

    /// Returns a filter for supported crtcs of this plane.
    ///
    /// Use with [control::ResourceHandles::filter_crtcs]
    /// to receive a list of crtcs.
    pub fn possible_crtcs(&self) -> control::CrtcListFilter {
        control::CrtcListFilter(self.pos_crtcs)
    }

    /// Returns the framebuffer this plane is attached to.
    pub fn framebuffer(&self) -> Option<control::framebuffer::Handle> {
        self.fb
    }

    /// Returns the formats this plane supports.
    pub fn formats(&self) -> &[u32] {
        let buf_len = std::cmp::min(self.formats.len(), self.fmt_len);
        &self.formats[..buf_len]
    }
}
