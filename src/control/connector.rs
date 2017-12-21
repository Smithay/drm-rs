//! # Connector
//!
//! A connector is a physical video connector found on your device, such as an
//! HDMI port.

use ffi;
use control::{self, ResourceHandle, ResourceInfo};
use result::*;

/// A [`ResourceHandle`] for a connector.
///
/// Like all control resources, every connector has a unique `Handle` associated
/// with it. This `Handle` can be used to acquire information about the
/// connector (see [`connector::Info`]) or change the connector's state.
///
/// These can be retrieved by using [`ResourceHandles::connectors`].
///
/// [`ResourceHandle`]: ../ResourceHandle.t.html
/// [`connector::Info`]: Info.t.html
/// [`ResourceHandles::connectors`]: ../ResourceHandles.t.html#method.connectors
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, From, Into)]
pub struct Handle(control::RawHandle);
impl ResourceHandle for Handle {}

/// A [`ResourceInfo`] for a connector.
///
/// [`ResourceInfo`]: ../ResourceInfo.t.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info {
    handle: Handle,
    // TODO: properties
    modes: ffi::Buffer<control::Mode>,
    encoder: control::encoder::Handle,
    encoders: ffi::Buffer<control::encoder::Handle>,
    con_type: Type,
    con_state: State,
    // TODO: Subpixel
    // TODO: Subconnector
    size: (u32, u32),
}

/// The physical type of connector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
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

/// The connection state of a connector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum State {
    Connected,
    Disconnected,
    Unknown,
}

impl Info {
    /// Returns the [`Type`] of connector.
    ///
    /// [`Type`]: Type.t.html
    pub fn connector_type(&self) -> Type {
        self.con_type
    }

    /// Returns the [`State`] of connector.
    ///
    /// [`State`]: State.t.html
    pub fn connection_state(&self) -> State {
        self.con_state
    }

    /// Returns a list containing each supported [`Mode`].
    ///
    /// [`Mode`]: ../Mode.t.html
    pub fn modes<'a>(&'a self) -> &'a [control::Mode] {
        &self.modes
    }

    /// Returns the currently active encoder
    pub fn current_encoder(&self) -> Option<control::encoder::Handle> {
        if self.encoder == control::encoder::Handle::from(0) {
            None
        } else {
            Some(self.encoder)
        }
    }

    /// Returns a list containing each supported [`encoder::Handle`].
    pub fn encoders<'a>(&'a self) -> &'a [control::encoder::Handle] {
        &self.encoders
    }
}

impl control::property::LoadProperties for Handle {
    const TYPE: u32 = ffi::DRM_MODE_OBJECT_CONNECTOR;
}

impl ResourceInfo for Info {
    type Handle = Handle;

    fn load_from_device<T>(device: &T, handle: Self::Handle) -> Result<Self>
    where
        T: control::Device,
    {
        let connector = {
            // TODO: Change to immutable once we no longer need to modify
            // raw.count_props
            let mut raw: ffi::drm_mode_get_connector = Default::default();
            raw.connector_id = handle.into();
            unsafe {
                try!(ffi::ioctl_mode_getconnector(device.as_raw_fd(), &mut raw));
            }

            // TODO: Start returning the list of properties too.
            raw.count_props = 0;

            let con = Self {
                handle: handle,
                modes: ffi_buf!(raw.modes_ptr, raw.count_modes),
                encoder: control::encoder::Handle::from(raw.encoder_id),
                encoders: ffi_buf!(raw.encoders_ptr, raw.count_encoders),
                con_type: Type::from(raw.connector_type),
                con_state: State::from(raw.connection),
                size: (raw.mm_width, raw.mm_height),
            };

            unsafe {
                try!(ffi::ioctl_mode_getconnector(device.as_raw_fd(), &mut raw));
            }

            con
        };

        Ok(connector)
    }

    fn handle(&self) -> Self::Handle {
        self.handle
    }
}

#[allow(non_snake_case)]
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
