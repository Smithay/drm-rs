extern crate drm;

/// Check the `util` module to see how the `Card` structure is implemented.
pub mod util;
use util::*;

pub fn main() {
    let card = Card::open_global();

    let res = card.resource_handles().unwrap();
    println!("{:#?}", res);

    for &i in res.connectors() {
        let info = card.info(i).unwrap();
        println!("{:#?}", card.info(i).unwrap());

        /*
        for &j in info.property_handles() {
            println!("{:#?}", card.info(j).unwrap());
        }
        */
    }

    for &i in res.encoders() {
        println!("{:#?}", card.info(i).unwrap());
    }

    for &i in res.crtcs() {
        println!("{:#?}", card.info(i).unwrap());
    }

    for &i in res.framebuffers() {
        println!("{:#?}", card.info(i).unwrap());
    }

    let res = card.plane_handles().unwrap();
    println!("{:#?}", res);

    for &i in res.planes() {
        println!("{:#?}", card.info(i).unwrap());
    }
}
