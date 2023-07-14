//! # SyncObj
//!
//! A SyncObj is a binding point for the DRM subsystem to attach single-use fences which are
//! signalled when a device task completes. They are typically provided as optional arguments to
//! device-specific command submission IOCTLs. In practice, they are used to implement Vulkan
//! fence objects.
//!
//! After a submission IOCTL sets a fence into a SyncObj, it may be exported as a sync file
//! descriptor. This sync file may be epoll()'d for EPOLLIN to implement asynchronous waiting on
//! multiple events. This file descriptor is also compatible with [`tokio::io::unix::AsyncFd`] for
//! Rust async/await integration.
//!
//! [`tokio::io::unix::AsyncFd`]: <https://docs.rs/tokio/latest/tokio/io/unix/struct.AsyncFd.html>

use control;
use std::os::unix::io::{AsFd, AsRawFd, FromRawFd, RawFd};

/// A handle to a specific syncobj
#[repr(transparent)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Handle(control::RawResourceHandle);

// Safety: Handle is repr(transparent) over NonZeroU32
unsafe impl bytemuck::ZeroableInOption for Handle {}
unsafe impl bytemuck::PodInOption for Handle {}
unsafe impl bytemuck::NoUninit for Handle {}

impl From<Handle> for control::RawResourceHandle {
    fn from(handle: Handle) -> Self {
        handle.0
    }
}

impl From<Handle> for u32 {
    fn from(handle: Handle) -> Self {
        handle.0.into()
    }
}

impl From<control::RawResourceHandle> for Handle {
    fn from(handle: control::RawResourceHandle) -> Self {
        Handle(handle)
    }
}

impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("syncobj::Handle").field(&self.0).finish()
    }
}

#[derive(Debug)]
/// A simple wrapper for a syncobj fd.
pub struct SyncFile(std::fs::File);

impl FromRawFd for SyncFile {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self(std::fs::File::from_raw_fd(fd))
    }
}

/// Implementing [`AsFd`] is a prerequisite to implementing the traits found in this crate.
/// Here, we are just calling [`std::fs::File::as_fd()`] on the inner File.
impl AsFd for SyncFile {
    fn as_fd(&self) -> std::os::unix::io::BorrowedFd<'_> {
        self.0.as_fd()
    }
}

/// Implementing [`AsRawFd`] allows SyncFile to be owned by [`tokio::io::unix::AsyncFd`];
/// thereby integrating with its async/await runtime.
///
/// [`tokio::io::unix::AsyncFd`]: <https://docs.rs/tokio/latest/tokio/io/unix/struct.AsyncFd.html>
impl AsRawFd for SyncFile {
    fn as_raw_fd(&self) -> RawFd {
        self.as_fd().as_raw_fd()
    }
}
