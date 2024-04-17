//! # Framebuffer
//!
//! Process specific GPU buffers that can be attached to a plane.

use crate::buffer;
use crate::control;
use drm_ffi as ffi;
use drm_fourcc::{DrmFourcc, DrmModifier};

/// A handle to a framebuffer
#[repr(transparent)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Handle(control::RawResourceHandle);

// Safety: Handle is repr(transparent) over NonZeroU32
unsafe impl bytemuck::ZeroableInOption for Handle {}
unsafe impl bytemuck::PodInOption for Handle {}

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
    pub(crate) buffer: Option<buffer::Handle>,
}

impl std::fmt::Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Framebuffer {}", self.handle.0)
    }
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

    /// Returns the buffer handle of this framebuffer.
    pub fn buffer(&self) -> Option<buffer::Handle> {
        self.buffer
    }
}

/// Information about a framebuffer (with modifiers)
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct PlanarInfo {
    pub(crate) handle: Handle,
    pub(crate) size: (u32, u32),
    pub(crate) pixel_format: DrmFourcc,
    pub(crate) flags: control::FbCmd2Flags,
    pub(crate) buffers: [Option<buffer::Handle>; 4],
    pub(crate) pitches: [u32; 4],
    pub(crate) offsets: [u32; 4],
    pub(crate) modifier: Option<DrmModifier>,
}

impl PlanarInfo {
    /// Returns the handle to this framebuffer.
    pub fn handle(&self) -> Handle {
        self.handle
    }

    /// Returns the size of this framebuffer.
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Returns the pixel format of this framebuffer.
    pub fn pixel_format(&self) -> DrmFourcc {
        self.pixel_format
    }

    /// Returns the flags of this framebuffer.
    pub fn flags(&self) -> control::FbCmd2Flags {
        self.flags
    }

    /// Returns the buffer handles of this framebuffer.
    pub fn buffers(&self) -> [Option<buffer::Handle>; 4] {
        self.buffers
    }

    /// Returns the pitches of this framebuffer.
    pub fn pitches(&self) -> [u32; 4] {
        self.pitches
    }

    /// Returns the offsets of this framebuffer.
    pub fn offsets(&self) -> [u32; 4] {
        self.offsets
    }

    /// Returns the modifier of this framebuffer.
    pub fn modifier(&self) -> Option<DrmModifier> {
        self.modifier
    }
}
