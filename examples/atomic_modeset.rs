extern crate drm;
extern crate image;

mod utils;
use utils::*;

use drm::control::Device as ControlDevice;
use drm::Device as BasicDevice;

use drm::buffer::DrmFourcc;

use drm::control::ResourceHandle;
use drm::control::{self, atomic, connector, crtc, property, AtomicCommitFlags};

fn find_prop_id<T: ResourceHandle>(
    card: &Card,
    handle: T,
    name: &'static str,
) -> Option<property::Handle> {
    let props = card
        .get_properties(handle)
        .expect("Could not get props of connector");
    let (ids, _vals) = props.as_props_and_values();
    ids.iter()
        .find(|&id| {
            let info = card.get_property(*id).unwrap();
            info.name().to_str().map(|x| x == name).unwrap_or(false)
        })
        .map(|x| *x)
}

pub fn main() {
    let card = Card::open_global();

    card.set_client_capability(drm::ClientCapability::UniversalPlanes, true)
        .expect("Unable to request UniversalPlanes capability");
    card.set_client_capability(drm::ClientCapability::Atomic, true)
        .expect("Unable to request Atomic capability");

    // Load the information.
    let res = card
        .resource_handles()
        .expect("Could not load normal resource ids.");
    let coninfo: Vec<connector::Info> = res
        .connectors()
        .iter()
        .flat_map(|con| card.get_connector(*con))
        .collect();
    let crtcinfo: Vec<crtc::Info> = res
        .crtcs()
        .iter()
        .flat_map(|crtc| card.get_crtc(*crtc))
        .collect();

    // Filter each connector until we find one that's connected.
    let con = coninfo
        .iter()
        .filter(|&i| i.state() == connector::State::Connected)
        .next()
        .expect("No connected connectors");

    // Get the first (usually best) mode
    let &mode = con
        .modes()
        .iter()
        .next()
        .expect("No modes found on connector");

    let (disp_width, disp_height) = mode.size();

    // Find a crtc and FB
    let crtc = crtcinfo.iter().next().expect("No crtcs found");

    // Select the pixel format
    let fmt = DrmFourcc::Rgba8888;

    // Create a DB
    // If buffer resolution is above display resolution, a ENOSPC (not enough GPU memory) error may
    // occur
    let mut db = card
        .create_dumb_buffer((disp_width.into(), disp_height.into()), fmt, 32)
        .expect("Could not create dumb buffer");

    // Map it and grey it out.
    {
        let mut map = card
            .map_dumb_buffer(&mut db)
            .expect("Could not map dumbbuffer");
        for b in map.as_mut() {
            *b = 128;
        }
    }

    // Create an FB:
    let fb = card
        .add_framebuffer(&db, 1, 32)
        .expect("Could not create FB");

    let planes = card.plane_handles().expect("Could not list planes");
    let (better_planes, compatible_planes): (
        Vec<control::plane::Handle>,
        Vec<control::plane::Handle>,
    ) = planes
        .planes()
        .iter()
        .map(|x| *x)
        .filter(|plane| {
            card.get_plane(*plane)
                .map(|plane_info| {
                    let compatible_crtcs = res.filter_crtcs(plane_info.possible_crtcs());
                    compatible_crtcs.contains(&crtc.handle())
                })
                .unwrap_or(false)
        })
        .partition(|plane| {
            if let Ok(props) = card.get_properties(*plane) {
                let (ids, vals) = props.as_props_and_values();
                for (&id, &val) in ids.iter().zip(vals.iter()) {
                    if let Ok(info) = card.get_property(id) {
                        if info.name().to_str().map(|x| x == "type").unwrap_or(false) {
                            return val == (drm::control::PlaneType::Primary as u32).into();
                        }
                    }
                }
            }
            false
        });
    let plane = *better_planes.get(0).unwrap_or(&compatible_planes[0]);

    println!("{:#?}", mode);
    println!("{:#?}", fb);
    println!("{:#?}", db);
    println!("{:#?}", plane);

    let mut atomic_req = atomic::AtomicModeReq::new();
    atomic_req.add_property(
        con.handle(),
        find_prop_id(&card, con.handle(), "CRTC_ID").expect("Could not get CRTC_ID"),
        property::Value::CRTC(Some(crtc.handle())),
    );
    let blob = card
        .create_property_blob(mode)
        .expect("Failed to create blob");
    atomic_req.add_property(
        crtc.handle(),
        find_prop_id(&card, crtc.handle(), "MODE_ID").expect("Could not get MODE_ID"),
        blob,
    );
    atomic_req.add_property(
        crtc.handle(),
        find_prop_id(&card, crtc.handle(), "ACTIVE").expect("Could not get ACTIVE"),
        property::Value::Boolean(true),
    );
    atomic_req.add_property(
        plane,
        find_prop_id(&card, plane, "FB_ID").expect("Could not get FB_ID"),
        property::Value::Framebuffer(Some(fb)),
    );
    atomic_req.add_property(
        plane,
        find_prop_id(&card, plane, "CRTC_ID").expect("Could not get CRTC_ID"),
        property::Value::CRTC(Some(crtc.handle())),
    );
    atomic_req.add_property(
        plane,
        find_prop_id(&card, plane, "SRC_X").expect("Could not get SRC_X"),
        property::Value::UnsignedRange(0),
    );
    atomic_req.add_property(
        plane,
        find_prop_id(&card, plane, "SRC_Y").expect("Could not get SRC_Y"),
        property::Value::UnsignedRange(0),
    );
    atomic_req.add_property(
        plane,
        find_prop_id(&card, plane, "SRC_W").expect("Could not get SRC_W"),
        property::Value::UnsignedRange((mode.size().0 as u64) << 16),
    );
    atomic_req.add_property(
        plane,
        find_prop_id(&card, plane, "SRC_H").expect("Could not get SRC_H"),
        property::Value::UnsignedRange((mode.size().1 as u64) << 16),
    );
    atomic_req.add_property(
        plane,
        find_prop_id(&card, plane, "CRTC_X").expect("Could not get CRTC_X"),
        property::Value::SignedRange(0),
    );
    atomic_req.add_property(
        plane,
        find_prop_id(&card, plane, "CRTC_Y").expect("Could not get CRTC_Y"),
        property::Value::SignedRange(0),
    );
    atomic_req.add_property(
        plane,
        find_prop_id(&card, plane, "CRTC_W").expect("Could not get CRTC_W"),
        property::Value::UnsignedRange(mode.size().0 as u64),
    );
    atomic_req.add_property(
        plane,
        find_prop_id(&card, plane, "CRTC_H").expect("Could not get CRTC_H"),
        property::Value::UnsignedRange(mode.size().1 as u64),
    );

    // Set the crtc
    // On many setups, this requires root access.
    card.atomic_commit(&[AtomicCommitFlags::AllowModeset], atomic_req)
        .expect("Failed to set mode");

    let five_seconds = ::std::time::Duration::from_millis(5000);
    ::std::thread::sleep(five_seconds);

    card.destroy_framebuffer(fb).unwrap();
    card.destroy_dumb_buffer(db).unwrap();
}
