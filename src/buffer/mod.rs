//! Memory management and buffer functionality that the DRM sybsystem exposes.
//!
//! # Summary
//!
//! The DRM subsystem exposes functionality for managing memory on modern GPU
//! devices using a system called the Graphics Execution Manager (GEM). This
//! system manages GPU buffers and exposes them to userspace using 32-bit
//! handles. These handles are automatically reference counted in the kernel.
//!
//! GEM provides a small API for sharing buffers between processes. However, it
//! does not provide any generic API for creating these. Instead, each driver
//! provides its own method of creating these buffers. The `libgbm` library
//! (part of the mesa project) provides a driver agnostic method of creating
//! these buffers.
//!
//! There are two methods of sharing a GEM handle between processes:
//!
//! 1. Using `Flink` to globally publish a handle using a 32-bit 'name'. This
//! requires either holding the DRM Master lock or having the process'
//! [AuthToken](../AuthToken.t.hmtl) authenticated. However, any process can
//! open these handles if they know (or even guess) the global name.
//!
//! 2. Converting the GEM handle into a PRIME file descriptor, and passing it
//! like a regular one. This allows better control and security, and is the
//! recommended method of sharing buffers.

use ffi::{self, Wrapper, gem::RawHandle};
use result::Result;

pub mod format;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
/// A handle to a GEM buffer.
///
/// # Notes
///
/// There are no guarantees that this handle is valid. It is up to the user
/// to make sure this handle does not outlive the underlying buffer, and to
/// prevent buffers from leaking by properly closing them after they are done.
pub struct Handle(RawHandle);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
/// The name of a GEM buffer.
///
/// # Notes
///
/// There are no guarantees that this name is valid. It is up to the user
/// to make sure this name does not outlive the underlying buffer, and to
/// prevent buffers from leaking by properly closing them after they are done.
pub struct Name(RawHandle);

/// Buffer related commands that can be executed by a [Device](../Device.t.html).
pub trait Commands: super::Device {
    /// Acquires the [Handle](Handle.t.html) of a GEM buffer given its global
    /// name.
    fn open(&self, name: Name) -> Result<Handle>;

    /// Closes the GEM buffer.
    fn close(&self, handle: Handle) -> Result<()>;

    /// Publishes a GEM buffer and returns a [Name](Name.t.html) that can be
    /// used by other processes to acquire it.
    fn flink(&self, handle: Handle) -> Result<Name>;
}

impl<T: super::Device> Commands for T {
    fn open(&self, name: Name) -> Result<Handle> {
        let mut t = ffi::gem::Open::default();
        t.raw_mut_ref().name = name.into();
        t.ioctl(self.as_raw_fd())?;
        Ok(Handle(t.raw_ref().handle))
    }

    fn close(&self, handle: Handle) -> Result<()> {
        let mut t = ffi::gem::Close::default();
        t.raw_mut_ref().handle = handle.into();
        t.ioctl(self.as_raw_fd())?;
        Ok(())
    }

    // TODO: Raw struct also has '__u64 size;'
    fn flink(&self, handle: Handle) -> Result<Name> {
        let mut t = ffi::gem::Flink::default();
        t.raw_mut_ref().handle = handle.into();
        t.ioctl(self.as_raw_fd())?;
        Ok(Name(t.raw_ref().name))
    }
}

/// Common functionality of all buffers.
pub trait Buffer {
    /// The width and height of the buffer.
    fn size(&self) -> (u32, u32);
    /// The format of the buffer.
    fn format(&self) -> format::PixelFormat;
    /// The pitch of the buffer.
    fn pitch(&self) -> u32;
    /// The GEM handle of the buffer.
    fn handle(&self) -> Handle;
}

