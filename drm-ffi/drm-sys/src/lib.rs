#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(any(target_os = "android", target_os = "linux"))]
pub use linux_raw_sys::general::__kernel_size_t;
#[cfg(any(target_os = "android", target_os = "linux"))]
pub type drm_handle_t = core::ffi::c_uint;

#[cfg(not(any(target_os = "android", target_os = "linux")))]
type __kernel_size_t = libc::size_t;
#[cfg(not(any(target_os = "android", target_os = "linux")))]
pub type drm_handle_t = core::ffi::c_ulong;

#[cfg(feature = "use_bindgen")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(not(feature = "use_bindgen"))]
include!("bindings.rs");

pub const DRM_PLANE_TYPE_OVERLAY: u32 = 0;
pub const DRM_PLANE_TYPE_PRIMARY: u32 = 1;
pub const DRM_PLANE_TYPE_CURSOR: u32 = 2;
