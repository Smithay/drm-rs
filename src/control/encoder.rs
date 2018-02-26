//! # Encoder
//!
//! An encoder is a bridge between a CRTC and a connector that takes the pixel
//! data of the CRTC and encodes it into a format the connector understands.

use ffi::{self, Wrapper, mode::RawHandle};
use control::{ResourceHandle, ResourceInfo, Device};
use control::crtc;
use result::Result;

#[derive(Copy, Clone, Hash, PartialEq, Eq, From, Into)]
/// A [ResourceHandle](../ResourceHandle.t.html) representing an encoder.
pub struct Handle(RawHandle);

impl ResourceHandle for Handle {
    type Info = Info;

    fn get_info<T: Device>(device: &T, handle: Self) -> Result<Info> {
        let mut t = ffi::mode::GetEncoder::default();
        t.raw_mut_ref().encoder_id = handle.into();
        t.ioctl(device.as_raw_fd())?;
        Ok(Info(t))
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
/// A [ResourceInfo](../ResourceInfo.t.html) object about an encoder.
pub struct Info(ffi::mode::GetEncoder);

impl ResourceInfo for Info {
    type Handle = Handle;

    fn handle(&self) -> Handle {
        Handle::from(self.0.raw_ref().encoder_id)
    }
}

impl Info {
    /// Returns the type of encoder associated.
    pub fn encoder_type(&self) -> Type {
        Type::from(self.0.raw_ref().encoder_type)
    }

    /// Returns a filter that can be used to determine which CRTC resources
    /// are compatible with this encoder.
    pub fn possible_crtcs(&self) -> u32 {
        self.0.raw_ref().possible_crtcs
    }

    /// Returns a filter that can be used to determine which encoder resources
    /// are likely clones of this one.
    pub fn possible_clones(&self) -> u32 {
        self.0.raw_ref().possible_clones
    }

    /// Returns the handle associated with the currently attached CRTC.
    pub fn current_crtc(&self) -> Option<crtc::Handle> {
        crtc::Handle::from_checked(self.0.raw_ref().crtc_id)
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
