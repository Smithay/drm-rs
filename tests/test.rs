#![feature(drop_types_in_const)]

extern crate drm;

use std::fs::{OpenOptions, File};
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::{Mutex, MutexGuard, Once, ONCE_INIT};

use drm::Device;

#[derive(Debug)]
// This is our customized struct that implements the traits in drm.
struct Card(File);

// Need to implement AsRawFd before we can implement drm::Device
impl AsRawFd for Card {
    fn as_raw_fd(&self) -> RawFd { self.0.as_raw_fd() }
}

impl drm::Device for Card { }

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
    // Open a DRM device.
    let mut options = OpenOptions::new();
    options.read(true);
    options.write(true);
    let card = Card(options.open("/dev/dri/card0").unwrap());

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
