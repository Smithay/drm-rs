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

pub use drm_fourcc::{DrmFourcc, UnrecognizedFourcc, DrmModifier, DrmVendor, UnrecognizedVendor};

/// A handle to a GEM buffer.
///
/// # Notes
///
/// There are no guarantees that this handle is valid. It is up to the user
/// to make sure this handle does not outlive the underlying buffer, and to
/// prevent buffers from leaking by properly closing them after they are done.
#[repr(transparent)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Handle(::control::RawResourceHandle);

impl Into<::control::RawResourceHandle> for Handle {
    fn into(self) -> ::control::RawResourceHandle {
        self.0
    }
}

impl Into<u32> for Handle {
    fn into(self) -> u32 {
        self.0.into()
    }
}

impl From<::control::RawResourceHandle> for Handle {
    fn from(handle: ::control::RawResourceHandle) -> Self {
        Handle(handle)
    }
}

impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("buffer::Handle")
            .field(&self.0)
            .finish()
    }
}

/// The name of a GEM buffer.
///
/// # Notes
///
/// There are no guarantees that this name is valid. It is up to the user
/// to make sure this name does not outlive the underlying buffer, and to
/// prevent buffers from leaking by properly closing them after they are done.
#[repr(transparent)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Name(u32);

impl Into<u32> for Name {
    fn into(self) -> u32 {
        self.0.into()
    }
}

impl std::fmt::Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("buffer::Name")
            .field(&self.0)
            .finish()
    }
}

/// Common functionality of all regular buffers.
pub trait Buffer {
    /// The width and height of the buffer.
    fn size(&self) -> (u32, u32);
    /// The format of the buffer.
    fn format(&self) -> DrmFourcc;
    /// The pitch of the buffer.
    fn pitch(&self) -> u32;
    /// The handle to the buffer.
    fn handle(&self) -> Handle;
}

/// Planar buffers are buffers where each channel/plane is in its own buffer.
///
/// Each plane has their own handle, pitch, and offsets.
pub trait PlanarBuffer {
    /// The width and height of the buffer.
    fn size(&self) -> (u32, u32);
    /// The format of the buffer.
    fn format(&self) -> DrmFourcc;
    /// The pitches of the buffer.
    fn pitches(&self) -> [u32; 4];
    /// The handles to the buffer.
    fn handles(&self) -> [Option<Handle>; 4];
    /// The offsets of the buffer.
    fn offsets(&self) -> [u32; 4];
}

impl<B: Buffer + ?Sized> PlanarBuffer for B {
    fn size(&self) -> (u32, u32) {
        Buffer::size(self)
    }

    fn format(&self) -> DrmFourcc {
        Buffer::format(self)
    }

    fn pitches(&self) -> [u32; 4] {
        [self.pitch(), 0, 0, 0]
    }

    fn handles(&self) -> [Option<Handle>; 4] {
        [Some(self.handle()), None, None, None]
    }

    fn offsets(&self) -> [u32; 4] {
        [0; 4]
    }
}