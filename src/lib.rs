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
//! To begin using this crate, the [Device trait](Device.t.html) must be
//! implemented. See the trait's [example](Device.t.html#example) section for
//! details on how to implement it.
//!

#![warn(missing_docs)]
extern crate core;

extern crate drm_ffi;

extern crate nix;

pub(crate) mod util;

pub mod control;
pub mod buffer;

use std::os::unix::io::AsRawFd;

pub use drm_ffi::result::SystemError;
use util::*;

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
/// extern crate drm;
///
/// use drm::Device;
///
/// use std::fs::File;
/// use std::fs::OpenOptions;
///
/// use std::os::unix::io::RawFd;
/// use std::os::unix::io::AsRawFd;
///
/// #[derive(Debug)]
/// // A simple wrapper for a device node.
/// struct Card(File);
///
/// // Implementing `AsRawFd` is a prerequisite to implementing the traits found
/// // in this crate. Here, we are just calling `as_raw_fd()` on the inner File.
/// impl AsRawFd for Card {
///     fn as_raw_fd(&self) -> RawFd {
///         self.0.as_raw_fd()
///     }
/// }
///
/// /// With `AsRawFd` implemented, we can now implement `drm::Device`.
/// impl Device for Card {}
///
/// // Simple helper method for opening a `Card`.
/// impl Card {
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
pub trait Device: AsRawFd {
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
    fn acquire_master_lock(&self) -> Result<(), SystemError> {
        drm_ffi::auth::acquire_master(self.as_raw_fd())?;
        Ok(())
    }

    /// Releases the DRM Master lock for another process to use.
    fn release_master_lock(&self) -> Result<(), SystemError> {
        drm_ffi::auth::release_master(self.as_raw_fd())?;
        Ok(())
    }

    /// Generates an [AuthToken](AuthToken.t.html) for this process.
    #[deprecated(note = "Consider opening a render node instead.")]
    fn generate_auth_token(&self) -> Result<AuthToken, SystemError> {
        let token = drm_ffi::auth::get_magic_token(self.as_raw_fd())?;
        Ok(AuthToken(token.magic))
    }

    /// Authenticates an [AuthToken](AuthToken.t.html) from another process.
    fn authenticate_auth_token(&self, token: AuthToken) -> Result<(), SystemError> {
        drm_ffi::auth::auth_magic_token(self.as_raw_fd(), token.0)?;
        Ok(())
    }

    /// Requests the driver to expose or hide certain capabilities. See
    /// [ClientCapability](ClientCapability.t.html) for more information.
    fn set_client_capability(
        &self,
        cap: ClientCapability,
        enable: bool,
    ) -> Result<(), SystemError> {
        drm_ffi::set_capability(self.as_raw_fd(), cap as u64, enable)?;
        Ok(())
    }

    /// Gets the [BusID](BusID.t.html) of this device.
    fn get_bus_id(&self) -> Result<BusID, SystemError> {
        let mut buffer = [0u8; 32];

        let buffer_len;

        let _busid = {
            let mut slice = &mut buffer[..];
            let busid = drm_ffi::get_bus_id(self.as_raw_fd(), Some(&mut slice))?;
            buffer_len = slice.len();
            busid
        };

        let bus_id = BusID(SmallOsString::from_u8_buffer(buffer, buffer_len));

        Ok(bus_id)
    }

    /// Check to see if our [AuthToken](AuthToken.t.html) has been authenticated
    /// by the DRM Master
    fn authenticated(&self) -> Result<bool, SystemError> {
        let client = drm_ffi::get_client(self.as_raw_fd(), 0)?;
        Ok(client.auth == 1)
    }

    /// Gets the value of a capability.
    fn get_driver_capability(&self, cap: DriverCapability) -> Result<u64, SystemError> {
        let cap = drm_ffi::get_capability(self.as_raw_fd(), cap as u64)?;
        Ok(cap.value)
    }

    /// Possible errors:
    ///   - EFAULT: Kernel could not copy fields into userspace
    #[allow(missing_docs)]
    fn get_driver(&self) -> Result<Driver, SystemError> {
        let mut name = [0i8; 32];
        let mut date = [0i8; 32];
        let mut desc = [0i8; 32];

        let name_len;
        let date_len;
        let desc_len;

        let _version = {
            let mut name_slice = &mut name[..];
            let mut date_slice = &mut date[..];
            let mut desc_slice = &mut desc[..];

            let version = drm_ffi::get_version(
                self.as_raw_fd(),
                Some(&mut name_slice),
                Some(&mut date_slice),
                Some(&mut desc_slice),
            )?;

            name_len = name_slice.len();
            date_len = date_slice.len();
            desc_len = desc_slice.len();

            version
        };

        let name = SmallOsString::from_i8_buffer(name, name_len);
        let date = SmallOsString::from_i8_buffer(date, date_len);
        let desc = SmallOsString::from_i8_buffer(desc, desc_len);

        let driver = Driver {
            name: name,
            date: date,
            desc: desc,
        };

        Ok(driver)
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

/// Bus ID of a device.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct BusID(SmallOsString);

impl AsRef<OsStr> for BusID {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

/// Driver version of a device.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Driver {
    name: SmallOsString,
    date: SmallOsString,
    desc: SmallOsString,
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
    /// PRIME handles are support
    Prime = drm_ffi::DRM_CAP_PRIME as u64,
    /// Unknown
    MonotonicTimestamp = drm_ffi::DRM_CAP_TIMESTAMP_MONOTONIC as u64,
    /// Asynchronous page flipping support
    ASyncPageFlip = drm_ffi::DRM_CAP_ASYNC_PAGE_FLIP as u64,
    /// Width of cursor buffers
    CursorWidth = drm_ffi::DRM_CAP_CURSOR_WIDTH as u64,
    /// Height of cursor buffers
    CursorHeight = drm_ffi::DRM_CAP_CURSOR_HEIGHT as u64,
    /// You can create framebuffers with modifiers
    AddFB2Modifiers = drm_ffi::DRM_CAP_ADDFB2_MODIFIERS as u64,
    /// Unknown
    PageFlipTarget = drm_ffi::DRM_CAP_PAGE_FLIP_TARGET as u64,
    /// Uses the CRTC's ID in vblank events
    CRTCInVBlankEvent = drm_ffi::DRM_CAP_CRTC_IN_VBLANK_EVENT as u64,
    /// SyncObj support
    SyncObj = drm_ffi::DRM_CAP_SYNCOBJ as u64,
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
}
