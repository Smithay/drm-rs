#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(any(target_os = "android", target_os = "linux"))]
mod platform {
    pub use linux_raw_sys::general::__kernel_size_t;
    pub type drm_handle_t = core::ffi::c_uint;
    pub const DRM_RDWR: u32 = linux_raw_sys::general::O_RDWR;
    pub const DRM_CLOEXEC: u32 = linux_raw_sys::general::O_CLOEXEC;
}

#[cfg(not(any(target_os = "android", target_os = "linux")))]
mod platform {
    pub type __kernel_size_t = libc::size_t;
    pub type drm_handle_t = core::ffi::c_ulong;
    pub const DRM_RDWR: u32 = libc::O_RDWR as u32;
    pub const DRM_CLOEXEC: u32 = libc::O_CLOEXEC as u32;
}

pub use platform::*;

#[cfg(feature = "use_bindgen")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(not(feature = "use_bindgen"))]
include!("bindings.rs");

pub const DRM_PLANE_TYPE_OVERLAY: u32 = 0;
pub const DRM_PLANE_TYPE_PRIMARY: u32 = 1;
pub const DRM_PLANE_TYPE_CURSOR: u32 = 2;
