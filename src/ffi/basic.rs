use drm_sys::*;
use ffi::ioctl;

use nix::libc::*;
use nix::Error;
use std::os::unix::io::RawFd;

use std::cmp;

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

/// Gets the length of this device's Bus ID.
pub fn get_bus_id_length(fd: RawFd) -> Result<usize, Error> {
    let mut busid = drm_unique::default();

    unsafe {
        ioctl::get_bus_id(fd, &mut busid)?;
    }

    Ok(busid.unique_len as usize)
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

    let min = cmp::min(busid.unique_len as _, buf.len());
    *buf = &buf[..min];

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

/// Manually set a driver version to restrict/enable capabilities.
pub fn set_version(
    fd: RawFd,
    di_maj: c_int,
    di_min: c_int,
    dd_maj: c_int,
    dd_min: c_int,
) -> Result<drm_set_version, Error> {
    let mut version = drm_set_version {
        drm_di_major: di_maj,
        drm_di_minor: di_min,
        drm_dd_major: dd_maj,
        drm_dd_minor: dd_min,
    };

    unsafe {
        ioctl::set_version(fd, &mut version)?;
    }

    Ok(version)
}

/// Gets the lengths of this device driver's name, date, and description.
pub fn get_driver_version_lengths(fd: RawFd) -> Result<(usize, usize, usize), Error> {
    let mut version = drm_version::default();

    unsafe {
        ioctl::get_version(fd, &mut version)?;
    }

    Ok((
        version.name_len as usize,
        version.date_len as usize,
        version.desc_len as usize,
    ))
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

    let min = cmp::min(version.name_len as _, name_buf.len());
    *name_buf = &name_buf[..min];
    let min = cmp::min(version.date_len as _, date_buf.len());
    *date_buf = &date_buf[..min];
    let min = cmp::min(version.desc_len as _, desc_buf.len());
    *desc_buf = &desc_buf[..min];

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
