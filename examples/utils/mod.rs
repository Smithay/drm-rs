#![allow(dead_code)]

pub use drm::control::Device as ControlDevice;
pub use drm::Device;

#[derive(Debug)]
/// A simple wrapper for a device node.
pub struct Card(std::fs::File);

/// Implementing `AsFd` is a prerequisite to implementing the traits found
/// in this crate. Here, we are just calling `as_fd()` on the inner File.
impl std::os::unix::io::AsFd for Card {
    fn as_fd(&self) -> std::os::unix::io::BorrowedFd<'_> {
        self.0.as_fd()
    }
}

/// With `AsFd` implemented, we can now implement `drm::Device`.
impl Device for Card {}
impl ControlDevice for Card {}

/// Simple helper methods for opening a `Card`.
impl Card {
    pub fn open(path: &str) -> Self {
        let mut options = std::fs::OpenOptions::new();
        options.read(true);
        options.write(true);
        Card(options.open(path).unwrap())
    }

    pub fn open_global() -> Self {
        Self::open("/dev/dri/card0")
    }
}

pub mod capabilities {
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
        DC::TimelineSyncObj,
    ];
}

pub mod images {
    use image;

    pub fn load_image(name: &str) -> image::RgbaImage {
        let path = format!("examples/images/{}", name);
        image::open(path).unwrap().to_rgba8()
    }
}
