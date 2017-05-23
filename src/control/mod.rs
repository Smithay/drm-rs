use drm_sys::*;
use super::result::*;
use super::ffi;

use std::ffi::CStr;

/// The underlying id for a resource.
pub type RawId = u32;

/// A trait for devices that provide control (modesetting) functionality.
pub trait Device : Sized + super::Device {
    /// Gets the resource ids for this device.
    fn resource_ids(&self) -> Result<ResourceIds> {
        ResourceIds::load_from_device(self)
    }

    /// Gets the plane ids for this device.
    fn plane_ids(&self) -> Result<PlaneResourceIds> {
        PlaneResourceIds::load_from_device(self)
    }

    /// Gets info on a resource.
    fn resource_info<T, U>(&self, id: T) -> Result<U>
        where T: ResourceId<U>, U: ResourceInfo<T> {

        U::load_from_device(self, id)
    }
}

/// A trait for a resource id to be referenced or created by a RawId
pub trait ResourceId<T> : Sized where T: ResourceInfo<Self> {
    /// Extracts the RawId.
    fn as_raw_id(&self) -> RawId;

    /// Creates a ResourceId from a RawId.
    ///
    /// While not actually unsafe, errors will appear that are hard to debug
    /// unless you are certain of what type of object a RawId represents. We
    /// designate this unsafe to ensure the user knows what they're doing.
    unsafe fn from_raw_id(id: RawId) -> Self;
}

/// A trait for an object that is owned by a control node.
pub trait ResourceInfo<T> : Sized where T: ResourceId<Self> {
    /// Load the info from the provided device.
    fn load_from_device<U>(device: &U, id: T) -> Result<Self> where U: Device;

    /// Get the associated ResourceId
    fn id(&self) -> T;
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The set of resource ids that are associated with a DRM device.
pub struct ResourceIds {
    connectors: ffi::Buffer<ConnectorId>,
    encoders: ffi::Buffer<EncoderId>,
    crtcs: ffi::Buffer<CrtcId>,
    framebuffers: ffi::Buffer<FramebufferId>,
    width: (u32, u32),
    height: (u32, u32)
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The set of plane ids that are associated with a DRM device.
pub struct PlaneResourceIds {
    planes: ffi::Buffer<PlaneId>
}

impl ResourceIds {
    /// Loads the resource ids from a device.
    pub fn load_from_device<T>(device: &T) -> Result<Self>
        where T: Device{

        let mut raw: drm_mode_card_res = Default::default();
        unsafe {
            try!(ffi::ioctl_mode_getresources(device.as_raw_fd(), &mut raw));
        }
        let conns = ffi_buf!(raw.connector_id_ptr, raw.count_connectors);
        let encs = ffi_buf!(raw.encoder_id_ptr, raw.count_encoders);
        let crtcs = ffi_buf!(raw.crtc_id_ptr, raw.count_crtcs);
        let fbs = ffi_buf!(raw.fb_id_ptr, raw.count_fbs);
        unsafe {
            try!(ffi::ioctl_mode_getresources(device.as_raw_fd(), &mut raw));
        }

        let res = ResourceIds {
            connectors: conns,
            encoders: encs,
            crtcs: crtcs,
            framebuffers: fbs,
            width: (raw.min_width, raw.max_width),
            height: (raw.min_height, raw.max_height)
        };

        Ok(res)
    }

    /// Returns a slice to the list of connector ids.
    pub fn connectors<'a>(&'a self) -> &'a [ConnectorId] {
        &self.connectors
    }

    /// Returns a slice to the list of encoder ids.
    pub fn encoders<'a>(&'a self) -> &'a [EncoderId] {
        &self.encoders
    }

    /// Returns a slice to the list of crtc ids.
    pub fn crtcs<'a>(&'a self) -> &'a [CrtcId] {
        &self.crtcs
    }

    /// Returns a slice to the list of framebuffer ids.
    pub fn framebuffers<'a>(&'a self) -> &'a [FramebufferId] {
        &self.framebuffers
    }

    /// TODO: Learn and document.
    pub fn width(&self) -> (u32, u32) {
        (self.width)
    }

    /// TODO: Learn and document.
    pub fn height(&self) -> (u32, u32) {
        (self.height)

    }

    pub fn filter_crtcs(&self, filter: CrtcListFilter) -> ffi::Buffer<CrtcId> {
        self.crtcs.iter().enumerate().filter(| &(n, _) | {
            (1 << n) & filter.0 != 0
        }).map(| (_, &e) | e).collect()
    }
}

