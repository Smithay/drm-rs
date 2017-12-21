extern crate drm;

use drm::Device as BasicDevice;
use drm::control::Device as ControlDevice;

use drm::control::ResourceInfo;
use drm::control::ResourceHandle;
use drm::control::{connector, crtc, dumbbuffer, encoder, framebuffer, plane};

use std::fs::File;
use std::fs::OpenOptions;

use std::os::unix::io::RawFd;
use std::os::unix::io::AsRawFd;

#[derive(Debug)]
pub struct Card(File);

impl AsRawFd for Card {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl BasicDevice for Card {}
impl ControlDevice for Card {}

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

pub fn main() {
    let card = Card::open_global();

    let res = card.resource_handles().expect("Can't get resource handles");
    let pres = card.plane_handles().expect("Could not load plane handles");

    let coninfo: Vec<connector::Info> = load_information(&card, res.connectors());
    let encinfo: Vec<encoder::Info> = load_information(&card, res.encoders());
    let crtcinfo: Vec<crtc::Info> = load_information(&card, res.crtcs());
    let fbinfo: Vec<framebuffer::Info> = load_information(&card, res.framebuffers());
    let plinfo: Vec<plane::Info> = load_information(&card, pres.planes());

    println!("{:#?}", coninfo);
    println!("{:#?}", encinfo);
    println!("{:#?}", crtcinfo);
    println!("{:#?}", fbinfo);
    println!("{:#?}", plinfo);
}

fn load_information<T, U>(card: &Card, handles: &[T]) -> Vec<U>
where
    T: ResourceHandle,
    U: ResourceInfo<Handle = T>,
{
    handles
        .iter()
        .map(|&h| card.resource_info(h).expect("Could not load resource info"))
        .collect()
}
