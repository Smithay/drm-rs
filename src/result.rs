use nix;

error_chain! {
    foreign_links {
        Unix(nix::Error);
    }

    errors {
        UnsupportedPixelFormat {
            description("PixelFormat is unsupported by operation")
            display("The provided PixelFormat is not supported by the operation")
        }
    }
}
