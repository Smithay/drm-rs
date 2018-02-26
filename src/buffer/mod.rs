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
use ffi::fourcc::*;
use result::Result;

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
    fn format(&self) -> PixelFormat;
    /// The pitch of the buffer.
    fn pitch(&self) -> u32;
    /// The GEM handle of the buffer.
    fn handle(&self) -> Handle;
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(missing_docs)]
/// Possible pixel formats of a buffer
pub enum PixelFormat {
    C8,
    R8,
    GR88,

    RGB332,
    BGR233,

    XRGB4444,
    XBGR4444,
    RGBX4444,
    BGRX4444,

    ARGB4444,
    ABGR4444,
    RGBA4444,
    BGRA4444,

    XRGB1555,
    XBGR1555,
    RGBX5551,
    BGRX5551,

    ARGB1555,
    ABGR1555,
    RGBA5551,
    BGRA5551,

    RGB565,
    BGR565,

    XRGB8888,
    XBGR8888,
    RGBX8888,
    BGRX8888,

    ARGB8888,
    ABGR8888,
    RGBA8888,
    BGRA8888,

    XRGB2101010,
    XBGR2101010,
    RGBX1010102,
    BGRX1010102,

    ARGB2101010,
    ABGR2101010,
    RGBA1010102,
    BGRA1010102,

    YUYV,
    YVYU,
    UYVY,
    VYUY,

    AYUV,
}

impl PixelFormat {
    /// Convert into the raw fourcc code
    pub fn as_raw(&self) -> u32 {
        use self::PixelFormat::*;
        match *self {
            C8 => DRM_FORMAT_C8,
            R8 => DRM_FORMAT_R8,
            GR88 => DRM_FORMAT_GR88,

            RGB332 => DRM_FORMAT_RGB332,
            BGR233 => DRM_FORMAT_BGR233,

            XRGB4444 => DRM_FORMAT_XRGB4444,
            XBGR4444 => DRM_FORMAT_XBGR4444,
            RGBX4444 => DRM_FORMAT_RGBX4444,
            BGRX4444 => DRM_FORMAT_BGRX4444,

            ARGB4444 => DRM_FORMAT_ARGB4444,
            ABGR4444 => DRM_FORMAT_ABGR4444,
            RGBA4444 => DRM_FORMAT_RGBA4444,
            BGRA4444 => DRM_FORMAT_BGRA4444,

            XRGB1555 => DRM_FORMAT_XRGB1555,
            XBGR1555 => DRM_FORMAT_XBGR1555,
            RGBX5551 => DRM_FORMAT_RGBX5551,
            BGRX5551 => DRM_FORMAT_BGRX5551,

            ARGB1555 => DRM_FORMAT_ARGB1555,
            ABGR1555 => DRM_FORMAT_ABGR1555,
            RGBA5551 => DRM_FORMAT_RGBA4444,
            BGRA5551 => DRM_FORMAT_RGBA5551,

            RGB565 => DRM_FORMAT_RGB565,
            BGR565 => DRM_FORMAT_BGR565,

            XRGB8888 => DRM_FORMAT_XRGB8888,
            XBGR8888 => DRM_FORMAT_XBGR8888,
            RGBX8888 => DRM_FORMAT_RGBX8888,
            BGRX8888 => DRM_FORMAT_BGRX8888,

            ARGB8888 => DRM_FORMAT_ARGB8888,
            ABGR8888 => DRM_FORMAT_ABGR8888,
            RGBA8888 => DRM_FORMAT_RGBA8888,
            BGRA8888 => DRM_FORMAT_BGRA8888,

            XRGB2101010 => DRM_FORMAT_XRGB2101010,
            XBGR2101010 => DRM_FORMAT_XBGR2101010,
            RGBX1010102 => DRM_FORMAT_RGBX1010102,
            BGRX1010102 => DRM_FORMAT_BGRX1010102,

            ARGB2101010 => DRM_FORMAT_ARGB2101010,
            ABGR2101010 => DRM_FORMAT_ABGR2101010,
            RGBA1010102 => DRM_FORMAT_RGBA1010102,
            BGRA1010102 => DRM_FORMAT_BGRA1010102,

            YUYV => DRM_FORMAT_YUYV,
            YVYU => DRM_FORMAT_YVYU,
            UYVY => DRM_FORMAT_UYVY,
            VYUY => DRM_FORMAT_VYUY,

            AYUV => DRM_FORMAT_AYUV,
        }
    }

    /// Depth value of the format
    pub fn depth(&self) -> Option<u8> {
        use self::PixelFormat::*;
        match *self {
            XRGB1555 => Some(15),
            RGB565 => Some(16),
            XRGB8888 => Some(24),
            ARGB8888 => Some(32),
            XRGB2101010 => Some(30),
            _ => None,
        }
    }

