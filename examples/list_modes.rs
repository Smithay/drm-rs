/// Check the `util` module to see how the `Card` structure is implemented.
pub mod utils;
use crate::utils::*;

pub fn main() {
    let card = Card::open_global();

    let resources = card.resource_handles().unwrap();
    for connector in resources.connectors().iter() {
        let info = card.get_connector(*connector, false).unwrap();
        println!("Connector {:?}: {:?}", info.interface(), info.state());
        if info.state() == drm::control::connector::State::Connected {
            println!("\t Modes:\n{:#?}", info.modes());
        }
    }
}
