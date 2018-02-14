//!
//! Foreign function interface
//!

#![allow(dead_code)]
#![allow(missing_docs)]

use nix::libc::{c_int, c_char, uint32_t, uint64_t};
pub use drm_sys::*;

mod ioctl;

/// Wrappers for the raw DRM structures.
pub(crate) trait Wrapper {
    type Raw;
    type Err;
    fn raw_mut_ref(&mut self) -> &mut Self::Raw;
    fn raw_ref(&self) -> &Self::Raw;
    fn ioctl(&mut self, fd: c_int) -> Result<(), Self::Err>;
}

macro_rules! impl_wrapper {
    ($type:ty, $raw:ty, $ioctl:expr) => {
        impl Wrapper for $type {
            type Raw = $raw;
            type Err = super::nix::Error;

            fn raw_mut_ref(&mut self) -> &mut Self::Raw {
                &mut self.0
            }

            fn raw_ref(&self) -> &Self::Raw {
                &self.0
            }

            fn ioctl(&mut self, fd: c_int) -> Result<(), Self::Err> {
                unsafe { $ioctl(fd, &mut self.0)? };
                Ok(())
            }
        }
    };
    (full $type:ty, $raw:ty, $ioctl:expr) => {
        impl Wrapper for $type {
            type Raw = $raw;
            type Err = super::nix::Error;

            fn raw_mut_ref(&mut self) -> &mut Self::Raw {
                &mut self.raw
            }

            fn raw_ref(&self) -> &Self::Raw {
                &self.raw
            }

            fn ioctl(&mut self, fd: c_int) -> Result<(), Self::Err> {
                use ffi::PrepareBuffers;
                self.prepare_buffers();
                unsafe { $ioctl(fd, &mut self.raw)? };
                Ok(())
            }
        }
    };
}

/// Many DRM structures have fields that act as pointers to buffers. In libdrm,
/// these buffers are allocated at runtime using `drmMalloc` after determining
/// the size of the buffer.
///
/// However, these buffers tend to be extremely tiny in nature. Therefore, we
/// wrap the DRM structures in a new type that also owns these buffers as
/// fixed-sized arrays. This provides us with two benefits:
///
/// 1. We only need to make the ioctl call once.
/// 2. We do not need to allocate memory on the heap.
///
/// If the actual number of elements exceeds our fixed-length array though, then
/// we will only fill the number of elements we can contain. If this happens on
/// a particular system, it's recommended to increase the length of these buffers
/// and consider filing a bug report.
pub type Buffer<T> = [T; 32];

