//! # Connector
//!
//! Respresents the physical output, such as a DisplayPort or VGA connector.
//!
//! A Connector is the physical connection between the display controller and
//! a display. These objects keep track of connection information and state,
//! including the modes that the current display supports.

use ffi::{self, Wrapper, mode::RawHandle};
use control::{ResourceHandle, ResourceInfo, Device};
use control::encoder;
use result::Result;

#[derive(Copy, Clone, Hash, PartialEq, Eq, From, Into)]
/// A [ResourceHandle](../ResourceHandle.t.html) representing a connector.
pub struct Handle(RawHandle);

impl ResourceHandle for Handle {
    type Info = Info;

    fn get_info<T: Device>(device: &T, handle: Self) -> Result<Info> {
        let mut t = ffi::mode::GetConnector::default();
        t.raw_mut_ref().connector_id = handle.into();
        t.ioctl(device.as_raw_fd())?;
        Ok(Info(t))
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
/// A [ResourceInfo](../ResourceInfo.t.html) object about a connector.
pub struct Info(ffi::mode::GetConnector);

impl ResourceInfo for Info {
    type Handle = Handle;

    fn handle(&self) -> Handle {
        Handle::from(self.0.raw_ref().connector_id)
    }
}

impl Info {
    /// Returns the physical type of connector associated.
    pub fn connector_type(&self) -> Type {
        Type::from(self.0.raw_ref().connector_type)
    }

    /// Returns whether a connector is currently connected or not.
    pub fn connection_state(&self) -> State {
        State::from(self.0.raw_ref().connection)
    }

    /// Returns the handle associated with the currently attached encoder.
    pub fn current_encoder(&self) -> Option<encoder::Handle> {
       encoder::Handle::from_checked(self.0.raw_ref().encoder_id)
    }

    /// Returns the set of compatible encoders.
    pub fn encoders(&self) -> &[encoder::Handle] {
        slice_from_wrapper!(self.0, enc_buf, count_encoders)
    }

    /// TODO: Document
    pub fn properties(&self) -> &[u32] {
        slice_from_wrapper!(self.0, prop_buf, count_props)
    }

    /// TODO: Document
    pub fn prop_values(&self) -> &[u64] {
        slice_from_wrapper!(self.0, prop_val_buf, count_props)
    }
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// The type of connector.
pub enum Type {
    Unknown,
    VGA,
    DVII,
    DVID,
    DVIA,
    Composite,
    SVideo,
    LVDS,
    Component,
    NinePinDIN,
    DisplayPort,
    HDMIA,
    HDMIB,
    TV,
    EmbeddedDisplayPort,
    Virtual,
    DSI,
    DPI,
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// The state of the connector.
pub enum State {
    Connected,
    Disconnected,
    Unknown,
}

impl From<u32> for Type {
    fn from(n: u32) -> Self {
        match n {
            ffi::DRM_MODE_CONNECTOR_Unknown => Type::Unknown,
            ffi::DRM_MODE_CONNECTOR_VGA => Type::VGA,
            ffi::DRM_MODE_CONNECTOR_DVII => Type::DVII,
            ffi::DRM_MODE_CONNECTOR_DVID => Type::DVID,
            ffi::DRM_MODE_CONNECTOR_DVIA => Type::DVIA,
            ffi::DRM_MODE_CONNECTOR_Composite => Type::Composite,
            ffi::DRM_MODE_CONNECTOR_SVIDEO => Type::SVideo,
            ffi::DRM_MODE_CONNECTOR_LVDS => Type::LVDS,
            ffi::DRM_MODE_CONNECTOR_Component => Type::Component,
            ffi::DRM_MODE_CONNECTOR_9PinDIN => Type::NinePinDIN,
            ffi::DRM_MODE_CONNECTOR_DisplayPort => Type::DisplayPort,
            ffi::DRM_MODE_CONNECTOR_HDMIA => Type::HDMIA,
            ffi::DRM_MODE_CONNECTOR_HDMIB => Type::HDMIB,
            ffi::DRM_MODE_CONNECTOR_TV => Type::TV,
            ffi::DRM_MODE_CONNECTOR_eDP => Type::EmbeddedDisplayPort,
            ffi::DRM_MODE_CONNECTOR_VIRTUAL => Type::Virtual,
            ffi::DRM_MODE_CONNECTOR_DSI => Type::DSI,
            ffi::DRM_MODE_CONNECTOR_DPI => Type::DPI,
            _ => Type::Unknown,
        }
    }
}

impl From<u32> for State {
    fn from(n: u32) -> Self {
        // These variables are not defined in drm_mode.h for some reason.
        // They were copied from libdrm's xf86DrmMode.h
        match n {
            1 => State::Connected,
            2 => State::Disconnected,
            _ => State::Unknown,
        }
    }
}
