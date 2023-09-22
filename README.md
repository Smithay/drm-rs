# drm-rs

[![Crates.io](https://img.shields.io/crates/v/drm.svg)](https://crates.io/crates/drm)
[![docs.rs](https://docs.rs/drm/badge.svg)](https://docs.rs/drm)
[![Build Status](https://github.com/Smithay/drm-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Smithay/drm-rs/actions/workflows/ci.yml)

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
device through the [AsFd](https://doc.rust-lang.org/std/os/fd/trait.AsFd.html)
trait. Here is a basic example using `File` as a backend:

```rust
/// A simple wrapper for a device node.
pub struct Card(std::fs::File);

/// Implementing [`AsFd`] is a prerequisite to implementing the traits found
/// in this crate. Here, we are just calling [`File::as_fd()`] on the inner
/// [`File`].
impl AsFd for Card {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
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

See [`drm::control::Device`](https://docs.rs/drm/*/drm/control/trait.Device.html)
as well as our mode-setting examples: [`atomic_modeset`](https://github.com/Smithay/drm-rs/blob/develop/examples/atomic_modeset.rs)
and [`legacy_modeset`](https://github.com/Smithay/drm-rs/blob/develop/examples/legacy_modeset.rs)

### Rendering

Rendering is done by [creating](https://docs.rs/drm/*/drm/control/trait.Device.html#method.add_framebuffer) and
[attaching](https://docs.rs/drm/*/drm/control/trait.Device.html#method.page_flip) [framebuffers](https://docs.rs/drm/*/drm/control/framebuffer/index.html)
to [crtcs](https://docs.rs/drm/*/drm/control/crtc/index.html).

A framebuffer is created from anything implementing [`Buffer`](https://docs.rs/drm/*/drm/buffer/trait.Buffer.html) like the always
available, but very limited, [`DumbBuffer`](https://docs.rs/drm/*/drm/control/dumbbuffer/struct.DumbBuffer.html).

For faster hardware-backed buffers, checkout [gbm.rs](https://github.com/Smithay/gbm.rs).
