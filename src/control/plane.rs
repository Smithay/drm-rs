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

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// A handle to a specific plane
pub struct Handle(u32);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// Information about a specific plane
pub struct Info;
