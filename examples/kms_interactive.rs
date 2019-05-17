#![feature(slice_patterns)]

extern crate drm;

/// Check the `util` module to see how the `Card` structure is implemented.
pub mod utils;
use utils::*;

pub fn main() {
    let card = Card::open_global();

    // Enable all possible client capabilities
    for &cap in capabilities::CLIENT_CAP_ENUMS {
        card.set_client_capability(cap, true);
    }

    run_repl(&card);
}

fn run_repl(card: &Card) {
    use std::io::{self, BufRead};

    let stdin = io::stdin();
    for line in stdin.lock().lines().map(|x| x.unwrap()) {
        println!("{}", line);
        let args: Vec<_> = line.split_whitespace().collect();

        match &args[..] {
            ["CreateFramebuffer", width, height, r, g, b] => {
                let width: u32 = str::parse(width).unwrap();
                let height: u32 = str::parse(height).unwrap();
                let r: u8 = str::parse(r).unwrap();
                let g: u8 = str::parse(g).unwrap();
                let b: u8 = str::parse(b).unwrap();

                let fmt = drm::buffer::format::PixelFormat::ARGB8888;

                let mut db = card.create_dumb_buffer((width, height), fmt).unwrap();

                {
                    let mut mapping = card.map_dumb_buffer(&mut db).unwrap();
                    let mut buffer = mapping.as_mut();
                    for byte in buffer.chunks_exact_mut(4) {
                        // Assuming little endian, it's BGRA
                        byte[0] = b;
                        byte[1] = g;
                        byte[2] = r;
                        byte[3] = 255;
                    }
                };

                println!("{:?}", db);

                let fb = card.add_framebuffer(&db).unwrap();
                println!("{:?}", fb);
            },
            ["GetResources"] => {
                let resources = card.resource_handles().unwrap();
                println!("\tConnectors: {:?}", resources.connectors());
                println!("\tEncoders: {:?}", resources.encoders());
                println!("\tCRTCS: {:?}", resources.crtcs());
                println!("\tFramebuffers: {:?}", resources.framebuffers());
                let planes = card.plane_handles().unwrap();
                println!("\tPlanes: {:?}", planes.planes());
            },
            ["GetProperties", "Connector", handle] => {
                let handle: u32 = str::parse(handle).unwrap();
                let handle: drm::control::connector::Handle = unsafe {
                    std::mem::transmute(handle)
                };
                let props = card.get_properties(handle).unwrap();
                let (ids, vals) = props.as_props_and_values();

                for (id, val) in ids.iter().zip(vals.iter()) {
                    println!("\tProperty: {:?}\tValue: {:?}", id, val);
                }
            },
            ["GetProperties", "CRTC", handle] => {
                let handle: u32 = str::parse(handle).unwrap();
                let handle: drm::control::crtc::Handle = unsafe {
                    std::mem::transmute(handle)
                };
                let props = card.get_properties(handle).unwrap();
                let (ids, vals) = props.as_props_and_values();

                for (id, val) in ids.iter().zip(vals.iter()) {
                    println!("\tProperty: {:?}\tValue: {:?}", id, val);
                }
            },
            ["GetProperties", "Plane", handle] => {
                let handle: u32 = str::parse(handle).unwrap();
                let handle: drm::control::plane::Handle = unsafe {
                    std::mem::transmute(handle)
                };
                let props = card.get_properties(handle).unwrap();
                let (ids, vals) = props.as_props_and_values();

                for (id, val) in ids.iter().zip(vals.iter()) {
                    println!("\tProperty: {:?}\tValue: {:?}", id, val);
                }
            },
            ["GetProperty", handle] => {
                let handle: u32 = str::parse(handle).unwrap();
                let handle: drm::control::property::Handle = unsafe {
                    std::mem::transmute(handle)
                };
                let property = card.get_property(handle).unwrap();
                println!("\tName: {:?}", property.name());
                println!("\tMutable: {:?}", property.mutable());
                println!("\tAtomic: {:?}", property.atomic());
                println!("\tValue: {:#?}", property.value_type());
            },
            ["SetProperty", "Connector", handle, property, value] => {
                let handle: u32 = str::parse(handle).unwrap();
                let handle: drm::control::connector::Handle = unsafe {
                    std::mem::transmute(handle)
                };
                let property: u32 = str::parse(property).unwrap();
                let property: drm::control::property::Handle = unsafe {
                    std::mem::transmute(property)
                };
                let value: u64 = str::parse(value).unwrap();
                println!("\t{:?}", card.set_property(handle, property, value));
            },
            ["SetProperty", "CRTC", handle, property, value] => {
                let handle: u32 = str::parse(handle).unwrap();
                let handle: drm::control::crtc::Handle = unsafe {
                    std::mem::transmute(handle)
                };
                let property: u32 = str::parse(property).unwrap();
                let property: drm::control::property::Handle = unsafe {
                    std::mem::transmute(property)
                };
                let value: u64 = str::parse(value).unwrap();
                println!("\t{:?}", card.set_property(handle, property, value));
            },
            ["SetProperty", "Plane", handle, property, value] => {
                let handle: u32 = str::parse(handle).unwrap();
                let handle: drm::control::plane::Handle = unsafe {
                    std::mem::transmute(handle)
                };
                let property: u32 = str::parse(property).unwrap();
                let property: drm::control::property::Handle = unsafe {
                    std::mem::transmute(property)
                };
                let value: u64 = str::parse(value).unwrap();
                println!("\t{:?}", card.set_property(handle, property, value));
            },
            [ ] => (),
            _ => {
                println!("Unknown command");
            }
        }
    }
}

