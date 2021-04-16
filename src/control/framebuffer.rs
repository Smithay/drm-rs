//! # Framebuffer
//!
//! Process specific GPU buffers that can be attached to a plane.

use control;
use drm_ffi as ffi;

/// A handle to a framebuffer
#[repr(transparent)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Handle(control::RawResourceHandle);

impl From<Handle> for control::RawResourceHandle {
    fn from(handle: Handle) -> Self {
        handle.0
    }
}

impl From<Handle> for u32 {
    fn from(handle: Handle) -> Self {
        handle.0.into()
    }
}

impl From<control::RawResourceHandle> for Handle {
    fn from(handle: control::RawResourceHandle) -> Self {
        Handle(handle)
    }
}

impl control::ResourceHandle for Handle {
    const FFI_TYPE: u32 = ffi::DRM_MODE_OBJECT_FB;
}

impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("framebuffer::Handle").field(&self.0).finish()
    }
}

/// Information about a framebuffer
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Info {
    pub(crate) handle: Handle,
    pub(crate) size: (u32, u32),
    pub(crate) pitch: u32,
    pub(crate) bpp: u32,
    pub(crate) depth: u32,
    pub(crate) buffer: u32,
}

impl Info {
    /// Returns the handle to this framebuffer.
    pub fn handle(&self) -> Handle {
        self.handle
    }

    /// Returns the size of this framebuffer.
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Returns the pitch of this framebuffer.
    pub fn pitch(&self) -> u32 {
        self.pitch
    }

    /// Returns the bits-per-pixel of this framebuffer.
    pub fn bpp(&self) -> u32 {
        self.bpp
    }

    /// Returns the depth of this framebuffer.
    pub fn depth(&self) -> u32 {
        self.depth
    }
}
