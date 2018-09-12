extern crate drm;

/// Check the `util` module to see how the `Card` structure is implemented.
pub mod util;
use util::*;

pub fn main() {
    let card = Card::open_global();

    let resources = card.resource_handles().unwrap();
    let plane_res = card.plane_handles().unwrap();

    // Print out all card resource handles
    println!("Connectors:\t{:?}", resources.connectors());
    println!("Encoders:\t{:?}", resources.encoders());
    println!("CRTCs:\t\t{:?}", resources.crtcs());
    println!("Framebuffers:\t{:?}", resources.framebuffers());
    println!("Planes:\t\t{:?}", plane_res.planes());

    for &handle in resources.connectors() {
        let info = card.get_connector(handle).unwrap();
        println!("Connector: {:?}", handle);
        println!("\t{:?}-{}", info.kind(), info.kind_id());
        println!("\t{:?}", info.state());
        println!("\t{:?}", info.size());
        println!("\t{:?}", info.current_encoder());
        println!("\t{:?}", info.encoders());

        for &mode in info.modes() {
            println!("\tMode:");
            println!("\t\tClock: {:?}", mode.clock());
            println!("\t\tSize: {:?}", mode.size());
            println!("\t\tHSync: {:?}", mode.hsync());
            println!("\t\tVSync: {:?}", mode.vsync());
            println!("\t\tHSkew: {:?}", mode.hskew());
            println!("\t\tVScan: {:?}", mode.vscan());
            println!("\t\tVRef: {:?}", mode.vrefresh());
        }
    }

    for &handle in resources.encoders() {
        let info = card.get_encoder(handle).unwrap();
        println!("Encoder: {:?}", handle);
        println!("\t{:?}", info.kind());
        println!("\t{:?}", info.crtc());
    }

    for &handle in resources.crtcs() {
        let info = card.get_crtc(handle).unwrap();
        println!("CRTC: {:?}", handle);
        println!("\tPosition: {:?}", info.position());
        println!("\tMode: {:?}", info.mode());
        println!("\tFramebuffer: {:?}", info.framebuffer());
        println!("\tGamma Length: {:?}", info.gamma_length());
    }

    for &handle in resources.framebuffers() {
        println!("{:#?}", card.get_framebuffer(handle));
    }

    for &handle in plane_res.planes() {
        println!("{:#?}", card.get_plane(handle));
    }
}
