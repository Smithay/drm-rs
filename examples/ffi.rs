use drm_ffi as ffi;

use std::fs::{File, OpenOptions};
use std::os::unix::io::{AsFd, BorrowedFd};

#[derive(Debug)]
// This is our customized struct that implements the traits in drm.
struct Card(File);

// Need to implement AsRawFd before we can implement drm::Device
impl AsFd for Card {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
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
}

fn print_busid(fd: BorrowedFd<'_>) {
    let mut buffer = Vec::new();
    let busid = ffi::get_bus_id(fd, Some(&mut buffer));
    println!("{:#?}", busid);
}

fn print_client(fd: BorrowedFd<'_>) {
    let client = ffi::get_client(fd, 0);
    println!("{:#?}", client);
}

fn print_version(fd: BorrowedFd<'_>) {
    let mut name = Vec::new();
    let mut date = Vec::new();
    let mut desc = Vec::new();

    let version = ffi::get_version(fd, Some(&mut name), Some(&mut date), Some(&mut desc));

    println!("{:#?}", version);
}

fn print_capabilities(fd: BorrowedFd<'_>) {
    for cty in 1.. {
        let cap = ffi::get_capability(fd, cty);
        match cap {
            Ok(_) => println!("{:#?}", cap),
            Err(_) => break,
        }
    }
}

fn print_token(fd: BorrowedFd<'_>) {
    let token = ffi::auth::get_magic_token(fd);
    println!("{:#?}", token);
}

fn main() {
    let card = Card::open_global();
    let fd = card.as_fd();

    print_busid(fd);
    print_client(fd);
    print_version(fd);
    print_capabilities(fd);
    print_token(fd);
    //print_stats(fd);
}
