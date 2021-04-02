/// Takes an `Option<&mut &mut [T]>` style buffer and gets its pointer.
macro_rules! map_ptr {
    ($buffer:expr) => {
        match $buffer {
            Some(b) => b.as_ptr() as _,
            None => 0 as _,
        }
    };
}

/// Takes an `Option<&mut &mut T>` style buffer and gets its length.
macro_rules! map_len {
    ($buffer:expr) => {
        match $buffer {
            Some(b) => b.len() as _,
            None => 0 as _,
        }
    };
}

/// Takes an `Option<&mut &mut T>` style buffer and shrinks it.
macro_rules! map_shrink {
    ($buffer:expr, $min:expr) => {
        match $buffer {
            Some(b) => utils::shrink(b, $min),
            _ => (),
        }
    };
}

/// Takes a `&mut &mut [T]` and shrinks the slice down to a specific size.
pub fn shrink<T>(slice_ref: &mut &mut [T], min: usize) {
    use std::mem::replace;
    use std::slice::from_raw_parts_mut;

    if min < slice_ref.len() {
        let ptr = slice_ref.as_mut_ptr();
        unsafe {
            let _ = replace(slice_ref, from_raw_parts_mut(ptr, min));
        }
    }
}
