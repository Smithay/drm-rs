extern crate drm_ffi;

use drm_ffi as ffi;

use std::fs::{File, OpenOptions};
use std::os::unix::io::{AsRawFd, RawFd};

#[derive(Debug)]
// This is our customized struct that implements the traits in drm.
struct Card(File);

// Need to implement AsRawFd before we can implement drm::Device
impl AsRawFd for Card {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl Card {
    fn open(path: &str) -> Self {
        let mut options = OpenOptions::new();
        options.read(true);
        options.write(true);
        Card(options.open(path).unwrap())
    }

    fn open_global() -> Self {
        Self::open("/dev/dri/card0")
    }

    fn open_control() -> Self {
        Self::open("/dev/dri/controlD64")
    }
}

fn print_busid(fd: RawFd) {
    let mut buffer = [0u8; 32];
    let mut slice = &mut buffer[..];
    let busid = ffi::get_bus_id(fd, Some(&mut slice));
    println!("{:#?}", busid);
}

fn print_client(fd: RawFd) {
    let client = ffi::get_client(fd, 0);
    println!("{:#?}", client);
}

fn print_version(fd: RawFd) {
    let mut name = [0i8; 32];
    let mut date = [0i8; 32];
    let mut desc = [0i8; 32];

    let mut name_slice = &mut name[..];
    let mut date_slice = &mut date[..];
    let mut desc_slice = &mut desc[..];

    let version = ffi::get_version(
        fd,
        Some(&mut name_slice),
        Some(&mut date_slice),
        Some(&mut desc_slice),
    );

    println!("{:#?}", version);
}

fn print_capabilities(fd: RawFd) {
    for cty in 1.. {
        let cap = ffi::get_capability(fd, cty);
        match cap {
            Ok(_) => println!("{:#?}", cap),
            Err(_) => break,
        }
    }
}

fn print_token(fd: RawFd) {
    let token = ffi::auth::get_magic_token(fd);
    println!("{:#?}", token);
}

/*
fn print_stats(fd: RawFd) {
    let stats = ffi::basic::get_stats(fd);
    println!("{:#?}", stats);
}
*/

fn main() {
    let card = Card::open_global();
    let fd = card.as_raw_fd();

    print_busid(fd);
    print_client(fd);
    print_version(fd);
    print_capabilities(fd);
    print_token(fd);
    //print_stats(fd);
}
