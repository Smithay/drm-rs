use drm_sys::*;
use result::*;
use ffi;

use std::ffi::CStr;

pub mod connector;
pub mod encoder;
pub mod crtc;
pub mod framebuffer;
pub mod plane;
pub mod property;

/// The underlying id for a resource.
pub type RawId = u32;

#[derive(Clone, Copy, PartialEq, Eq)]
/// An array to hold the name of a property.
pub struct RawName([i8; 32]);

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

    fn connector_info(&self, handle: connector::Id) -> Result<connector::Info> {
        connector::Info::load_from_device(self, handle)
    }

    fn encoder_info(&self, handle: encoder::Id) -> Result<encoder::Info> {
        encoder::Info::load_from_device(self, handle)
    }

    fn crtc_info(&self, handle: crtc::Id) -> Result<crtc::Info> {
        crtc::Info::load_from_device(self, handle)
    }

    fn fb_info(&self, handle: framebuffer::Id) -> Result<framebuffer::Info> {
        framebuffer::Info::load_from_device(self, handle)
    }

    fn plane_info(&self, handle: plane::Id) -> Result<plane::Info> {
        plane::Info::load_from_device(self, handle)
    }

    fn property_info(&self, handle: property::Id) -> Result<property::Info> {
        property::Info::load_from_device(self, handle)
    }

}

/// A generalization of a handle that represents an object managed by a `Device`.
///
/// In most cases, all objects and resources that are managed by a `Device`
/// provide some sort of handle that we can use to refer to them. Almost all
/// operations performed on a `Device` that use an object or resource require
/// making requests using a handle.
pub trait ResourceHandle: Eq + Copy {
    /// The underlying handle type.
    type RawHandle;

    /// Create this handle from its raw part.
    fn from_raw(Self::RawHandle) -> Self;

    /// Get the raw part from this handle.
    fn as_raw(&self) -> Self::RawHandle;
}

/// Information about a resource or object managed by a `Device`.
///
/// Due to external events such as hot plugging, other tasks, and even buggy
/// drivers, object information could change at any time. In fact, there are no
/// guarantees that this resource has existed or will exist at any point in
/// time. A process should treat a `ResourceInfo` as merely a hint to the
/// current state of the `Device`.
pub trait ResourceInfo : Clone + Eq {
    /// The type of handle used to load this trait.
    type Handle: ResourceHandle;

    /// Load the resource from a `Device` given its `ResourceHandle`
    fn load_from_device<T>(&T, Self::Handle) -> Result<Self> where T: Device;

    /// Get the `ResourceHandle` for this resource.
    fn handle(&self) -> Self::Handle;
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The set of resource ids that are associated with a DRM device.
pub struct ResourceIds {
    connectors: ffi::Buffer<connector::Id>,
    encoders: ffi::Buffer<encoder::Id>,
    crtcs: ffi::Buffer<crtc::Id>,
    framebuffers: ffi::Buffer<framebuffer::Id>,
    width: (u32, u32),
    height: (u32, u32)
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The set of plane ids that are associated with a DRM device.
pub struct PlaneResourceIds {
    planes: ffi::Buffer<plane::Id>
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
    pub fn connectors<'a>(&'a self) -> &'a [connector::Id] {
        &self.connectors
    }

    /// Returns a slice to the list of encoder ids.
    pub fn encoders<'a>(&'a self) -> &'a [encoder::Id] {
        &self.encoders
    }

    /// Returns a slice to the list of crtc ids.
    pub fn crtcs<'a>(&'a self) -> &'a [crtc::Id] {
        &self.crtcs
    }

    /// Returns a slice to the list of framebuffer ids.
    pub fn framebuffers<'a>(&'a self) -> &'a [framebuffer::Id] {
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

    pub fn filter_crtcs(&self, filter: CrtcListFilter) -> ffi::Buffer<crtc::Id> {
        self.crtcs.iter().enumerate().filter(| &(n, _) | {
            (1 << n) & filter.0 != 0
        }).map(| (_, &e) | e).collect()
    }
}

impl PlaneResourceIds {
    /// Loads the plane ids from a device.
    pub fn load_from_device<T>(device: &T) -> Result<Self>
        where T: Device {

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
    pub fn planes<'a>(&'a self) -> &'a [plane::Id] {
        &self.planes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Connector,
    Encoder,
    Property,
    Framebuffer,
    Blob,
    Plane,
    Crtc,
    Unknown
}

#[warn(non_upper_case_globals)]
impl From<Type> for u32 {
    fn from(n: Type) -> Self {
        match n {
            Type::Connector => DRM_MODE_OBJECT_CONNECTOR,
            Type::Encoder => DRM_MODE_OBJECT_ENCODER,
            //Type::Mode => DRM_MODE_OBJECT_MODE,
            Type::Property => DRM_MODE_OBJECT_PROPERTY,
            Type::Framebuffer => DRM_MODE_OBJECT_FB,
            Type::Blob => DRM_MODE_OBJECT_BLOB,
            Type::Plane => DRM_MODE_OBJECT_PLANE,
            Type::Crtc => DRM_MODE_OBJECT_CRTC,
            Type::Unknown => DRM_MODE_OBJECT_ANY,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// A ResourceId for a Property.
pub struct PropertyId(RawId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A handle to a generic resource id
pub enum ResourceIdType {
    Connector(connector::Id),
    Encoder(encoder::Id),
    Crtc(crtc::Id),
    Framebuffer(framebuffer::Id),
    Plane(plane::Id),
    Property(property::Id)
}

#[derive(Debug, Clone, PartialEq, Eq)]
// TODO: Document
pub struct GammaLookupTable {
    pub red: ffi::Buffer<u16>,
    pub green: ffi::Buffer<u16>,
    pub blue: ffi::Buffer<u16>,
}

#[derive(Clone, Copy)]
/// A filter that can be used with a ResourceIds to determine the set of Crtcs
/// that can attach to a specific encoder.
pub struct CrtcListFilter(u32);

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


impl ::std::fmt::Debug for RawName {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let cstr = unsafe {
            CStr::from_ptr(::std::mem::transmute(&self))
        };
        write!(f, "{:?}", cstr)
    }
}
