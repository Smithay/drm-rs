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
//! Each CRTC has a built in plane, which can have a framebuffer attached to it,
//! but they can also use pixel data from other planes to perform hardware
//! compositing.

use crate::control;
use drm_ffi as ffi;

/// A handle to a specific CRTC
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
    const FFI_TYPE: u32 = ffi::DRM_MODE_OBJECT_CRTC;
}

impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("crtc::Handle").field(&self.0).finish()
    }
}

/// Information about a specific CRTC
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Info {
    pub(crate) handle: Handle,
    pub(crate) position: (u32, u32),
    pub(crate) mode: Option<control::Mode>,
    pub(crate) fb: Option<control::framebuffer::Handle>,
    pub(crate) gamma_length: u32,
}

impl std::fmt::Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CRTC {}", self.handle.0)
    }
}

impl Info {
    /// Returns the handle to this CRTC.
    pub fn handle(&self) -> Handle {
        self.handle
    }

    /// Returns the position of the CRTC.
    pub fn position(&self) -> (u32, u32) {
        self.position
    }

    /// Returns the current mode of the CRTC.
    pub fn mode(&self) -> Option<control::Mode> {
        self.mode
    }

    /// Returns the framebuffer currently attached to this CRTC.
    pub fn framebuffer(&self) -> Option<control::framebuffer::Handle> {
        self.fb
    }

    /// Returns the size of the gamma LUT.
    pub fn gamma_length(&self) -> u32 {
        self.gamma_length
    }
}
