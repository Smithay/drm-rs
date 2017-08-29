#![feature(associated_consts)]

#[macro_use]
extern crate nix;
extern crate drm_sys;

#[macro_use]
extern crate error_chain;

#[macro_use]
pub mod ffi;
pub mod result;

pub mod control;
pub mod buffer;

use std::os::unix::io::AsRawFd;
use result::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A token unique to the process that determines who opened the device.
///
/// This token can be sent to another process that acts as the DRM Master and
/// then authenticated to give extra privileges.
pub struct AuthToken(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Capabilities that the process understands.
///
/// These can be used to tell the DRM device what capabilities the process can
/// use.
pub enum ClientCapability {
    Stereo3D = ffi::DRM_CLIENT_CAP_STEREO_3D as isize,
    UniversalPlanes = ffi::DRM_CLIENT_CAP_UNIVERSAL_PLANES as isize,
    Atomic = ffi::DRM_CLIENT_CAP_ATOMIC as isize
}

/// A trait for all DRM devices.
pub trait Device : AsRawFd {
    /// Generates and returns a magic token unique to the current process.
    ///
    /// This token can be used to authenticate with the DRM Master.
    fn get_auth_token(&self) -> Result<AuthToken> {
        let token = {
            let mut raw: ffi::drm_auth_t = Default::default();
            unsafe {
                ffi::ioctl_get_magic(self.as_raw_fd(), &mut raw)?
            };
            raw.magic
        };

        Ok(AuthToken(token))
    }

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

        unsafe {
            ffi::ioctl_set_client_cap(self.as_raw_fd(), &mut raw)?
        };

        Ok(())
    }

    /// Attempts to acquire the DRM Master lock.
    fn set_master(&self) -> Result<()> {
        unsafe {
            ffi::ioctl_set_master(self.as_raw_fd())?
        };

        Ok(())
    }

    /// Attempts to release the DRM Master lock.
    fn drop_master(&self) -> Result<()> {
        unsafe {
            ffi::ioctl_drop_master(self.as_raw_fd())?
        };

        Ok(())
    }
}

#[allow(non_camel_case_types)]
pub type iPoint = (i32, i32);
#[allow(non_camel_case_types)]
pub type uPoint = (u32, u32);
pub type Dimensions = (u32, u32);
#[allow(non_camel_case_types)]
pub type iRect = (iPoint, Dimensions);
#[allow(non_camel_case_types)]
pub type uRect = (uPoint, Dimensions);
