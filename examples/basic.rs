extern crate drm;

/// Check the `util` module to see how the `Card` structure is implemented.
pub mod util;
use util::*;

use drm::ClientCapability;

pub fn main() {
    let card = Card::open_global();

    println!("Generating AuthToken:");
    let res = card.generate_auth_token();
    println!("\t{:?}", res);

    println!("Requesting Stero3D functionality:");
    let res = card.toggle_capability(ClientCapability::Stereo3D, true);
    println!("\t{:?}", res);

    println!("Requesting UniversalPlanes functionality:");
    let res = card.toggle_capability(ClientCapability::UniversalPlanes, true);
    println!("\t{:?}", res);

    println!("Requesting Atomic functionality:");
    let res = card.toggle_capability(ClientCapability::Atomic, true);
    println!("\t{:?}", res);

    println!("Attempting to acquire DRM Master Lock:");
    let res = card.acquire_master_lock();
    println!("\t{:?}", res);

    println!("Attempting to release DRM Master Lock:");
    let res = card.release_master_lock();
    println!("\t{:?}", res);
}
