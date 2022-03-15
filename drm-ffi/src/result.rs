//!
//! Error types
//!

use nix::errno::Errno;
use std::error::Error;
use std::fmt;

/// A general system error that can be returned by any DRM command.
///
/// Receiving this error likely indicates a bug in either the program, this
/// crate, or the underlying operating system.
#[derive(Debug)]
pub enum SystemError {
    /// A command was attempted using an invalid file descriptor.
    InvalidFileDescriptor,

    /// Provided memory area is inaccessible.
    ///
    /// Receiving this error indicates a bug in this crate.
    MemoryFault,

    /// One or more arguments used are invalid.
    ///
    /// This can be due to the system not supporting a feature or value.
    InvalidArgument,

    /// A command was attempted using a non-DRM device.
    InvalidFileType,

    /// Permission denied.
    PermissionDenied,

    /// An unknown fourcc code was received.
    ///
    /// This likely indicates that the drm-fourcc crate needs updating.
    UnknownFourcc,

    /// Unknown system error.
    Unknown {
        /// Unknown [`nix::errno::Errno`] returned by the system call.
        errno: Errno,
    },
}

impl fmt::Display for SystemError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(match self {
            SystemError::InvalidFileDescriptor => "invalid file descriptor",
            SystemError::MemoryFault => "invalid memory access",
            SystemError::InvalidArgument => "invalid argument",
            SystemError::InvalidFileType => "invalid file type",
            SystemError::PermissionDenied => "permission denied",
            SystemError::UnknownFourcc => "unknown fourcc",
            SystemError::Unknown { errno } => {
                return write!(fmt, "unknown system error: {}", errno)
            }
        })
    }
}

impl Error for SystemError {}

impl From<Errno> for SystemError {
    fn from(errno: Errno) -> SystemError {
        match errno {
            Errno::EBADF => SystemError::InvalidFileDescriptor,
            Errno::EFAULT => SystemError::MemoryFault,
            Errno::EINVAL => SystemError::InvalidArgument,
            Errno::ENOTTY => SystemError::InvalidFileDescriptor,
            Errno::EACCES => SystemError::PermissionDenied,
            _ => SystemError::Unknown { errno },
        }
    }
}
