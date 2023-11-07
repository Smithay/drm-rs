//!
//! Bindings to the Graphics Execution Manager
//!

use crate::ioctl;
use drm_sys::*;

use std::{
    io,
    os::unix::io::{AsRawFd, BorrowedFd},
};

/// Open a GEM object given it's 32-bit name, returning the handle.
pub fn open(fd: BorrowedFd<'_>, name: u32) -> io::Result<drm_gem_open> {
    let mut gem = drm_gem_open {
        name,
        ..Default::default()
    };

    unsafe {
        ioctl::gem::open(fd, &mut gem)?;
    }

    Ok(gem)
}

/// Closes a GEM object given it's handle.
pub fn close(fd: BorrowedFd<'_>, handle: u32) -> io::Result<drm_gem_close> {
    let gem = drm_gem_close {
        handle,
        ..Default::default()
    };

    unsafe {
        ioctl::gem::close(fd, &gem)?;
    }

    Ok(gem)
}

/// Converts a GEM object's handle to a PRIME file descriptor.
pub fn handle_to_fd(fd: BorrowedFd<'_>, handle: u32, flags: u32) -> io::Result<drm_prime_handle> {
    let mut prime = drm_prime_handle {
        handle,
        flags,
        ..Default::default()
    };

    unsafe {
        ioctl::gem::prime_handle_to_fd(fd, &mut prime)?;
    }

    Ok(prime)
}

/// Converts a PRIME file descriptor to a GEM object's handle.
pub fn fd_to_handle(fd: BorrowedFd<'_>, primefd: BorrowedFd<'_>) -> io::Result<drm_prime_handle> {
    let mut prime = drm_prime_handle {
        fd: primefd.as_raw_fd(),
        ..Default::default()
    };

    unsafe {
        ioctl::gem::prime_fd_to_handle(fd, &mut prime)?;
    }

    Ok(prime)
}
