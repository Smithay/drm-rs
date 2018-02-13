//!
//! Foreign function interface
//!

#![allow(dead_code)]
#![allow(missing_docs)]

use nix::libc::{c_uint, c_char, uint32_t, uint64_t};
pub use drm_sys::*;

/// The type to be used as an ffi buffer.
pub type Buffer<T> = Vec<T>;

pub type FFIBuffer<T> = [T; 32];

// Creates a buffer to be modified by an FFI function.
macro_rules! ffi_buf {
    ( $ptr:expr, $sz:expr) => (
        {
            use std::mem;

            let mut buf = unsafe { vec![mem::zeroed(); $sz as usize] };
            *(&mut $ptr) = unsafe { mem::transmute(buf.as_mut_ptr()) };
            buf
        }
    )
}

pub(crate) trait FFIBufferSetup {
    fn ffi_buffer_setup(&mut self);
}

/// Gets the bus ID of the device
///
/// # Locks DRM mutex: Yes
/// # Permissions: None
/// # Nodes: Primary
ioctl!(readwrite ioctl_get_unique with DRM_IOCTL_BASE, 0x01; drm_unique);

#[derive(Debug, Default, Copy, Clone, Hash)]
pub(crate) struct DRMUnique {
    pub raw: drm_unique,
    pub unique_buf: FFIBuffer<c_char>
}

impl FFIBufferSetup for DRMUnique {
    fn ffi_buffer_setup(&mut self) {
        self.raw.unique = (&mut self.unique_buf).as_mut_ptr();
        self.raw.unique_len = self.unique_buf.len() as u64;
    }
}

/// Get information about the client
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary
ioctl!(readwrite ioctl_get_client with DRM_IOCTL_BASE, 0x05; drm_client);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMClient(pub drm_client);

/// Gets statistical information from the device
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary
ioctl!(read ioctl_get_stats with DRM_IOCTL_BASE, 0x06; drm_stats);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMStats(pub drm_stats);

/// Get capabilities of the device.
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary, Render
ioctl!(readwrite ioctl_get_cap with DRM_IOCTL_BASE, 0x0c; drm_get_cap);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMGetCap(pub drm_get_cap);

/// Tells the device we understand a capability
///
/// # Locks DRM mutex: Yes
/// # Permissions: None
/// # Nodes: Primary
ioctl!(write_ptr ioctl_set_client_cap with DRM_IOCTL_BASE, 0x0d; drm_set_client_cap);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMSetClientCap(pub drm_set_client_cap);

/// Sets the requested interface version
///
/// # Locks DRM mutex: Yes
/// # Permissions: Master
/// # Nodes: Primary, control
ioctl!(readwrite ioctl_set_version with DRM_IOCTL_BASE, 0x07; drm_set_version);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMSetVersion(pub drm_set_version);

/// Gets the current interface version
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: All
ioctl!(readwrite ioctl_version with DRM_IOCTL_BASE, 0x00; drm_version);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMVersion(pub drm_version);

/// Generates the client's authentication token
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary
ioctl!(read ioctl_get_magic with DRM_IOCTL_BASE, 0x02; drm_auth);

/// Authenticates a client via their authentication token
///
/// # Locks DRM mutex: No
/// # Permissions: Auth, Master
/// # Nodes: Primary
ioctl!(write_ptr ioctl_auth_magic with DRM_IOCTL_BASE, 0x11; drm_auth);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMAuth(pub drm_auth);

/// Acquires the DRM Master lock
///
/// # Locks DRM mutex: No
/// # Permissions: Root
/// # Nodes: Primary
ioctl!(none ioctl_set_master with DRM_IOCTL_BASE, 0x1e);

/// Drops the DRM Master lock
///
/// # Locks DRM mutex: No
/// # Permissions: Root
/// # Nodes: Primary
ioctl!(none ioctl_drop_master with DRM_IOCTL_BASE, 0x1f);

