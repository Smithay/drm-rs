extern crate drm;
extern crate image;

mod utils;
use utils::*;

use drm::control::Device as ControlDevice;
use drm::Device as BasicDevice;

use drm::buffer::DrmFourcc;

use drm::control::ResourceHandle;
use drm::control::{connector, crtc, dumbbuffer, framebuffer};

pub fn main() {
    let card = Card::open_global();

    // Load the information.
    let res = card
        .resource_handles()
        .expect("Could not load normal resource ids.");
    let coninfo: Vec<connector::Info> = res.connectors().iter().flat_map(|con| card.get_connector(*con)).collect();
    let crtcinfo: Vec<crtc::Info> = res.crtcs().iter().flat_map(|crtc| card.get_crtc(*crtc)).collect();

    // Filter each connector until we find one that's connected.
    let con = coninfo
        .iter()
        .filter(|&i| i.state() == connector::State::Connected)
        .next()
        .expect("No connected connectors");

    // Get the first (usually best) mode
    let &mode = con
        .modes()
        .iter()
        .next()
        .expect("No modes found on connector");

	let (disp_width, disp_height) = mode.size();
	
    // Find a crtc and FB
    let crtc = crtcinfo.iter().next().expect("No crtcs found");

    // Select the pixel format
    let fmt = DrmFourcc::Rgba8888;

    // Create a DB
    // If buffer resolution is larger than display resolution, an ENOSPC (not enough video memory)
    // error may occur
    let mut db = card.create_dumb_buffer((disp_width.into(), disp_height.into()), fmt, 32)
        .expect("Could not create dumb buffer");

    // Map it and grey it out.
    {
        let mut map = card.map_dumb_buffer(&mut db).expect("Could not map dumbbuffer");
        for mut b in map.as_mut() {
            *b = 128;
        }
    }

    // Create an FB:
    let fb = card.add_framebuffer(&db, 1, 32).expect("Could not create FB");

    println!("{:#?}", mode);
    println!("{:#?}", fb);
    println!("{:#?}", db);

    // Set the crtc
    // On many setups, this requires root access.
    card.set_crtc(
        crtc.handle(),
        Some(fb),
        (0, 0),
        &[con.handle()],
        Some(mode),
    )
    .expect("Could not set CRTC");

    let five_seconds = ::std::time::Duration::from_millis(5000);
    ::std::thread::sleep(five_seconds);

    card.destroy_framebuffer(fb).unwrap();
    card.destroy_dumb_buffer(db).unwrap();
}
