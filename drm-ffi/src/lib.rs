//!
//! Foreign function interface
//!

#![warn(missing_docs)]
#![allow(unused_doc_comments)]

pub use drm_sys::{self, *};

#[macro_use]
pub(crate) mod utils;

pub mod gem;
mod ioctl;
pub mod mode;
pub mod syncobj;

use std::{
    ffi::{c_int, c_ulong},
    io,
    os::unix::io::BorrowedFd,
};

///
/// Bindings to the methods of authentication the DRM provides.
///
pub mod auth {
    use crate::ioctl;
    use drm_sys::*;

    use std::{io, os::unix::io::BorrowedFd};

    /// Get the 'Magic Authentication Token' for this file descriptor.
    pub fn get_magic_token(fd: BorrowedFd<'_>) -> io::Result<drm_auth> {
        unsafe { ioctl::get_token(fd) }
    }

    /// Authorize another process' 'Magic Authentication Token'.
    pub fn auth_magic_token(fd: BorrowedFd<'_>, auth: u32) -> io::Result<drm_auth> {
        let token = drm_auth { magic: auth };

        unsafe {
            ioctl::auth_token(fd, &token)?;
        }

        Ok(token)
    }

    /// Acquire the 'Master DRM Lock' for this file descriptor.
    pub fn acquire_master(fd: BorrowedFd<'_>) -> io::Result<()> {
        unsafe { ioctl::acquire_master(fd) }
    }

    /// Release the 'Master DRM Lock' for this file descriptor.
    pub fn release_master(fd: BorrowedFd<'_>) -> io::Result<()> {
        unsafe { ioctl::release_master(fd) }
    }
}

/// Load this device's Bus ID into a buffer.
pub fn get_bus_id(fd: BorrowedFd<'_>, mut buf: Option<&mut Vec<u8>>) -> io::Result<drm_unique> {
    let mut sizes = drm_unique::default();
    unsafe {
        ioctl::get_bus_id(fd, &mut sizes)?;
    }

    if buf.is_none() {
        return Ok(sizes);
    }

    map_reserve!(buf, sizes.unique_len as usize);

    let mut busid = drm_unique {
        unique_len: sizes.unique_len,
        unique: map_ptr!(&buf),
    };

    unsafe {
        ioctl::get_bus_id(fd, &mut busid)?;
    }

    map_set!(buf, busid.unique_len as usize);

    Ok(busid)
}

/// Get a device's IRQ.
pub fn get_interrupt_from_bus_id(
    fd: BorrowedFd<'_>,
    bus: c_int,
    dev: c_int,
    func: c_int,
) -> io::Result<drm_irq_busid> {
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
pub fn get_client(fd: BorrowedFd<'_>, idx: c_int) -> io::Result<drm_client> {
    let mut client = drm_client {
        idx,
        ..Default::default()
    };

    unsafe {
        ioctl::get_client(fd, &mut client)?;
    }

    Ok(client)
}

/// Check if a capability is set.
pub fn get_capability(fd: BorrowedFd<'_>, cty: u64) -> io::Result<drm_get_cap> {
    let mut cap = drm_get_cap {
        capability: cty,
        ..Default::default()
    };

    unsafe {
        ioctl::get_cap(fd, &mut cap)?;
    }

    Ok(cap)
}

/// Attempt to enable/disable a client's capability.
pub fn set_capability(fd: BorrowedFd<'_>, cty: u64, val: bool) -> io::Result<drm_set_client_cap> {
    let cap = drm_set_client_cap {
        capability: cty,
        value: val as u64,
    };

    unsafe {
        ioctl::set_cap(fd, &cap)?;
    }

    Ok(cap)
}

/// Sets the requested interface version
pub fn set_version(fd: BorrowedFd<'_>, version: &mut drm_set_version) -> io::Result<()> {
    unsafe { ioctl::set_version(fd, version) }
}

/// Gets the driver version for this device.
pub fn get_version(
    fd: BorrowedFd<'_>,
    mut name_buf: Option<&mut Vec<i8>>,
    mut date_buf: Option<&mut Vec<i8>>,
    mut desc_buf: Option<&mut Vec<i8>>,
) -> io::Result<drm_version> {
    let mut sizes = drm_version::default();
    unsafe { ioctl::get_version(fd, &mut sizes) }?;

    map_reserve!(name_buf, sizes.name_len as usize);
    map_reserve!(date_buf, sizes.date_len as usize);
    map_reserve!(desc_buf, sizes.desc_len as usize);

    let mut version = drm_version {
        name_len: map_len!(&name_buf),
        name: map_ptr!(&name_buf),
        date_len: map_len!(&date_buf),
        date: map_ptr!(&date_buf),
        desc_len: map_len!(&desc_buf),
        desc: map_ptr!(&desc_buf),
        ..Default::default()
    };

    unsafe { ioctl::get_version(fd, &mut version) }?;

    map_set!(name_buf, version.name_len as usize);
    map_set!(date_buf, version.date_len as usize);
    map_set!(desc_buf, version.desc_len as usize);

    Ok(version)
}

/// Waits for a vblank.
pub fn wait_vblank(
    fd: BorrowedFd<'_>,
    type_: u32,
    sequence: u32,
    signal: usize,
) -> io::Result<drm_wait_vblank_reply> {
    // We can't assume the kernel will completely fill the reply in the union
    // with valid data (it won't populate the timestamp if the event flag is
    // set, for example), so use `default` to ensure the structure is completely
    // initialized with zeros
    let mut wait_vblank = drm_wait_vblank::default();
    wait_vblank.request = drm_wait_vblank_request {
        type_,
        sequence,
        signal: signal as c_ulong,
    };

    unsafe {
        ioctl::wait_vblank(fd, &mut wait_vblank)?;
    };

    Ok(unsafe { wait_vblank.reply })
}
