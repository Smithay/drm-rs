//! Utilities used internally by this crate.

#![allow(dead_code)]
#![allow(missing_docs)]

pub unsafe fn transmute_vec<T, U>(from: Vec<T>) -> Vec<U> {
    let mut from = std::mem::ManuallyDrop::new(from);

    Vec::from_raw_parts(from.as_mut_ptr() as *mut U, from.len(), from.capacity())
}
