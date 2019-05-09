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

use drm_ffi as ffi;
use drm_ffi::result::SystemError;

pub mod connector;
pub mod crtc;
pub mod encoder;
pub mod framebuffer;
pub mod plane;

pub mod property;

use std::mem;

use core::num::NonZeroU32;
type ResourceHandle = NonZeroU32;

#[doc(hidden)]
pub trait ResourceType : AsRef<ResourceHandle> + Copy {
    const FFI_TYPE: u32;
}

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
    /// Gets the set of resource handles that this device currently controls
    fn resource_handles(&self) -> Result<ResourceHandles, SystemError> {
        let mut fbs = [0u32; 32];
        let mut crtcs = [0u32; 32];
        let mut connectors = [0u32; 32];
        let mut encoders = [0u32; 32];

        let mut fb_slice = &mut fbs[..];
        let mut crtc_slice = &mut crtcs[..];
        let mut conn_slice = &mut connectors[..];
        let mut enc_slice = &mut encoders[..];

        let ffi_res = ffi::mode::get_resources(
            self.as_raw_fd(),
            Some(&mut fb_slice),
            Some(&mut crtc_slice),
            Some(&mut conn_slice),
            Some(&mut enc_slice),
            )?;

        let fb_len = fb_slice.len();
        let crtc_len = crtc_slice.len();
        let conn_len = conn_slice.len();
        let enc_len = enc_slice.len();

        let res = ResourceHandles {
            fbs: unsafe { mem::transmute(fbs) },
            fb_len: fb_len,
            crtcs: unsafe { mem::transmute(crtcs) },
            crtc_len: crtc_len,
            connectors: unsafe { mem::transmute(connectors) },
            conn_len: conn_len,
            encoders: unsafe { mem::transmute(encoders) },
            enc_len: enc_len,
            width: (ffi_res.min_width, ffi_res.max_width),
            height: (ffi_res.min_height, ffi_res.max_height),
        };

        Ok(res)
    }

    /// Gets the set of plane handles that this device currently has
    fn plane_handles(&self) -> Result<PlaneResourceHandles, SystemError> {
        let mut planes = [0u32; 32];
        let mut plane_slice = &mut planes[..];

        let _ffi_res = ffi::mode::get_plane_resources(self.as_raw_fd(), Some(&mut plane_slice))?;

        let plane_len = plane_slice.len();

        let res = PlaneResourceHandles {
            planes: unsafe { mem::transmute(planes) },
            plane_len: plane_len,
        };

        Ok(res)
    }

    /// Returns information about a specific connector
    fn get_connector(&self, handle: connector::Handle) -> Result<connector::Info, SystemError> {
        // Maximum number of encoders is 3 due to kernel restrictions
        let mut encoders = [0u32; 3];
        let mut enc_slice = &mut encoders[..];

        let ffi_info = ffi::mode::get_connector(
            self.as_raw_fd(),
            unsafe { mem::transmute(*handle.as_ref()) },
            None,
            None,
            None,
            Some(&mut enc_slice),
            )?;

        let connector = connector::Info {
            handle: handle,
            interface: connector::Interface::from(ffi_info.connector_type),
            interface_id: ffi_info.connector_type_id,
            connection: connector::State::from(ffi_info.connection),
            size: match (ffi_info.mm_width, ffi_info.mm_height) {
                (0, 0) => None,
                (x, y) => Some((x, y)),
            },
            encoders: unsafe { mem::transmute(encoders) },
            curr_enc: unsafe { mem::transmute(ffi_info.encoder_id) },
        };

        Ok(connector)
    }

    /// Returns information about a specific encoder
    fn get_encoder(&self, handle: encoder::Handle) -> Result<encoder::Info, SystemError> {
        let info = ffi::mode::get_encoder(
            self.as_raw_fd(),
            unsafe { mem::transmute(*handle.as_ref()) },
            )?;

        let enc = encoder::Info {
            handle: handle,
            enc_type: encoder::Kind::from(info.encoder_type),
            crtc: unsafe { mem::transmute(info.crtc_id) },
            pos_crtcs: info.possible_crtcs,
            pos_clones: info.possible_clones,
        };

        Ok(enc)
    }

    /// Returns information about a specific CRTC
    fn get_crtc(&self, handle: crtc::Handle) -> Result<crtc::Info, SystemError> {
        let info = ffi::mode::get_crtc(
            self.as_raw_fd(),
            unsafe { mem::transmute(*handle.as_ref()) },
            )?;

        let crtc = crtc::Info {
            handle: handle,
            position: (info.x, info.y),
            mode: match info.mode_valid {
                0 => None,
                _ => Some(Mode::from(info.mode)),
            },
            fb: unsafe { mem::transmute(info.fb_id) },
            gamma_length: info.gamma_size,
        };

        Ok(crtc)
    }

    /// Returns information about a specific framebuffer
    fn get_framebuffer(
        &self,
        handle: framebuffer::Handle,
        ) -> Result<framebuffer::Info, SystemError> {
        let info = ffi::mode::get_framebuffer(
            self.as_raw_fd(),
            unsafe { mem::transmute(*handle.as_ref()) },
            )?;

        let fb = framebuffer::Info {
            handle: handle,
            size: (info.width, info.height),
            pitch: info.pitch,
            bpp: info.bpp,
            depth: info.depth,
            buffer: info.handle,
        };

        Ok(fb)
    }

    /// Returns information about a specific plane
    fn get_plane(&self, handle: plane::Handle) -> Result<plane::Info, SystemError> {
        let mut formats = [0u32; 8];
        let mut fmt_slice = &mut formats[..];

        let info = ffi::mode::get_plane(
            self.as_raw_fd(),
            unsafe { mem::transmute(*handle.as_ref()) },
            Some(&mut fmt_slice)
            )?;

        let fmt_len = fmt_slice.len();

        let plane = plane::Info {
            handle: handle,
            crtc: unsafe { mem::transmute(info.crtc_id) },
            fb: unsafe { mem::transmute(info.fb_id) },
            pos_crtcs: info.possible_crtcs,
            formats: formats,
            fmt_len: fmt_len
        };

        Ok(plane)
    }

    /// Returns information about a specific property.
    fn get_property(&self, handle: property::Handle) -> Result<property::Info, SystemError> {
        let mut values = [0u64; 24];
        let mut enums = [ffi::drm_mode_property_enum::default(); 24];

        let mut val_slice = &mut values[..];
        let mut enum_slice = &mut enums[..];

        let info = ffi::mode::get_property(
            self.as_raw_fd(),
            unsafe { mem::transmute(*handle.as_ref()) },
            Some(&mut val_slice),
            Some(&mut enum_slice)
            )?;

        let val_len = val_slice.len();
        let enum_len = enum_slice.len();

        let val_type = {
            use self::property::ValueType;
            let flags = info.flags;

            if flags & ffi::DRM_MODE_PROP_RANGE != 0 {
                let min = values[0];
                let max = values[1];

                match (min, max) {
                    (0, 1) => ValueType::Boolean,
                    (min, max) => ValueType::UnsignedRange(min, max)
                }
            } else if flags & ffi::DRM_MODE_PROP_SIGNED_RANGE != 0 {
                let min = values[0];
                let max = values[1];

                ValueType::SignedRange(min as i64, max as i64)
            } else if flags & ffi::DRM_MODE_PROP_ENUM != 0 {
                let enum_values = self::property::EnumValues {
                    values: values,
                    enums: unsafe { mem::transmute(enums) },
                    length: val_len
                };

                ValueType::Enum(enum_values)
            } else if flags & ffi::DRM_MODE_PROP_BLOB != 0 {
                ValueType::Blob
            } else if flags & ffi::DRM_MODE_PROP_BITMASK != 0 {
                ValueType::Bitmask
            } else if flags & ffi::DRM_MODE_PROP_OBJECT != 0 {
                match values[0] as u32 {
                    ffi::DRM_MODE_OBJECT_CRTC => ValueType::CRTC,
                    ffi::DRM_MODE_OBJECT_CONNECTOR => ValueType::Connector,
                    ffi::DRM_MODE_OBJECT_ENCODER => ValueType::Encoder,
                    ffi::DRM_MODE_OBJECT_FB => ValueType::Framebuffer,
                    ffi::DRM_MODE_OBJECT_PLANE => ValueType::Plane,
                    ffi::DRM_MODE_OBJECT_PROPERTY => ValueType::Property,
                    ffi::DRM_MODE_OBJECT_BLOB => ValueType::Blob,
                    ffi::DRM_MODE_OBJECT_ANY => ValueType::Object,
                    _ => ValueType::Unknown,
                }
            } else {
                ValueType::Unknown
            }
        };

        let property = property::Info {
            handle: handle,
            val_type: val_type,
            mutable: info.flags & ffi::DRM_MODE_PROP_IMMUTABLE == 0,
            atomic: info.flags & ffi::DRM_MODE_PROP_ATOMIC == 0,
            info: info
        };

        Ok(property)
    }

    /// Sets a property for a specific resource.
    fn set_property<T: ResourceType>(
        &self,
        handle: T,
        prop: property::Handle,
        value: property::RawValue
        ) -> Result<(), SystemError> {

        ffi::mode::set_property(
            self.as_raw_fd(),
            unsafe { mem::transmute(*prop.as_ref()) },
            unsafe { mem::transmute(*handle.as_ref()) },
            T::FFI_TYPE,
            value
            )?;

        Ok(())
    }

    /// Returns the set of `Mode`s that a particular connector supports.
    fn get_modes(&self, handle: connector::Handle) -> Result<ModeList, SystemError> {
        let mut modes = [ffi::drm_mode_modeinfo::default(); 38];
        let mut mode_slice = &mut modes[..];

        let _ffi_info = ffi::mode::get_connector(
            self.as_raw_fd(),
            unsafe { mem::transmute(*handle.as_ref()) },
            None,
            None,
            Some(&mut mode_slice),
            None,
            )?;

        let mode_len = mode_slice.len();

        let list = ModeList {
            modes: unsafe { mem::transmute(modes) },
            mode_len: mode_len
        };

        Ok(list)
    }

    /// Gets a list of property handles and values for this resource.
    fn get_properties<T: ResourceType>(&self, handle: T) -> Result<PropertyValueSet, SystemError> {
        let mut prop_ids = [0u32; 32];
        let mut prop_vals = [0u64; 32];

        let mut prop_id_slice = &mut prop_ids[..];
        let mut prop_val_slice = &mut prop_vals[..];

        ffi::mode::get_properties(
            self.as_raw_fd(),
            unsafe { mem::transmute(*handle.as_ref()) },
            T::FFI_TYPE,
            Some(&mut prop_id_slice),
            Some(&mut prop_val_slice),
            )?;

        let prop_len = prop_id_slice.len();

        let prop_val_set = PropertyValueSet {
            prop_ids: unsafe { mem::transmute(prop_ids) },
            prop_vals: unsafe { mem::transmute(prop_vals) },
            len: prop_len
        };

        Ok(prop_val_set)
    }
}

