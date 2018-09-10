//! # Framebuffer
//!
//! Process specific GPU buffers that can be attached to a plane.

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// A handle to a specific framebuffer
pub struct Handle(u32);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// Information about a specific framebuffer
pub struct Info;
