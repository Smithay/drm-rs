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

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// A handle to a specific CRTC
pub struct Handle(u32);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// Information about a specific CRTC
pub struct Info;
