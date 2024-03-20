use std::{ffi::c_uint, io, os::unix::io::BorrowedFd};

use drm_sys::*;
use rustix::ioctl::{
    ioctl, Getter, NoArg, NoneOpcode, ReadOpcode, ReadWriteOpcode, Setter, Updater, WriteOpcode,
};

macro_rules! ioctl_readwrite {
    ($name:ident, $ioty:expr, $nr:expr, $ty:ty) => {
        pub unsafe fn $name(fd: BorrowedFd, data: &mut $ty) -> io::Result<()> {
            type Opcode = ReadWriteOpcode<$ioty, $nr, $ty>;
            Ok(ioctl(fd, Updater::<Opcode, $ty>::new(data))?)
        }
    };
}

macro_rules! ioctl_read {
    ($name:ident, $ioty:expr, $nr:expr, $ty:ty) => {
        pub unsafe fn $name(fd: BorrowedFd) -> io::Result<$ty> {
            type Opcode = ReadOpcode<$ioty, $nr, $ty>;
            Ok(ioctl(fd, Getter::<Opcode, $ty>::new())?)
        }
    };
}

macro_rules! ioctl_write_ptr {
    ($name:ident, $ioty:expr, $nr:expr, $ty:ty) => {
        pub unsafe fn $name(fd: BorrowedFd, data: &$ty) -> io::Result<()> {
            type Opcode = WriteOpcode<$ioty, $nr, $ty>;
            Ok(ioctl(fd, Setter::<Opcode, $ty>::new(*data))?)
        }
    };
}

macro_rules! ioctl_none {
    ($name:ident, $ioty:expr, $nr:expr) => {
        pub unsafe fn $name(fd: BorrowedFd) -> io::Result<()> {
            type Opcode = NoneOpcode<$ioty, $nr, ()>;
            Ok(ioctl(fd, NoArg::<Opcode>::new())?)
        }
    };
}

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
    use super::*;

    /// Modesetting resources
    ioctl_readwrite!(get_resources, DRM_IOCTL_BASE, 0xA0, drm_mode_card_res);

    ioctl_readwrite!(
        get_plane_resources,
        DRM_IOCTL_BASE,
        0xB5,
        drm_mode_get_plane_res
    );

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
    ioctl_readwrite!(get_fb2, DRM_IOCTL_BASE, 0xCE, drm_mode_fb_cmd2);
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

    ioctl_readwrite!(
        connector_set_property,
        DRM_IOCTL_BASE,
        0xAB,
        drm_mode_connector_set_property
    );

    ioctl_readwrite!(
        obj_get_properties,
        DRM_IOCTL_BASE,
        0xB9,
        drm_mode_obj_get_properties
    );

    ioctl_readwrite!(
        obj_set_property,
        DRM_IOCTL_BASE,
        0xBA,
        drm_mode_obj_set_property
    );

    /// Property blobs
    ioctl_readwrite!(get_blob, DRM_IOCTL_BASE, 0xAC, drm_mode_get_blob);

    // TODO: Property blobs probably require a large buffer

    ioctl_readwrite!(create_blob, DRM_IOCTL_BASE, 0xBD, drm_mode_create_blob);

    ioctl_readwrite!(destroy_blob, DRM_IOCTL_BASE, 0xBE, drm_mode_destroy_blob);

    /// Atomic modesetting related functions
    ioctl_readwrite!(
        crtc_page_flip,
        DRM_IOCTL_BASE,
        0xB0,
        drm_mode_crtc_page_flip
    );

    ioctl_readwrite!(dirty_fb, DRM_IOCTL_BASE, 0xB1, drm_mode_fb_dirty_cmd);

    ioctl_readwrite!(atomic, DRM_IOCTL_BASE, 0xBC, drm_mode_atomic);

    ioctl_readwrite!(create_lease, DRM_IOCTL_BASE, 0xC6, drm_mode_create_lease);
    ioctl_readwrite!(list_lessees, DRM_IOCTL_BASE, 0xC7, drm_mode_list_lessees);
    ioctl_readwrite!(get_lease, DRM_IOCTL_BASE, 0xC8, drm_mode_get_lease);
    ioctl_readwrite!(revoke_lease, DRM_IOCTL_BASE, 0xC9, drm_mode_revoke_lease);
}

pub(crate) mod gem {
    use super::*;

    /// GEM related functions
    ioctl_readwrite!(open, DRM_IOCTL_BASE, 0x0b, drm_gem_open);
    ioctl_write_ptr!(close, DRM_IOCTL_BASE, 0x09, drm_gem_close);

    /// Converts a buffer handle into a dma-buf file descriptor.
    ioctl_readwrite!(prime_handle_to_fd, DRM_IOCTL_BASE, 0x2d, drm_prime_handle);

    /// Converts a dma-buf file descriptor into a buffer handle.
    ioctl_readwrite!(prime_fd_to_handle, DRM_IOCTL_BASE, 0x2e, drm_prime_handle);
}

pub(crate) mod syncobj {
    use super::*;

    /// Creates a syncobj.
    ioctl_readwrite!(create, DRM_IOCTL_BASE, 0xBF, drm_syncobj_create);
    /// Destroys a syncobj.
    ioctl_readwrite!(destroy, DRM_IOCTL_BASE, 0xC0, drm_syncobj_destroy);
    /// Exports a syncobj as an inter-process file descriptor or as a poll()-able sync file.
    ioctl_readwrite!(handle_to_fd, DRM_IOCTL_BASE, 0xC1, drm_syncobj_handle);
    /// Imports a file descriptor exported by [`handle_to_fd`] back into a process-local handle.
    ioctl_readwrite!(fd_to_handle, DRM_IOCTL_BASE, 0xC2, drm_syncobj_handle);
    /// Waits for one or more syncobjs to become signalled.
    ioctl_readwrite!(wait, DRM_IOCTL_BASE, 0xC3, drm_syncobj_wait);
    /// Resets (un-signals) one or more syncobjs.
    ioctl_readwrite!(reset, DRM_IOCTL_BASE, 0xC4, drm_syncobj_array);
    /// Signals one or more syncobjs.
    ioctl_readwrite!(signal, DRM_IOCTL_BASE, 0xC5, drm_syncobj_array);

    /// Waits for one or more specific timeline syncobj points.
    ioctl_readwrite!(
        timeline_wait,
        DRM_IOCTL_BASE,
        0xCA,
        drm_syncobj_timeline_wait
    );
    /// Queries for state of one or more timeline syncobjs.
    ioctl_readwrite!(query, DRM_IOCTL_BASE, 0xCB, drm_syncobj_timeline_array);
    /// Transfers one timeline syncobj point to another.
    ioctl_readwrite!(transfer, DRM_IOCTL_BASE, 0xCC, drm_syncobj_transfer);
    /// Signals one or more specific timeline syncobj points.
    ioctl_readwrite!(
        timeline_signal,
        DRM_IOCTL_BASE,
        0xCD,
        drm_syncobj_timeline_array
    );
    /// Register an eventfd to be signalled by a syncobj.
    ioctl_readwrite!(eventfd, DRM_IOCTL_BASE, 0xCF, drm_syncobj_eventfd);
}
