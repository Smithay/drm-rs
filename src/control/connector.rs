//! # Connector
//!
//! Respresents the physical output, such as a DisplayPort or VGA connector.
//!
//! A Connector is the physical connection between the display controller and
//! a display. These objects keep track of connection information and state,
//! including the modes that the current display supports.

use control;
use drm_ffi as ffi;

/// A handle to a connector
#[repr(transparent)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Handle(control::RawResourceHandle);

impl Into<control::RawResourceHandle> for Handle {
    fn into(self) -> control::RawResourceHandle {
        self.0
    }
}

impl Into<u32> for Handle {
    fn into(self) -> u32 {
        self.0.into()
    }
}

impl From<control::RawResourceHandle> for Handle {
    fn from(handle: control::RawResourceHandle) -> Self {
        Handle(handle)
    }
}

impl control::ResourceHandle for Handle {
    const FFI_TYPE: u32 = ffi::DRM_MODE_OBJECT_CONNECTOR;
}

impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("connector::Handle")
            .field(&self.0)
            .finish()
    }
}

/// Information about a connector
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Info {
    pub(crate) handle: Handle,
    pub(crate) interface: Interface,
    pub(crate) interface_id: u32,
    pub(crate) connection: State,
    pub(crate) size: Option<(u32, u32)>,
    pub(crate) encoders: [Option<control::encoder::Handle>; 3],
    pub(crate) curr_enc: Option<control::encoder::Handle>,
}

impl Info {
    /// Returns the handle to this connector.
    pub fn handle(&self) -> Handle {
        self.handle
    }

    /// Returns the type of `Interface` of this connector.
    pub fn interface(&self) -> Interface {
        self.interface
    }

    /// Returns the interface ID of this connector.
    ///
    /// When multiple connectors have the same `Interface`, they will have
    /// different interface IDs.
    pub fn interface_id(&self) -> u32 {
        self.interface_id
    }

    /// Returns the `State` of this connector.
    pub fn state(&self) -> State {
        self.connection
    }

    /// Returns the size of the display (in millimeters) if connected.
    pub fn size(&self) -> Option<(u32, u32)> {
        self.size
    }

    /// Returns a list of encoders that can be possibly used by this connector.
    pub fn encoders(&self) -> &[Option<control::encoder::Handle>] {
        &self.encoders
    }

    /// Returns the current encoder attached to this connector.
    pub fn current_encoder(&self) -> Option<control::encoder::Handle> {
        self.curr_enc
    }
}

/// A physical interface type.
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Interface {
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

impl From<u32> for Interface {
    fn from(n: u32) -> Self {
        match n {
            ffi::DRM_MODE_CONNECTOR_Unknown => Interface::Unknown,
            ffi::DRM_MODE_CONNECTOR_VGA => Interface::VGA,
            ffi::DRM_MODE_CONNECTOR_DVII => Interface::DVII,
            ffi::DRM_MODE_CONNECTOR_DVID => Interface::DVID,
            ffi::DRM_MODE_CONNECTOR_DVIA => Interface::DVIA,
            ffi::DRM_MODE_CONNECTOR_Composite => Interface::Composite,
            ffi::DRM_MODE_CONNECTOR_SVIDEO => Interface::SVideo,
            ffi::DRM_MODE_CONNECTOR_LVDS => Interface::LVDS,
            ffi::DRM_MODE_CONNECTOR_Component => Interface::Component,
            ffi::DRM_MODE_CONNECTOR_9PinDIN => Interface::NinePinDIN,
            ffi::DRM_MODE_CONNECTOR_DisplayPort => Interface::DisplayPort,
            ffi::DRM_MODE_CONNECTOR_HDMIA => Interface::HDMIA,
            ffi::DRM_MODE_CONNECTOR_HDMIB => Interface::HDMIB,
            ffi::DRM_MODE_CONNECTOR_TV => Interface::TV,
            ffi::DRM_MODE_CONNECTOR_eDP => Interface::EmbeddedDisplayPort,
            ffi::DRM_MODE_CONNECTOR_VIRTUAL => Interface::Virtual,
            ffi::DRM_MODE_CONNECTOR_DSI => Interface::DSI,
            ffi::DRM_MODE_CONNECTOR_DPI => Interface::DPI,
            _ => Interface::Unknown,
        }
    }
}

impl Into<u32> for Interface {
    fn into(self) -> u32 {
        match self {
            Interface::Unknown => ffi::DRM_MODE_CONNECTOR_Unknown,
            Interface::VGA => ffi::DRM_MODE_CONNECTOR_VGA,
            Interface::DVII => ffi::DRM_MODE_CONNECTOR_DVII,
            Interface::DVID => ffi::DRM_MODE_CONNECTOR_DVID,
            Interface::DVIA => ffi::DRM_MODE_CONNECTOR_DVIA,
            Interface::Composite => ffi::DRM_MODE_CONNECTOR_Composite,
            Interface::SVideo => ffi::DRM_MODE_CONNECTOR_SVIDEO,
            Interface::LVDS => ffi::DRM_MODE_CONNECTOR_LVDS,
            Interface::Component => ffi::DRM_MODE_CONNECTOR_Component,
            Interface::NinePinDIN => ffi::DRM_MODE_CONNECTOR_9PinDIN,
            Interface::DisplayPort => ffi::DRM_MODE_CONNECTOR_DisplayPort,
            Interface::HDMIA => ffi::DRM_MODE_CONNECTOR_HDMIA,
            Interface::HDMIB => ffi::DRM_MODE_CONNECTOR_HDMIB,
            Interface::TV => ffi::DRM_MODE_CONNECTOR_TV,
            Interface::EmbeddedDisplayPort => ffi::DRM_MODE_CONNECTOR_eDP,
            Interface::Virtual => ffi::DRM_MODE_CONNECTOR_VIRTUAL,
            Interface::DSI => ffi::DRM_MODE_CONNECTOR_DSI,
            Interface::DPI => ffi::DRM_MODE_CONNECTOR_DPI,
        }
    }
}

/// The state of a connector.
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
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