/// The set of [ResourceHandles](ResourceHandle.t.html) that a
/// [Device](Device.t.html) exposes. Excluding Plane resources.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct ResourceHandles {
    fbs: [Option<framebuffer::Handle>; 32],
    fb_len: usize,
    crtcs: [Option<crtc::Handle>; 32],
    crtc_len: usize,
    connectors: [Option<connector::Handle>; 32],
    conn_len: usize,
    encoders: [Option<encoder::Handle>; 32],
    enc_len: usize,
    width: (u32, u32),
    height: (u32, u32),
}

impl ResourceHandles {
    /// Returns the set of [connector::Handles](connector/Handle.t.html)
    pub fn connectors(&self) -> &[connector::Handle] {
        let buf_len = std::cmp::min(self.connectors.len(), self.conn_len);
        unsafe { mem::transmute(&self.connectors[..buf_len]) }
    }

    /// Returns the set of [encoder::Handles](encoder/Handle.t.html)
    pub fn encoders(&self) -> &[encoder::Handle] {
        let buf_len = std::cmp::min(self.encoders.len(), self.enc_len);
        unsafe { mem::transmute(&self.encoders[..buf_len]) }
    }

    /// Returns the set of [crtc::Handles](crtc/Handle.t.html)
    pub fn crtcs(&self) -> &[crtc::Handle] {
        let buf_len = std::cmp::min(self.crtcs.len(), self.crtc_len);
        unsafe { mem::transmute(&self.crtcs[..buf_len]) }
    }

