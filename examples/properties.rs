extern crate drm;

/// Check the `util` module to see how the `Card` structure is implemented.
pub mod util;
use util::*;

pub fn main() {
    let card = Card::open_global();

    // Enable all possible client capabilities
    for &cap in util::CLIENT_CAP_ENUMS {
        card.set_client_capability(cap, true);
    }

    let resources = card.resource_handles().unwrap();
    let plane_res = card.plane_handles().unwrap();

    for &handle in resources.connectors() {
        let props = card.get_properties(handle).unwrap();

        println!("Connector: {:?}", handle);
        println!("\t{:?}", props.handles());
        println!("\t{:?}", props.nonbounded_values());
    }
}
