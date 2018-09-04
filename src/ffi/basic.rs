use drm_sys::*;
use ffi::ioctl;

use nix::libc::*;
use nix::Error;
use std::os::unix::io::RawFd;

use super::minimize_slice;

pub mod auth {
    use drm_sys::*;
    use ffi::ioctl;

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
    pub fn auth_magic_token(fd: RawFd, mut auth: drm_auth) -> Result<drm_auth, Error> {
        unsafe {
            ioctl::auth_token(fd, &mut auth)?;
        }

        Ok(auth)
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
pub fn get_bus_id(fd: RawFd, buf: &mut &[u8]) -> Result<(), Error> {
    let mut busid = drm_unique {
        unique: buf.as_ptr() as _,
        unique_len: buf.len() as _,
    };

    unsafe {
        ioctl::get_bus_id(fd, &mut busid)?;
    }

    minimize_slice(buf, busid.unique_len as _);

    Ok(())
}

/// Get a device's IRQ.
pub fn get_interrupt_from_bus_id(
    fd: RawFd,
    bus: c_int,
    dev: c_int,
    func: c_int,
) -> Result<c_int, Error> {
    let mut irq = drm_irq_busid {
        busnum: bus,
        devnum: dev,
        funcnum: func,
        ..Default::default()
    };

    unsafe {
        ioctl::get_irq_from_bus_id(fd, &mut irq)?;
    }

    Ok(irq.irq)
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

/// Check if a capability is set.
pub fn get_capability(fd: RawFd, cty: u64) -> Result<drm_get_cap, Error> {
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
pub fn set_capability(fd: RawFd, cty: u64, val: bool) -> Result<drm_set_client_cap, Error> {
    let mut cap = drm_set_client_cap {
        capability: cty,
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
    name_buf: &mut &[u8],
    date_buf: &mut &[u8],
    desc_buf: &mut &[u8],
) -> Result<(), Error> {
    let mut version = drm_version {
        name: name_buf.as_ptr() as _,
        name_len: name_buf.len() as _,
        date: date_buf.as_ptr() as _,
        date_len: date_buf.len() as _,
        desc: desc_buf.as_ptr() as _,
        desc_len: desc_buf.len() as _,
        ..Default::default()
    };

    unsafe {
        ioctl::get_version(fd, &mut version)?;
    }

    minimize_slice(name_buf, version.name_len as _);
    minimize_slice(date_buf, version.date_len as _);
    minimize_slice(desc_buf, version.desc_len as _);

    Ok(())
}

/// Gets statistic information about this device.
pub fn get_stats(fd: RawFd) -> Result<drm_stats, Error> {
    let mut stats = drm_stats::default();

    unsafe {
        ioctl::get_stats(fd, &mut stats)?;
    }

    Ok(stats)
}
