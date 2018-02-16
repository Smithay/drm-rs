extern crate drm;

/// Check the `util` module to see how the `Card` structure is implemented.
pub mod util;
use util::*;

pub fn main() {
    let card = Card::open_global();

    let res = card.resource_handles().unwrap();
    println!("{:?}", res.connectors());
    println!("{:?}", res.encoders());
    println!("{:?}", res.crtcs());
    println!("{:?}", res.framebuffers());
}
