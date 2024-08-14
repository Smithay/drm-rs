//! A safe interface to the Direct Rendering Manager subsystem found in various
//! operating systems.
//!
//! # Summary
//!
//! The Direct Rendering Manager (DRM) is subsystem found in various operating
//! systems that exposes graphical functionality to userspace processes. It can
//! be used to send data and commands to a GPU driver that implements the
//! interface.
//!
//! Userspace processes can access the DRM by opening a 'device node' (usually
//! found in `/dev/dri/*`) and using various `ioctl` commands on the open file
//! descriptor. Most processes use the libdrm library (part of the mesa project)
//! to execute these commands. This crate takes a more direct approach,
//! bypassing libdrm and executing the commands directly and doing minimal
//! abstraction to keep the interface safe.
//!
//! While the DRM subsystem exposes many powerful GPU interfaces, it is not
//! recommended for rendering or GPGPU operations. There are many standards made
//! for these use cases, and they are far more fitting for those sort of tasks.
//!
//! ## Usage
//!
//! To begin using this crate, the [`Device`] trait must be
//! implemented. See the trait's [example section](trait@Device#example) for
//! details on how to implement it.
//!

#![warn(missing_docs)]

pub(crate) mod util;

pub mod buffer;
pub mod control;
pub mod node;

use std::ffi::{OsStr, OsString};
use std::time::Duration;
use std::{
    io,
    os::unix::{ffi::OsStringExt, io::AsFd},
};

use rustix::io::Errno;

use crate::util::*;

pub use drm_ffi::{DRM_CLOEXEC as CLOEXEC, DRM_RDWR as RDWR};

/// This trait should be implemented by any object that acts as a DRM device. It
/// is a prerequisite for using any DRM functionality.
///
/// This crate does not provide a concrete device object due to the various ways
/// it can be implemented. The user of this crate is expected to implement it
/// themselves and derive this trait as necessary. The example below
/// demonstrates how to do this using a small wrapper.
///
/// # Example
///
/// ```
/// use drm::Device;
///
/// use std::fs::File;
/// use std::fs::OpenOptions;
///
/// use std::os::unix::io::AsFd;
/// use std::os::unix::io::BorrowedFd;
///
/// #[derive(Debug)]
/// /// A simple wrapper for a device node.
/// struct Card(File);
///
/// /// Implementing [`AsFd`] is a prerequisite to implementing the traits found
/// /// in this crate. Here, we are just calling [`File::as_fd()`] on the inner
/// /// [`File`].
/// impl AsFd for Card {
///     fn as_fd(&self) -> BorrowedFd<'_> {
///         self.0.as_fd()
///     }
/// }
///
/// /// With [`AsFd`] implemented, we can now implement [`drm::Device`].
/// impl Device for Card {}
///
/// impl Card {
///     /// Simple helper method for opening a [`Card`].
///     fn open() -> Self {
///         let mut options = OpenOptions::new();
///         options.read(true);
///         options.write(true);
///
///         // The normal location of the primary device node on Linux
///         Card(options.open("/dev/dri/card0").unwrap())
///     }
/// }
/// ```
pub trait Device: AsFd {
    /// Acquires the DRM Master lock for this process.
    ///
    /// # Notes
    ///
    /// Acquiring the DRM Master is done automatically when the primary device
    /// node is opened. If you opened the primary device node and did not
    /// acquire the lock, another process likely has the lock.
    ///
    /// This function is only available to processes with CAP_SYS_ADMIN
    /// privileges (usually as root)
    fn acquire_master_lock(&self) -> io::Result<()> {
        drm_ffi::auth::acquire_master(self.as_fd())?;
        Ok(())
    }

    /// Releases the DRM Master lock for another process to use.
    fn release_master_lock(&self) -> io::Result<()> {
        drm_ffi::auth::release_master(self.as_fd())?;
        Ok(())
    }

    /// Generates an [`AuthToken`] for this process.
    #[deprecated(note = "Consider opening a render node instead.")]
    fn generate_auth_token(&self) -> io::Result<AuthToken> {
        let token = drm_ffi::auth::get_magic_token(self.as_fd())?;
        Ok(AuthToken(token.magic))
    }

    /// Authenticates an [`AuthToken`] from another process.
    fn authenticate_auth_token(&self, token: AuthToken) -> io::Result<()> {
        drm_ffi::auth::auth_magic_token(self.as_fd(), token.0)?;
        Ok(())
    }

    /// Requests the driver to expose or hide certain capabilities. See
    /// [`ClientCapability`] for more information.
    fn set_client_capability(&self, cap: ClientCapability, enable: bool) -> io::Result<()> {
        drm_ffi::set_capability(self.as_fd(), cap as u64, enable)?;
        Ok(())
    }

