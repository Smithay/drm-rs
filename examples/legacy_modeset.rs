extern crate drm;

use drm::Device as BasicDevice;
use drm::control::Device as ControlDevice;

use drm::buffer::PixelFormat;

use drm::control::ResourceInfo;
use drm::control::ResourceHandle;
use drm::control::{connector, crtc, framebuffer, dumbbuffer};

use std::fs::File;
use std::fs::OpenOptions;

use std::os::unix::io::RawFd;
use std::os::unix::io::AsRawFd;

#[derive(Debug)]
pub struct Card(File);

impl AsRawFd for Card {
    fn as_raw_fd(&self) -> RawFd { self.0.as_raw_fd() }
}

impl BasicDevice for Card { }
impl ControlDevice for Card { }

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

pub fn main()
{
    let card = Card::open_global();

    // Load the information.
    let res = card.resource_handles().expect("Could not load normal resource ids.");
    let coninfo: Vec<connector::Info> = load_information(&card, res.connectors());
    let crtcinfo: Vec<crtc::Info> = load_information(&card, res.crtcs());

    // Filter each connector until we find one that's connected.
    let con = coninfo.iter().filter(| &i | {
        i.connection_state() == connector::State::Connected
    }).next().expect("No connected connectors");

    // Get the first (usually best) mode
    let &mode = con.modes().iter().next().expect("No modes found on connector");

    // Find a crtc and FB
    let crtc = crtcinfo.iter().next().expect("No crtcs found");

    // Select the pixel format
    let fmt = PixelFormat::XRGB8888;
    //let fmt = PixelFormat::RGBA8888;
    //let fmt = PixelFormat::ARGB4444;

    // Create a DB
    let mut db = dumbbuffer::DumbBuffer::create_from_device(&card, (1920, 1080), fmt)
        .expect("Could not create dumb buffer");

    // Map it and grey it out.
    {
        let mut map = db.map(&card).expect("Could not map dumbbuffer");
        for mut b in map.as_mut() {
            *b = 128;
        }
    }

    // Create an FB:
    let fbinfo = framebuffer::create(&card, &db)
        .expect("Could not create FB");

    println!("{:#?}", mode);
    println!("{:#?}", fbinfo);
    println!("{:#?}", db);

    // Set the crtc
    // On many setups, this requires root access.
    crtc::set(&card, crtc.handle(), fbinfo.handle(), &[con.handle()], (0, 0), Some(mode))
        .expect("Could not set CRTC");

    let five_seconds = ::std::time::Duration::from_millis(5000);
    ::std::thread::sleep(five_seconds);

    framebuffer::destroy(&card, fbinfo.handle()).unwrap();
    db.destroy(&card).unwrap();
}

fn load_information<T, U>(card: &Card, handles: &[T]) -> Vec<U>
    where T: ResourceHandle, U: ResourceInfo<Handle=T> {

    handles.iter().map(| &h | {
        card.resource_info(h).expect("Could not load resource info")
    }).collect()
}