/// For wrappers that need the buffers mentioned above, we implement this trait
/// to set up the inner DRM structure's fields to point to them properly.
pub trait PrepareBuffers {
    fn prepare_buffers(&mut self);
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct BusID {
    raw: drm_unique,
    pub unique_buf: Buffer<c_char>
}

impl_wrapper!(full BusID, drm_unique, ioctl::get_bus_id);

impl PrepareBuffers for BusID {
    fn prepare_buffers(&mut self) {
        self.raw.unique = (&mut self.unique_buf).as_mut_ptr();
        self.raw.unique_len = self.unique_buf.len() as u64;
    }
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct Client(drm_client);
impl_wrapper!(Client, drm_client, ioctl::get_client);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct Stats(drm_stats);
impl_wrapper!(Stats, drm_stats, ioctl::get_stats);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct GetCap(drm_get_cap);
impl_wrapper!(GetCap, drm_get_cap, ioctl::get_cap);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct SetCap(drm_set_client_cap);
impl_wrapper!(SetCap, drm_set_client_cap, ioctl::set_cap);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct SetVersion(drm_set_version);
impl_wrapper!(SetVersion, drm_set_version, ioctl::set_version);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct GetVersion(drm_version);
impl_wrapper!(GetVersion, drm_version, ioctl::get_version);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct GetToken(drm_auth);
impl_wrapper!(GetToken, drm_auth, ioctl::get_token);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct AuthToken(drm_auth);
impl_wrapper!(AuthToken, drm_auth, ioctl::auth_token);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct IRQControl(drm_control);
impl_wrapper!(IRQControl, drm_control, ioctl::irq_control);

#[derive(Default, Copy, Clone, From, Into)]
pub(crate) struct WaitVBlank(drm_wait_vblank);
impl_wrapper!(WaitVBlank, drm_wait_vblank, ioctl::wait_vblank);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct ModesetCtl(drm_modeset_ctl);
impl_wrapper!(ModesetCtl, drm_modeset_ctl, ioctl::modeset_ctl);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct PrimeHandleToFD(drm_prime_handle);
impl_wrapper!(PrimeHandleToFD, drm_prime_handle, ioctl::prime_handle_to_fd);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct PrimeFDToHandle(drm_prime_handle);
impl_wrapper!(PrimeFDToHandle, drm_prime_handle, ioctl::prime_fd_to_handle);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct ModeCardRes {
    raw: drm_mode_card_res,
    pub con_buf: Buffer<uint32_t>,
    pub enc_buf: Buffer<uint32_t>,
    pub crtc_buf: Buffer<uint32_t>,
    pub fb_buf: Buffer<uint32_t>
}

impl_wrapper!(full ModeCardRes, drm_mode_card_res, ioctl::mode::get_resources);

impl PrepareBuffers for ModeCardRes {
    fn prepare_buffers(&mut self) {
        self.raw.connector_id_ptr = (&mut self.con_buf).as_mut_ptr() as u64;
        self.raw.count_connectors = self.con_buf.len() as u32;
        self.raw.encoder_id_ptr = (&mut self.enc_buf).as_mut_ptr() as u64;
        self.raw.count_encoders = self.enc_buf.len() as u32;
        self.raw.crtc_id_ptr = (&mut self.crtc_buf).as_mut_ptr() as u64;
        self.raw.count_crtcs = self.crtc_buf.len() as u32;
        self.raw.fb_id_ptr = (&mut self.fb_buf).as_mut_ptr() as u64;
        self.raw.count_fbs = self.fb_buf.len() as u32;
    }
}
#[derive(Debug, Default, Copy, Clone, Hash)]
pub(crate) struct ModePlaneRes {
    pub raw: drm_mode_get_plane_res,
    pub plane_buf: Buffer<uint32_t>
}

impl_wrapper!(full ModePlaneRes, drm_mode_get_plane_res,
              ioctl::mode::get_plane_resources);

impl PrepareBuffers for ModePlaneRes {
    fn prepare_buffers(&mut self) {
        self.raw.plane_id_ptr = (&mut self.plane_buf).as_mut_ptr() as u64;
        self.raw.count_planes = self.plane_buf.len() as u32;
    }
}

#[derive(Debug, Default, Copy, Clone, Hash)]
pub(crate) struct DRMModeGetConnector {
    pub raw: drm_mode_get_connector,
    pub enc_buf: Buffer<uint32_t>,
    pub prop_buf: Buffer<uint32_t>,
    pub prop_val_buf: Buffer<uint64_t>,
    pub mode_buf: Buffer<drm_mode_modeinfo>
}

impl PrepareBuffers for DRMModeGetConnector {
    fn prepare_buffers(&mut self) {
        self.raw.encoders_ptr = (&mut self.enc_buf).as_mut_ptr() as u64;
        self.raw.count_encoders = self.enc_buf.len() as u32;
        self.raw.props_ptr = (&mut self.prop_buf).as_mut_ptr() as u64;
        self.raw.prop_values_ptr = (&mut self.prop_val_buf).as_mut_ptr() as u64;
        self.raw.count_props = self.prop_buf.len() as u32;
        self.raw.modes_ptr = (&mut self.mode_buf).as_mut_ptr() as u64;
        self.raw.count_modes = self.mode_buf.len() as u32;
    }
}
#[derive(Debug, Default, Copy, Clone, Hash)]
pub(crate) struct DRMModeCrtc {
    pub raw: drm_mode_crtc,
    pub conn_buf: Buffer<uint32_t>
}

impl PrepareBuffers for DRMModeCrtc {
    fn prepare_buffers(&mut self) {
        self.raw.set_connectors_ptr = (&mut self.conn_buf).as_mut_ptr() as u64;
        self.raw.count_connectors = self.conn_buf.len() as u32;
    }
}
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct DRMModeFbCmd(pub drm_mode_fb_cmd);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct DRMModeFbCmd2(pub drm_mode_fb_cmd2);
#[derive(Debug, Default, Copy, Clone, Hash)]
pub(crate) struct DRMModeGetPlane {
    pub raw: drm_mode_get_plane,
    pub format_buf: Buffer<uint32_t>
}

impl PrepareBuffers for DRMModeGetPlane {
    fn prepare_buffers(&mut self) {
        self.raw.format_type_ptr = (&mut self.format_buf).as_mut_ptr() as u64;
        self.raw.count_format_types = self.format_buf.len() as u32;
    }
}
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct DRMModeSetPlane(pub drm_mode_set_plane);
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct DRMModeCreateDumb(pub drm_mode_create_dumb);
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct DRMModeMapDumb(pub drm_mode_map_dumb);
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct DRMModeDestroyDumb(pub drm_mode_destroy_dumb);
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct DRMModeCursor(pub drm_mode_cursor);
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct DRMModeCursor2(pub drm_mode_cursor2);
// TODO: Requires some extra work for setting up buffers

#[derive(Debug, Default, Copy, Clone, Hash)]
pub(crate) struct DRMModeGetProperty {
    pub raw: drm_mode_get_property,
}
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct DRMModeConnectorSetProperty(drm_mode_connector_set_property);
#[derive(Debug, Default, Copy, Clone, Hash)]
pub(crate) struct DRMModeObjGetProperties {
    pub raw: drm_mode_obj_get_properties,
    pub prop_buf: Buffer<uint32_t>,
    pub vals_buf: Buffer<uint64_t>
}
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct DRMModeObjSetProperty(pub drm_mode_obj_set_property);
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct DRMModeCreateBlob(pub drm_mode_create_blob);
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct DRMModeDestroyBlob(pub drm_mode_destroy_blob);
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct DRMModeCrtcPageFlip(pub drm_mode_crtc_page_flip);
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub(crate) struct DRMModeFBDirtyCmd(pub drm_mode_fb_dirty_cmd);
#[derive(Debug, Default, Copy, Clone, Hash)]
pub(crate) struct DRMModeAtomic {
    pub raw: drm_mode_atomic,
    pub objs_buf: Buffer<uint32_t>,
    pub count_props_buf: Buffer<uint32_t>,
    pub props_buf: Buffer<uint32_t>,
    pub vals_buf: Buffer<uint64_t>
}

impl PrepareBuffers for DRMModeAtomic {
    fn prepare_buffers(&mut self) {
        self.raw.objs_ptr = (&mut self.objs_buf).as_mut_ptr() as u64;
        self.raw.count_props_ptr = (&mut self.count_props_buf).as_mut_ptr() as u64;
        self.raw.props_ptr = (&mut self.props_buf).as_mut_ptr() as u64;
        self.raw.prop_values_ptr = (&mut self.vals_buf).as_mut_ptr() as u64;
        self.raw.count_objs = self.objs_buf.len() as u32;
    }
}
