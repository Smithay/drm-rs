use result::*;
use ffi;
use std::ffi::CStr;
use std::hash::Hash;
pub mod connector;
pub mod encoder;
pub mod crtc;
pub mod framebuffer;
pub mod plane;
pub mod property;
pub mod dumbbuffer;

/// The underlying id for a resource.
pub type RawHandle = u32;

#[derive(Clone, Copy, PartialEq, Eq, Hash, From, Into)]
/// An array to hold the name of a property.
pub struct RawName([i8; 32]);

/// A trait for devices that provide control (modesetting) functionality.
pub trait Device: Sized + super::Device {
    /// See [`ResourceHandles::load_from_device`]
    ///
    /// [`ResourceHandles::load_from_device`]:
    ///     ResourceHandles.t.html#method.load_from_device
    fn resource_handles(&self) -> Result<ResourceHandles> {
        ResourceHandles::load_from_device(self)
    }

    /// See [`PlaneResourceHandles::load_from_device`]
    ///
    /// [`PlaneResourceHandles::load_from_device`]:
    ///     PlaneResourceHandles.t.html#method.load_from_device
    fn plane_handles(&self) -> Result<PlaneResourceHandles> {
        PlaneResourceHandles::load_from_device(self)
    }

    /// See [`ResourceInfo::load_from_device`]
    ///
    /// [`ResourceInfo::load_from_device`]:
    ///     ResourceInfo.t.html#method.load_from_device
    fn resource_info<T>(&self, handle: T::Handle) -> Result<T>
    where
        T: ResourceInfo,
    {
        T::load_from_device(self, handle)
    }

    /// Attaches a framebuffer to a CRTC's built-in plane, attaches the CRTC to
    /// a connector, and sets the CRTC's mode to output the pixel data.
    fn set_crtc(
        &self,
        crtc: crtc::Handle,
        fb: framebuffer::Handle,
        cons: &[connector::Handle],
        position: (u32, u32),
        mode: Option<Mode>,
    ) -> Result<()> {
        crtc::set(self, crtc, fb, cons, position, mode)
    }

    /// Creates a framebuffer from a [`Buffer`], returning
    /// [`framebuffer::Info`].
    ///
    /// [`Buffer`]: ../buffer/Buffer.t.html
    /// [`framebuffer::Info`]: framebuffer/Info.t.html
    fn create_framebuffer<U>(&self, buffer: &U) -> Result<framebuffer::Info>
    where
        U: super::buffer::Buffer,
    {
        framebuffer::create(self, buffer)
    }
}

/// A generalization of a handle that represents an object managed by a `Device`.
///
/// In most cases, all objects and resources that are managed by a `Device`
/// provide some sort of handle that we can use to refer to them. Almost all
/// operations performed on a `Device` that use an object or resource require
/// making requests using a handle.
pub trait ResourceHandle: Copy + Eq + Hash + From<RawHandle> + Into<RawHandle> {}

/// Information about a resource or object managed by a `Device`.
///
/// Due to external events such as hot plugging, other tasks, and even buggy
/// drivers, object information could change at any time. In fact, there are no
/// guarantees that this resource has existed or will exist at any point in
/// time. A process should treat a `ResourceInfo` as merely a hint to the
/// current state of the `Device`.
pub trait ResourceInfo: Clone + Eq {
    /// The type of handle used to load this trait.
    type Handle: ResourceHandle;

    /// Load the resource from a `Device` given its `ResourceHandle`
    fn load_from_device<T>(&T, Self::Handle) -> Result<Self>
    where
        T: Device;

    /// Get the `ResourceHandle` for this resource.
    fn handle(&self) -> Self::Handle;
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The set of resource ids that are associated with a DRM device.
pub struct ResourceHandles {
    connectors: ffi::Buffer<connector::Handle>,
    encoders: ffi::Buffer<encoder::Handle>,
    crtcs: ffi::Buffer<crtc::Handle>,
    framebuffers: ffi::Buffer<framebuffer::Handle>,
    width: (u32, u32),
    height: (u32, u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The set of plane ids that are associated with a DRM device.
pub struct PlaneResourceHandles {
    planes: ffi::Buffer<plane::Handle>,
}

impl ResourceHandles {
    /// Attempts to acquire a copy of the device's ResourceHandles.
    pub fn load_from_device<T>(device: &T) -> Result<Self>
    where
        T: Device,
    {
        let fd = device.as_raw_fd();

        let ids = {
            let mut raw: ffi::drm_mode_card_res = Default::default();
            unsafe { ffi::ioctl_mode_getresources(fd, &mut raw)? };

            let ids = ResourceHandles {
                connectors: ffi_buf!(raw.connector_id_ptr, raw.count_connectors),
                encoders: ffi_buf!(raw.encoder_id_ptr, raw.count_encoders),
                crtcs: ffi_buf!(raw.crtc_id_ptr, raw.count_crtcs),
                framebuffers: ffi_buf!(raw.fb_id_ptr, raw.count_fbs),
                width: (raw.min_width, raw.max_width),
                height: (raw.min_height, raw.max_height),
            };

            unsafe {
                try!(ffi::ioctl_mode_getresources(device.as_raw_fd(), &mut raw));
            }

            ids
        };

        Ok(ids)
    }

    /// Returns a slice to the list of connector handles.
    pub fn connectors<'a>(&'a self) -> &'a [connector::Handle] {
        &self.connectors
    }