impl PlaneResourceIds {
    /// Loads the plane ids from a device.
    pub fn load_from_device<T>(device: &T) -> Result<Self>
        where T: Device{

        let mut raw: drm_mode_get_plane_res = Default::default();
        unsafe {
            try!(ffi::ioctl_mode_getplaneresources(device.as_raw_fd(),
                                                   &mut raw));
        }
        let planes = ffi_buf!(raw.plane_id_ptr, raw.count_planes);
        unsafe {
            try!(ffi::ioctl_mode_getplaneresources(device.as_raw_fd(),
                                                   &mut raw));
        }

        let res = PlaneResourceIds {
            planes: planes
        };

        Ok(res)
    }

    /// Returns a slice to the list of plane ids.
    pub fn planes<'a>(&'a self) -> &'a [PlaneId] {
        &self.planes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A ResourceId for a Connector.
pub struct ConnectorId(RawId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A ResourceId for an Encoder.
pub struct EncoderId(RawId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A ResourceId for a Crtc.
pub struct CrtcId(RawId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A ResourceId for a Framebuffer.
pub struct FramebufferId(RawId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A ResourceId for a Plane.
pub struct PlaneId(RawId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A handle to a generic resource id
pub enum ResourceIdType {
    Connector(ConnectorId),
    Encoder(EncoderId),
    Crtc(CrtcId),
    Framebuffer(FramebufferId),
    Plane(PlaneId)
}

impl ResourceId<ConnectorInfo> for ConnectorId {
    fn as_raw_id(&self) -> RawId { self.0 }
    unsafe fn from_raw_id(id: RawId) -> Self{ ConnectorId(id) }
}

impl ResourceId<EncoderInfo> for EncoderId {
    fn as_raw_id(&self) -> RawId { self.0 }
    unsafe fn from_raw_id(id: RawId) -> Self { EncoderId(id) }
}

impl ResourceId<CrtcInfo> for CrtcId {
    fn as_raw_id(&self) -> RawId { self.0 }
    unsafe fn from_raw_id(id: RawId) -> Self { CrtcId(id) }
}

impl ResourceId<FramebufferInfo> for FramebufferId {
    fn as_raw_id(&self) -> RawId { self.0 }
    unsafe fn from_raw_id(id: RawId) -> FramebufferId { FramebufferId(id) }
}

impl ResourceId<PlaneInfo> for PlaneId {
    fn as_raw_id(&self) -> RawId { self.0 }
    unsafe fn from_raw_id(id: RawId) -> Self { PlaneId(id) }
}

#[derive(Debug, Clone)]
pub struct ConnectorInfo {
    id: ConnectorId,
    // TODO: properties
    modes: ffi::Buffer<Mode>,
    encoders: ffi::Buffer<EncoderId>,
    con_type: ConnectorType,
    con_state: ConnectorState,
    // TODO: Subpixel
    // TODO: Subconnector
    size: (u32, u32)
}

#[derive(Debug, Clone)]
pub struct EncoderInfo {
    id: EncoderId,
    crtc_id: CrtcId,
    enc_type: EncoderType,
    possible_crtcs: CrtcListFilter,
    // TODO: possible_clones
}

#[derive(Debug, Clone)]
pub struct CrtcInfo {
    id: CrtcId,
    position: (u32, u32),
    // TODO: mode
    fb: FramebufferId,
    gamma_length: u32
}

#[derive(Debug, Clone)]
pub struct FramebufferInfo {
    id: FramebufferId,
    size: (u32, u32),
    pitch: u32,
    bpp: u32,
    // TODO: Handle?
    depth: u32
}

#[derive(Debug, Clone)]
pub struct PlaneInfo {
    id: PlaneId,
    crtc_id: CrtcId,
    fb_id: FramebufferId,
    // TODO: count_formats,
    // TODO: possible_crtcs
    gamma_length: u32,
    // TODO: formats
}

impl ResourceInfo<ConnectorId> for ConnectorInfo {
    fn load_from_device<T>(device: &T, id: ConnectorId) -> Result<Self>
        where T: Device {

        let mut raw: drm_mode_get_connector = Default::default();
        raw.connector_id = id.0;
        unsafe {
            try!(ffi::ioctl_mode_getconnector(device.as_raw_fd(), &mut raw));
        }
        // TODO: Figure out properties
        // let props = ffi_buf!(raw.props_ptr, raw.count_props);
        raw.count_props = 0;
        let encs = ffi_buf!(raw.encoders_ptr, raw.count_encoders);
        let modes = ffi_buf!(raw.modes_ptr, raw.count_modes);
        unsafe {
            try!(ffi::ioctl_mode_getconnector(device.as_raw_fd(), &mut raw));
        }

        let con = Self {
            id: id,
            modes: modes,
            encoders: encs,
            con_type: ConnectorType::from(raw.connector_type),
            con_state: ConnectorState::from(raw.connection),
            size: (raw.mm_width, raw.mm_height)
        };

        Ok(con)
    }

    fn id(&self) -> ConnectorId {
        self.id
    }
}

impl ResourceInfo<EncoderId> for EncoderInfo {
    fn load_from_device<T>(device: &T, id: EncoderId) -> Result<Self>
        where T: Device {

        let mut raw: drm_mode_get_encoder = Default::default();
        raw.encoder_id = id.0;
        unsafe {
            try!(ffi::ioctl_mode_getencoder(device.as_raw_fd(), &mut raw));
        }

        let enc = Self {
            id: id,
            crtc_id: CrtcId(raw.crtc_id),
            enc_type: EncoderType::from(raw.encoder_type),
            possible_crtcs: CrtcListFilter(raw.possible_crtcs)
        };

        Ok(enc)
    }

    fn id(&self) -> EncoderId {
        self.id
    }
}

impl ResourceInfo<CrtcId> for CrtcInfo {
    fn load_from_device<T>(device: &T, id: CrtcId) -> Result<Self>
        where T: Device {

        let mut raw: drm_mode_crtc = Default::default();
        raw.crtc_id = id.0;
        unsafe {
            try!(ffi::ioctl_mode_getcrtc(device.as_raw_fd(), &mut raw));
        }

        let crtc = Self {
            id: id,
            position: (raw.x, raw.y),
            fb: FramebufferId(raw.fb_id),
            gamma_length: raw.gamma_size
        };

        Ok(crtc)
    }

    fn id(&self) -> CrtcId {
        self.id
    }
}

impl ResourceInfo<FramebufferId> for FramebufferInfo {
    fn load_from_device<T>(device: &T, id: FramebufferId) -> Result<Self>
        where T: Device {

        let mut raw: drm_mode_fb_cmd = Default::default();
        raw.fb_id = id.0;
        unsafe {
            try!(ffi::ioctl_mode_getfb(device.as_raw_fd(), &mut raw));
        }

        let fb = Self {
            id: id,
            size: (raw.width, raw.height),
            pitch: raw.pitch,
            bpp: raw.bpp,
            depth: raw.depth
        };

        Ok(fb)
    }

    fn id(&self) -> FramebufferId {
        self.id
    }
}

impl ResourceInfo<PlaneId> for PlaneInfo {
    fn load_from_device<T>(device: &T, id: PlaneId) -> Result<Self>
        where T: Device {

        let mut raw: drm_mode_get_plane = Default::default();
        raw.plane_id = id.0;
        unsafe {
            try!(ffi::ioctl_mode_getplane(device.as_raw_fd(), &mut raw));
        }

        let plane = Self {
            id: id,
            crtc_id: CrtcId(raw.crtc_id),
            fb_id: FramebufferId(raw.fb_id),
            gamma_length: raw.gamma_size,
        };

        Ok(plane)
    }

    fn id(&self) -> PlaneId {
        self.id
    }
}

impl ConnectorInfo {
    /// Returns the type of connector this is
    pub fn connector_type(&self) -> ConnectorType {
        self.con_type
    }

    /// Returns the state of this connector.
    pub fn connection_state(&self) -> ConnectorState {
        self.con_state
    }

    /// Returns a slice of each possible mode.
    pub fn modes(&self) -> &[Mode] {
        &self.modes
    }
}

impl EncoderInfo {
    /// Returns the type of encoder this is.
    pub fn encoder_type(&self) -> EncoderType {
        self.enc_type
    }

    /// Returns a CrtcListFilter that can be used to find which Crtc can work
    /// with this Encoder.
    pub fn possible_crtcs(&self) -> CrtcListFilter {
        self.possible_crtcs
    }
}

impl CrtcInfo {
    /// Returns the position the Crtc is attached to.
    pub fn position(&self) -> (u32, u32) {
        self.position
    }

    /// Returns the id of the framebuffer the Crtc is attached to, or None if
    /// not attached.
    pub fn framebuffer(&self) -> Option<FramebufferId> {
        match self.fb.0 {
            0 => None,
            _ => Some(self.fb)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The underlying type of connector.
pub enum ConnectorType {
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
pub enum ConnectorState {
    Connected,
    Disconnected,
    Unknown
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The underlying type of encoder.
pub enum EncoderType {
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

#[derive(Debug, Clone, PartialEq, Eq)]
// TODO: Document
pub struct GammaLookupTable {
    pub red: ffi::Buffer<u16>,
    pub green: ffi::Buffer<u16>,
    pub blue: ffi::Buffer<u16>,
}

#[derive(Debug, Clone, Copy)]
/// A filter that can be used with a ResourceIds to determine the set of Crtcs
/// that can attach to a specific encoder.
pub struct CrtcListFilter(u32);

#[allow(non_upper_case_globals)]
impl From<u32> for ConnectorType {
    fn from(n: u32) -> Self {
        match n {
            DRM_MODE_CONNECTOR_Unknown => ConnectorType::Unknown,
            DRM_MODE_CONNECTOR_VGA => ConnectorType::VGA,
            DRM_MODE_CONNECTOR_DVII => ConnectorType::DVII,
            DRM_MODE_CONNECTOR_DVID => ConnectorType::DVID,
            DRM_MODE_CONNECTOR_DVIA => ConnectorType::DVIA,
            DRM_MODE_CONNECTOR_Composite => ConnectorType::Composite,
            DRM_MODE_CONNECTOR_SVIDEO => ConnectorType::SVideo,
            DRM_MODE_CONNECTOR_LVDS => ConnectorType::LVDS,
            DRM_MODE_CONNECTOR_Component => ConnectorType::Component,
            DRM_MODE_CONNECTOR_9PinDIN => ConnectorType::NinePinDIN,
            DRM_MODE_CONNECTOR_DisplayPort => ConnectorType::DisplayPort,
            DRM_MODE_CONNECTOR_HDMIA => ConnectorType::HDMIA,
            DRM_MODE_CONNECTOR_HDMIB => ConnectorType::HDMIB,
            DRM_MODE_CONNECTOR_TV => ConnectorType::TV,
            DRM_MODE_CONNECTOR_eDP => ConnectorType::EmbeddedDisplayPort,
            DRM_MODE_CONNECTOR_VIRTUAL => ConnectorType::Virtual,
            DRM_MODE_CONNECTOR_DSI => ConnectorType::DSI,
            DRM_MODE_CONNECTOR_DPI => ConnectorType::DPI,
            _ => ConnectorType::Unknown
        }
    }
}

impl From<u32> for ConnectorState {
    fn from(n: u32) -> Self {
        // These variables are not defined in drm_mode.h for some reason.
        // They were copied from libdrm's xf86DrmMode.h
        match n {
            1 => ConnectorState::Connected,
            2 => ConnectorState::Disconnected,
            _ => ConnectorState::Unknown
        }
    }
}

impl From<u32> for EncoderType {
    fn from(n: u32) -> Self {
        match n {
            DRM_MODE_ENCODER_NONE => EncoderType::None,
            DRM_MODE_ENCODER_DAC => EncoderType::DAC,
            DRM_MODE_ENCODER_TMDS => EncoderType::TMDS,
            DRM_MODE_ENCODER_LVDS => EncoderType::LVDS,
            DRM_MODE_ENCODER_TVDAC => EncoderType::TVDAC,
            DRM_MODE_ENCODER_VIRTUAL => EncoderType::Virtual,
            DRM_MODE_ENCODER_DSI => EncoderType::DSI,
            DRM_MODE_ENCODER_DPMST => EncoderType::DPMST,
            DRM_MODE_ENCODER_DPI => EncoderType::DPI,
            _ => EncoderType::None
        }
    }
}

// TODO: Implement PartialEq and Eq
#[derive(Debug, Clone, Copy)]
pub struct Mode {
    // We're using the FFI struct because the DRM API expects it when giving it
    // to a CRTC or creating a blob from it. Maybe in the future we can look at
    // another option.
    mode: drm_mode_modeinfo
}

impl Mode {
    /// Returns the clock speed of this mode.
    pub fn clock(&self) -> u32 {
        self.mode.clock
    }

    /// Returns the size (resolution) of the mode.
    pub fn size(&self) -> (u16, u16) {
        (self.mode.hdisplay, self.mode.vdisplay)
    }

    /// Returns the horizontal sync start, end, and total.
    pub fn hsync(&self) -> (u16, u16, u16) {
        (self.mode.hsync_start, self.mode.hsync_end, self.mode.htotal)
    }

    /// Returns the vertical sync start, end, and total.
    pub fn vsync(&self) -> (u16, u16, u16) {
        (self.mode.vsync_start, self.mode.vsync_end, self.mode.vtotal)
    }

    /// Returns the horizontal skew of this mode.
    pub fn hskew(&self) -> u16 {
        self.mode.hskew
    }

    /// Returns the vertical scan of this mode.
    pub fn vscan(&self) -> u16 {
        self.mode.vscan
    }

    /// Returns the name of the mode.
    pub fn name(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(&self.mode.name as *const _)
        }
    }
}