    /// Gets the bus ID of this device.
    fn get_bus_id(&self) -> io::Result<OsString> {
        let mut buffer = Vec::new();
        let _ = drm_ffi::get_bus_id(self.as_fd(), Some(&mut buffer))?;
        let bus_id = OsString::from_vec(buffer);

        Ok(bus_id)
    }

    /// Check to see if our [`AuthToken`] has been authenticated
    /// by the DRM Master
    fn authenticated(&self) -> io::Result<bool> {
        let client = drm_ffi::get_client(self.as_fd(), 0)?;
        Ok(client.auth == 1)
    }

    /// Gets the value of a capability.
    fn get_driver_capability(&self, cap: DriverCapability) -> io::Result<u64> {
        let cap = drm_ffi::get_capability(self.as_fd(), cap as u64)?;
        Ok(cap.value)
    }

    /// # Possible errors:
    ///   - `EFAULT`: Kernel could not copy fields into userspace
    #[allow(missing_docs)]
    fn get_driver(&self) -> io::Result<Driver> {
        let mut name = Vec::new();
        let mut date = Vec::new();
        let mut desc = Vec::new();

        let v = drm_ffi::get_version(
            self.as_fd(),
            Some(&mut name),
            Some(&mut date),
            Some(&mut desc),
        )?;

        let version = (v.version_major, v.version_minor, v.version_patchlevel);
        let name = OsString::from_vec(unsafe { transmute_vec(name) });
        let date = OsString::from_vec(unsafe { transmute_vec(date) });
        let desc = OsString::from_vec(unsafe { transmute_vec(desc) });

        let driver = Driver {
            version,
            name,
            date,
            desc,
        };

        Ok(driver)
    }

    /// Waits for a vblank.
    fn wait_vblank(
        &self,
        target_sequence: VblankWaitTarget,
        flags: VblankWaitFlags,
        high_crtc: u32,
        user_data: usize,
    ) -> io::Result<VblankWaitReply> {
        use drm_ffi::drm_vblank_seq_type::_DRM_VBLANK_HIGH_CRTC_MASK;
        use drm_ffi::_DRM_VBLANK_HIGH_CRTC_SHIFT;

        let high_crtc_mask = _DRM_VBLANK_HIGH_CRTC_MASK >> _DRM_VBLANK_HIGH_CRTC_SHIFT;
        if (high_crtc & !high_crtc_mask) != 0 {
            return Err(Errno::INVAL.into());
        }

        let (sequence, wait_type) = match target_sequence {
            VblankWaitTarget::Absolute(n) => {
                (n, drm_ffi::drm_vblank_seq_type::_DRM_VBLANK_ABSOLUTE)
            }
            VblankWaitTarget::Relative(n) => {
                (n, drm_ffi::drm_vblank_seq_type::_DRM_VBLANK_RELATIVE)
            }
        };

        let type_ = wait_type | (high_crtc << _DRM_VBLANK_HIGH_CRTC_SHIFT) | flags.bits();
        let reply = drm_ffi::wait_vblank(self.as_fd(), type_, sequence, user_data)?;

        let time = match (reply.tval_sec, reply.tval_usec) {
            (0, 0) => None,
            (sec, usec) => Some(Duration::new(sec as u64, (usec * 1000) as u32)),
        };

        Ok(VblankWaitReply {
            frame: reply.sequence,
            time,
        })
    }
}

/// An authentication token, unique to the file descriptor of the device.
///
/// This token can be sent to another process that owns the DRM Master lock to
/// allow unprivileged use of the device, such as rendering.
///
/// # Deprecation Notes
///
/// This method of authentication is somewhat deprecated. Accessing unprivileged
/// functionality is best done by opening a render node. However, some other
/// processes may still use this method of authentication. Therefore, we still
/// provide functionality for generating and authenticating these tokens.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct AuthToken(u32);

/// Driver version of a device.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Driver {
    /// Version of the driver in `(major, minor, patchlevel)` format
    pub version: (i32, i32, i32),
    /// Name of the driver
    pub name: OsString,
    /// Date driver was published
    pub date: OsString,
    /// Driver description
    pub desc: OsString,
}

impl Driver {
    /// Name of driver
    pub fn name(&self) -> &OsStr {
        self.name.as_ref()
    }

    /// Date driver was published
    pub fn date(&self) -> &OsStr {
        self.date.as_ref()
    }

    /// Driver description
    pub fn description(&self) -> &OsStr {
        self.desc.as_ref()
    }
}

