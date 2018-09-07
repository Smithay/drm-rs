extern crate drm;

/// Check the `util` module to see how the `Card` structure is implemented.
pub mod util;
use util::*;

pub fn main() {
    let card = Card::open_global();

    println!("Generating AUTH token");
    let token = card.generate_auth_token().unwrap();
    println!("\t{:?}", token);

    println!("Aquiring Master Lock");
    println!("\t{:?}", card.acquire_master_lock());

    println!("Authenticating AUTH token");
    println!("\t{:?}", card.authenticate_auth_token(token));

    println!("Releasing Master Lock");
    println!("\t{:?}", card.release_master_lock());

    println!("Getting Bus ID");
    println!("\t{:?}", card.get_bus_id().unwrap().as_ref());

    println!("Getting driver info");
    let driver = card.get_driver().unwrap();
    println!("\tName: {:?}", driver.name());
    println!("\tDate: {:?}", driver.date());
    println!("\tDesc: {:?}", driver.description());

    println!("Setting client capabilities");
    for &cap in util::CLIENT_CAP_ENUMS {
        println!("\t{:?}: {:?}", cap, card.set_client_capability(cap, true));
    }

    println!("Getting driver capabilities");
    for &cap in util::DRIVER_CAP_ENUMS {
        println!("\t{:?}: {:?}", cap, card.get_driver_capability(cap));
    }
}
