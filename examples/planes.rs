use drm::control::property::Value;
use std::{collections::HashMap, fs, io};

pub mod utils;
use crate::utils::*;

fn convert_value((type_, value): &(drm::control::property::ValueType, u64)) -> Value<'_> {
    type_.convert_value(*value)
}

// 16.16 fixed point
fn convert_value_fixed((type_, value): &(drm::control::property::ValueType, u64)) -> Option<f64> {
    let val = type_.convert_value(*value).as_unsigned_range()? as u32;
    Some(val as f64 / 0xffff as f64)
}

fn display_plane(
    card: &Card,
    plane_handle: drm::control::plane::Handle,
    plane: drm::control::plane::Info,
) -> io::Result<()> {
    let props = card.get_properties(plane_handle)?;
    let mut prop_map = HashMap::new();
    for (handle, value) in &props {
        let info = card.get_property(*handle)?;
        let name = info.name().to_str().unwrap().to_owned();
        prop_map.insert(name, (info.value_type().clone(), *value));
    }
    let Value::Enum(Some(type_)) = convert_value(&prop_map["type"]) else {
        panic!("failed to convert plane type enum");
    };
    println!("        Type: {}", type_.name().to_str().unwrap());
    println!(
        "        CRTC_XYWH: ({}, {}) {}x{}",
        convert_value(&prop_map["CRTC_X"])
            .as_signed_range()
            .unwrap(),
        convert_value(&prop_map["CRTC_Y"])
            .as_signed_range()
            .unwrap(),
        convert_value(&prop_map["CRTC_W"])
            .as_unsigned_range()
            .unwrap(),
        convert_value(&prop_map["CRTC_H"])
            .as_unsigned_range()
            .unwrap()
    );
    println!(
        "        SRC_XYWH: ({:.1}, {:.1}) {:.1}x{:.1}",
        convert_value_fixed(&prop_map["SRC_X"]).unwrap(),
        convert_value_fixed(&prop_map["SRC_Y"]).unwrap(),
        convert_value_fixed(&prop_map["SRC_W"]).unwrap(),
        convert_value_fixed(&prop_map["SRC_H"]).unwrap()
    );
    if let Some(framebuffer_handle) = plane.framebuffer() {
        println!("        {:?}", framebuffer_handle);
        let framebuffer = card.get_planar_framebuffer(framebuffer_handle).unwrap();
        println!("          Format: {:?}", framebuffer.pixel_format());
        if let Some(modifier) = framebuffer.modifier() {
            println!("          Modifier: {:?}", modifier);
        }
        println!(
            "          Size: {}x{}",
            framebuffer.size().0,
            framebuffer.size().1
        );

        println!("          Planes");
        for i in 0..4 {
            println!(
                "            Plane {} (offset: {}, pitch: {})",
                i,
                framebuffer.offsets()[i],
                framebuffer.pitches()[i]
            );
        }
    }
    Ok(())
}

fn display_card(card: &Card) -> io::Result<()> {
    for &cap in capabilities::CLIENT_CAP_ENUMS {
        card.set_client_capability(cap, true)?;
    }

    let resources = card.resource_handles()?;

    let mut planes = Vec::new();
    for connector_handle in resources.connectors() {
        let connector = card.get_connector(*connector_handle, false)?;
        if connector.state() != drm::control::connector::State::Connected {
            continue;
        }

        println!("  {:?}", connector_handle);
        println!(
            "    {:?}{}",
            connector.interface(),
            connector.interface_id()
        );
        if let Some(encoder_handle) = connector.current_encoder() {
            let encoder = card.get_encoder(encoder_handle)?;
            println!("    {:?}", encoder_handle);
            println!("      Kind: {:?}", encoder.kind());
            if let Some(crtc_handle) = encoder.crtc() {
                println!("    {:?}", crtc_handle);
                let crtc = card.get_crtc(crtc_handle)?;
                if let Some(mode) = crtc.mode() {
                    println!("      {:?}", mode);
                }
                for plane_handle in card.plane_handles()? {
                    let plane = card.get_plane(plane_handle)?;
                    if plane.crtc() != Some(crtc_handle) {
                        continue;
                    }
                    planes.push(plane_handle);
                    println!("      {:?}", plane_handle);
                    display_plane(card, plane_handle, plane)?;
                }
            }
        }
    }

    println!("Planes not associated with connector");
    for plane_handle in card.plane_handles()? {
        if !planes.contains(&plane_handle) {
            let plane = card.get_plane(plane_handle)?;
            println!("      {:?}", plane_handle);
            display_plane(card, plane_handle, plane)?;
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    for i in fs::read_dir("/dev/dri")? {
        let i = i?;
        if i.file_name().to_str().unwrap().starts_with("card") {
            if let Ok(card) = Card::open(i.path().to_str().unwrap()) {
                let driver = card.get_driver()?;
                println!(
                    "{} ({}, {}, version {}.{}.{})",
                    i.path().display(),
                    driver.name.to_str().unwrap(),
                    driver.desc.to_str().unwrap(),
                    driver.version.0,
                    driver.version.1,
                    driver.version.2
                );
                display_card(&card)?;
            }
        }
    }
    Ok(())
}
