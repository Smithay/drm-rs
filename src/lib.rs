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
#![feature(str_internals)]
extern crate core;

extern crate drm_sys;

#[macro_use]
extern crate failure;
#[macro_use]
extern crate nix;

pub mod ffi;
pub mod result;
pub(crate) mod util;

pub mod control;
//pub mod buffer;

use result::SystemError;
use util::*;

use std::os::unix::io::AsRawFd;

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
        ffi::basic::auth::acquire_master(self.as_raw_fd())
            .map_err(|e| SystemError::from(result::unwrap_errno(e)))
    }

    /// Releases the DRM Master lock for another process to use.
    fn release_master_lock(&self) -> Result<(), SystemError> {
        ffi::basic::auth::release_master(self.as_raw_fd())
            .map_err(|e| SystemError::from(result::unwrap_errno(e)))
    }

    /// Generates an [AuthToken](AuthToken.t.html) for this process.
    #[deprecated(note = "Consider opening a render node instead.")]
    fn generate_auth_token(&self) -> Result<AuthToken, SystemError> {
        let token = ffi::basic::auth::get_magic_token(self.as_raw_fd())
            .map_err(|e| SystemError::from(result::unwrap_errno(e)))?;
        Ok(AuthToken(token.magic))
    }

    /// Authenticates an [AuthToken](AuthToken.t.html) from another process.
    fn authenticate_auth_token(&self, token: AuthToken) -> Result<(), SystemError> {
        ffi::basic::auth::auth_magic_token(self.as_raw_fd(), token.0)
            .map_err(|e| SystemError::from(result::unwrap_errno(e)))
    }

    /// Requests the driver to expose or hide certain capabilities. See
    /// [ClientCapability](ClientCapability.t.html) for more information.
    fn set_client_capability(
        &self,
        cap: ClientCapability,
        enable: bool,
    ) -> Result<(), SystemError> {
        ffi::basic::set_capability(self.as_raw_fd(), cap as u64, enable)
            .map_err(|e| SystemError::from(result::unwrap_errno(e)))?;
        Ok(())
    }

    /// Gets the [BusID](BusID.t.html) of this device.
    fn get_bus_id(&self) -> Result<BusID, SystemError> {
        let mut buffer = [0u8; 32];
        let len = {
            let mut slice = &mut buffer[..];
            ffi::basic::get_bus_id(self.as_raw_fd(), &mut slice)
                .map_err(|e| SystemError::from(result::unwrap_errno(e)))?;
            slice.len()
        };

        let bus_id = BusID(SmallOsString::new(buffer, len));

        Ok(bus_id)
    }

    /// Check to see if our [AuthToken](AuthToken.t.html) has been authenticated
    /// by the DRM Master
    fn authenticated(&self) -> Result<bool, SystemError> {
        let client = ffi::basic::get_client(self.as_raw_fd(), 0)
            .map_err(|e| SystemError::from(result::unwrap_errno(e)))?;
        Ok(client.auth == 1)
    }

    /// Gets the value of a capability.
    fn get_driver_capability(&self, cap: DriverCapability) -> Result<u64, SystemError> {
        ffi::basic::get_capability(self.as_raw_fd(), cap as u64)
            .map_err(|e| SystemError::from(result::unwrap_errno(e)))
    }

    /// Possible errors:
    ///   - EFAULT: Kernel could not copy fields into userspace
    #[allow(missing_docs)]
    fn get_driver(&self) -> Result<Driver, SystemError> {
        let mut name = [0u8; 32];
        let mut date = [0u8; 32];
        let mut desc = [0u8; 32];

        let (name_len, date_len, desc_len) = {
            let mut name_slice = &mut name[..];
            let mut date_slice = &mut date[..];
            let mut desc_slice = &mut desc[..];

            ffi::basic::get_version(
                self.as_raw_fd(),
                &mut name_slice,
                &mut date_slice,
                &mut desc_slice,
            ).map_err(|e| SystemError::from(result::unwrap_errno(e)))?;

            (name_slice.len(), date_slice.len(), desc_slice.len())
        };

        let name = SmallOsString::new(name, name_len);
        let date = SmallOsString::new(date, date_len);
        let desc = SmallOsString::new(desc, desc_len);

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

/// Capabilities that can be toggled by the driver.
///
/// # Notes
///
/// Some DRM functionality is not immediately exposed by the driver. Before
/// a process can access this functionality, we must ask the driver to
/// expose it. This can be done using
/// [toggle_capability](toggle_capability.t.html).
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ClientCapability {
    /// Stereoscopic 3D Support
    Stereo3D = ffi::DRM_CLIENT_CAP_STEREO_3D as isize,
    /// Universal plane access and api
    UniversalPlanes = ffi::DRM_CLIENT_CAP_UNIVERSAL_PLANES as isize,
    /// Atomic modesetting support
    Atomic = ffi::DRM_CLIENT_CAP_ATOMIC as isize,
}

/// Immutable capabilities and attributes of this driver.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum DriverCapability {
    /// Do we support VBlanks at CRTC high
    VBlankHighCrtc = ffi::DRM_CAP_VBLANK_HIGH_CRTC as isize,
    /// Driver's preferred dumb buffer depth
    DumbBufferDepthPref = ffi::DRM_CAP_DUMB_PREFERRED_DEPTH as isize,
    /// Does the driver prefer shadow dumb buffers
    DumbBufferShadowPref = ffi::DRM_CAP_DUMB_PREFER_SHADOW as isize,
    /// Are timestamps monotonic
    MonotonicTimestamp = ffi::DRM_CAP_TIMESTAMP_MONOTONIC as isize,
    /// Do we support asynchronous page flips.
    AsyncPageFlip = ffi::DRM_CAP_ASYNC_PAGE_FLIP as isize,
    /// What is the width of the cursor plane
    CursorWidth = ffi::DRM_CAP_CURSOR_WIDTH as isize,
    /// What is the height of the cursor plane
    CursorHeight = ffi::DRM_CAP_CURSOR_HEIGHT as isize,
    /// Can we use modifiers when adding frame buffers
    AddFBModifiers = ffi::DRM_CAP_ADDFB2_MODIFIERS as isize,
    /// Target page flip
    PageFlipTarget = ffi::DRM_CAP_PAGE_FLIP_TARGET as isize,
    /// Does a VBlank event include the CRTC id
    CrtcInVBlank = ffi::DRM_CAP_CRTC_IN_VBLANK_EVENT as isize,
    /// Do we support syncobj
    SyncObj = ffi::DRM_CAP_SYNCOBJ as isize,
}

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

/// Signed point
#[allow(non_camel_case_types)]
pub type iPoint = (i32, i32);

/// Unsigned point
#[allow(non_camel_case_types)]
pub type uPoint = (u32, u32);

/// Dimensions (width, height)
pub type Dimensions = (u32, u32);

/// Rectangle with a signed upper left corner
#[allow(non_camel_case_types)]
pub type iRect = (iPoint, Dimensions);

/// Rectangle with an unsigned upper left corner
#[allow(non_camel_case_types)]
pub type uRect = (uPoint, Dimensions);
