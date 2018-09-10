//! # Connector
//!
//! Respresents the physical output, such as a DisplayPort or VGA connector.
//!
//! A Connector is the physical connection between the display controller and
//! a display. These objects keep track of connection information and state,
//! including the modes that the current display supports.

use ffi;
use control::Mode;
use control::encoder::Handle as EncoderHandle;

use util::*;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// A handle to a specific connector
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
/// Information about a specific connector
pub struct Info {
    pub(crate) handle: Handle,
    pub(crate) conn_type: Type,
    pub(crate) conn_type_id: u32,
    pub(crate) connection: State,
    pub(crate) size: (u32, u32),
    pub(crate) subpixel: (),
    pub(crate) encoders: SmallBuffer<EncoderHandle>,
    pub(crate) modes: SmallBuffer<Mode>,
    pub(crate) curr_enc: Option<EncoderHandle>,
}

impl Info {
    /// The connector's handle
    pub fn handle(&self) -> Handle {
        self.handle
    }

    /// The connector's type
    pub fn kind(&self) -> Type {
        self.conn_type
    }

    /// The connector's state
    pub fn state(&self) -> State {
        self.connection
    }

    /// The size of this display in millimeters
    pub fn size(&self) -> (u32, u32) {
        self.size
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
