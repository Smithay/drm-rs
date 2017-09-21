//!
//! Error types
//!

use std::io;
use nix;

error_chain! {
    foreign_links {
        Unix(nix::Error) #[doc = "Unix error"];
        Io(io::Error) #[doc = "I/O error"];
    }

    errors {
        #[doc = "Size of the given gamma ramp does not match the device"]
        InvalidGammaSize(set: usize, size: u32) {
            description("Invalid Gamma Ramp Size")
            display("Invalid Gamma Ramp Size: '{}', expected: '{}'", set, size)
        }

        #[doc = "Pixel format is not supported by the operation/device"]
        UnsupportedPixelFormat {
            description("PixelFormat is unsupported by operation")
            display("The provided PixelFormat is not supported by the operation")
        }
    }
}
