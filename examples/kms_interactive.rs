/// Check the `util` module to see how the `Card` structure is implemented.
pub mod utils;
use crate::utils::*;
use drm::control::{from_u32, RawResourceHandle};

pub fn main() {
    let card = Card::open_global();

    // Enable all possible client capabilities
    for &cap in capabilities::CLIENT_CAP_ENUMS {
        if let Err(e) = card.set_client_capability(cap, true) {
            eprintln!("Unable to activate capability {:?}: {}", cap, e);
            return;
        }
    }

    run_repl(&card);
}

fn run_repl(card: &Card) {
    // Load a set of numbered images
    let images = [
        images::load_image("1.png"),
        images::load_image("2.png"),
        images::load_image("3.png"),
        images::load_image("4.png"),
    ];

    for image in &images {
        // Create the Dumbbuffer
        let fmt = drm::buffer::DrmFourcc::Xrgb8888;
        let mut db = card
            .create_dumb_buffer(image.dimensions(), fmt, 32)
            .unwrap();

        // Create a Framebuffer to represent it
        let _fb = card.add_framebuffer(&db, 24, 32).unwrap();

        // Load the image into the buffer
        {
            let mut mapping = card.map_dumb_buffer(&mut db).unwrap();
            let buffer = mapping.as_mut();
            for (img_px, map_px) in image.pixels().zip(buffer.chunks_exact_mut(4)) {
                // Assuming little endian, it's BGRA
                map_px[0] = img_px[0]; // Blue
                map_px[1] = img_px[1]; // Green
                map_px[2] = img_px[2]; // Red
                map_px[3] = img_px[3]; // Alpha
            }
        };
    }

    // Using rustyline to create the interactive prompt.
    let editor_config = rustyline::config::Builder::new()
        .max_history_size(256)
        .unwrap()
        .completion_type(rustyline::config::CompletionType::List)
        .edit_mode(rustyline::config::EditMode::Vi)
        .auto_add_history(true)
        .build();
    let mut kms_editor = rustyline::Editor::<(), _>::with_config(editor_config).unwrap();
    let mut atomic_editor = rustyline::Editor::<(), _>::with_config(editor_config).unwrap();

    for line in kms_editor.iter("KMS>> ").map(|x| x.unwrap()) {
        let args: Vec<_> = line.split_whitespace().collect();
        match &args[..] {
            ["CreateAtomicSet"] => {
                for line in atomic_editor.iter("Atomic>> ").map(|x| x.unwrap()) {
                    let args: Vec<_> = line.split_whitespace().collect();
                    match &args[..] {
                        ["Quit"] => break,
                        args => println!("{:?}", args),
                    }
                }
            }
            // Destroying a framebuffer
            ["DestroyFramebuffer", handle] => {
                let handle: u32 = str::parse(handle).unwrap();
                let handle: drm::control::framebuffer::Handle = from_u32(handle).unwrap();
                if let Err(err) = card.destroy_framebuffer(handle) {
                    println!("Unable to destroy framebuffer ({:?}): {}", handle, err);
                }
            }
            // Print out all resources
            ["GetResources"] => {
                let resources = card.resource_handles().unwrap();
                println!("\tConnectors: {:?}", resources.connectors());
                println!("\tEncoders: {:?}", resources.encoders());
                println!("\tCRTCS: {:?}", resources.crtcs());
                println!("\tFramebuffers: {:?}", resources.framebuffers());
                let planes = card.plane_handles().unwrap();
                println!("\tPlanes: {:?}", planes);
            }
            // Print out the values of a specific property
            ["GetProperty", handle] => {
                let handle: u32 = str::parse(handle).unwrap();
                let handle: drm::control::property::Handle = from_u32(handle).unwrap();
                let property = card.get_property(handle).unwrap();
                println!("\tName: {:?}", property.name());
                println!("\tMutable: {:?}", property.mutable());
                println!("\tAtomic: {:?}", property.atomic());
                println!("\tValue: {:#?}", property.value_type());
            }
            // Get the property-value pairs of a single resource
            ["GetProperties", handle] => match HandleWithProperties::from_str(card, handle) {
                Ok(handle) => {
                    let props = match handle {
                        HandleWithProperties::Connector(handle) => {
                            card.get_properties(handle).unwrap()
                        }
                        HandleWithProperties::CRTC(handle) => card.get_properties(handle).unwrap(),
                        HandleWithProperties::Plane(handle) => card.get_properties(handle).unwrap(),
                    };
                    for (id, val) in props.iter() {
                        println!("\tProperty: {:?}\tValue: {:?}", id, val);
                    }
                }
                Err(_) => println!("Unknown handle or handle has no properties"),
            },
            // Set a property's value on a resource
            ["SetProperty", handle, property, value] => {
                let property: u32 = str::parse(property).unwrap();
                let property: drm::control::property::Handle = from_u32(property).unwrap();
                let value: u64 = str::parse(value).unwrap();

                match HandleWithProperties::from_str(card, handle) {
                    Ok(handle) => {
                        match handle {
                            HandleWithProperties::Connector(handle) => {
                                println!("\t{:?}", card.set_property(handle, property, value));
                            }
                            HandleWithProperties::CRTC(handle) => {
                                println!("\t{:?}", card.set_property(handle, property, value));
                            }
                            HandleWithProperties::Plane(handle) => {
                                println!("\t{:?}", card.set_property(handle, property, value));
                            }
                        };
                    }
                    Err(_) => println!("Unknown handle or handle has no properties"),
                };
            }
            ["GetModes", handle] => match HandleWithProperties::from_str(card, handle) {
                Ok(HandleWithProperties::Connector(handle)) => {
                    let modes = card.get_modes(handle).unwrap();
                    for mode in modes {
                        println!("\tName:\t{:?}", mode.name());
                        println!("\t\tSize:\t{:?}", mode.size());
                        println!("\t\tRefresh:\t{:?}", mode.vrefresh());
                    }
                }
                _ => println!("Unknown handle or handle is not a connector"),
            },
            ["help"] => {
                println!("CreateAtomicSet");
                println!("DestroyFramebuffer <handle>");
                println!("GetResources");
                println!("GetProperty <handle>");
                println!("GetProperties <handle>");
                println!("SetProperty <handle> <poperty> <value>");
                println!("GetModes <handle>");
            }
            ["quit"] => break,
            [] => (),
            _ => {
                println!("Unknown command");
            }
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
enum HandleWithProperties {
    Connector(drm::control::connector::Handle),
    CRTC(drm::control::crtc::Handle),
    Plane(drm::control::plane::Handle),
}

impl HandleWithProperties {
    // This is a helper command that will take a string of a number and lookup
    // the corresponding resource.
    fn from_str(card: &Card, handle: &str) -> Result<Self, ()> {
        let handle: u32 = str::parse(handle).unwrap();
        let handle = RawResourceHandle::new(handle).unwrap();

        let rhandles = card.resource_handles().unwrap();
        for connector in rhandles.connectors().iter().map(|h| (*h).into()) {
            if handle == connector {
                return Ok(HandleWithProperties::Connector(handle.into()));
            }
        }

        for crtc in rhandles.crtcs().iter().map(|h| (*h).into()) {
            if handle == crtc {
                return Ok(HandleWithProperties::CRTC(handle.into()));
            }
        }

        let phandles = card.plane_handles().unwrap();
        for plane in phandles.iter().map(|h| (*h).into()) {
            if handle == plane {
                return Ok(HandleWithProperties::Plane(handle.into()));
            }
        }

        Err(())
    }
}
