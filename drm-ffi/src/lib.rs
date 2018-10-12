//!
//! Foreign function interface
//!

#![warn(missing_docs)]
#![feature(str_internals)]
extern crate core;

pub extern crate drm_sys;
use drm_sys::*;

#[macro_use]
extern crate failure;
#[macro_use]
extern crate nix;

#[macro_use]
pub(crate) mod utils;

use result::SystemError as Error;
pub mod gem;
pub mod ioctl;
pub mod result;
pub mod mode;

use nix::libc::*;
use std::os::unix::io::RawFd;

///
/// Bindings to the methods of authentication the DRM provides.
///
pub mod auth {
    use drm_sys::*;
    use ioctl;

    use nix::Error;
    use std::os::unix::io::RawFd;

    /// Get the 'Magic Authentication Token' for this file descriptor.
    pub fn get_magic_token(fd: RawFd) -> Result<drm_auth, Error> {
        let mut auth = drm_auth::default();

        unsafe {
            ioctl::get_token(fd, &mut auth)?;
        }

        Ok(auth)
    }

    /// Authorize another process' 'Magic Authentication Token'.
    pub fn auth_magic_token(fd: RawFd, auth: u32) -> Result<drm_auth, Error> {
        let mut token = drm_auth { magic: auth };

        unsafe {
            ioctl::auth_token(fd, &mut token)?;
        }

        Ok(token)
    }

    /// Acquire the 'Master DRM Lock' for this file descriptor.
    pub fn acquire_master(fd: RawFd) -> Result<(), Error> {
        unsafe {
            ioctl::acquire_master(fd)?;
        }

        Ok(())
    }

    /// Release the 'Master DRM Lock' for this file descriptor.
    pub fn release_master(fd: RawFd) -> Result<(), Error> {
        unsafe {
            ioctl::release_master(fd)?;
        }

        Ok(())
    }
}

/// Load this device's Bus ID into a buffer.
///
/// If the buffer is too small, this will load the maximum bytes in the buffer.
/// If the buffer is too big, this will coerce the buffer to the proper size.
pub fn get_bus_id(fd: RawFd, buf: Option<&mut &mut [u8]>) -> Result<drm_unique, Error> {
    let mut busid = drm_unique {
        unique: map_ptr!(&buf),
        unique_len: map_len!(&buf),
    };

    unsafe {
        ioctl::get_bus_id(fd, &mut busid)?;
    }

    map_shrink!(buf, busid.unique_len as usize);

    Ok(busid)
}

/// Get a device's IRQ.
pub fn get_interrupt_from_bus_id(
    fd: RawFd,
    bus: c_int,
    dev: c_int,
    func: c_int,
) -> Result<drm_irq_busid, Error> {
    let mut irq = drm_irq_busid {
        busnum: bus,
        devnum: dev,
        funcnum: func,
        ..Default::default()
    };

    unsafe {
        ioctl::get_irq_from_bus_id(fd, &mut irq)?;
    }

    Ok(irq)
}

/// Get client information given a client's ID.
pub fn get_client(fd: RawFd, idx: c_int) -> Result<drm_client, Error> {
    let mut client = drm_client {
        idx: idx,
        ..Default::default()
    };

    unsafe {
        ioctl::get_client(fd, &mut client)?;
    }

    Ok(client)
}

/// Used to check which capabilities your graphics driver has.
#[repr(u64)]
pub enum DriverCapability {
    /// DumbBuffer support for scanout
    DumbBuffer = DRM_CAP_DUMB_BUFFER as u64,
    /// Unknown
    VBlankHighCRTC = DRM_CAP_VBLANK_HIGH_CRTC as u64,
    /// Preferred depth to use for dumb buffers
    DumbPreferredDepth = DRM_CAP_DUMB_PREFERRED_DEPTH as u64,
    /// Unknown
    DumbPreferShadow = DRM_CAP_DUMB_PREFER_SHADOW as u64,
    /// PRIME handles are support
    Prime = DRM_CAP_PRIME as u64,
    /// Unknown
    TimestampMonotonic = DRM_CAP_TIMESTAMP_MONOTONIC as u64,
    /// Asynchronous page flipping support
    ASyncPageFlip = DRM_CAP_ASYNC_PAGE_FLIP as u64,
    /// Width of cursor buffers
    CursorWidth = DRM_CAP_CURSOR_WIDTH as u64,
    /// Height of cursor buffers
    CursorHeight = DRM_CAP_CURSOR_HEIGHT as u64,
    /// You can create framebuffers with modifiers
    AddFB2Modifiers = DRM_CAP_ADDFB2_MODIFIERS as u64,
    /// Unknown
    PageFlipTarget = DRM_CAP_PAGE_FLIP_TARGET as u64,
    /// Uses the CRTC's ID in vblank events
    CRTCInVBlankEvent = DRM_CAP_CRTC_IN_VBLANK_EVENT as u64,
    /// SyncObj support
    SyncObj = DRM_CAP_SYNCOBJ as u64,
}

/// Check if a capability is set.
pub fn get_capability(fd: RawFd, cty: DriverCapability) -> Result<drm_get_cap, Error> {
    let mut cap = drm_get_cap {
        capability: cty as u64,
        ..Default::default()
    };

    unsafe {
        ioctl::get_cap(fd, &mut cap)?;
    }

    Ok(cap)
}

#[repr(u64)]
/// Used to enable/disable capabilities for the process.
pub enum ClientCapability {
    /// The driver provides 3D screen control
    Stereo3D = DRM_CLIENT_CAP_STEREO_3D as u64,
    /// The driver provides more plane types for modesetting
    UniversalPlanes = DRM_CLIENT_CAP_UNIVERSAL_PLANES as u64,
    /// The driver provides atomic modesetting
    Atomic = DRM_CLIENT_CAP_ATOMIC as u64,
}

/// Attempt to enable/disable a client's capability.
pub fn set_capability(fd: RawFd, cty: u64, val: bool) -> Result<drm_set_client_cap, Error> {
    let mut cap = drm_set_client_cap {
        capability: cty as u64,
        value: val as u64,
    };

    unsafe {
        ioctl::set_cap(fd, &mut cap)?;
    }

    Ok(cap)
}

/// Gets the driver version for this device.
///
/// If any buffer is too small, this will load the maximum bytes in the buffer.
/// If any buffer is too big, this will coerce the buffer to the proper size.
pub fn get_version(
    fd: RawFd,
    name_buf: Option<&mut &mut [i8]>,
    date_buf: Option<&mut &mut [i8]>,
    desc_buf: Option<&mut &mut [i8]>,
) -> Result<drm_version, Error> {
    let mut version = drm_version {
        name: map_ptr!(&name_buf),
        name_len: map_len!(&name_buf),
        date: map_ptr!(&date_buf),
        date_len: map_len!(&date_buf),
        desc: map_ptr!(&desc_buf),
        desc_len: map_len!(&desc_buf),
        ..Default::default()
    };

    unsafe {
        ioctl::get_version(fd, &mut version)?;
    }

    map_shrink!(name_buf, version.name_len as usize);
    map_shrink!(date_buf, version.date_len as usize);
    map_shrink!(desc_buf, version.desc_len as usize);

    Ok(version)
}