/// Used to check which capabilities your graphics driver has.
#[allow(clippy::upper_case_acronyms)]
#[repr(u64)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum DriverCapability {
    /// DumbBuffer support for scanout
    DumbBuffer = drm_ffi::DRM_CAP_DUMB_BUFFER as u64,
    /// Unknown
    VBlankHighCRTC = drm_ffi::DRM_CAP_VBLANK_HIGH_CRTC as u64,
    /// Preferred depth to use for dumb buffers
    DumbPreferredDepth = drm_ffi::DRM_CAP_DUMB_PREFERRED_DEPTH as u64,
    /// Unknown
    DumbPreferShadow = drm_ffi::DRM_CAP_DUMB_PREFER_SHADOW as u64,
    /// PRIME handles are supported
    Prime = drm_ffi::DRM_CAP_PRIME as u64,
    /// Unknown
    MonotonicTimestamp = drm_ffi::DRM_CAP_TIMESTAMP_MONOTONIC as u64,
    /// Asynchronous page flipping support
    ASyncPageFlip = drm_ffi::DRM_CAP_ASYNC_PAGE_FLIP as u64,
    /// Asynchronous page flipping support for atomic API
    AtomicASyncPageFlip = drm_ffi::DRM_CAP_ATOMIC_ASYNC_PAGE_FLIP as u64,
    /// Width of cursor buffers
    CursorWidth = drm_ffi::DRM_CAP_CURSOR_WIDTH as u64,
    /// Height of cursor buffers
    CursorHeight = drm_ffi::DRM_CAP_CURSOR_HEIGHT as u64,
    /// Create framebuffers with modifiers
    AddFB2Modifiers = drm_ffi::DRM_CAP_ADDFB2_MODIFIERS as u64,
    /// Unknown
    PageFlipTarget = drm_ffi::DRM_CAP_PAGE_FLIP_TARGET as u64,
    /// Uses the CRTC's ID in vblank events
    CRTCInVBlankEvent = drm_ffi::DRM_CAP_CRTC_IN_VBLANK_EVENT as u64,
    /// SyncObj support
    SyncObj = drm_ffi::DRM_CAP_SYNCOBJ as u64,
    /// Timeline SyncObj support
    TimelineSyncObj = drm_ffi::DRM_CAP_SYNCOBJ_TIMELINE as u64,
}

/// Used to enable/disable capabilities for the process.
#[repr(u64)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ClientCapability {
    /// The driver provides 3D screen control
    Stereo3D = drm_ffi::DRM_CLIENT_CAP_STEREO_3D as u64,
    /// The driver provides more plane types for modesetting
    UniversalPlanes = drm_ffi::DRM_CLIENT_CAP_UNIVERSAL_PLANES as u64,
    /// The driver provides atomic modesetting
    Atomic = drm_ffi::DRM_CLIENT_CAP_ATOMIC as u64,
    /// If set to 1, the DRM core will provide aspect ratio information in modes.
    AspectRatio = drm_ffi::DRM_CLIENT_CAP_ASPECT_RATIO as u64,
    /// If set to 1, the DRM core will expose special connectors to be used for
    /// writing back to memory the scene setup in the commit.
    ///
    /// The client must enable [`Self::Atomic`] first.
    WritebackConnectors = drm_ffi::DRM_CLIENT_CAP_WRITEBACK_CONNECTORS as u64,
    /// Drivers for para-virtualized hardware have additional restrictions for cursor planes e.g.
    /// they need cursor planes to act like one would expect from a mouse
    /// cursor and have correctly set hotspot properties.
    /// If this client cap is not set the DRM core will hide cursor plane on
    /// those virtualized drivers because not setting it implies that the
    /// client is not capable of dealing with those extra restictions.
    /// Clients which do set cursor hotspot and treat the cursor plane
    /// like a mouse cursor should set this property.
    ///
    /// The client must enable [`Self::Atomic`] first.
    CursorPlaneHotspot = drm_ffi::DRM_CLIENT_CAP_CURSOR_PLANE_HOTSPOT as u64,
}

/// Used to specify a vblank sequence to wait for
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum VblankWaitTarget {
    /// Wait for a specific vblank sequence number
    Absolute(u32),
    /// Wait for a given number of vblanks
    Relative(u32),
}

bitflags::bitflags! {
    /// Flags to alter the behaviour when waiting for a vblank
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct VblankWaitFlags : u32 {
        /// Send event instead of blocking
        const EVENT = drm_ffi::drm_vblank_seq_type::_DRM_VBLANK_EVENT;
        /// If missed, wait for next vblank
        const NEXT_ON_MISS = drm_ffi::drm_vblank_seq_type::_DRM_VBLANK_NEXTONMISS;
    }
}

/// Data returned from a vblank wait
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct VblankWaitReply {
    frame: u32,
    time: Option<Duration>,
}

impl VblankWaitReply {
    /// Sequence of the frame
    pub fn frame(&self) -> u32 {
        self.frame
    }

    /// Time at which the vblank occurred. [`None`] if an asynchronous event was
    /// requested
    pub fn time(&self) -> Option<Duration> {
        self.time
    }
}
