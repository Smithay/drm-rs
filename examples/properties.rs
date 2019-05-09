extern crate drm;

/// Check the `util` module to see how the `Card` structure is implemented.
pub mod util;
use util::*;

fn print_properties<T: drm::control::ResourceType>(card: &Card, handle: T) {
    let props = card.get_properties(handle).unwrap();

    let (ids, vals) = props.as_props_and_values();

    for (&id, &val) in ids.iter().zip(vals.iter()) {
        println!("Property: {:?}", id);
        let info = card.get_property(id).unwrap();
        println!("{:?}", info.name());
        println!("{:#?}", info.value_type());
        println!("Mutable: {}", info.mutable());
        println!("Atomic: {}", info.atomic());
        println!("Value: {:?}", info.value_type().convert_value(val));
        println!("");
    }
}

pub fn main() {
    let card = Card::open_global();

    // Enable all possible client capabilities
    for &cap in util::CLIENT_CAP_ENUMS {
        card.set_client_capability(cap, true);
    }

    let resources = card.resource_handles().unwrap();
    let plane_res = card.plane_handles().unwrap();

    for &handle in resources.connectors() {
        print_properties(&card, handle);
    }

    for &handle in resources.framebuffers() {
        print_properties(&card, handle);
    }

    for &handle in resources.crtcs() {
        print_properties(&card, handle);
    }

    for &handle in plane_res.planes() {
        print_properties(&card, handle);
    }
}
