/// Check the `util` module to see how the `Card` structure is implemented.
pub mod utils;
use crate::utils::*;

pub fn main() {
    let card = Card::open_global();

    // Attempt to acquire and release master lock
    println!("Get Master lock: {:?}", card.acquire_master_lock());
    println!("Release Master lock: {:?}", card.release_master_lock());

    // Get the Bus ID of the device
    println!("Getting Bus ID: {:?}", card.get_bus_id().unwrap());

    // Figure out driver in use
    println!("Getting driver info");
    let driver = card.get_driver().unwrap();
    println!("\tName: {:?}", driver.name());
    println!("\tDate: {:?}", driver.date());
    println!("\tDesc: {:?}", driver.description());

    // Enable all possible client capabilities
    println!("Setting client capabilities");
    for &cap in capabilities::CLIENT_CAP_ENUMS {
        println!("\t{:?}: {:?}", cap, card.set_client_capability(cap, true));
    }

    // Get driver capabilities
    println!("Getting driver capabilities");
    for &cap in capabilities::DRIVER_CAP_ENUMS {
        println!("\t{:?}: {:?}", cap, card.get_driver_capability(cap));
    }
}
