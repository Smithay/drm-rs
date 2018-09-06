//!
//! Foreign function interface
//!

#![allow(dead_code)]
#![allow(missing_docs)]

pub mod basic;
pub mod gem;
pub mod ioctl;
pub mod mode;

pub use drm_sys::*;

/// Allow this crate to easily shrink slices
pub(crate) trait ShrinkableSlice {
    fn shrink(&mut self, size: usize);
}

impl<'a, T> ShrinkableSlice for &'a mut [T] {
    #[inline]
    fn shrink(&mut self, size: usize) {
        use std::cmp;
        use std::mem;

        let shrink_to = cmp::min(size, self.len());
        let mut new_slice = mem::replace(self, &mut []);
        new_slice = &mut {new_slice}[..shrink_to];
        mem::replace(self, new_slice);
    }
}

