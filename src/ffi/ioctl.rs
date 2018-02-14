use drm_sys::*;

/// Gets the bus ID of the device
///
/// # Locks DRM mutex: Yes
/// # Permissions: None
/// # Nodes: Primary
ioctl!(readwrite get_bus_id with DRM_IOCTL_BASE, 0x01; drm_unique);

/// Get information about the client
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary
ioctl!(readwrite get_client with DRM_IOCTL_BASE, 0x05; drm_client);

/// Gets statistical information from the device
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary
ioctl!(read get_stats with DRM_IOCTL_BASE, 0x06; drm_stats);

/// Get capabilities of the device.
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary, Render
ioctl!(readwrite get_cap with DRM_IOCTL_BASE, 0x0c; drm_get_cap);

/// Tells the device we understand a capability
///
/// # Locks DRM mutex: Yes
/// # Permissions: None
/// # Nodes: Primary
ioctl!(write_ptr set_cap with DRM_IOCTL_BASE, 0x0d; drm_set_client_cap);

/// Sets the requested interface version
///
/// # Locks DRM mutex: Yes
/// # Permissions: Master
/// # Nodes: Primary, control
ioctl!(readwrite set_version with DRM_IOCTL_BASE, 0x07; drm_set_version);

/// Gets the current interface version
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: All
ioctl!(readwrite get_version with DRM_IOCTL_BASE, 0x00; drm_version);

/// Generates the client's authentication token
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary
ioctl!(read get_token with DRM_IOCTL_BASE, 0x02; drm_auth);

/// Authenticates a client via their authentication token
///
/// # Locks DRM mutex: No
/// # Permissions: Auth, Master
/// # Nodes: Primary
ioctl!(write_ptr auth_token with DRM_IOCTL_BASE, 0x11; drm_auth);

/// Acquires the DRM Master lock
///
/// # Locks DRM mutex: No
/// # Permissions: Root
/// # Nodes: Primary
ioctl!(none acquire_master with DRM_IOCTL_BASE, 0x1e);

/// Drops the DRM Master lock
///
/// # Locks DRM mutex: No
/// # Permissions: Root
/// # Nodes: Primary
ioctl!(none drop_master with DRM_IOCTL_BASE, 0x1f);

/// IRQ Control
///
/// # Locks DRM mutex: Yes
/// # Permissions: Root, Master, Auth
/// # Nodes: Primary
ioctl!(write_ptr irq_control with DRM_IOCTL_BASE, 0x14; drm_control);

/// Enable the vblank interrupt and sleep until the requested sequence occurs
///
/// # Locks DRM mutex: No
/// # Permissions: None
/// # Nodes: Primary
ioctl!(readwrite wait_vblank with DRM_IOCTL_BASE, 0x3a; drm_wait_vblank);

/// Handle vblank counter changes across mode switches
///
/// # Locks DRM mutex: Yes
/// # Permissions: None
/// # Nodes: Primary
ioctl!(write_ptr modeset_ctl with DRM_IOCTL_BASE, 0x08; drm_modeset_ctl);

/// Converts a buffer handle into a dma-buf file descriptor.
ioctl!(readwrite prime_handle_to_fd
       with DRM_IOCTL_BASE, 0x2d; drm_prime_handle);

/// Converts a dma-buf file descriptor into a buffer handle.
ioctl!(readwrite prime_fd_to_handle
       with DRM_IOCTL_BASE, 0x2e; drm_prime_handle);

pub(crate) mod mode {
    use nix::libc::c_uint;
    use drm_sys::*;

    /// Modesetting resources
    ioctl!(readwrite get_resources
           with DRM_IOCTL_BASE, 0xA0; drm_mode_card_res);

    ioctl!(readwrite get_plane_resources
           with DRM_IOCTL_BASE, 0xB5; drm_mode_get_plane_res);

    /// Connector related functions
    ioctl!(readwrite get_connector
           with DRM_IOCTL_BASE, 0xA7; drm_mode_get_connector);

    /// Encoder related functions
    ioctl!(readwrite get_encoder
           with DRM_IOCTL_BASE, 0xA6; drm_mode_get_encoder);

