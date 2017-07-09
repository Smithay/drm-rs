#![feature(drop_types_in_const)]

extern crate drm;

use std::fs::{OpenOptions, File};
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::{Mutex, MutexGuard, Once, ONCE_INIT};

use drm::Device as BasicDevice;
use drm::control::Device as ControlDevice;

use drm::control::{connector, encoder, crtc, framebuffer, plane, property};

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
    fn open() -> Self {
        let mut options = OpenOptions::new();
        options.read(true);
        options.write(true);
        Card(options.open("/dev/dri/card0").unwrap())
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


#[test]
fn unprivileged_global() {
    let card = Card::open();

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
    let card = Card::open();

    // Load the resource ids
    let res = card.resource_ids().expect("Could not load normal resource ids.");
    let pres = card.plane_ids().expect("Could not load plane ids");

    let cons: Vec<_> = res.connectors().iter().map(| &id | {
        card.connector_info(id).expect("Could not load connector info")
    }).collect();

    let encs: Vec<_> = res.encoders().iter().map(| &id | {
        card.encoder_info(id).expect("Could not load encoder info")
    }).collect();

    let crtcs: Vec<_> = res.crtcs().iter().map(| &id | {
        card.crtc_info(id).expect("Could not load crtc info")
    }).collect();

    let fbs: Vec<_> = res.framebuffers().iter().map(| &id | {
        card.fb_info(id).expect("Could not load fbs info")
    }).collect();

    let planes: Vec<_> = pres.planes().iter().map(| &id | {
        card.plane_info(id).expect("Could not load plane info")
    }).collect();

    println!("{:#?}", cons);
    println!("{:#?}", encs);
    println!("{:#?}", crtcs);
    println!("{:#?}", fbs);
    println!("{:#?}", planes);
}
