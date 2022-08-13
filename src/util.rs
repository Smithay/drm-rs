//! Utilities used internally by this crate.

use crate::control::{from_u32, RawResourceHandle};

pub unsafe fn transmute_vec<T, U>(from: Vec<T>) -> Vec<U> {
    let mut from = std::mem::ManuallyDrop::new(from);

    Vec::from_raw_parts(from.as_mut_ptr() as *mut U, from.len(), from.capacity())
}

pub unsafe fn transmute_vec_from_u32<T: From<RawResourceHandle>>(raw: Vec<u32>) -> Vec<T> {
    if cfg!(debug_assertions) {
        raw.into_iter()
            .map(|handle| from_u32(handle).unwrap())
            .collect()
    } else {
        transmute_vec(raw)
    }
}
