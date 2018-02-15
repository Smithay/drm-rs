extern crate drm;
pub use drm::Device as BasicDevice;

use std::fs::File;
use std::fs::OpenOptions;

pub use std::os::unix::io::RawFd;
pub use std::os::unix::io::AsRawFd;

#[derive(Debug)]
pub struct Card(File);

impl AsRawFd for Card {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl BasicDevice for Card {}

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
