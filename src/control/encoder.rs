//! # Encoder
//!
//! An encoder is a bridge between a CRTC and a connector that takes the pixel
//! data of the CRTC and encodes it into a format the connector understands.

use control::{self, ResourceHandle, ResourceInfo};
use result::*;
use ffi;

/// A [`ResourceHandle`] for an encoder.
///
/// Like all control resources, every encoder has a unique `Handle` associated
/// with it. This `Handle` can be used to acquire information about the encoder
/// (see [`encoder::Info`]) or change the encoder's state.
///
/// These can be retrieved by using [`ResourceHandles::encoders`].
///
/// [`ResourceHandle`]: ResourceHandle.t.html
/// [`encoder::Info`]: Info.t.html
/// [`ResourceHandles::encoders`]: ResourceHandles.t.html#method.encoders
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, From, Into)]
pub struct Handle(control::RawHandle);
impl ResourceHandle for Handle {}

/// A [`ResourceInfo`] for an encoder.
///
/// [`ResourceInfo`]: ResourceInfo.t.html
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Info {
    handle: Handle,
    crtc_id: control::crtc::Handle,
    enc_type: Type,
    possible_crtcs: control::CrtcListFilter,
    possible_clones: u32,
}

/// The type of encoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
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

impl Info {
    /// Returns the [`Type`] of the connector.
    ///
    /// [`Type`]: Type.t.html
    pub fn encoder_type(&self) -> Type {
        self.enc_type
    }

    /// Returns a filter for determining which CRTCs are compatible with this
    /// encoder.
    pub fn possible_crtcs(&self) -> control::CrtcListFilter {
        self.possible_crtcs
    }

    /// Returns the currently connected `crtc::Handle`
    pub fn current_crtc(&self) -> Option<control::crtc::Handle> {
        if self.crtc_id == control::crtc::Handle::from(0) {
            None
        } else {
            Some(self.crtc_id)
        }
    }
}

impl ResourceInfo for Info {
    type Handle = Handle;

    fn load_from_device<T>(device: &T, handle: Handle) -> Result<Self>
    where
        T: control::Device,
    {
        let enc = {
            let mut raw: ffi::drm_mode_get_encoder = Default::default();
            raw.encoder_id = handle.into();
            unsafe {
                try!(ffi::ioctl_mode_getencoder(device.as_raw_fd(), &mut raw));
            }

            Self {
                handle: handle,
                crtc_id: control::crtc::Handle::from(raw.crtc_id),
                enc_type: Type::from(raw.encoder_type),
                possible_crtcs: control::CrtcListFilter(raw.possible_crtcs),
                possible_clones: raw.possible_clones,
            }
        };

        Ok(enc)
    }

    fn handle(&self) -> Self::Handle {
        self.handle
    }
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
