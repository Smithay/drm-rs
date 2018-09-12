//! # Encoder
//!
//! An encoder is a bridge between a CRTC and a connector that takes the pixel
//! data of the CRTC and encodes it into a format the connector understands.

use control::crtc::Handle as CrtcHandle;
use ffi;

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
pub struct Info {
    pub(crate) handle: Handle,
    pub(crate) enc_type: Type,
    pub(crate) crtc: Option<CrtcHandle>,
    pub(crate) pos_crtcs: u32,
    pub(crate) pos_clones: u32,
}

impl Info {
    /// The encoder's handle
    pub fn handle(&self) -> Handle {
        self.handle
    }

    /// The encoder's type
    pub fn kind(&self) -> Type {
        self.enc_type
    }

    /// Gets the handle of the CRTC currently used if it exists
    pub fn crtc(&self) -> Option<CrtcHandle> {
        self.crtc
    }

    /// Returns a filter for the possible CRTCs that can use this encoder
    pub fn possible_crtcs(&self) -> () {
        unimplemented!()
    }

    /// Returns a filter for the possible encoders that clones this one
    pub fn possible_clones(&self) -> () {
        unimplemented!()
    }
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// The type of encoder.
pub enum Type {
    None,
    DAC,
    TMDS,
    LVDS,
    TVDAC,
    Virtual,
    DSI,
    DPMST,
    DPI,
}

impl From<u32> for Type {
    fn from(n: u32) -> Self {
        match n {
            ffi::DRM_MODE_ENCODER_NONE => Type::None,
            ffi::DRM_MODE_ENCODER_DAC => Type::DAC,
            ffi::DRM_MODE_ENCODER_TMDS => Type::TMDS,
            ffi::DRM_MODE_ENCODER_LVDS => Type::LVDS,
            ffi::DRM_MODE_ENCODER_TVDAC => Type::TVDAC,
            ffi::DRM_MODE_ENCODER_VIRTUAL => Type::Virtual,
            ffi::DRM_MODE_ENCODER_DSI => Type::DSI,
            ffi::DRM_MODE_ENCODER_DPMST => Type::DPMST,
            ffi::DRM_MODE_ENCODER_DPI => Type::DPI,
            _ => Type::None,
        }
    }
}

impl Into<u32> for Type {
    fn into(self) -> u32 {
        match self {
            Type::None => ffi::DRM_MODE_ENCODER_NONE,
            Type::DAC => ffi::DRM_MODE_ENCODER_DAC,
            Type::TMDS => ffi::DRM_MODE_ENCODER_TMDS,
            Type::LVDS => ffi::DRM_MODE_ENCODER_LVDS,
            Type::TVDAC => ffi::DRM_MODE_ENCODER_TVDAC,
            Type::Virtual => ffi::DRM_MODE_ENCODER_VIRTUAL,
            Type::DSI => ffi::DRM_MODE_ENCODER_DSI,
            Type::DPMST => ffi::DRM_MODE_ENCODER_DPMST,
            Type::DPI => ffi::DRM_MODE_ENCODER_DPI,
        }
    }
}

