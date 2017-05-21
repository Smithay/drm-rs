use nix;

error_chain! {
    foreign_links {
        Unix(nix::Error);
    }
}
