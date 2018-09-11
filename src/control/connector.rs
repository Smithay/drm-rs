//! # Connector
//!
//! Respresents the physical output, such as a DisplayPort or VGA connector.
//!
//! A Connector is the physical connection between the display controller and
//! a display. These objects keep track of connection information and state,
//! including the modes that the current display supports.

use control::encoder::Handle as EncoderHandle;
use control::Mode;
use ffi;

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
    pub(crate) props: SmallBuffer<u32>,
    pub(crate) pvals: SmallBuffer<u64>,
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

    /// If there is more than one of a specific kind of connector on a card,
    /// each one will be given a different id.
    pub fn kind_id(&self) -> u32 {
        self.conn_type_id
    }

    /// The connector's state
    pub fn state(&self) -> State {
        self.connection
    }

    /// The size of this display in millimeters
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Returns a list of encoders that can be used on this connector
    pub fn encoders(&self) -> &[EncoderHandle] {
        self.encoders.as_ref()
    }

    /// Gets the handle of the encoder currently used if it exists
    pub fn current_encoder(&self) -> Option<EncoderHandle> {
        self.curr_enc
    }

    /// Returns the list of modes this connector supports.
    pub fn modes(&self) -> &[Mode] {
        self.modes.as_ref()
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

impl Into<u32> for Type {
    fn into(self) -> u32 {
        match self {
            Type::Unknown => ffi::DRM_MODE_CONNECTOR_Unknown,
            Type::VGA => ffi::DRM_MODE_CONNECTOR_VGA,
            Type::DVII => ffi::DRM_MODE_CONNECTOR_DVII,
            Type::DVID => ffi::DRM_MODE_CONNECTOR_DVID,
            Type::DVIA => ffi::DRM_MODE_CONNECTOR_DVIA,
            Type::Composite => ffi::DRM_MODE_CONNECTOR_Composite,
            Type::SVideo => ffi::DRM_MODE_CONNECTOR_SVIDEO,
            Type::LVDS => ffi::DRM_MODE_CONNECTOR_LVDS,
            Type::Component => ffi::DRM_MODE_CONNECTOR_Component,
            Type::NinePinDIN => ffi::DRM_MODE_CONNECTOR_9PinDIN,
            Type::DisplayPort => ffi::DRM_MODE_CONNECTOR_DisplayPort,
            Type::HDMIA => ffi::DRM_MODE_CONNECTOR_HDMIA,
            Type::HDMIB => ffi::DRM_MODE_CONNECTOR_HDMIB,
            Type::TV => ffi::DRM_MODE_CONNECTOR_TV,
            Type::EmbeddedDisplayPort => ffi::DRM_MODE_CONNECTOR_eDP,
            Type::Virtual => ffi::DRM_MODE_CONNECTOR_VIRTUAL,
            Type::DSI => ffi::DRM_MODE_CONNECTOR_DSI,
            Type::DPI => ffi::DRM_MODE_CONNECTOR_DPI,
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// The state of the connector.
pub enum State {
    Connected,
    Disconnected,
    Unknown,
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

impl Into<u32> for State {
    fn into(self) -> u32 {
        // These variables are not defined in drm_mode.h for some reason.
        // They were copied from libdrm's xf86DrmMode.h
        match self {
            State::Connected => 1,
            State::Disconnected => 2,
            State::Unknown => 3,
        }
    }
}