    /// Returns a slice to the list of encoder handle.
    pub fn encoders<'a>(&'a self) -> &'a [encoder::Handle] {
        &self.encoders
    }

    /// Returns a slice to the list of CRTC handles.
    pub fn crtcs<'a>(&'a self) -> &'a [crtc::Handle] {
        &self.crtcs
    }

    /// Returns a slice to the list of framebuffer handle.
    pub fn framebuffers<'a>(&'a self) -> &'a [framebuffer::Handle] {
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

    pub fn filter_crtcs(&self, filter: CrtcListFilter) -> ffi::Buffer<crtc::Handle> {
        self.crtcs
            .iter()
            .enumerate()
            .filter(|&(n, _)| (1 << n) & filter.0 != 0)
            .map(|(_, &e)| e)
            .collect()
    }
}

impl PlaneResourceHandles {
    /// Loads the plane ids from a device.
    pub fn load_from_device<T>(device: &T) -> Result<Self>
    where
        T: Device,
    {
        let phandles = {
            let mut raw: ffi::drm_mode_get_plane_res = Default::default();
            unsafe {
                try!(ffi::ioctl_mode_getplaneresources(
                    device.as_raw_fd(),
                    &mut raw
                ));
            }

            let phandles = PlaneResourceHandles {
                planes: ffi_buf!(raw.plane_id_ptr, raw.count_planes),
            };

            unsafe {
                try!(ffi::ioctl_mode_getplaneresources(
                    device.as_raw_fd(),
                    &mut raw
                ));
            }

            phandles
        };

        Ok(phandles)
    }

    /// Returns a slice to the list of plane ids.
    pub fn planes<'a>(&'a self) -> &'a [plane::Handle] {
        &self.planes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Type {
    Connector,
    Encoder,
    Property,
    Framebuffer,
    Blob,
    Plane,
    Crtc,
    Unknown,
}

#[warn(non_upper_case_globals)]
impl From<Type> for u32 {
    fn from(n: Type) -> Self {
        match n {
            Type::Connector => ffi::DRM_MODE_OBJECT_CONNECTOR,
            Type::Encoder => ffi::DRM_MODE_OBJECT_ENCODER,
            //Type::Mode => ffi::DRM_MODE_OBJECT_MODE,
            Type::Property => ffi::DRM_MODE_OBJECT_PROPERTY,
            Type::Framebuffer => ffi::DRM_MODE_OBJECT_FB,
            Type::Blob => ffi::DRM_MODE_OBJECT_BLOB,
            Type::Plane => ffi::DRM_MODE_OBJECT_PLANE,
            Type::Crtc => ffi::DRM_MODE_OBJECT_CRTC,
            Type::Unknown => ffi::DRM_MODE_OBJECT_ANY,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A handle to a generic resource id
pub enum ResourceHandleType {
    Connector(connector::Handle),
    Encoder(encoder::Handle),
    Crtc(crtc::Handle),
    Framebuffer(framebuffer::Handle),
    Plane(plane::Handle),
    Property(property::Handle),
}

#[derive(Debug, Clone, PartialEq, Eq)]
// TODO: Document
pub struct GammaLookupTable {
    pub red: ffi::Buffer<u16>,
    pub green: ffi::Buffer<u16>,
    pub blue: ffi::Buffer<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A filter that can be used with a ResourceHandles to determine the set of
/// Crtcs that can attach to a specific encoder.
pub struct CrtcListFilter(u32);

#[derive(Debug, Clone, Copy)]
pub struct Mode {
    // We're using the FFI struct because the DRM API expects it when giving it
    // to a CRTC or creating a blob from it. Maybe in the future we can look at
    // another option.
    mode: ffi::drm_mode_modeinfo,
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
        unsafe { CStr::from_ptr(&self.mode.name as *const _) }
    }
}

// We need to implement PartialEq manually for Mode
impl PartialEq for Mode {
    fn eq(&self, other: &Mode) -> bool {
        self.mode.clock == other.mode.clock && self.mode.clock == other.mode.clock
            && self.mode.hdisplay == other.mode.hdisplay
            && self.mode.hsync_start == other.mode.hsync_start
            && self.mode.hsync_end == other.mode.hsync_end
            && self.mode.htotal == other.mode.htotal && self.mode.hskew == other.mode.hskew
            && self.mode.vdisplay == other.mode.vdisplay
            && self.mode.vsync_start == other.mode.vsync_start
            && self.mode.vsync_end == other.mode.vsync_end
            && self.mode.vtotal == other.mode.vtotal && self.mode.vscan == other.mode.vscan
            && self.mode.vrefresh == other.mode.vrefresh
            && self.mode.flags == other.mode.flags && self.mode.type_ == other.mode.type_
            && self.mode.name == other.mode.name
    }
}

impl Eq for Mode {}

impl ::std::fmt::Debug for RawName {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let cstr = unsafe { CStr::from_ptr(::std::mem::transmute(&self)) };
        write!(f, "{:?}", cstr)
    }
}
