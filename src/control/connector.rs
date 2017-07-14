use ffi;
use control::{self, ResourceHandle, ResourceInfo};
use result::*;

#[derive(Clone, Copy, PartialEq, Eq)]
/// A `ResourceHandle` to a connector.
pub struct Id(control::RawId);

#[derive(Debug, Clone, PartialEq, Eq)]
/// The `ResourceInfo` on a connector.
pub struct Info {
    handle: Id,
    // TODO: properties
    modes: ffi::Buffer<control::Mode>,
    encoders: ffi::Buffer<control::encoder::Id>,
    con_type: Type,
    con_state: State,
    // TODO: Subpixel
    // TODO: Subconnector
    size: (u32, u32)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The physical type of connector
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
    DPI
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The state of a connector.
pub enum State {
    Connected,
    Disconnected,
    Unknown
}

impl Info {
    /// Returns the type of connector this is
    pub fn connector_type(&self) -> Type {
        self.con_type
    }

    /// Returns the state of this connector.
    pub fn connection_state(&self) -> State {
        self.con_state
    }

    /// Returns a list of supported Modes.
    pub fn modes<'a>(&'a self) -> &'a [control::Mode] {
        &self.modes
    }
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

impl control::property::LoadProperties for Id {
    const TYPE: u32 = ffi::DRM_MODE_OBJECT_CONNECTOR;
}

impl ResourceInfo for Info {
    type Handle = Id;

    fn load_from_device<T>(device: &T, handle: Id) -> Result<Self>
        where T: control::Device {

        let mut raw: ffi::drm_mode_get_connector = Default::default();
        raw.connector_id = handle.0;
        unsafe {
            try!(ffi::ioctl_mode_getconnector(device.as_raw_fd(), &mut raw));
        }
        // TODO: Figure out properties
        // let props = ffi_buf!(raw.props_ptr, raw.count_props);
        raw.count_props = 0;
        let encs = ffi_buf!(raw.encoders_ptr, raw.count_encoders);
        let modes: Vec<ffi::drm_mode_modeinfo> = ffi_buf!(raw.modes_ptr, raw.count_modes);
        unsafe {
            try!(ffi::ioctl_mode_getconnector(device.as_raw_fd(), &mut raw));
        }

        let encs = encs.iter().map(| &x |
                                   control::encoder::Id::from_raw(x)
        ).collect();

        let con = Self {
            handle: handle,
            modes: unsafe { ::std::mem::transmute(modes) },
            encoders: encs,
            con_type: Type::from(raw.connector_type),
            con_state: State::from(raw.connection),
            size: (raw.mm_width, raw.mm_height)
        };

        Ok(con)
    }

    fn handle(&self) -> Self::Handle { self.handle }
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
            _ => Type::Unknown
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
            _ => State::Unknown
        }
    }
}

impl ::std::fmt::Debug for Id {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "connector::Id({})", self.0)
    }
}
