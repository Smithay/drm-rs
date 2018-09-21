//! # Framebuffer
//!
//! Process specific GPU buffers that can be attached to a plane.

/// A handle to an framebuffer
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