    /// Returns the set of [framebuffer::Handles](framebuffer/Handle.t.html)
    pub fn framebuffers(&self) -> &[framebuffer::Handle] {
        let buf_len = std::cmp::min(self.fbs.len(), self.fb_len);
        unsafe { mem::transmute(&self.fbs[..buf_len]) }
    }
}

impl std::fmt::Debug for ResourceHandles {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ResourceHandles")
            .field("fbs", &self.framebuffers())
            .field("crtcs", &self.crtcs())
            .field("connectors", &self.connectors())
            .field("encoders", &self.encoders())
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

/// The set of [plane::Handles](plane/Handle.t.html) that a
/// [Device](Device.t.html) exposes.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct PlaneResourceHandles {
    planes: [Option<plane::Handle>; 32],
    plane_len: usize,
}

impl PlaneResourceHandles {
    /// Returns the set of [plane::Handles](plane/Handle.t.html)
    pub fn planes(&self) -> &[plane::Handle] {
        let buf_len = std::cmp::min(self.planes.len(), self.plane_len);
        unsafe { mem::transmute(&self.planes[..buf_len]) }
    }
}

impl std::fmt::Debug for PlaneResourceHandles {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("PlaneResourceHandles")
            .field("planes", &self.planes())
            .finish()
    }
}

