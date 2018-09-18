//! # Connector
//!
//! Respresents the physical output, such as a DisplayPort or VGA connector.
//!
//! A Connector is the physical connection between the display controller and
//! a display. These objects keep track of connection information and state,
//! including the modes that the current display supports.

use control::encoder::Handle as EncoderHandle;
use control::Mode;
use control::HandleBuffer3;
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
    pub(crate) kind: Kind,
    pub(crate) id: u32,
    pub(crate) connection: State,
    pub(crate) size: (u32, u32),
    pub(crate) subpixel: (),
    pub(crate) encoders: HandleBuffer3<EncoderHandle>,
    pub(crate) curr_enc: Option<EncoderHandle>,
}

impl Info {
    /// The connector's handle
    pub fn handle(&self) -> Handle {
        self.handle
    }

    /// The connector's type
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// If there are multiple of this `Kind` of connector, this ID will be different
    pub fn kind_id(&self) -> u32 {
        self.id
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
        self.encoders.as_slice()
    }

    /// Gets the handle of the encoder currently used if it exists
    pub fn current_encoder(&self) -> Option<EncoderHandle> {
        self.curr_enc
    }
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// The type of connector.
pub enum Kind {
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

impl From<u32> for Kind {
    fn from(n: u32) -> Self {
        match n {
            ffi::DRM_MODE_CONNECTOR_Unknown => Kind::Unknown,
            ffi::DRM_MODE_CONNECTOR_VGA => Kind::VGA,
            ffi::DRM_MODE_CONNECTOR_DVII => Kind::DVII,
            ffi::DRM_MODE_CONNECTOR_DVID => Kind::DVID,
            ffi::DRM_MODE_CONNECTOR_DVIA => Kind::DVIA,
            ffi::DRM_MODE_CONNECTOR_Composite => Kind::Composite,
            ffi::DRM_MODE_CONNECTOR_SVIDEO => Kind::SVideo,
            ffi::DRM_MODE_CONNECTOR_LVDS => Kind::LVDS,
            ffi::DRM_MODE_CONNECTOR_Component => Kind::Component,
            ffi::DRM_MODE_CONNECTOR_9PinDIN => Kind::NinePinDIN,
            ffi::DRM_MODE_CONNECTOR_DisplayPort => Kind::DisplayPort,
            ffi::DRM_MODE_CONNECTOR_HDMIA => Kind::HDMIA,
            ffi::DRM_MODE_CONNECTOR_HDMIB => Kind::HDMIB,
            ffi::DRM_MODE_CONNECTOR_TV => Kind::TV,
            ffi::DRM_MODE_CONNECTOR_eDP => Kind::EmbeddedDisplayPort,
            ffi::DRM_MODE_CONNECTOR_VIRTUAL => Kind::Virtual,
            ffi::DRM_MODE_CONNECTOR_DSI => Kind::DSI,
            ffi::DRM_MODE_CONNECTOR_DPI => Kind::DPI,
            _ => Kind::Unknown,
        }
    }
}

impl Into<u32> for Kind {
    fn into(self) -> u32 {
        match self {
            Kind::Unknown => ffi::DRM_MODE_CONNECTOR_Unknown,
            Kind::VGA => ffi::DRM_MODE_CONNECTOR_VGA,
            Kind::DVII => ffi::DRM_MODE_CONNECTOR_DVII,
            Kind::DVID => ffi::DRM_MODE_CONNECTOR_DVID,
            Kind::DVIA => ffi::DRM_MODE_CONNECTOR_DVIA,
            Kind::Composite => ffi::DRM_MODE_CONNECTOR_Composite,
            Kind::SVideo => ffi::DRM_MODE_CONNECTOR_SVIDEO,
            Kind::LVDS => ffi::DRM_MODE_CONNECTOR_LVDS,
            Kind::Component => ffi::DRM_MODE_CONNECTOR_Component,
            Kind::NinePinDIN => ffi::DRM_MODE_CONNECTOR_9PinDIN,
            Kind::DisplayPort => ffi::DRM_MODE_CONNECTOR_DisplayPort,
            Kind::HDMIA => ffi::DRM_MODE_CONNECTOR_HDMIA,
            Kind::HDMIB => ffi::DRM_MODE_CONNECTOR_HDMIB,
            Kind::TV => ffi::DRM_MODE_CONNECTOR_TV,
            Kind::EmbeddedDisplayPort => ffi::DRM_MODE_CONNECTOR_eDP,
            Kind::Virtual => ffi::DRM_MODE_CONNECTOR_VIRTUAL,
            Kind::DSI => ffi::DRM_MODE_CONNECTOR_DSI,
            Kind::DPI => ffi::DRM_MODE_CONNECTOR_DPI,
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
