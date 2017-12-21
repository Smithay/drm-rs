#![feature(drop_types_in_const)]

extern crate drm;
extern crate nix;

use std::fs::{File, OpenOptions};
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::{Mutex, MutexGuard, Once, ONCE_INIT};
use std::time::{Duration, Instant};

use drm::Device as BasicDevice;
use drm::control::Device as ControlDevice;
use drm::control::{ResourceHandle, ResourceInfo};

use drm::control::{connector, crtc, dumbbuffer, encoder, framebuffer, plane};

#[derive(Debug)]
// This is our customized struct that implements the traits in drm.
struct Card(File);

// Need to implement AsRawFd before we can implement drm::Device
impl AsRawFd for Card {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl BasicDevice for Card {}
impl ControlDevice for Card {}

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
    LOCK_INIT.call_once(|| unsafe {
        GLOBAL_LOCK = Some(Mutex::new(()));
    });

    unsafe { GLOBAL_LOCK.as_ref().unwrap().lock().unwrap() }
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
fn vblank_modeset() {
    use std::any::Any;
    use std::sync::atomic::{AtomicBool, Ordering};
    use nix::sys::{select, time};
    use nix::sys::time::TimeValLike;

    let cleanup = AtomicBool::new(false);

    // Can't run with other modesetting tests.
    let _guard = wait_for_lock();

    let card = Card::open_control();

    // Load the information.
    let res = card.resource_handles()
        .expect("Could not load normal resource ids.");
    let coninfo: Vec<connector::Info> = load_information(&card, res.connectors());
    let crtcinfo: Vec<crtc::Info> = load_information(&card, res.crtcs());

    // Filter each connector until we find one that's connected.
    let con = coninfo
        .iter()
        .filter(|&i| i.connection_state() == connector::State::Connected)
        .next()
        .expect("No connected connectors");

    // Get the first (usually best) mode
    let &mode = con.modes()
        .iter()
        .next()
        .expect("No modes found on connector");

    // Find a crtc and FB
    let crtc = crtcinfo.iter().next().expect("No crtcs found");

    // Create a DB
    let db1 = dumbbuffer::DumbBuffer::create_from_device(&card, (1920, 1080), 32)
        .expect("Could not create dumb buffer");
    let db2 = dumbbuffer::DumbBuffer::create_from_device(&card, (1920, 1080), 32)
        .expect("Could not create dumb buffer");

    // Map them and set one white and one black.
    {
        let mut map1 = db1.map(&card).expect("Could not map dumbbuffer");
        for mut b1 in map1.as_mut() {
            *b1 = 255;
        }
        let mut map2 = db2.map(&card).expect("Could not map dumbbuffer");
        for mut b2 in map2.as_mut() {
            *b2 = 0;
        }
    }

    // Create FBs:
    let fb_infos = [
        framebuffer::create(&card, &db1).expect("Could not create FB1"),
        framebuffer::create(&card, &db2).expect("Could not create FB2"),
    ];

    println!("{:#?}", mode);
    println!("{:#?}", fb_infos);
    println!("{:#?}", db1);
    println!("{:#?}", db2);

    // Set the crtc
    crtc::set(
        &card,
        crtc.handle(),
        fb_infos[0].handle(),
        &[con.handle()],
        (0, 0),
        Some(mode),
    ).expect("Could not set CRTC");
    crtc::page_flip(
        &card,
        crtc.handle(),
        fb_infos[1].handle(),
        &[crtc::PageFlipFlags::PageFlipEvent],
        None::<Box<()>>,
    ).expect("Failed to queue Page Flip");

    struct PageFlipHandler<'a> {
        index: usize,
        cleanup: &'a AtomicBool,
        crtc: crtc::Handle,
        fb_infos: [framebuffer::Info; 2],
    }

    impl<'a, T: ControlDevice> crtc::PageFlipHandler<T> for PageFlipHandler<'a> {
        fn handle_event(&mut self, device: &T, _: u32, _: Duration, userdata: Box<Any>) {
            if !self.cleanup.load(Ordering::Acquire) {
                crtc::page_flip(
                    device,
                    self.crtc,
                    self.fb_infos[self.index].handle(),
                    &[crtc::PageFlipFlags::PageFlipEvent],
                    userdata,
                ).expect("Failed to queue Page Flip");
                self.index = (self.index + 1) % 2;
            }
        }
    }

    let mut handler = PageFlipHandler {
        index: 0,
        cleanup: &cleanup,
        crtc: crtc.handle(),
        fb_infos: fb_infos,
    };

    let start = Instant::now();
    while Instant::now().duration_since(start) < Duration::new(5, 0) {
        let mut readfds = select::FdSet::new();
        readfds.insert(card.as_raw_fd());
        match select::select(
            card.as_raw_fd() + 1,
            Some(&mut readfds),
            None,
            None,
            Some(&mut time::TimeVal::seconds(5)),
        ) {
            Ok(1) => crtc::handle_event(
                &card,
                2,
                None::<&mut ()>,
                Some(&mut handler),
                None::<&mut ()>,
            ).expect("Unable to handle event"),
            Ok(0) => break,
            Ok(_) => unreachable!(),
            Err(_) => break,
        }
    }

    cleanup.store(true, Ordering::Release);
    crtc::handle_event(
        &card,
        2,
        None::<&mut ()>,
        Some(&mut handler),
        None::<&mut ()>,
    ).expect("Unable to handle event");

    let mut fbs = fb_infos.to_vec();
    for fb in fbs.drain(..) {
        framebuffer::destroy(&card, fb.handle()).unwrap();
    }

    db1.destroy(&card).unwrap();
    db2.destroy(&card).unwrap();
}

#[test]
fn gamma_test() {
    // Can't run with other modesetting tests.
    let guard = wait_for_lock();

    let card = Card::open_control();

    // Load the information.
    let res = card.resource_handles()
        .expect("Could not load normal resource ids.");
    let coninfo: Vec<connector::Info> = load_information(&card, res.connectors());
    let crtcinfo: Vec<crtc::Info> = load_information(&card, res.crtcs());

    // Filter each connector until we find one that's connected.
    let con = coninfo
        .iter()
        .filter(|&i| i.connection_state() == connector::State::Connected)
        .next()
        .expect("No connected connectors");

    // Get the first (usually best) mode
    let &mode = con.modes()
        .iter()
        .next()
        .expect("No modes found on connector");

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
    let fbinfo = framebuffer::create(&card, &db).expect("Could not create FB");

    println!("{:#?}", mode);
    println!("{:#?}", fbinfo);
    println!("{:#?}", db);

    // Set the crtc
    crtc::set(
        &card,
        crtc.handle(),
        fbinfo.handle(),
        &[con.handle()],
        (0, 0),
        Some(mode),
    ).expect("Could not set CRTC");

    let five_seconds = ::std::time::Duration::from_millis(5000);
    ::std::thread::sleep(five_seconds);

    let gamma = crtc::gamma(&card, crtc.handle()).unwrap();
    println!("{:?}", gamma.red);
    crtc::set_gamma(
        &card,
        crtc.handle(),
        crtc::GammaRamp {
            red: gamma
                .red
                .iter()
                .map(|_| ::std::u16::MAX)
                .collect::<Vec<u16>>()
                .into_boxed_slice(),
            green: gamma.green.clone(),
            blue: gamma.blue.clone(),
        },
    ).unwrap();

    ::std::thread::sleep(five_seconds);
    crtc::set_gamma(&card, crtc.handle(), gamma).unwrap();
}
