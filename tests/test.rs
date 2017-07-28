#![feature(drop_types_in_const)]

extern crate drm;

use std::fs::{OpenOptions, File};
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::{Mutex, MutexGuard, Once, ONCE_INIT};

use drm::Device as BasicDevice;
use drm::control::Device as ControlDevice;
use drm::control::{ResourceInfo, ResourceHandle};

use drm::control::{connector, encoder, crtc, framebuffer, plane, dumbbuffer};

#[derive(Debug)]
// This is our customized struct that implements the traits in drm.
struct Card(File);

// Need to implement AsRawFd before we can implement drm::Device
impl AsRawFd for Card {
    fn as_raw_fd(&self) -> RawFd { self.0.as_raw_fd() }
}

impl BasicDevice for Card { }
impl ControlDevice for Card { }

impl Card {
    fn open(path: &str) -> Self {
        let mut options = OpenOptions::new();
        options.read(true);
        options.write(true);
        Card(options.open(path).unwrap())
    }

    fn open_global() -> Self {
        Self::open("/dev/dri/card0")
    }

    fn open_control() -> Self {
        Self::open("/dev/dri/controlD64")
    }
}

// Some tests cannot be done in parallel. We will use this lock in threads that
// require single access.
static mut GLOBAL_LOCK: Option<Mutex<()>> = None;
static LOCK_INIT: Once = ONCE_INIT;

// Call this function at the beginning of the test if it cannot run parallel.
fn wait_for_lock() -> MutexGuard<'static, ()> {
    LOCK_INIT.call_once(|| {
        unsafe {
            GLOBAL_LOCK = Some(Mutex::new(()));
        }
    });

    unsafe {
        GLOBAL_LOCK.as_ref().unwrap().lock().unwrap()
    }
}

fn load_information<T, U>(card: &Card, handles: &[T]) -> Vec<U>
    where T: ResourceHandle, U: ResourceInfo<Handle=T> {

    handles.iter().map(| &h | {
        card.resource_info(h).expect("Could not load resource info")
    }).collect()
}

#[test]
fn unprivileged_global() {
    let card = Card::open_global();

    // AuthToken
    card.get_auth_token().expect("Could not get AuthToken");

    // Client capabilities
    card.set_client_cap(drm::ClientCapability::Stereo3D, true)
        .expect("Could not enable Stereo3D capability");
    card.set_client_cap(drm::ClientCapability::UniversalPlanes, true)
        .expect("Could not enable UniversalPlanes capability");
    card.set_client_cap(drm::ClientCapability::Atomic, true)
        .expect("Could not enable Atomic capability");
}

#[test]
fn load_resources() {
    let card = Card::open_control();

    // Load the resource ids
    let res = card.resource_handles().expect("Could not load handles.");
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

#[test]
fn legacy_modeset() {
    // Can't run with other modesetting tests.
    let guard = wait_for_lock();

    let card = Card::open_control();

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

    // Create a DB
    let db = dumbbuffer::DumbBuffer::create_from_device(&card, (1920, 1080), 32)
        .expect("Could not create dumb buffer");

    // Map it and grey it out.
    let mut map = db.map(&card).expect("Could not map dumbbuffer");
    for mut b in map.as_mut() {
        *b = 128;
    }

    // Create an FB:
    let fbinfo = framebuffer::create(&card, &db)
        .expect("Could not create FB");

    println!("{:#?}", mode);
    println!("{:#?}", fbinfo);
    println!("{:#?}", db);

    // Set the crtc
    crtc::set(&card, crtc.handle(), fbinfo.handle(), &[con.handle()], (0, 0), Some(mode))
        .expect("Could not set CRTC");

    let five_seconds = ::std::time::Duration::from_millis(5000);
    ::std::thread::sleep(five_seconds);
}
