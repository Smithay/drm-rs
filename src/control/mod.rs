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
        // Buffers to hold the handles.
        let mut fbs = [0u32; 32];
        let mut crtcs = [0u32; 32];
        let mut connectors = [0u32; 32];
        let mut encoders = [0u32; 32];

        let (ffi_card, fb_len, crtc_len, conn_len, enc_len) = {
            let mut fb_slice = &mut fbs[..];
            let mut crtc_slice = &mut crtcs[..];
            let mut conn_slice = &mut connectors[..];
            let mut enc_slice = &mut encoders[..];

            let ffi_card = ffi::mode::get_resources(
                self.as_raw_fd(),
                &mut fb_slice,
                &mut crtc_slice,
                &mut conn_slice,
                &mut enc_slice,
            ).map_err(|e| SystemError::from(result::unwrap_errno(e)))?;

            (
                ffi_card,
                fb_slice.len(),
                crtc_slice.len(),
                conn_slice.len(),
                enc_slice.len(),
            )
        };

        let res = unsafe {
            ResourceHandles {
                fbs: SmallBuffer::new(mem::transmute(fbs), fb_len),
                crtcs: SmallBuffer::new(mem::transmute(crtcs), crtc_len),
                connectors: SmallBuffer::new(mem::transmute(connectors), conn_len),
                encoders: SmallBuffer::new(mem::transmute(encoders), enc_len),
                width: (ffi_card.min_width, ffi_card.max_width),
                height: (ffi_card.min_height, ffi_card.max_height),
            }
        };

        Ok(res)
    }

    /// Gets the set of plane handles that this device currently has
    fn plane_handles(&self) -> Result<PlaneResourceHandles, SystemError> {
        let mut planes = [0u32; 32];

        let len = {
            let mut plane_slice = &mut planes[..];

            ffi::mode::get_plane_resources(self.as_raw_fd(), &mut plane_slice)
                .map_err(|e| SystemError::from(result::unwrap_errno(e)))?;

            plane_slice.len()
        };

        let res = unsafe {
            PlaneResourceHandles {
                planes: SmallBuffer::new(mem::transmute(planes), len),
            }
        };

        Ok(res)
    }

    /// Returns information about a specific connector
    fn get_connector(&self, handle: connector::Handle) -> Result<connector::Info, SystemError> {
        // Maximum number of encoders is 3 due to the kernel
        let mut encoders = [0u32; 3];

        let (enc_len, info) = {
            let mut enc_slice = &mut encoders[..];

            let info = ffi::mode::get_connector_without_props_or_modes(
                self.as_raw_fd(),
                handle.into(),
                &mut enc_slice,
            ).map_err(|e| SystemError::from(result::unwrap_errno(e)))?;

            (enc_slice.len(), info)
        };

        let conn = unsafe {
            connector::Info {
                handle: handle,
                id: (connector::Kind::from(info.connector_type), info.connector_type_id),
                connection: connector::State::from(info.connection),
                size: (info.mm_width, info.mm_height),
                subpixel: (),
                encoders: connector::EncodersBuffer::new(mem::transmute(encoders), enc_len),
                curr_enc: match info.encoder_id {
                    0 => None,
                    x => Some(encoder::Handle::from(x)),
                },
            }
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
        Ok(crtc::Info)
    }

    /// Returns information about a specific framebuffer
    fn get_framebuffer(
        &self,
        handle: framebuffer::Handle,
    ) -> Result<framebuffer::Info, SystemError> {
        Ok(framebuffer::Info)
    }

    /// Returns information about a specific plane
    fn get_plane(&self, handle: plane::Handle) -> Result<plane::Info, SystemError> {
        Ok(plane::Info)
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// The set of [ResourceHandles](ResourceHandle.t.html) that a
/// [Device](Device.t.html) exposes. Excluding Plane resources.
pub struct ResourceHandles {
    fbs: SmallBuffer<framebuffer::Handle>,
    crtcs: SmallBuffer<crtc::Handle>,
    connectors: SmallBuffer<connector::Handle>,
    encoders: SmallBuffer<encoder::Handle>,
    width: (u32, u32),
    height: (u32, u32),
}

impl ResourceHandles {
    /// Returns the set of [connector::Handles](connector/Handle.t.html)
    pub fn connectors(&self) -> &[connector::Handle] {
        self.connectors.as_ref()
    }

    /// Returns the set of [encoder::Handles](encoder/Handle.t.html)
    pub fn encoders(&self) -> &[encoder::Handle] {
        self.encoders.as_ref()
    }

    /// Returns the set of [crtc::Handles](crtc/Handle.t.html)
    pub fn crtcs(&self) -> &[crtc::Handle] {
        self.crtcs.as_ref()
    }

    /// Returns the set of [framebuffer::Handles](framebuffer/Handle.t.html)
    pub fn framebuffers(&self) -> &[framebuffer::Handle] {
        self.fbs.as_ref()
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// The set of [plane::Handles](plane/Handle.t.html) that a
/// [Device](Device.t.html) exposes.
pub struct PlaneResourceHandles {
    planes: SmallBuffer<plane::Handle>,
}

impl PlaneResourceHandles {
    /// Returns the set of [plane::Handles](plane/Handle.t.html)
    pub fn planes(&self) -> &[plane::Handle] {
        self.planes.as_ref()
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