    /// Bytes per pixel of the used format
    pub fn bpp(&self) -> Option<u8> {
        use self::PixelFormat::*;
        match *self {
            XRGB1555 => Some(16),
            RGB565 => Some(16),
            XRGB8888 => Some(32),
            ARGB8888 => Some(32),
            XRGB2101010 => Some(32),
            _ => None,
        }
    }

    /// Try to parse format from raw fourcc code
    pub fn from_raw(raw: u32) -> Option<PixelFormat> {
        use self::PixelFormat::*;

        match raw {
            x if x == DRM_FORMAT_C8 as u32 => Some(C8),
            x if x == DRM_FORMAT_R8 as u32 => Some(R8),
            x if x == DRM_FORMAT_GR88 as u32 => Some(GR88),

            x if x == DRM_FORMAT_RGB332 as u32 => Some(RGB332),
            x if x == DRM_FORMAT_BGR233 as u32 => Some(BGR233),

            x if x == DRM_FORMAT_XRGB4444 as u32 => Some(XRGB4444),
            x if x == DRM_FORMAT_XBGR4444 as u32 => Some(XBGR4444),
            x if x == DRM_FORMAT_RGBX4444 as u32 => Some(RGBX4444),
            x if x == DRM_FORMAT_BGRX4444 as u32 => Some(BGRX4444),

            x if x == DRM_FORMAT_ARGB4444 as u32 => Some(ARGB4444),
            x if x == DRM_FORMAT_ABGR4444 as u32 => Some(ABGR4444),
            x if x == DRM_FORMAT_RGBA4444 as u32 => Some(RGBA4444),
            x if x == DRM_FORMAT_BGRA4444 as u32 => Some(BGRA4444),

            x if x == DRM_FORMAT_XRGB1555 as u32 => Some(XRGB1555),
            x if x == DRM_FORMAT_XBGR1555 as u32 => Some(XBGR1555),
            x if x == DRM_FORMAT_RGBX5551 as u32 => Some(RGBX5551),
            x if x == DRM_FORMAT_BGRX5551 as u32 => Some(BGRX5551),

            x if x == DRM_FORMAT_ARGB1555 as u32 => Some(ARGB1555),
            x if x == DRM_FORMAT_ABGR1555 as u32 => Some(ABGR1555),
            x if x == DRM_FORMAT_RGBA5551 as u32 => Some(RGBA5551),
            x if x == DRM_FORMAT_BGRA5551 as u32 => Some(BGRA5551),

            x if x == DRM_FORMAT_RGB565 as u32 => Some(RGB565),
            x if x == DRM_FORMAT_BGR565 as u32 => Some(BGR565),

            x if x == DRM_FORMAT_XRGB8888 as u32 => Some(XRGB8888),
            x if x == DRM_FORMAT_XBGR8888 as u32 => Some(XBGR8888),
            x if x == DRM_FORMAT_RGBX8888 as u32 => Some(RGBX8888),
            x if x == DRM_FORMAT_BGRX8888 as u32 => Some(BGRX8888),

            x if x == DRM_FORMAT_ARGB8888 as u32 => Some(ARGB8888),
            x if x == DRM_FORMAT_ABGR8888 as u32 => Some(ABGR8888),
            x if x == DRM_FORMAT_RGBA8888 as u32 => Some(RGBA8888),
            x if x == DRM_FORMAT_BGRA8888 as u32 => Some(BGRA8888),

            x if x == DRM_FORMAT_XRGB2101010 as u32 => Some(XRGB2101010),
            x if x == DRM_FORMAT_XBGR2101010 as u32 => Some(XBGR2101010),
            x if x == DRM_FORMAT_RGBX1010102 as u32 => Some(RGBX1010102),
            x if x == DRM_FORMAT_BGRX1010102 as u32 => Some(BGRX1010102),

            x if x == DRM_FORMAT_ARGB2101010 as u32 => Some(ARGB2101010),
            x if x == DRM_FORMAT_ABGR2101010 as u32 => Some(ABGR2101010),
            x if x == DRM_FORMAT_RGBA1010102 as u32 => Some(RGBA1010102),
            x if x == DRM_FORMAT_BGRA1010102 as u32 => Some(BGRA1010102),

            x if x == DRM_FORMAT_YUYV as u32 => Some(YUYV),
            x if x == DRM_FORMAT_YVYU as u32 => Some(YVYU),
            x if x == DRM_FORMAT_UYVY as u32 => Some(UYVY),
            x if x == DRM_FORMAT_VYUY as u32 => Some(VYUY),

            x if x == DRM_FORMAT_AYUV as u32 => Some(AYUV),

            _ => None,
        }
    }
}
