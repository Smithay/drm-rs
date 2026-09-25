/// Check the `util` module to see how the `Card` structure is implemented.
pub mod utils;
use drm::control::property::{Value, ValueType};
use crate::utils::*;

fn print_pipeline(card: &Card, stage: Value) {
    let stage = match stage.as_colorop() {
        Some(s) => s,
        None => return,
    };
    let props = card.get_properties(stage).unwrap();

    println!("Colorop: {stage:?}",);

    let mut next = None;
    for (&id, &val) in props.iter() {
        let info = card.get_property(id).unwrap();
        let value_type = info.value_type();
        let name = match info.name().to_str() {
            Ok(name) => name,
            Err(_) => continue,
        };

        println!("Property: {id:?}");
        println!("{}", name);
        println!("{:#?}", &value_type);
        println!("Mutable: {}", info.mutable());
        println!("Value: {:?}", value_type.convert_value(val));
        println!();

        if name == "NEXT" {
            if let Value::Colorop(handle) = value_type.convert_value(val) {
                next = handle;
            }
        }
    }

    print_pipeline(card, Value::Colorop(next));
}

fn print_plane_pipelines(card: &Card, handle: drm::control::plane::Handle) {
    let props = card.get_properties(handle).unwrap();

    for (&id, &val) in props.iter() {
        let info = card.get_property(id).unwrap();
        if !info.name().to_str().map(|x| x == "COLOR_PIPELINE").unwrap_or(false) {
            continue;
        }

        println!("Plane: {handle:?}",);

        let value_type = info.value_type();
        println!("Property: {id:?}");
        println!("COLOR_PIPELINE");
        println!("{:#?}", value_type);
        println!("Mutable: {}", info.mutable());
        println!("Value: {:?}", value_type.convert_value(val));
        println!();

        if let ValueType::Enum(values) = value_type {
            for handle in values.values().0 {
                let handle = ValueType::Colorop.convert_value(*handle);
                print_pipeline(card, handle);
            }
        }
    }
}

pub fn main() {
    let card = Card::open_global();

    // Enable all possible client capabilities
    for &cap in capabilities::CLIENT_CAP_ENUMS {
        if let Err(e) = card.set_client_capability(cap, true) {
            eprintln!("Unable to activate capability {cap:?}: {e}");
            return;
        }
    }

    let plane_res = card.plane_handles().unwrap();

    for handle in plane_res {
        print_plane_pipelines(&card, handle);
    }
}