    /// CRTC related functions
    ioctl!(readwrite get_crtc with DRM_IOCTL_BASE, 0xA1; drm_mode_crtc);
    ioctl!(readwrite set_crtc with DRM_IOCTL_BASE, 0xA2; drm_mode_crtc);

    /// Gamma related functions
    ioctl!(readwrite get_gamma
           with DRM_IOCTL_BASE, 0xA4; drm_mode_crtc_lut);
    ioctl!(readwrite set_gamma
           with DRM_IOCTL_BASE, 0xA5; drm_mode_crtc_lut);

    // TODO: Figure out GAMMA LUT arrays

    /// FB related functions
    ioctl!(readwrite get_fb with DRM_IOCTL_BASE, 0xAD; drm_mode_fb_cmd);
    ioctl!(readwrite add_fb with DRM_IOCTL_BASE, 0xAE; drm_mode_fb_cmd);
    ioctl!(readwrite add_fb2 with DRM_IOCTL_BASE, 0xB8; drm_mode_fb_cmd2);
    ioctl!(readwrite rm_fb with DRM_IOCTL_BASE, 0xAF; c_uint);

    /// Plane related functions
    ioctl!(readwrite get_plane
           with DRM_IOCTL_BASE, 0xB6; drm_mode_get_plane);

    ioctl!(readwrite set_plane
           with DRM_IOCTL_BASE, 0xB7; drm_mode_set_plane);

    /// Dumbbuffer related functions
    ioctl!(readwrite create_dumb
           with DRM_IOCTL_BASE, 0xB2; drm_mode_create_dumb);

    ioctl!(readwrite map_dumb
           with DRM_IOCTL_BASE, 0xB3; drm_mode_map_dumb);

    ioctl!(readwrite destroy_dumb
           with DRM_IOCTL_BASE, 0xB4; drm_mode_destroy_dumb);

    /// Cursor related functions
    ioctl!(readwrite cursor with DRM_IOCTL_BASE, 0xA3; drm_mode_cursor);

    ioctl!(readwrite cursor2 with DRM_IOCTL_BASE, 0xBB; drm_mode_cursor2);

    /// Property related functions
    ioctl!(readwrite get_property
           with DRM_IOCTL_BASE, 0xAA; drm_mode_get_property);

    ioctl!(readwrite connector_set_property
           with DRM_IOCTL_BASE, 0xAB; drm_mode_connector_set_property);

    ioctl!(readwrite obj_get_properties
           with DRM_IOCTL_BASE, 0xB9; drm_mode_obj_get_properties);

    ioctl!(readwrite obj_set_property
           with DRM_IOCTL_BASE, 0xBA; drm_mode_obj_set_property);

    /// Property blobs
    ioctl!(readwrite get_blob
           with DRM_IOCTL_BASE, 0xAC; drm_mode_get_blob);

    // TODO: Property blobs probably require a large buffer

    ioctl!(readwrite create_blob
           with DRM_IOCTL_BASE, 0xBD; drm_mode_create_blob);


    ioctl!(readwrite destroy_blob
           with DRM_IOCTL_BASE, 0xBE; drm_mode_destroy_blob);


    /// Atomic modesetting related functions
    ioctl!(readwrite crtc_page_flip
           with DRM_IOCTL_BASE, 0xB0; drm_mode_crtc_page_flip);


    ioctl!(readwrite dirty_fb
           with DRM_IOCTL_BASE, 0xB1; drm_mode_fb_dirty_cmd);


    ioctl!(readwrite atomic with DRM_IOCTL_BASE, 0xBC; drm_mode_atomic);
}

pub(crate) mod gem {
    use drm_sys::*;

    /// GEM related functions
    ioctl!(readwrite open with DRM_IOCTL_BASE, 0x0b; drm_gem_open);
    ioctl!(write_ptr close with DRM_IOCTL_BASE, 0x09; drm_gem_close);
    ioctl!(readwrite flink with DRM_IOCTL_BASE, 0x0a; drm_gem_flink);
}

