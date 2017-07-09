use drm_sys;
use control::{self, ResourceHandle, ResourceInfo};
use result::*;
use ffi;

#[derive(Clone, Copy, PartialEq, Eq)]
/// A `ResourceHandle` to an encoder.
pub struct Id(control::RawId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The `ResourceInfo` on an encoder.
pub struct Info {
    handle: Id,
    crtc_id: control::crtc::Id,
    enc_type: Type,
    //possible_crtcs: CrtcListFilter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The underlying type of encoder.
pub enum Type {
    None,
    DAC,
    TMDS,
    LVDS,
    TVDAC,
    Virtual,
    DSI,
    DPMST,
    DPI
}

impl ResourceHandle for Id {
    type RawHandle = control::RawId;

    fn from_raw(raw: Self::RawHandle) -> Self {
        Id(raw)
    }

    fn as_raw(&self) -> Self::RawHandle {
        self.0
    }
}

impl ResourceInfo for Info {
    type Handle = Id;

    fn load_from_device<T>(device: &T, handle: Id) -> Result<Self>
        where T: control::Device {

        let mut raw: drm_sys::drm_mode_get_encoder = Default::default();
        raw.encoder_id = handle.0;
        unsafe {
            try!(ffi::ioctl_mode_getencoder(device.as_raw_fd(), &mut raw));
        }

        let enc = Self {
            handle: handle,
            crtc_id: control::crtc::Id::from_raw(raw.crtc_id),
            enc_type: Type::from(raw.encoder_type),
            //possible_crtcs: CrtcListFilter(raw.possible_crtcs)
        };

        Ok(enc)
    }

    fn handle(&self) -> Self::Handle { self.handle }
}

impl From<u32> for Type {
    fn from(n: u32) -> Self {
        match n {
            drm_sys::DRM_MODE_ENCODER_NONE => Type::None,
            drm_sys::DRM_MODE_ENCODER_DAC => Type::DAC,
            drm_sys::DRM_MODE_ENCODER_TMDS => Type::TMDS,
            drm_sys::DRM_MODE_ENCODER_LVDS => Type::LVDS,
            drm_sys::DRM_MODE_ENCODER_TVDAC => Type::TVDAC,
            drm_sys::DRM_MODE_ENCODER_VIRTUAL => Type::Virtual,
            drm_sys::DRM_MODE_ENCODER_DSI => Type::DSI,
            drm_sys::DRM_MODE_ENCODER_DPMST => Type::DPMST,
            drm_sys::DRM_MODE_ENCODER_DPI => Type::DPI,
            _ => Type::None
        }
    }
}

impl ::std::fmt::Debug for Id {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "encoder::Id({})", self.0)
    }
}
