use drm_sys::*;
use ffi::ioctl;

use nix::Error;
use std::os::unix::io::RawFd;

// Open a GEM object given it's 32-bit name, returning the handle.
pub fn open(fd: RawFd, name: u32) -> Result<u32, Error> {
    let mut gem = drm_gem_open::default();
    gem.name = name;

    unsafe {
        ioctl::gem::open(fd, &mut gem)?;
    }

    Ok(gem.handle)
}

// Open a GEM object given it's handle.
pub fn close(fd: RawFd, handle: u32) -> Result<(), Error> {
    let mut gem = drm_gem_close::default();
    gem.handle = handle;

    unsafe {
        ioctl::gem::close(fd, &mut gem)?;
    }

    Ok(())
}

// Converts a GEM object's handle to a PRIME file descriptor.
pub fn handle_to_fd(fd: RawFd, handle: u32, flags: u32) -> Result<RawFd, Error> {
    let mut prime = drm_prime_handle::default();
    prime.handle = handle;
    prime.flags = flags;

    unsafe {
        ioctl::gem::prime_handle_to_fd(fd, &mut prime)?;
    }

    Ok(prime.fd as RawFd)
}

// Converts a PRIME file descriptor to a GEM object's handle.
pub fn fd_to_handle(fd: RawFd, primefd: RawFd) -> Result<u32, Error> {
    let mut prime = drm_prime_handle::default();
    prime.fd = primefd;

    unsafe {
        ioctl::gem::prime_fd_to_handle(fd, &mut prime)?;
    }

    Ok(prime.handle)
}
