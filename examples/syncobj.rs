/// Check the `util` module to see how the `Card` structure is implemented.
pub mod utils;

use crate::utils::*;
use rustix::event::PollFlags;
use std::{
    io,
    os::unix::io::{AsFd, OwnedFd},
};

impl Card {
    fn simulate_command_submission(&self) -> io::Result<OwnedFd> {
        // Create a temporary syncobj to receive the command fence.
        let syncobj = self.create_syncobj(false)?;

        let sync_file = {
            // Fake a command submission by signalling the syncobj immediately. The kernel
            // attaches a null fence object which is always signalled. Other than this, there
            // isn't a good way to create and signal a fence object from user-mode, so an actual
            // device is required to test this properly.
            //
            // For a real device, the syncobj handle should be passed to a command submission
            // which is expected to set a fence to be signalled upon completion.
            self.syncobj_signal(&[syncobj])?;

            // Export fence set by previous ioctl to file descriptor.
            self.syncobj_to_fd(syncobj, true)
        };

        // The sync file descriptor constitutes ownership of the fence, so the syncobj can be
        // safely destroyed.
        self.destroy_syncobj(syncobj)?;

        sync_file
    }
}

fn main() {
    let card = Card::open_global();
    let sync_file = card.simulate_command_submission().unwrap();
    let fd = sync_file.as_fd();

    // Poll for readability. The DRM fence object will directly wake the thread when signalled.
    //
    // Alternatively, Tokio's AsyncFd may be used like so:
    //
    // use tokio::io::{Interest, unix::AsyncFd};
    // let afd = AsyncFd::with_interest(sync_file, Interest::READABLE).unwrap();
    // let future = async move { afd.readable().await.unwrap().retain_ready() };
    // future.await;
    let mut poll_fds = [rustix::event::PollFd::new(&fd, PollFlags::IN)];
    rustix::event::poll(&mut poll_fds, -1).unwrap();
}
