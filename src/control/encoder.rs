//! # Encoder
//!
//! An encoder is a bridge between a CRTC and a connector that takes the pixel
//! data of the CRTC and encodes it into a format the connector understands.

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// A handle to a specific encoder
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

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// Information about a specific encoder
pub struct Info;
