#![feature(drop_types_in_const)]

extern crate drm;

use std::fs::{OpenOptions, File};
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::{Mutex, MutexGuard, Once, ONCE_INIT};

use drm::Device as BasicDevice;
use drm::control::Device as ControlDevice;
use drm::control::property::LoadProperties;
use drm::control::ResourceInfo;

use drm::control::{connector, encoder, crtc, framebuffer, plane, property, dumbbuffer};

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

// Get the handle of a valid Connector
fn get_connected(card: &Card) -> drm::control::connector::Id {
    let res = card.resource_ids().expect("Could not load normal resource ids.");

    // Find a connected connector.
    let &con = res.connectors().iter().filter(| &&c | {
        let info = card.connector_info(c).expect("Could not get connector info");
        info.connection_state() == drm::control::connector::State::Connected
    }).next().expect("No connectors available");

    con
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
    let pres = card.plane_ids().expect("Could not load plane handles");

    for &con in res.connectors() {
        let info = card.resource_info(con)
            .expect("Could not get connector info");
        println!("{:#?}", info);
    }

    for &enc in res.encoders() {
        let info = card.resource_info(enc)
            .expect("Could not get encoder info");
        println!("{:#?}", info);
    }

    for &crtc in res.crtcs() {
        let info = card.resource_info(crtc)
            .expect("Could not get crtc info");
        println!("{:#?}", info);
    }

    for &fb in res.framebuffers() {
        let info = card.resource_info(fb)
            .expect("Could not get framebuffer info");
        println!("{:#?}", info);
    }
}

#[test]
fn legacy_modeset() {
    // Can't run with other modesetting tests.
    let guard = wait_for_lock();

    let card = Card::open_control();
    let res = card.resource_ids().expect("Could not load normal resource ids.");

    // Find a connector that's connected
    let con = get_connected(&card);
    let coninfo = card.connector_info(con).expect("Could not load connector info");

    // Get the first (usually best) mode
    let &mode = coninfo.modes().iter().next().expect("No modes found on connector");

    // Find a crtc and FB
    let &crtc = res.crtcs().iter().next().expect("No crtcs available");

    // Create a DB
    let db = dumbbuffer::DumbBuffer::create_from_device(&card, (1920, 1080), 32)
        .expect("Could not create dumb buffer");

    // Map it and grey it out.
    let mut map = db.map(&card).expect("Could not map dumbbuffer");
    for mut b in map.as_mut() {
        *b = 128;
    }

    // Create an FB:
    let fbinfo = framebuffer::Info::create_from_buffer(&card, &db)
        .expect("Could not create FB");

    println!("{:#?}", mode);
    println!("{:#?}", fbinfo);
    println!("{:#?}", db);

    // Set the crtc
    crtc.set_on_device(&card, fbinfo.handle(), &[con], (0, 0), Some(mode))
        .expect("Could not set Crtc");

    ::std::thread::sleep_ms(5000);
}
