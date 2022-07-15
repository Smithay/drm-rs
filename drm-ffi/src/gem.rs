//!
//! Bindings to the Graphics Execution Manager
//!

use drm_sys::*;

use result::SystemError as Error;

use std::os::unix::io::RawFd;

/// Open a GEM object given it's 32-bit name, returning the handle.
pub fn open(fd: RawFd, name: u32) -> Result<drm_gem_open, Error> {
    let mut gem = drm_gem_open {
        name,
        ..Default::default()
    };

    unsafe {
        crate::ioctl::gem::open(fd, &mut gem)?;
    }

    Ok(gem)
}

/// Closes a GEM object given it's handle.
pub fn close(fd: RawFd, handle: u32) -> Result<drm_gem_close, Error> {
    let gem = drm_gem_close {
        handle,
        ..Default::default()
    };

    unsafe {
        crate::ioctl::gem::close(fd, &gem)?;
    }

    Ok(gem)
}

/// Converts a GEM object's handle to a PRIME file descriptor.
pub fn handle_to_fd(fd: RawFd, handle: u32, flags: u32) -> Result<drm_prime_handle, Error> {
    let mut prime = drm_prime_handle {
        handle,
        flags,
        ..Default::default()
    };

    unsafe {
        crate::ioctl::gem::prime_handle_to_fd(fd, &mut prime)?;
    }

    Ok(prime)
}

/// Converts a PRIME file descriptor to a GEM object's handle.
pub fn fd_to_handle(fd: RawFd, primefd: RawFd) -> Result<drm_prime_handle, Error> {
    let mut prime = drm_prime_handle {
        fd: primefd,
        ..Default::default()
    };

    unsafe {
        crate::ioctl::gem::prime_fd_to_handle(fd, &mut prime)?;
    }

    Ok(prime)
}
