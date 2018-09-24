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
        println!("\t{:?}-{}", info.interface(), info.interface_id());
        println!("\t{:?}", info.state());
        println!("\t{:?}", info.size());
        println!("\t{:?}", info.encoders());
        println!("\t{:?}", info.current_encoder());

        for &mode in card.get_modes(handle).unwrap().as_slice() {
            //println!("{:?}", mode);
        }
    }
    println!("\n");

    for &handle in resources.encoders() {
        let info = card.get_encoder(handle).unwrap();
        println!("Encoder: {:?}", handle);
        println!("\t{:?}", info.kind());
        println!("\t{:?}", info.crtc());
    }
    println!("\n");

    for &handle in resources.crtcs() {
        let info = card.get_crtc(handle).unwrap();
        println!("CRTC: {:?}", handle);
        println!("\tPosition: {:?}", info.position());
        println!("\tMode: {:?}", info.mode());
        println!("\tFramebuffer: {:?}", info.framebuffer());
        println!("\tGamma Length: {:?}", info.gamma_length());
    }
    println!("\n");

    for &handle in resources.framebuffers() {
        let info = card.get_framebuffer(handle).unwrap();
        println!("Framebuffer: {:?}", handle);
        println!("\tSize: {:?}", info.size());
        println!("\tPitch: {:?}", info.pitch());
        println!("\tBPP: {:?}", info.bpp());
        println!("\tDepth: {:?}", info.depth());
    }

    println!("\n");

    for &handle in plane_res.planes() {
        let info = card.get_plane(handle).unwrap();
        println!("Plane: {:?}", handle);
        println!("\tCRTC: {:?}", info.crtc());
        println!("\tFramebuffer: {:?}", info.framebuffer());
        println!("\tFormats: {:?}", info.formats());
    }
}
