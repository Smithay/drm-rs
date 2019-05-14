#![allow(missing_docs)]

use drm_sys::*;

/// Gets the bus ID of the device
///
/// # Locks DRM mutex: Yes
/// # Permissions: None
/// # Nodes: Primary
ioctl_readwrite!(get_bus_id, DRM_IOCTL_BASE, 0x01, drm_unique);

/// Get information about the client
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary
ioctl_readwrite!(get_client, DRM_IOCTL_BASE, 0x05, drm_client);

/// Gets statistical information from the device
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary
ioctl_read!(get_stats, DRM_IOCTL_BASE, 0x06, drm_stats);

/// Get capabilities of the device.
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary, Render
ioctl_readwrite!(get_cap, DRM_IOCTL_BASE, 0x0c, drm_get_cap);

/// Tells the device we understand a capability
///
/// # Locks DRM mutex: Yes
/// # Permissions: None
/// # Nodes: Primary
ioctl_write_ptr!(set_cap, DRM_IOCTL_BASE, 0x0d, drm_set_client_cap);

/// Sets the requested interface version
///
/// # Locks DRM mutex: Yes
/// # Permissions: Master
/// # Nodes: Primary, control
ioctl_readwrite!(set_version, DRM_IOCTL_BASE, 0x07, drm_set_version);

/// Gets the current interface version
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: All
ioctl_readwrite!(get_version, DRM_IOCTL_BASE, 0x00, drm_version);

/// Generates the client's authentication token
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary
ioctl_read!(get_token, DRM_IOCTL_BASE, 0x02, drm_auth);

/// Authenticates a client via their authentication token
///
/// # Locks DRM mutex: No
/// # Permissions: Auth, Master
/// # Nodes: Primary
ioctl_write_ptr!(auth_token, DRM_IOCTL_BASE, 0x11, drm_auth);

/// Acquires the DRM Master lock
///
/// # Locks DRM mutex: No
/// # Permissions: Root
/// # Nodes: Primary
ioctl_none!(acquire_master, DRM_IOCTL_BASE, 0x1e);

/// Drops the DRM Master lock
///
/// # Locks DRM mutex: No
/// # Permissions: Root
/// # Nodes: Primary
ioctl_none!(release_master, DRM_IOCTL_BASE, 0x1f);

/// Gets the IRQ number
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary
ioctl_readwrite!(get_irq_from_bus_id, DRM_IOCTL_BASE, 0x03, drm_irq_busid);

/// Enable the vblank interrupt and sleep until the requested sequence occurs
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary
ioctl_readwrite!(wait_vblank, DRM_IOCTL_BASE, 0x3a, drm_wait_vblank);

pub(crate) mod mode {
    use drm_sys::*;
    use nix::libc::c_uint;

    /// Modesetting resources
    ioctl_readwrite!(get_resources, DRM_IOCTL_BASE, 0xA0, drm_mode_card_res);

    ioctl_readwrite!(get_plane_resources, DRM_IOCTL_BASE, 0xB5, drm_mode_get_plane_res);

    /// Connector related functions
    ioctl_readwrite!(get_connector, DRM_IOCTL_BASE, 0xA7, drm_mode_get_connector);

    /// Encoder related functions
    ioctl_readwrite!(get_encoder, DRM_IOCTL_BASE, 0xA6, drm_mode_get_encoder);

    /// CRTC related functions
    ioctl_readwrite!(get_crtc, DRM_IOCTL_BASE, 0xA1, drm_mode_crtc);
    ioctl_readwrite!(set_crtc, DRM_IOCTL_BASE, 0xA2, drm_mode_crtc);

    /// Gamma related functions
    ioctl_readwrite!(get_gamma, DRM_IOCTL_BASE, 0xA4, drm_mode_crtc_lut);
    ioctl_readwrite!(set_gamma, DRM_IOCTL_BASE, 0xA5, drm_mode_crtc_lut);

    // TODO: Figure out GAMMA LUT arrays

    /// FB related functions
    ioctl_readwrite!(get_fb, DRM_IOCTL_BASE, 0xAD, drm_mode_fb_cmd);
    ioctl_readwrite!(add_fb, DRM_IOCTL_BASE, 0xAE, drm_mode_fb_cmd);
    ioctl_readwrite!(add_fb2, DRM_IOCTL_BASE, 0xB8, drm_mode_fb_cmd2);
    ioctl_readwrite!(rm_fb, DRM_IOCTL_BASE, 0xAF, c_uint);

    /// Plane related functions
    ioctl_readwrite!(get_plane, DRM_IOCTL_BASE, 0xB6, drm_mode_get_plane);

    ioctl_readwrite!(set_plane, DRM_IOCTL_BASE, 0xB7, drm_mode_set_plane);

    /// Dumbbuffer related functions
    ioctl_readwrite!(create_dumb, DRM_IOCTL_BASE, 0xB2, drm_mode_create_dumb);

    ioctl_readwrite!(map_dumb, DRM_IOCTL_BASE, 0xB3, drm_mode_map_dumb);

    ioctl_readwrite!(destroy_dumb, DRM_IOCTL_BASE, 0xB4, drm_mode_destroy_dumb);

    /// Cursor related functions
    ioctl_readwrite!(cursor, DRM_IOCTL_BASE, 0xA3, drm_mode_cursor);
    ioctl_readwrite!(cursor2, DRM_IOCTL_BASE, 0xBB, drm_mode_cursor2);

    /// Property related functions
    ioctl_readwrite!(get_property, DRM_IOCTL_BASE, 0xAA, drm_mode_get_property);

    ioctl_readwrite!(connector_set_property, DRM_IOCTL_BASE, 0xAB, drm_mode_connector_set_property);

    ioctl_readwrite!(obj_get_properties, DRM_IOCTL_BASE, 0xB9, drm_mode_obj_get_properties);

    ioctl_readwrite!(obj_set_property, DRM_IOCTL_BASE, 0xBA, drm_mode_obj_set_property);

    /// Property blobs
    ioctl_readwrite!(get_blob, DRM_IOCTL_BASE, 0xAC, drm_mode_get_blob);

    // TODO: Property blobs probably require a large buffer

    ioctl_readwrite!(create_blob, DRM_IOCTL_BASE, 0xBD, drm_mode_create_blob);

    ioctl_readwrite!(destroy_blob, DRM_IOCTL_BASE, 0xBE, drm_mode_destroy_blob);

    /// Atomic modesetting related functions
    ioctl_readwrite!(crtc_page_flip, DRM_IOCTL_BASE, 0xB0, drm_mode_crtc_page_flip);

    ioctl_readwrite!(dirty_fb, DRM_IOCTL_BASE, 0xB1, drm_mode_fb_dirty_cmd);

    ioctl_readwrite!(atomic, DRM_IOCTL_BASE, 0xBC, drm_mode_atomic);
}

pub(crate) mod gem {
    use drm_sys::*;

    /// GEM related functions
    ioctl_readwrite!(open, DRM_IOCTL_BASE, 0x0b, drm_gem_open);
    ioctl_write_ptr!(close, DRM_IOCTL_BASE, 0x09, drm_gem_close);

    /// Converts a buffer handle into a dma-buf file descriptor.
    ioctl_readwrite!(prime_handle_to_fd, DRM_IOCTL_BASE, 0x2d, drm_prime_handle);

    /// Converts a dma-buf file descriptor into a buffer handle.
    ioctl_readwrite!(prime_fd_to_handle, DRM_IOCTL_BASE, 0x2e, drm_prime_handle);
}
