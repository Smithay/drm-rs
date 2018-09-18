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

use ffi;
use result;
use result::SystemError;
use util::*;

use std::mem;

pub mod connector;
pub mod crtc;
pub mod encoder;
pub mod framebuffer;
pub mod plane;

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
        let mut fbs: HandleBuffer32<framebuffer::Handle> = Default::default();
        let mut crtcs: HandleBuffer32<crtc::Handle> = Default::default();
        let mut connectors: HandleBuffer32<connector::Handle> = Default::default();
        let mut encoders: HandleBuffer32<encoder::Handle> = Default::default();

        let mut fb_slice = fbs.as_mut_u32_slice();
        let mut crtc_slice = crtcs.as_mut_u32_slice();
        let mut conn_slice = connectors.as_mut_u32_slice();
        let mut enc_slice = encoders.as_mut_u32_slice();

        let ffi_card = ffi::mode::get_resources(
            self.as_raw_fd(),
            &mut fb_slice,
            &mut crtc_slice,
            &mut conn_slice,
            &mut enc_slice,
        ).map_err(|e| SystemError::from(result::unwrap_errno(e)))?;

        fbs.len = fb_slice.len();
        crtcs.len = crtc_slice.len();
        connectors.len = conn_slice.len();
        encoders.len = enc_slice.len();

        let res = ResourceHandles {
            fbs: fbs,
            crtcs: crtcs,
            connectors: connectors,
            encoders: encoders,
            width: (ffi_card.min_width, ffi_card.max_width),
            height: (ffi_card.min_height, ffi_card.max_height),
        };

        Ok(res)
    }

    /// Gets the set of plane handles that this device currently has
    fn plane_handles(&self) -> Result<PlaneResourceHandles, SystemError> {
        let mut planes: HandleBuffer32<plane::Handle> = Default::default();
        let mut plane_slice = planes.as_mut_u32_slice();

        ffi::mode::get_plane_resources(self.as_raw_fd(), &mut plane_slice)
            .map_err(|e| SystemError::from(result::unwrap_errno(e)))?;

        planes.len = plane_slice.len();

        let res = PlaneResourceHandles {
            planes: planes,
        };

        Ok(res)
    }

    /// Returns information about a specific connector
    fn get_connector(&self, handle: connector::Handle) -> Result<connector::Info, SystemError> {
        // Maximum number of encoders is 3 due to kernel restrictions
        let mut encoders: HandleBuffer3<encoder::Handle> = Default::default();
        let mut enc_slice = encoders.as_mut_u32_slice();

        let info = ffi::mode::get_connector_without_props_or_modes(
            self.as_raw_fd(),
            handle.into(),
            &mut enc_slice,
        ).map_err(|e| SystemError::from(result::unwrap_errno(e)))?;

        let conn = connector::Info {
            handle: handle,
            kind: connector::Kind::from(info.connector_type),
            id: info.connector_type_id,
            connection: connector::State::from(info.connection),
            size: (info.mm_width, info.mm_height),
            subpixel: (),
            encoders: encoders,
            curr_enc: match info.encoder_id {
                0 => None,
                x => Some(encoder::Handle::from(x)),
            },
        };

        Ok(conn)
    }

    /// Returns information about a specific encoder
    fn get_encoder(&self, handle: encoder::Handle) -> Result<encoder::Info, SystemError> {
        let info = ffi::mode::get_encoder(self.as_raw_fd(), handle.into())
            .map_err(|e| SystemError::from(result::unwrap_errno(e)))?;

        let enc = encoder::Info {
            handle: handle,
            enc_type: encoder::Kind::from(info.encoder_type),
            crtc: match info.crtc_id {
                0 => None,
                x => Some(crtc::Handle::from(x)),
            },
            pos_crtcs: info.possible_crtcs,
            pos_clones: info.possible_clones,
        };

        Ok(enc)
    }

    /// Returns information about a specific CRTC
    fn get_crtc(&self, handle: crtc::Handle) -> Result<crtc::Info, SystemError> {
        let info = ffi::mode::get_crtc(self.as_raw_fd(), handle.into())
            .map_err(|e| SystemError::from(result::unwrap_errno(e)))?;

        let crtc = crtc::Info {
            handle: handle,
            position: (info.x, info.y),
            mode: match info.mode_valid {
                0 => None,
                _ => Some(Mode::from(info.mode)),
            },
            fb: match info.fb_id {
                0 => None,
                x => Some(framebuffer::Handle::from(x)),
            },
            gamma_length: info.gamma_size,
        };

        Ok(crtc)
    }

    /// Returns information about a specific framebuffer
    fn get_framebuffer(
        &self,
        handle: framebuffer::Handle,
    ) -> Result<framebuffer::Info, SystemError> {
        let info = ffi::mode::get_framebuffer(self.as_raw_fd(), handle.into())
            .map_err(|e| SystemError::from(result::unwrap_errno(e)))?;

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
        let mut formats = [0u32; 32];

        let (fmt_len, info) = {
            let mut fmt_slice = &mut formats[..];
            let info = ffi::mode::get_plane(self.as_raw_fd(), handle.into(), &mut fmt_slice)
                .map_err(|e| SystemError::from(result::unwrap_errno(e)))?;

            let plane = plane::Info {
                handle: handle,
                crtc: match info.crtc_id {
                    0 => None,
                    x => Some(crtc::Handle::from(x)),
                },
                fb: match info.fb_id {
                    0 => None,
                    x => Some(framebuffer::Handle::from(x)),
                },
                pos_crtcs: info.possible_crtcs,
                gamma_size: info.gamma_size,
            };

            (fmt_slice.len(), plane)
        };

        println!("{:?}", formats);

        Ok(info)
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// The set of [ResourceHandles](ResourceHandle.t.html) that a
/// [Device](Device.t.html) exposes. Excluding Plane resources.
pub struct ResourceHandles {
    fbs: HandleBuffer32<framebuffer::Handle>,
    crtcs: HandleBuffer32<crtc::Handle>,
    connectors: HandleBuffer32<connector::Handle>,
    encoders: HandleBuffer32<encoder::Handle>,
    width: (u32, u32),
    height: (u32, u32),
}

impl ResourceHandles {
    /// Returns the set of [connector::Handles](connector/Handle.t.html)
    pub fn connectors(&self) -> &[connector::Handle] {
        self.connectors.as_slice()
    }

    /// Returns the set of [encoder::Handles](encoder/Handle.t.html)
    pub fn encoders(&self) -> &[encoder::Handle] {
        self.encoders.as_slice()
    }

    /// Returns the set of [crtc::Handles](crtc/Handle.t.html)
    pub fn crtcs(&self) -> &[crtc::Handle] {
        self.crtcs.as_slice()
    }

    /// Returns the set of [framebuffer::Handles](framebuffer/Handle.t.html)
    pub fn framebuffers(&self) -> &[framebuffer::Handle] {
        self.fbs.as_slice()
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// The set of [plane::Handles](plane/Handle.t.html) that a
/// [Device](Device.t.html) exposes.
pub struct PlaneResourceHandles {
    planes: HandleBuffer32<plane::Handle>,
}

impl PlaneResourceHandles {
    /// Returns the set of [plane::Handles](plane/Handle.t.html)
    pub fn planes(&self) -> &[plane::Handle] {
        self.planes.as_slice()
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Mode {
    // We're using the FFI struct because the DRM API expects it when giving it
    // to a CRTC or creating a blob from it. Rather than rearranging the fields
    // to convert to/from an abstracted type, just use the raw object.
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

use std::marker::PhantomData;

/// Buffer that can hold up to 32 handles.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct HandleBuffer32<T> where T: From<u32> + Into<u32> {
    data: [u32; 32],
    len: usize,
    _phantom: PhantomData<T>
}

impl<T> Default for HandleBuffer32<T> where T: From<u32> + Into<u32> {
    fn default() -> HandleBuffer32<T> {
        HandleBuffer32 {
            len: 32,
            ..Default::default()
        }
    }
}

impl<T> HandleBuffer32<T> where T: From<u32> + Into<u32> {
    /// Returns the data as an immutable `u32` slice.
    fn as_u32_slice(&self) -> &[u32] {
        &self.data[..self.len]
    }

    /// Returns the data as a mutable `u32` slice.
    fn as_mut_u32_slice(&mut self) -> &mut [u32] {
        &mut self.data[..self.len]
    }

    /// Returns the underlying data as an immutable slice.
    unsafe fn as_slice(&self) -> &[T] {
        mem::transmute(self.as_u32_slice())
    }

    /// Returns the underlying data as a mutable slice.
    unsafe fn as_mut_slice(&mut self) -> &mut [T] {
        mem::transmute(self.as_mut_u32_slice())
    }
}

/// Buffer that can hold up to 3 handles.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct HandleBuffer3<T> where T: From<u32> + Into<u32> {
    data: [u32; 3],
    len: usize,
    _phantom: PhantomData<T>
}

impl<T> Default for HandleBuffer3<T> where T: From<u32> + Into<u32> {
    fn default() -> HandleBuffer3<T> {
        HandleBuffer3 {
            len: 3,
            ..Default::default()
        }
    }
}

impl<T> HandleBuffer3<T> where T: From<u32> + Into<u32> {
    /// Returns the data as an immutable `u32` slice.
    fn as_u32_slice(&self) -> &[u32] {
        &self.data[..self.len]
    }

    /// Returns the data as a mutable `u32` slice.
    fn as_mut_u32_slice(&mut self) -> &mut [u32] {
        &mut self.data[..self.len]
    }

    /// Returns the underlying data as an immutable slice.
    unsafe fn as_slice(&self) -> &[T] {
        mem::transmute(self.as_u32_slice())
    }

    /// Returns the underlying data as a mutable slice.
    unsafe fn as_mut_slice(&mut self) -> &mut [T] {
        mem::transmute(self.as_mut_u32_slice())
    }
}
