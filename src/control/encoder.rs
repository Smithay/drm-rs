//! # Encoder
//!
//! An encoder is a bridge between a CRTC and a connector that takes the pixel
//! data of the CRTC and encodes it into a format the connector understands.

use crate::control;
use drm_ffi as ffi;

/// A handle to an encoder
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
    const FFI_TYPE: u32 = ffi::DRM_MODE_OBJECT_ENCODER;
}

impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("encoder::Handle").field(&self.0).finish()
    }
}

/// Information about an encoder
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Info {
    pub(crate) handle: Handle,
    pub(crate) enc_type: Kind,
    pub(crate) crtc: Option<control::crtc::Handle>,
    pub(crate) pos_crtcs: u32,
    pub(crate) pos_clones: u32,
}

impl std::fmt::Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Encoder {}", self.handle.0)
    }
}

impl Info {
    /// Returns the handle to this encoder.
    pub fn handle(&self) -> Handle {
        self.handle
    }

    /// Returns the `Kind` of encoder this is.
    pub fn kind(&self) -> Kind {
        self.enc_type
    }

    /// Returns a handle to the CRTC this encoder is attached to.
    pub fn crtc(&self) -> Option<control::crtc::Handle> {
        self.crtc
    }

    /// Returns a filter for the possible CRTCs that can use this encoder.
    ///
    /// Use with [`control::ResourceHandles::filter_crtcs`]
    /// to receive a list of crtcs.
    pub fn possible_crtcs(&self) -> control::CrtcListFilter {
        control::CrtcListFilter(self.pos_crtcs)
    }

    /// Returns a filter for the possible encoders that clones this one.
    pub fn possible_clones(&self) {
        unimplemented!()
    }
}

/// The type of encoder.
#[allow(missing_docs)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Kind {
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

impl From<u32> for Kind {
    fn from(n: u32) -> Self {
        match n {
            ffi::DRM_MODE_ENCODER_NONE => Kind::None,
            ffi::DRM_MODE_ENCODER_DAC => Kind::DAC,
            ffi::DRM_MODE_ENCODER_TMDS => Kind::TMDS,
            ffi::DRM_MODE_ENCODER_LVDS => Kind::LVDS,
            ffi::DRM_MODE_ENCODER_TVDAC => Kind::TVDAC,
            ffi::DRM_MODE_ENCODER_VIRTUAL => Kind::Virtual,
            ffi::DRM_MODE_ENCODER_DSI => Kind::DSI,
            ffi::DRM_MODE_ENCODER_DPMST => Kind::DPMST,
            ffi::DRM_MODE_ENCODER_DPI => Kind::DPI,
            _ => Kind::None,
        }
    }
}

impl From<Kind> for u32 {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::None => ffi::DRM_MODE_ENCODER_NONE,
            Kind::DAC => ffi::DRM_MODE_ENCODER_DAC,
            Kind::TMDS => ffi::DRM_MODE_ENCODER_TMDS,
            Kind::LVDS => ffi::DRM_MODE_ENCODER_LVDS,
            Kind::TVDAC => ffi::DRM_MODE_ENCODER_TVDAC,
            Kind::Virtual => ffi::DRM_MODE_ENCODER_VIRTUAL,
            Kind::DSI => ffi::DRM_MODE_ENCODER_DSI,
            Kind::DPMST => ffi::DRM_MODE_ENCODER_DPMST,
            Kind::DPI => ffi::DRM_MODE_ENCODER_DPI,
        }
    }
}
