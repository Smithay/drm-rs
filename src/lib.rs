//!
//! # drm-rs
//!
//! drm-rs
//!
//! This library is a safe interface to the Direct Rendering Manager API found on various operating systems.
//!
//! This library is currently a work in progress.
//!
//! ## Usage
//!
//! The user is expected to implement their own functionality for opening and
//! accessing the file descriptor of the device. Here we create a small wrapper
//! around `File` and implement `AsRawFd`, `drm::Device`, and
//! `drm::control::Device`:
//!
//! ```rust,ignore
//! extern crate drm;
//!
//! use std::fs::{OpenOptions, File};
//! use std::os::unix::io::{AsRawFd, RawFd};
//!
//! use drm::Device as BasicDevice;
//! use drm::control::Device as ControlDevice;
//!
//! // The drm crate does not provide a method of opening the device.
//! // It is expected to be implemented by the user.
//! struct Card(File);
//!
//! // Required to implement drm::Device
//! impl AsRawFd for Card {
//!     fn as_raw_fd(&self) -> RawFd { self.0.as_raw_fd() }
//! }
//!
//! // Required to implement drm::control::Device
//! impl BasicDevice for Card { }
//!
//! // Allows modesetting functionality to be performed.
//! impl ControlDevice for Card { }
//!
//! ```
//!
//! Assuming the program used the above wrapper, the user now opens the card:
//!
//! ```rust,ignore
//! // Open the device (usually located at /dev/dri/*) with rw access.
//! let mut options = OpenOptions::new();
//! options.read(true);
//! options.write(true);
//! let file = options.open("/dev/dri/card0");
//! let card = Card(file);
//! ```
//!
//! Now we can check out what resources are available:
//!
//! ```rust,ignore
//! // Get a set of all modesetting resource handles (excluding planes):
//! let res_handles = card.resource_handles().unwrap();
//!
//! // Print all connector information
//! for &con in res_handles.connectors() {
//!     let info = card.resource_info(con).unwrap();
//!
//!     println!("{:#?}")
//! }
//!
//! // Print all CRTC information
//! for &crtc in res_handles.crtcs() {
//!     let info = card.resource_info(crtc).unwrap();
//!
//!     println!("{:#?}")
//! }
//! ```
//!
//! You'll also want to find a suitable mode:
//!
//! ```rust,ignore
//! // Assuming we found a good connector and loaded the info into `connector_info`
//! let &mode = connector_info.modes().iter(); // Search until you find one you want.
//! ```
//!
//! Once you find a suitable connector and CRTC, it's time to create a framebuffer.
//! Here we use a simple dumbbuffer as the backend:'
//!
//! ```rust,ignore
//! // Create a DB of size 1920x1080
//! let db = dumbbuffer::DumbBuffer::create_from_device(&card, (1920, 1080), 32)
//!     .expect("Could not create dumb buffer");
//!
//! // Map it and grey it out.
//! let mut map = db.map(&card).expect("Could not map dumbbuffer");
//! for mut b in map.as_mut() {
//!     *b = 128; // Grey
//! }
//!
//! let fb_info = framebuffer::Info::create_from_buffer(&card, &db)
//! let fb_handle = fb_info.handle();
//! ```
//!
//! Now we can apply the framebuffer onto the CRTC's internal plane, and connect it
//! to a connector with the proper mode:
//!
//! ```rust,ignore
//! // Assuming `crtc` is a crtc handle and `con` is a connector handle
//! crtc.set_on_device(&card, fb_handle, &[con], (0, 0), Some(mode))
//!     .expect("Could not set Crtc");
//! ```
//!
//! The contents of the dumb buffer will now appear onto the screen.

#![warn(missing_docs)]

extern crate drm_sys;

#[macro_use]
extern crate nix;

#[macro_use]
extern crate derive_more;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate failure;

#[macro_use]
pub mod ffi;
pub mod result;

pub mod control;
pub mod buffer;

use std::os::unix::io::AsRawFd;
use result::Result;

use failure::Error;
/// This trait can be implemented by any object that acts as a DRM device. It is
/// a prerequisite for using any DRM functionality.

pub trait Device: AsRawFd {
    /// Tells the DRM device whether we understand or do not understand a
    /// particular capability.
    ///
    /// Some features, such as atomic modesetting, require informing the device
    /// that the process can use such features before it will expose them.
    fn set_client_cap(&self, cap: ClientCapability, set: bool) -> Result<()> {
        let mut raw = {
            let mut raw: ffi::drm_set_client_cap = Default::default();
            raw.capability = cap as u64;
            raw.value = set as u64;
            raw
        };

        unsafe { ffi::ioctl_set_client_cap(self.as_raw_fd(), &mut raw)? };

        Ok(())
    }

    /// Acquires the DRM Master lock for this process.
    ///
    /// # Notes
    ///
    /// Acquiring the DRM Master is done automatically when the device is
    /// opened. If you do not already have the lock, another process probably
    /// beat it to it.
    ///
    /// This function is only available to processes with CAP_SYS_ADMIN
    /// privileges (usually as root)
    fn set_master(&self) -> Result<()> {
        unsafe { ffi::ioctl_set_master(self.as_raw_fd())? };

        Ok(())
    }

    /// Releases the DRM Master lock for another process to use.
    fn drop_master(&self) -> Result<()> {
        unsafe { ffi::ioctl_drop_master(self.as_raw_fd())? };

        Ok(())
    }

    #[deprecated(note = "DRM Deprecated: Consider opening a render node instead")]
    /// Generates an [AuthToken](AuthToken.t.html) for this process.
    fn generate_auth_token(&self) -> Result<AuthToken> {
        let token = {
            let mut raw: ffi::drm_auth_t = Default::default();
            unsafe { ffi::ioctl_get_magic(self.as_raw_fd(), &mut raw)? };
            raw.magic
        };

        Ok(AuthToken(token))
    }

    #[deprecated(note = "DRM Deprecated: Consider opening a render node instead")]
    /// Authenticates an [AuthToken](AuthToken.t.html) from another process.
    fn authenticate_auth_token(&self, token: AuthToken) -> Result<()> {
        let mut raw = ffi::drm_auth_t {
            magic: token.into(),
        };
        unsafe { ffi::ioctl_auth_magic(self.as_raw_fd(), &mut raw)? };
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// Capabilities that the process understands.
///
/// These can be used to tell the DRM device what capabilities the process can
/// use.
pub enum ClientCapability {
    /// Stereoscopic 3D Support
    Stereo3D = ffi::DRM_CLIENT_CAP_STEREO_3D as isize,
    /// Universal plane access and api
    UniversalPlanes = ffi::DRM_CLIENT_CAP_UNIVERSAL_PLANES as isize,
    /// Atomic modesetting support
    Atomic = ffi::DRM_CLIENT_CAP_ATOMIC as isize,
}

#[deprecated(note = "DRM Deprecated: Consider opening a render node instead")]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
/// An authentication token, unique to the process that determines who opened the
/// device.
///
/// This token can be sent to another process that acts as the DRM Master to
/// allow unprivileged use of the device.
///
/// # Deprecated
///
/// This method of authentication is deprecated. If you need to use the DRM
/// device for unprivileged use, consider opening a Render Node, which requires
/// no authentication to use.
pub struct AuthToken(u32);

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
