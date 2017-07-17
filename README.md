# drm-rs

This library is a safe interface to the Direct Rendering Manager API found on
various operating systems.

This library is currently a work in progress.

## Usage

The user is expected to implement their own functionality for opening and
accessing the file descriptor of the device. Here we create a small wrapper
around `File` and implement `AsRawFd`, `drm::Device`, and
`drm::control::Device`:

```rust
extern crate drm;

use std::fs::{OpenOptions, File};
use std::os::unix::io::{AsRawFd, RawFd};

use drm::Device as BasicDevice;
use drm::control::Device as ControlDevice;

// The drm crate does not provide a method of opening the device.
// It is expected to be implemented by the user.
struct Card(File);

// Required to implement drm::Device
impl AsRawFd for Card {
    as_raw_fd(&self) -> RawFd { self.0.as_raw_fd() }
}

// Required to implement drm::control::Device
impl BasicDevice for Card { }

// Allows modesetting functionality to be performed.
impl ControlDevice for Card { }

```

Assuming the program used the above wrapper, the user now opens the card:

```rust
// Open the device (usually located at /dev/dri/*) with rw access.
let mut options = OpenOptions::new();
options.read(true);
options.write(true);
let file = options.open("/dev/dri/card0");
let card = Card(file);
```

Now we can check out what resources are available:

```rust
// Get a set of all modesetting resource handles (excluding planes):
let res_handles = card.resource_handles().unwrap();

// Print all connector information
for &con in res_handles.connectors() {
    let info = card.resource_info(con).unwrap();

    println!("{:#?}")
}

// Print all CRTC information
for &crtc in res_handles.crtcs() {
    let info = card.resource_info(crtc).unwrap();

    println!("{:#?}")
}
```

You'll also want to find a suitable mode:

```rust
// Assuming we found a good connector and loaded the info into `connector_info`
let &mode = connector_info.modes().iter(); // Search until you find one you want.
```

Once you find a suitable connector and CRTC, it's time to create a framebuffer.
Here we use a simple dumbbuffer as the backend:'

```rust

// Create a DB of size 1920x1080
let db = dumbbuffer::DumbBuffer::create_from_device(&card, (1920, 1080), 32)
    .expect("Could not create dumb buffer");

// Map it and grey it out.
let mut map = db.map(&card).expect("Could not map dumbbuffer");
for mut b in map.as_mut() {
    *b = 128; // Grey
}

let fb_info = framebuffer::Info::create_from_buffer(&card, &db)
let fb_handle = fb_info.handle();
```

Now we can apply the framebuffer onto the CRTC's internal plane, and connect it
to a connector with the proper mode:

```rust
// Assuming `crtc` is a crtc handle and `con` is a connector handle
crtc.set_on_device(&card, fb_handle, &[con], (0, 0), Some(mode))
    .expect("Could not set Crtc");
```

The contents of the dumb buffer will now appear onto the screen.
