pub use drm::control::Device as ControlDevice;
pub use drm::Device;

use std::fs::File;
use std::fs::OpenOptions;

pub use std::os::unix::io::AsRawFd;
pub use std::os::unix::io::RawFd;

use drm::ClientCapability as CC;
pub const CLIENT_CAP_ENUMS: &[CC] = &[CC::Stereo3D, CC::UniversalPlanes, CC::Atomic];

use drm::DriverCapability as DC;
pub const DRIVER_CAP_ENUMS: &[DC] = &[
    DC::DumbBuffer,
    DC::VBlankHighCRTC,
    DC::DumbPreferredDepth,
    DC::DumbPreferShadow,
    DC::Prime,
    DC::MonotonicTimestamp,
    DC::ASyncPageFlip,
    DC::CursorWidth,
    DC::CursorHeight,
    DC::AddFB2Modifiers,
    DC::PageFlipTarget,
    DC::CRTCInVBlankEvent,
    DC::SyncObj,
];

#[derive(Debug)]
/// A simple wrapper for a device node.
pub struct Card(File);

/// Implementing `AsRawFd` is a prerequisite to implementing the traits found
/// in this crate. Here, we are just calling `as_raw_fd()` on the inner File.
impl AsRawFd for Card {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

/// With `AsRawFd` implemented, we can now implement `drm::Device`.
impl Device for Card {}
impl ControlDevice for Card {}

/// Simple helper methods for opening a `Card`.
impl Card {
    pub fn open(path: &str) -> Self {
        let mut options = OpenOptions::new();
        options.read(true);
        options.write(true);
        Card(options.open(path).unwrap())
    }

    pub fn open_global() -> Self {
        Self::open("/dev/dri/card0")
    }

    pub fn open_control() -> Self {
        Self::open("/dev/dri/controlD64")
    }
}
