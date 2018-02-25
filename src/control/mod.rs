//! Modesetting operations that the DRM subsystem exposes.
//!
//! # Summary
//!
//! The DRM subsystem provides Kernel Modesetting (KMS) functionality by
//! exposing the following resource types:
//!
//! * FrameBuffer - Specific to an individual process, these wrap around generic
//! GPU buffers so that they can be attached to a Plane.
//!
//! * Planes - Dedicated memory objects which contain a buffer that can then be
//! scanned out by a CRTC. There exist a few different types of planes depending
//! on the use case.
//!
//! * CRTC - Scanout engines that read pixel data from a Plane and sends it to
//! a Connector. Each CRTC has at least one Primary Plane.
//!
//! * Connector - Respresents the physical output, such as a DisplayPort or
//! VGA connector.
//!
//! * Encoder - Encodes pixel data from a CRTC into something a Connector can
//! understand.
//!
//! Further details on each resource can be found in their respective modules.
//!
//! # Usage
//!
//! To begin using modesetting functionality, the [Device trait](Device.t.html)
//! must be implemented on top of the [basic Device trait](../Device.t.html).

use ffi::{self, Wrapper, mode::RawHandle};
use result::Result;

mod debug;

pub mod connector;
pub mod encoder;
pub mod crtc;
pub mod framebuffer;
//pub mod plane;
//pub mod property;
//pub mod dumbbuffer;

/// This trait should be implemented by any object that acts as a DRM device and
/// provides modesetting functionality.
///
/// Like the parent [Device](../Device.t.html) trait, this crate does not
/// provide a concrete object for this trait.
///
/// # Example
/// ```
/// use drm::control::Device as ControlDevice;
///
/// // Assuming the `Card` wrapper already implements drm::Device
/// impl ControlDevice for Card {}
/// ```
pub trait Device: super::Device {
    /// Gets the set of resource handles that this device currently controls.
    fn resource_handles(&self) -> Result<ResourceHandles> {
        let mut t = ffi::mode::CardRes::default();
        t.ioctl(self.as_raw_fd())?;
        Ok(ResourceHandles(t))
    }

    /// Gets the set of plane handles that this device currently controls.
    fn plane_handles(&self) -> Result<PlaneResourceHandles> {
        let mut t = ffi::mode::PlaneRes::default();
        t.ioctl(self.as_raw_fd())?;
        Ok(PlaneResourceHandles(t))
    }

    /// Returns detailed information of an object given its handle.
    fn info<T: ResourceInfo>(&self, handle: T::Handle) -> Result<T>
    where Self: Sized {
        T::load_from_device(self, handle)
    }
}

/// Objects that derive this trait are handles to device resources, and are
/// used for nearly all modesetting operations.
pub trait ResourceHandle: From<RawHandle> + Into<RawHandle> {
    #[doc(hidden)]
    const DEBUG_NAME: &'static str;

    /// Returns None if the input is zero.
    fn from_checked(n: RawHandle) -> Option<Self> {
        match n {
            0 => None,
            id => Some(Self::from(id))
        }
    }
}

/// Objects that derive this trait can be retrieved using a
/// [ResourceHandle](ResourceHandle.t.html) on a [Device](Device.t.html)
pub trait ResourceInfo: Sized {
    #[allow(missing_docs)]
    type Handle: ResourceHandle;
    /// Attempts to retrieve information about a resource.
    fn load_from_device<T: Device>(&T, Self::Handle) -> Result<Self>;
    /// Returns the handle used to acquire this object.
    fn handle(&self) -> Self::Handle;
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
/// The set of [ResourceHandles](ResourceHandle.t.html) that a
/// [Device](Device.t.html) exposes. Excluding Plane resources.
pub struct ResourceHandles(ffi::mode::CardRes);

impl ResourceHandles {
    /// Returns the set of [connector::Handles](connector/Handle.t.html)
    pub fn connectors(&self) -> &[connector::Handle] {
        slice_from_wrapper!(self.0, conn_buf, count_connectors)
    }

    /// Returns the set of [encoder::Handles](encoder/Handle.t.html)
    pub fn encoders(&self) -> &[encoder::Handle] {
        slice_from_wrapper!(self.0, enc_buf, count_encoders)
    }

    /// Returns the set of [crtc::Handles](crtc/Handle.t.html)
    pub fn crtcs(&self) -> &[crtc::Handle] {
        slice_from_wrapper!(self.0, crtc_buf, count_crtcs)
    }

    /// Returns the set of [framebuffer::Handles](framebuffer/Handle.t.html)
    pub fn framebuffers(&self) -> &[framebuffer::Handle] {
        slice_from_wrapper!(self.0, fb_buf, count_fbs)
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
/// The set of [plane::Handles](plane/Handle.t.html) that a
/// [Device](Device.t.html) exposes.
pub struct PlaneResourceHandles(ffi::mode::PlaneRes);

impl PlaneResourceHandles {
    /// Returns the set of [plane::Handles](plane/Handle.t.html)
    pub fn planes(&self) -> &[u32] {
        slice_from_wrapper!(self.0, plane_buf, count_planes)
    }
}

/*
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Type {
    Connector,
    Encoder,
    Property, Framebuffer, Blob,
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

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// A handle to a generic resource id
pub enum ResourceHandleType {
    Connector(connector::Handle),
    Encoder(encoder::Handle),
    Crtc(crtc::Handle),
    Framebuffer(framebuffer::Handle),
    Plane(plane::Handle),
    Property(property::Handle),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
// TODO: Document
pub struct GammaLookupTable {
    pub red: ffi::Buffer<u16>,
    pub green: ffi::Buffer<u16>,
    pub blue: ffi::Buffer<u16>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
/// A filter that can be used with a ResourceHandles to determine the set of
/// Crtcs that can attach to a specific encoder.
pub struct CrtcListFilter(u32);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
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

    /// Returns the vertical refresh rate of this mode
    pub fn vrefresh(&self) -> u32 {
        self.mode.vrefresh
    }

    /// Returns the name of the mode.
    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr(&self.mode.name as *const _) }
    }
}

*/
