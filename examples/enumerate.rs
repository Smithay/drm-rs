pub fn main() {
    let devices = drm::node::devices().unwrap();

    for dev in devices {
        println!("{:?}", dev);
    }
}