/// IRQ Control
///
/// # Locks DRM mutex: Yes
/// # Permissions: Root, Master, Auth
/// # Nodes: Primary
ioctl!(write_ptr ioctl_control with DRM_IOCTL_BASE, 0x14; drm_control);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMControl(pub drm_control);

/// Enable the vblank interrupt and sleep until the requested sequence occurs
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary
ioctl!(readwrite ioctl_wait_vblank with DRM_IOCTL_BASE, 0x3a; drm_wait_vblank);

#[derive(Default, Copy, Clone)]
pub(crate) struct DRMWaitVblank(pub drm_wait_vblank);

/// Handle vblank counter changes across mode switches
///
/// # Locks DRM mutex: Yes
/// # Permissions: None
/// # Nodes: Primary
ioctl!(write_ptr ioctl_modeset_ctl with DRM_IOCTL_BASE, 0x08; drm_modeset_ctl);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMModesetCtl(pub drm_modeset_ctl);

/// Converts a buffer handle into a dma-buf file descriptor.
ioctl!(readwrite ioctl_prime_handle_to_fd
       with DRM_IOCTL_BASE, 0x2d; drm_prime_handle);

/// Converts a dma-buf file descriptor into a buffer handle.
ioctl!(readwrite ioctl_prime_fd_to_handle
       with DRM_IOCTL_BASE, 0x2e; drm_prime_handle);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMPrimeHandle(pub drm_prime_handle);

/// Modesetting resources
ioctl!(readwrite ioctl_mode_getresources
       with DRM_IOCTL_BASE, 0xA0; drm_mode_card_res);

#[derive(Debug, Default, Copy, Clone, Hash)]
pub(crate) struct DRMModeCardRes {
    pub raw: drm_mode_card_res,
    pub con_buf: FFIBuffer<uint32_t>,
    pub enc_buf: FFIBuffer<uint32_t>,
    pub crtc_buf: FFIBuffer<uint32_t>,
    pub fb_buf: FFIBuffer<uint32_t>
}

