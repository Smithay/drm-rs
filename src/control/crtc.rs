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

use control::framebuffer::Handle as FramebufferHandle;
use control::Mode;

/// A handle to a specific CRTC
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Handle(u32);

impl From<u32> for Handle {
    fn from(raw: u32) -> Self {
        Handle(raw)
    }
}

impl Into<u32> for Handle {
    fn into(self) -> u32 {
        self.0
    }
}

/// Information about a specific CRTC
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Info {
    pub(crate) handle: Handle,
    pub(crate) position: (u32, u32),
    pub(crate) mode: Option<Mode>,
    pub(crate) fb: Option<FramebufferHandle>,
    pub(crate) gamma_length: u32,
}

impl Info {
    /// The CRTC's handle
    pub fn handle(&self) -> Handle {
        self.handle
    }

    /// The CRTC's position
    pub fn position(&self) -> (u32, u32) {
        self.position
    }

    /// The current mode this CRTC is using
    pub fn mode(&self) -> Option<Mode> {
        self.mode
    }

    /// The framebuffer currently attached to this CRTC
    pub fn framebuffer(&self) -> Option<FramebufferHandle> {
        self.fb
    }

    /// The length of the gamma buffer
    pub fn gamma_length(&self) -> u32 {
        self.gamma_length
    }
}
