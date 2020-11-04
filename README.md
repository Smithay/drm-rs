# drm-rs

__This library is currently a work in progress.__

__A nightly compiler is required__

A safe interface to the Direct Rendering Manager.

## Direct Rendering Manager

The Direct Rendering Manager is a subsystem found on multiple Unix-based
operating systems that provides a userspace API to graphics hardware.
See the [Wikipedia article](https://en.wikipedia.org/wiki/Direct_Rendering_Manager)
for more details.

## Usage

### Basic

The DRM is accessed using [ioctls](https://en.wikipedia.org/wiki/Ioctl)
on a file representing a graphics card. These can normally be
found in `/dev/dri`, but can also be opened in other ways (ex. udev).

This crate does not provide a method of opening these files. Instead, the
user program must provide a way to access the file descriptor representing the
device through the [AsRawFd](https://doc.rust-lang.org/std/os/unix/io/trait.AsRawFd.html)
trait. Here is a basic example using `File` as a backend:

```rust
/// A simple wrapper for a device node.
pub struct Card(std::fs::File);

/// Implementing `AsRawFd` is a prerequisite to implementing the traits found
/// in this crate. Here, we are just calling `as_raw_fd()` on the inner File.
impl std::os::unix::io::AsRawFd for Card {
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        self.0.as_raw_fd()
    }
}

/// Simple helper methods for opening a `Card`.
impl Card {
    pub fn open(path: &str) -> Self {
        let mut options = std::fs::OpenOptions::new();
        options.read(true);
        options.write(true);
        Card(options.open(path).unwrap())
    }
}
```

Finally, you can implement `drm::Device` to gain access to the basic DRM
functionality:

```rust
impl drm::Device for Card {}

fn main() {
    let gpu = Card::open("/dev/dri/card0");
    println!("{:#?}", gpu.get_driver().unwrap());
}
```

### Control (modesetting)

**WIP** - See `drm::control::Device`

### Rendering

**WIP**
