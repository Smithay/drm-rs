//!
//! Foreign function interface
//!

#![allow(dead_code)]
#![allow(missing_docs)]

use nix::libc;
pub use drm_sys::*;

/// The type to be used as an ffi buffer.
pub type Buffer<T> = Vec<T>;

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

/// Gets the bus ID of the device
///
/// # Locks DRM mutex: Yes
/// # Permissions: None
/// # Nodes: Primary
ioctl!(readwrite ioctl_get_unique with DRM_IOCTL_BASE, 0x01; drm_unique);

/// Get information about the client
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary
ioctl!(readwrite ioctl_get_client with DRM_IOCTL_BASE, 0x05; drm_client);

/// Gets statistical information from the device
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary
ioctl!(read ioctl_get_stats with DRM_IOCTL_BASE, 0x06; drm_stats);

/// Get capabilities of the device.
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary, Render
ioctl!(readwrite ioctl_get_cap with DRM_IOCTL_BASE, 0x0c; drm_get_cap);

/// Tells the device we understand a capability
///
/// # Locks DRM mutex: Yes
/// # Permissions: None
/// # Nodes: Primary
ioctl!(write_ptr ioctl_set_client_cap with DRM_IOCTL_BASE, 0x0d; drm_set_client_cap);

/// Sets the requested interface version
///
/// # Locks DRM mutex: Yes
/// # Permissions: Master
/// # Nodes: Primary, control
ioctl!(readwrite ioctl_set_version with DRM_IOCTL_BASE, 0x07; drm_set_version);

/// Gets the current interface version
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: All
ioctl!(readwrite ioctl_version with DRM_IOCTL_BASE, 0x00; drm_version);

/// Authenticates a client via their authentication token
///
/// # Locks DRM mutex: No
/// # Permissions: Auth, Master
/// # Nodes: Primary
ioctl!(write_ptr ioctl_auth_magic with DRM_IOCTL_BASE, 0x11; drm_auth);

/// Generates the client's authentication token
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary
ioctl!(read ioctl_get_magic with DRM_IOCTL_BASE, 0x02; drm_auth);

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

/// Enable the vblank interrupt and sleep until the requested sequence occurs
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary
ioctl!(readwrite ioctl_wait_vblank with DRM_IOCTL_BASE, 0x3a; drm_wait_vblank);

/// Handle vblank counter changes across mode switches
///
/// # Locks DRM mutex: Yes
/// # Permissions: None
/// # Nodes: Primary
ioctl!(write_ptr ioctl_modeset_ctl with DRM_IOCTL_BASE, 0x08; drm_modeset_ctl);


ioctl!(readwrite ioctl_prime_handle_to_fd
       with DRM_IOCTL_BASE, 0x2d; drm_prime_handle);
ioctl!(readwrite ioctl_prime_fd_to_handle
       with DRM_IOCTL_BASE, 0x2e; drm_prime_handle);


ioctl!(readwrite ioctl_mode_getresources
       with DRM_IOCTL_BASE, 0xA0; drm_mode_card_res);
ioctl!(readwrite ioctl_mode_getcrtc with DRM_IOCTL_BASE, 0xA1; drm_mode_crtc);
ioctl!(readwrite ioctl_mode_setcrtc with DRM_IOCTL_BASE, 0xA2; drm_mode_crtc);
ioctl!(readwrite ioctl_mode_cursor with DRM_IOCTL_BASE, 0xA3; drm_mode_cursor);
ioctl!(readwrite ioctl_mode_getgamma
       with DRM_IOCTL_BASE, 0xA4; drm_mode_crtc_lut);
ioctl!(readwrite ioctl_mode_setgamma
       with DRM_IOCTL_BASE, 0xA5; drm_mode_crtc_lut);
ioctl!(readwrite ioctl_mode_getencoder
       with DRM_IOCTL_BASE, 0xA6; drm_mode_get_encoder);
ioctl!(readwrite ioctl_mode_getconnector
       with DRM_IOCTL_BASE, 0xA7; drm_mode_get_connector);

ioctl!(readwrite ioctl_mode_getproperty
       with DRM_IOCTL_BASE, 0xAA; drm_mode_get_property);
ioctl!(readwrite ioctl_mode_setproperty
       with DRM_IOCTL_BASE, 0xAB; drm_mode_connector_set_property);
ioctl!(readwrite ioctl_mode_getpropblob
       with DRM_IOCTL_BASE, 0xAC; drm_mode_get_blob);
ioctl!(readwrite ioctl_mode_getfb with DRM_IOCTL_BASE, 0xAD; drm_mode_fb_cmd);
ioctl!(readwrite ioctl_mode_addfb with DRM_IOCTL_BASE, 0xAE; drm_mode_fb_cmd);
ioctl!(readwrite ioctl_mode_rmfb with DRM_IOCTL_BASE, 0xAF; libc::c_uint);
ioctl!(readwrite ioctl_mode_page_flip
       with DRM_IOCTL_BASE, 0xB0; drm_mode_crtc_page_flip);
ioctl!(readwrite ioctl_mode_dirtyfb
       with DRM_IOCTL_BASE, 0xB1; drm_mode_fb_dirty_cmd);

ioctl!(readwrite ioctl_mode_create_dumb
       with DRM_IOCTL_BASE, 0xB2; drm_mode_create_dumb);
ioctl!(readwrite ioctl_mode_map_dumb
       with DRM_IOCTL_BASE, 0xB3; drm_mode_map_dumb);
ioctl!(readwrite ioctl_mode_destroy_dumb
       with DRM_IOCTL_BASE, 0xB4; drm_mode_destroy_dumb);
ioctl!(readwrite ioctl_mode_getplaneresources
       with DRM_IOCTL_BASE, 0xB5; drm_mode_get_plane_res);
ioctl!(readwrite ioctl_mode_getplane
       with DRM_IOCTL_BASE, 0xB6; drm_mode_get_plane);
ioctl!(readwrite ioctl_mode_setplane
       with DRM_IOCTL_BASE, 0xB7; drm_mode_set_plane);
ioctl!(readwrite ioctl_mode_addfb2 with DRM_IOCTL_BASE, 0xB8; drm_mode_fb_cmd2);
ioctl!(readwrite ioctl_mode_obj_getproperties
       with DRM_IOCTL_BASE, 0xB9; drm_mode_obj_get_properties);
ioctl!(readwrite ioctl_mode_obj_setproperty
       with DRM_IOCTL_BASE, 0xBA; drm_mode_obj_set_property);
ioctl!(readwrite ioctl_mode_cursor2 with DRM_IOCTL_BASE, 0xBB; drm_mode_cursor2);
ioctl!(readwrite ioctl_mode_atomic with DRM_IOCTL_BASE, 0xBC; drm_mode_atomic);
ioctl!(readwrite ioctl_mode_createpropblob
       with DRM_IOCTL_BASE, 0xBD; drm_mode_create_blob);
ioctl!(readwrite ioctl_mode_destroypropblob
       with DRM_IOCTL_BASE, 0xBE; drm_mode_destroy_blob);

ioctl!(write_ptr ioctl_gem_close with DRM_IOCTL_BASE, 0x09; drm_gem_close);
ioctl!(readwrite ioctl_gem_flink with DRM_IOCTL_BASE, 0x0a; drm_gem_flink);
ioctl!(readwrite ioctl_gem_open with DRM_IOCTL_BASE, 0x0b; drm_gem_open);
