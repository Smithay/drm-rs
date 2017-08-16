use std::io;
use nix;

error_chain! {
    foreign_links {
        Unix(nix::Error);
        Io(io::Error);
    }
}
