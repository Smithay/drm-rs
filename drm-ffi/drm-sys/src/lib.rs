#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate libc;

#[cfg(feature = "use_bindgen")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(all(not(feature = "use_bindgen"), target_os = "linux", target_arch = "x86"))]
include!(concat!("platforms/linux/x86/bindings.rs"));

#[cfg(all(
    not(feature = "use_bindgen"),
    target_os = "linux",
    target_arch = "x86_64"
))]
include!(concat!("platforms/linux/x86_64/bindings.rs"));

#[cfg(all(not(feature = "use_bindgen"), target_os = "linux", target_arch = "arm"))]
include!(concat!("platforms/linux/arm/bindings.rs"));

#[cfg(all(
    not(feature = "use_bindgen"),
    target_os = "linux",
    target_arch = "aarch64"
))]
include!(concat!("platforms/linux/aarch64/bindings.rs"));

#[cfg(all(
    not(feature = "use_bindgen"),
    target_os = "freebsd",
    target_arch = "x86_64"
))]
include!(concat!("platforms/freebsd/x86_64/bindings.rs"));

pub const DRM_PLANE_TYPE_OVERLAY: u32 = 0;
pub const DRM_PLANE_TYPE_PRIMARY: u32 = 1;
pub const DRM_PLANE_TYPE_CURSOR: u32 = 2;
