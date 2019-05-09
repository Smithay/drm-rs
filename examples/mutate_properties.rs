extern crate drm;

/// Check the `util` module to see how the `Card` structure is implemented.
pub mod util;
use util::*;

fn mutate_properties<T: drm::control::ResourceType + std::fmt::Debug>(card: &Card, handle: T) {
    let props = card.get_properties(handle).unwrap();

    let (ids, vals) = props.as_props_and_values();

    for (&id, &val) in ids.iter().zip(vals.iter()) {
        let info = card.get_property(id).unwrap();

        if info.mutable() {
            println!("Found mutable property for resource {:?}: {:?}", handle, info.name());
            println!("\tProperty has type of: {:?}", info.value_type());

            let vtype = info.value_type();
            println!("\tIs set to: {:?}", vtype.convert_value(val));

            match vtype {
                ValueType::Boolean => {
                    println!("\tIs a boolean");
                },
                ValueType::UnsignedRange(min, max) => {
                    println!("\tURange: {} {}", min, max);
                },
                ValueType::SignedRange(min, max) => {
                    println!("\tIRange: {} {}", min, max);
                },
                ValueType::Enum(values) => {
                    let (vals, enums) = values.values();
                    println!("\tEnum with {} possible values", vals.len());
                },
                ValueType::CRTC => {
                    println!("\tCRTC");
                },
                x => println!("\tNot mutating: {:?}", x)
            }
        }
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
        println!("Connector {:?}", handle);
        mutate_properties(&card, handle);
    }

    for &handle in resources.crtcs() {
        println!("CRTC {:?}", handle);
        mutate_properties(&card, handle);
    }

    for &handle in resources.framebuffers() {
        println!("Framebuffer {:?}", handle);
        mutate_properties(&card, handle);
    }

    for &handle in plane_res.planes() {
        println!("Plane {:?}", handle);
        mutate_properties(&card, handle);
    }
}
