//! # Connector
//!
//! Represents the physical output, such as a DisplayPort or VGA connector.
//!
//! A Connector is the physical connection between the display controller and
//! a display. These objects keep track of connection information and state,
//! including the modes that the current display supports.

use crate::control;
use drm_ffi as ffi;

/// A handle to a connector
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
    const FFI_TYPE: u32 = ffi::DRM_MODE_OBJECT_CONNECTOR;
}

impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("connector::Handle").field(&self.0).finish()
    }
}

/// Information about a connector
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Info {
    pub(crate) handle: Handle,
    pub(crate) interface: Interface,
    pub(crate) interface_id: u32,
    pub(crate) connection: State,
    pub(crate) size: Option<(u32, u32)>,
    pub(crate) modes: Vec<control::Mode>,
    pub(crate) encoders: Vec<control::encoder::Handle>,
    pub(crate) curr_enc: Option<control::encoder::Handle>,
    pub(crate) subpixel: SubPixel,
}

impl std::fmt::Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.interface.as_str(), self.interface_id)
    }
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
    pub fn encoders(&self) -> &[control::encoder::Handle] {
        &self.encoders
    }

    /// Returns a list of modes this connector reports as supported.
    pub fn modes(&self) -> &[control::Mode] {
        &self.modes
    }

    /// Returns the current encoder attached to this connector.
    pub fn current_encoder(&self) -> Option<control::encoder::Handle> {
        self.curr_enc
    }

    /// Subpixel order of the connected sink
    pub fn subpixel(&self) -> SubPixel {
        self.subpixel
    }
}

/// A physical interface type.
#[allow(missing_docs)]
#[allow(clippy::upper_case_acronyms)]
#[non_exhaustive]
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
    Writeback,
    SPI,
    USB,
}

impl Interface {
    /// Get interface name as string
    pub fn as_str(&self) -> &'static str {
        // source: https://github.com/torvalds/linux/blob/489fa31ea873282b41046d412ec741f93946fc2d/drivers/gpu/drm/drm_connector.c#L89
        match self {
            Interface::Unknown => "Unknown",
            Interface::VGA => "VGA",
            Interface::DVII => "DVI-I",
            Interface::DVID => "DVI-D",
            Interface::DVIA => "DVI-A",
            Interface::Composite => "Composite",
            Interface::SVideo => "SVIDEO",
            Interface::LVDS => "LVDS",
            Interface::Component => "Component",
            Interface::NinePinDIN => "DIN",
            Interface::DisplayPort => "DP",
            Interface::HDMIA => "HDMI-A",
            Interface::HDMIB => "HDMI-B",
            Interface::TV => "TV",
            Interface::EmbeddedDisplayPort => "eDP",
            Interface::Virtual => "Virtual",
            Interface::DSI => "DSI",
            Interface::DPI => "DPI",
            Interface::Writeback => "Writeback",
            Interface::SPI => "SPI",
            Interface::USB => "USB",
        }
    }
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
            ffi::DRM_MODE_CONNECTOR_WRITEBACK => Interface::Writeback,
            ffi::DRM_MODE_CONNECTOR_SPI => Interface::SPI,
            ffi::DRM_MODE_CONNECTOR_USB => Interface::USB,
            _ => Interface::Unknown,
        }
    }
}

impl From<Interface> for u32 {
    fn from(interface: Interface) -> Self {
        match interface {
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
            Interface::Writeback => ffi::DRM_MODE_CONNECTOR_WRITEBACK,
            Interface::SPI => ffi::DRM_MODE_CONNECTOR_SPI,
            Interface::USB => ffi::DRM_MODE_CONNECTOR_USB,
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

impl From<State> for u32 {
    fn from(state: State) -> Self {
        // These variables are not defined in drm_mode.h for some reason.
        // They were copied from libdrm's xf86DrmMode.h
        match state {
            State::Connected => 1,
            State::Disconnected => 2,
            State::Unknown => 3,
        }
    }
}

/// Subpixel order of the connected sink
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SubPixel {
    /// Unknown geometry
    Unknown,
    /// Horizontal RGB
    HorizontalRgb,
    /// Horizontal BGR
    HorizontalBgr,
    /// Vertical RGB
    VerticalRgb,
    /// Vertical BGR
    VerticalBgr,
    /// No geometry
    None,
    /// Encountered value not supported by drm-rs
    NotImplemented,
}

impl SubPixel {
    pub(super) fn from_raw(n: u32) -> Self {
        // These values are not defined in drm_mode.h
        // They were copied from linux's drm_connector.h
        // Don't mistake those for ones used in xf86DrmMode.h (libdrm offsets those by 1)
        match n {
            0 => Self::Unknown,
            1 => Self::HorizontalRgb,
            2 => Self::HorizontalBgr,
            3 => Self::VerticalRgb,
            4 => Self::VerticalBgr,
            5 => Self::None,
            _ => Self::NotImplemented,
        }
    }
}
