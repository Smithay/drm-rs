//!
//! # DumbBuffer
//!
//! Memory-supported, slow, but easy & cross-platform buffer implementation
//!

use crate::buffer;

use std::borrow::{Borrow, BorrowMut};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Slow, but generic [`buffer::Buffer`] implementation
pub struct DumbBuffer {
    pub(crate) size: (u32, u32),
    pub(crate) length: usize,
    pub(crate) format: buffer::DrmFourcc,
    pub(crate) pitch: u32,
    pub(crate) handle: buffer::Handle,
}

/// Mapping of a [`DumbBuffer`]
pub struct DumbMapping<'a> {
    pub(crate) _phantom: core::marker::PhantomData<&'a ()>,
    pub(crate) map: &'a mut [u8],
}

impl AsRef<[u8]> for DumbMapping<'_> {
    fn as_ref(&self) -> &[u8] {
        self.map
    }
}

impl AsMut<[u8]> for DumbMapping<'_> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.map
    }
}

impl Borrow<[u8]> for DumbMapping<'_> {
    fn borrow(&self) -> &[u8] {
        self.map
    }
}

impl BorrowMut<[u8]> for DumbMapping<'_> {
    fn borrow_mut(&mut self) -> &mut [u8] {
        self.map
    }
}

impl Deref for DumbMapping<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.map
    }
}

impl DerefMut for DumbMapping<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.map
    }
}

impl<'a> Drop for DumbMapping<'a> {
    fn drop(&mut self) {
        unsafe {
            rustix::mm::munmap(self.map.as_mut_ptr() as *mut _, self.map.len())
                .expect("Unmap failed");
        }
    }
}

impl buffer::Buffer for DumbBuffer {
    fn size(&self) -> (u32, u32) {
        self.size
    }
    fn format(&self) -> buffer::DrmFourcc {
        self.format
    }
    fn pitch(&self) -> u32 {
        self.pitch
    }
    fn handle(&self) -> buffer::Handle {
        self.handle
    }
}
