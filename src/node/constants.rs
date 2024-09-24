//! OS-Specific DRM constants.

/// DRM major value.
#[cfg(target_os = "dragonfly")]
pub const DRM_MAJOR: u32 = 145;

/// DRM major value.
#[cfg(target_os = "netbsd")]
pub const DRM_MAJOR: u32 = 34;

/// DRM major value.
#[cfg(all(target_os = "openbsd", target_arch = "x86"))]
pub const DRM_MAJOR: u32 = 88;

/// DRM major value.
#[cfg(all(target_os = "openbsd", not(target_arch = "x86")))]
pub const DRM_MAJOR: u32 = 87;

/// DRM major value.
#[cfg(not(any(target_os = "dragonfly", target_os = "netbsd", target_os = "openbsd")))]
pub const DRM_MAJOR: u32 = 226;

/// Primary DRM node prefix.
#[cfg(not(target_os = "openbsd"))]
pub const PRIMARY_NAME: &str = "card";

/// Primary DRM node prefix.
#[cfg(target_os = "openbsd")]
pub const PRIMARY_NAME: &str = "drm";

/// Control DRM node prefix.
#[cfg(not(target_os = "openbsd"))]
pub const CONTROL_NAME: &str = "controlD";

/// Control DRM node prefix.
#[cfg(target_os = "openbsd")]
pub const CONTROL_NAME: &str = "drmC";

/// Render DRM node prefix.
#[cfg(not(target_os = "openbsd"))]
pub const RENDER_NAME: &str = "renderD";

/// Render DRM node prefix.
#[cfg(target_os = "openbsd")]
pub const RENDER_NAME: &str = "drmR";
