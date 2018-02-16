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

#![warn(missing_docs)]

extern crate drm_sys;

#[macro_use]
extern crate nix;

#[macro_use]
extern crate derive_more;

#[macro_use]
extern crate error_chain;

#[macro_use]
pub(crate) mod ffi;
pub mod result;

pub mod control;

use ffi::Wrapper;

use std::os::unix::io::AsRawFd;
use result::Result;

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
    fn acquire_master_lock(&self) -> Result<()> {
        unsafe { ffi::ioctl::acquire_master(self.as_raw_fd())?; }
        Ok(())
    }

    /// Releases the DRM Master lock for another process to use.
    fn release_master_lock(&self) -> Result<()> {
        unsafe { ffi::ioctl::release_master(self.as_raw_fd())?; }
        Ok(())
    }

    #[deprecated(note="Consider opening a render node instead.")]
    /// Generates an [AuthToken](AuthToken.t.html) for this process.
    fn generate_auth_token(&self) -> Result<AuthToken> {
        let mut t = ffi::GetToken::default();
        t.ioctl(self.as_raw_fd())?;

        let token = AuthToken(t.raw_ref().magic);
        Ok(token)
    }

    /// Authenticates an [AuthToken](AuthToken.t.html) from another process.
    ///
    /// # Deprecation Notes
    ///
    /// A process should consider opening a render node instead of using
    /// authentication tokens. However, this particular function is not marked
    /// deprecated due to the need to authenticate older processes that do not
    /// yet know about render nodes.
    fn authenticate_auth_token(&self, token: AuthToken) -> Result<()> {
        let mut t = ffi::AuthToken::default();
        t.raw_mut_ref().magic = token.into();
        t.ioctl(self.as_raw_fd())?;
        Ok(())
    }

    /// Requests the driver to expose or hide certain capabilities. See
    /// [ClientCapability](ClientCapability.t.html) for more information.
    fn toggle_capability(&self, cap: ClientCapability, enable: bool)
                         -> Result<()> {
        let mut t = ffi::SetCap::default();
        t.raw_mut_ref().capability = cap as u64;
        t.raw_mut_ref().value = enable as u64;
        t.ioctl(self.as_raw_fd())?;

        Ok(())
    }

    #[allow(missing_docs)]
    fn get_bus_id(&self) {
        unimplemented!();
    }

    #[allow(missing_docs)]
    fn get_client(&self) {
        unimplemented!();
    }

    #[allow(missing_docs)]
    fn get_stats(&self) {
        unimplemented!();
    }

    #[allow(missing_docs)]
    fn get_capability(&self) {
        unimplemented!();
    }

    #[allow(missing_docs)]
    fn set_version(&self) {
        unimplemented!();
    }

    #[allow(missing_docs)]
    fn get_version(&self) {
        unimplemented!();
    }

    #[allow(missing_docs)]
    fn set_irq_handler(&self) {
        unimplemented!();
    }

    #[allow(missing_docs)]
    fn wait_vblank(&self) {
        unimplemented!();
    }

    #[allow(missing_docs)]
    fn modeset_control(&self) {
        unimplemented!();
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
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
pub struct AuthToken(u32);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// Capabilities that can be toggled by the driver.
///
/// # Notes
///
/// Some DRM functionality is not immediately exposed by the driver. Before
/// a process can access this functionality, we must ask the driver to
/// expose it. This can be done using
/// [toggle_capability](toggle_capability.t.html).
pub enum ClientCapability {
    /// Stereoscopic 3D Support
    Stereo3D = ffi::DRM_CLIENT_CAP_STEREO_3D as isize,
    /// Universal plane access and api
    UniversalPlanes = ffi::DRM_CLIENT_CAP_UNIVERSAL_PLANES as isize,
    /// Atomic modesetting support
    Atomic = ffi::DRM_CLIENT_CAP_ATOMIC as isize,
}

#[allow(non_camel_case_types)]
/// Signed point
pub type iPoint = (i32, i32);
#[allow(non_camel_case_types)]
/// Unsigned point
pub type uPoint = (u32, u32);
/// Dimensions (width, height)
pub type Dimensions = (u32, u32);
#[allow(non_camel_case_types)]
/// Rectangle with a signed upper left corner
pub type iRect = (iPoint, Dimensions);
#[allow(non_camel_case_types)]
/// Rectangle with an unsigned upper left corner
pub type uRect = (uPoint, Dimensions);
