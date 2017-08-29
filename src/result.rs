use std::io;
use nix;

error_chain! {
    foreign_links {
        Unix(nix::Error);
        Io(io::Error);
    }

    errors {
        InvalidGammaSize(set: usize, size: u32) {
            description("Invalid Gamma Ramp Size")
            display("Invalid Gamma Ramp Size: '{}', expected: '{}'", set, size)
        }

        UnsupportedPixelFormat {
            description("PixelFormat is unsupported by operation")
            display("The provided PixelFormat is not supported by the operation")
        }
    }
}
