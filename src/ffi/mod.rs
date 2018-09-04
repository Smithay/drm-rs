//!
//! Foreign function interface
//!

#![allow(dead_code)]
#![allow(missing_docs)]

pub mod basic;
pub mod gem;
pub mod ioctl;
pub mod mode;

/// Will coerce a buffer to be smaller if necessary.
pub(crate) fn minimize_slice<T>(slice: &mut &[T], size: usize) {
    use std::cmp;
    let min = cmp::min(size, slice.len());
    *slice = &slice[..min];
}