impl FFIBufferSetup for DRMModeCardRes {
    fn ffi_buffer_setup(&mut self) {
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

ioctl!(readwrite ioctl_mode_getplaneresources
       with DRM_IOCTL_BASE, 0xB5; drm_mode_get_plane_res);

#[derive(Debug, Default, Copy, Clone, Hash)]
pub(crate) struct DRMModePlaneRes {
    pub raw: drm_mode_get_plane_res,
    pub plane_buf: FFIBuffer<uint32_t>
}

impl FFIBufferSetup for DRMModePlaneRes {
    fn ffi_buffer_setup(&mut self) {
        self.raw.plane_id_ptr = (&mut self.plane_buf).as_mut_ptr() as u64;
        self.raw.count_planes = self.plane_buf.len() as u32;
    }
}

/// Connector related functions
ioctl!(readwrite ioctl_mode_getconnector
       with DRM_IOCTL_BASE, 0xA7; drm_mode_get_connector);

#[derive(Debug, Default, Copy, Clone, Hash)]
pub(crate) struct DRMModeGetConnector {
    pub raw: drm_mode_get_connector,
    pub enc_buf: FFIBuffer<uint32_t>,
    pub prop_buf: FFIBuffer<uint32_t>,
    pub prop_val_buf: FFIBuffer<uint64_t>,
    pub mode_buf: FFIBuffer<drm_mode_modeinfo>
}

impl FFIBufferSetup for DRMModeGetConnector {
    fn ffi_buffer_setup(&mut self) {
        self.raw.encoders_ptr = (&mut self.enc_buf).as_mut_ptr() as u64;
        self.raw.count_encoders = self.enc_buf.len() as u32;
        self.raw.props_ptr = (&mut self.prop_buf).as_mut_ptr() as u64;
        self.raw.prop_values_ptr = (&mut self.prop_val_buf).as_mut_ptr() as u64;
        self.raw.count_props = self.prop_buf.len() as u32;
        self.raw.modes_ptr = (&mut self.mode_buf).as_mut_ptr() as u64;
        self.raw.count_modes = self.mode_buf.len() as u32;
    }
}

/// Encoder related functions
ioctl!(readwrite ioctl_mode_getencoder
       with DRM_IOCTL_BASE, 0xA6; drm_mode_get_encoder);

/// CRTC related functions
ioctl!(readwrite ioctl_mode_getcrtc with DRM_IOCTL_BASE, 0xA1; drm_mode_crtc);
ioctl!(readwrite ioctl_mode_setcrtc with DRM_IOCTL_BASE, 0xA2; drm_mode_crtc);

#[derive(Debug, Default, Copy, Clone, Hash)]
pub(crate) struct DRMModeCrtc {
    pub raw: drm_mode_crtc,
    pub conn_buf: FFIBuffer<uint32_t>
}

impl FFIBufferSetup for DRMModeCrtc {
    fn ffi_buffer_setup(&mut self) {
        self.raw.set_connectors_ptr = (&mut self.conn_buf).as_mut_ptr() as u64;
        self.raw.count_connectors = self.conn_buf.len() as u32;
    }
}

/// Gamma related functions
ioctl!(readwrite ioctl_mode_getgamma
       with DRM_IOCTL_BASE, 0xA4; drm_mode_crtc_lut);
ioctl!(readwrite ioctl_mode_setgamma
       with DRM_IOCTL_BASE, 0xA5; drm_mode_crtc_lut);

// TODO: Figure out GAMMA LUT arrays

/// FB related functions
ioctl!(readwrite ioctl_mode_getfb with DRM_IOCTL_BASE, 0xAD; drm_mode_fb_cmd);
ioctl!(readwrite ioctl_mode_addfb with DRM_IOCTL_BASE, 0xAE; drm_mode_fb_cmd);
ioctl!(readwrite ioctl_mode_addfb2 with DRM_IOCTL_BASE, 0xB8; drm_mode_fb_cmd2);
ioctl!(readwrite ioctl_mode_rmfb with DRM_IOCTL_BASE, 0xAF; c_uint);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMModeFbCmd(pub drm_mode_fb_cmd);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMModeFbCmd2(pub drm_mode_fb_cmd2);

/// Plane related functions
ioctl!(readwrite ioctl_mode_getplane
       with DRM_IOCTL_BASE, 0xB6; drm_mode_get_plane);

#[derive(Debug, Default, Copy, Clone, Hash)]
pub(crate) struct DRMModeGetPlane {
    pub raw: drm_mode_get_plane,
    pub format_buf: FFIBuffer<uint32_t>
}

impl FFIBufferSetup for DRMModeGetPlane {
    fn ffi_buffer_setup(&mut self) {
        self.raw.format_type_ptr = (&mut self.format_buf).as_mut_ptr() as u64;
        self.raw.count_format_types = self.format_buf.len() as u32;
    }
}

ioctl!(readwrite ioctl_mode_setplane
       with DRM_IOCTL_BASE, 0xB7; drm_mode_set_plane);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMModeSetPlane(pub drm_mode_set_plane);

/// Dumbbuffer related functions
ioctl!(readwrite ioctl_mode_create_dumb
       with DRM_IOCTL_BASE, 0xB2; drm_mode_create_dumb);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMModeCreateDumb(pub drm_mode_create_dumb);

ioctl!(readwrite ioctl_mode_map_dumb
       with DRM_IOCTL_BASE, 0xB3; drm_mode_map_dumb);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMModeMapDumb(pub drm_mode_map_dumb);

ioctl!(readwrite ioctl_mode_destroy_dumb
       with DRM_IOCTL_BASE, 0xB4; drm_mode_destroy_dumb);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMModeDestroyDumb(pub drm_mode_destroy_dumb);

/// Cursor related functions
ioctl!(readwrite ioctl_mode_cursor with DRM_IOCTL_BASE, 0xA3; drm_mode_cursor);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMModeCursor(pub drm_mode_cursor);

ioctl!(readwrite ioctl_mode_cursor2 with DRM_IOCTL_BASE, 0xBB; drm_mode_cursor2);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMModeCursor2(pub drm_mode_cursor2);

/// Property related functions
ioctl!(readwrite ioctl_mode_getproperty
       with DRM_IOCTL_BASE, 0xAA; drm_mode_get_property);

// TODO: Requires some extra work for setting up buffers

#[derive(Debug, Default, Copy, Clone, Hash)]
pub(crate) struct DRMModeGetProperty {
    pub raw: drm_mode_get_property,
}

ioctl!(readwrite ioctl_mode_setproperty
       with DRM_IOCTL_BASE, 0xAB; drm_mode_connector_set_property);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMModeConnectorSetProperty(drm_mode_connector_set_property);

ioctl!(readwrite ioctl_mode_obj_getproperties
       with DRM_IOCTL_BASE, 0xB9; drm_mode_obj_get_properties);

#[derive(Debug, Default, Copy, Clone, Hash)]
pub(crate) struct DRMModeObjGetProperties {
    pub raw: drm_mode_obj_get_properties,
    pub prop_buf: FFIBuffer<uint32_t>,
    pub vals_buf: FFIBuffer<uint64_t>
}

ioctl!(readwrite ioctl_mode_obj_setproperty
       with DRM_IOCTL_BASE, 0xBA; drm_mode_obj_set_property);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMModeObjSetProperty(pub drm_mode_obj_set_property);

/// Property blobs
ioctl!(readwrite ioctl_mode_getpropblob
       with DRM_IOCTL_BASE, 0xAC; drm_mode_get_blob);

// TODO: Property blobs probably require a large buffer

ioctl!(readwrite ioctl_mode_createpropblob
       with DRM_IOCTL_BASE, 0xBD; drm_mode_create_blob);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMModeCreateBlob(pub drm_mode_create_blob);

ioctl!(readwrite ioctl_mode_destroypropblob
       with DRM_IOCTL_BASE, 0xBE; drm_mode_destroy_blob);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMModeDestroyBlob(pub drm_mode_destroy_blob);

/// Atomic modesetting related functions
ioctl!(readwrite ioctl_mode_page_flip
       with DRM_IOCTL_BASE, 0xB0; drm_mode_crtc_page_flip);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMModeCrtcPageFlip(pub drm_mode_crtc_page_flip);

ioctl!(readwrite ioctl_mode_dirtyfb
       with DRM_IOCTL_BASE, 0xB1; drm_mode_fb_dirty_cmd);

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DRMModeFBDirtyCmd(pub drm_mode_fb_dirty_cmd);

ioctl!(readwrite ioctl_mode_atomic with DRM_IOCTL_BASE, 0xBC; drm_mode_atomic);

#[derive(Debug, Default, Copy, Clone, Hash)]
pub(crate) struct DRMModeAtomic {
    pub raw: drm_mode_atomic,
    pub objs_buf: FFIBuffer<uint32_t>,
    pub count_props_buf: FFIBuffer<uint32_t>,
    pub props_buf: FFIBuffer<uint32_t>,
    pub vals_buf: FFIBuffer<uint64_t>
}

impl FFIBufferSetup for DRMModeAtomic {
    fn ffi_buffer_setup(&mut self) {
        self.raw.objs_ptr = (&mut self.objs_buf).as_mut_ptr() as u64;
        self.raw.count_props_ptr = (&mut self.count_props_buf).as_mut_ptr() as u64;
        self.raw.props_ptr = (&mut self.props_buf).as_mut_ptr() as u64;
        self.raw.prop_values_ptr = (&mut self.vals_buf).as_mut_ptr() as u64;
        self.raw.count_objs = self.objs_buf.len() as u32;
    }
}


/// GEM related functions
ioctl!(write_ptr ioctl_gem_close with DRM_IOCTL_BASE, 0x09; drm_gem_close);
ioctl!(readwrite ioctl_gem_flink with DRM_IOCTL_BASE, 0x0a; drm_gem_flink);
ioctl!(readwrite ioctl_gem_open with DRM_IOCTL_BASE, 0x0b; drm_gem_open);