/// Resolution and timing information for a display mode.
#[repr(transparent)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Mode {
    // We're using the FFI struct because the DRM API expects it when giving it
    // to a CRTC or creating a blob from it. Rather than rearranging the fields
    // to convert to/from an abstracted type, just use the raw object.
    mode: ffi::drm_mode_modeinfo,
}

impl Mode {
    /// Returns the name of this mode.
    pub fn name(&self) -> &std::ffi::CStr {
        unsafe { std::ffi::CStr::from_ptr(&self.mode.name[0] as _) }
    }

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
}

impl From<ffi::drm_mode_modeinfo> for Mode {
    fn from(raw: ffi::drm_mode_modeinfo) -> Mode {
        Mode { mode: raw }
    }
}

impl Into<ffi::drm_mode_modeinfo> for Mode {
    fn into(self) -> ffi::drm_mode_modeinfo {
        self.mode
    }
}

impl std::fmt::Debug for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Mode")
            .field("name", &self.name())
            .field("clock", &self.clock())
            .field("size", &self.size())
            .field("hsync", &self.hsync())
            .field("vsync", &self.vsync())
            .field("hskew", &self.hskew())
            .field("vscan", &self.vscan())
            .field("vrefresh", &self.vrefresh())
            .finish()
    }
}

/// A simple list of `Mode`s
#[derive(Copy, Clone)]
pub struct ModeList {
    modes: [Mode; 38],
    mode_len: usize
}

impl ModeList {
    /// Returns the list as a slice.
    pub fn modes(&self) -> &[Mode] {
        let buf_len = std::cmp::min(self.modes.len(), self.mode_len);
        unsafe { mem::transmute(&self.modes[..buf_len]) }
    }
}

impl std::fmt::Debug for ModeList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ModeList")
            .field("modes", &self.modes())
            .finish()
    }
}

/// Wrapper around a set of property IDs and their raw values.
#[derive(Debug, Copy, Clone)]
pub struct PropertyValueSet {
    prop_ids: [Option<property::Handle>; 32],
    prop_vals: [property::RawValue; 32],
    len: usize
}

impl PropertyValueSet {
    /// Returns a pair representing a set of [property::Handles](property/Handle.t.html) and their raw values
    pub fn as_props_and_values(&self) -> (&[property::Handle], &[property::RawValue]) {
        unsafe {
            mem::transmute((&self.prop_ids[..self.len], &self.prop_vals[..self.len]))
        }
    }
}
