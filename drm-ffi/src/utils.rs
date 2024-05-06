/// Takes an `Option<&mut Vec<T>>` style buffer and gets its pointer.
macro_rules! map_ptr {
    ($buffer:expr) => {
        match $buffer {
            Some(b) => b.as_ptr() as _,
            None => 0 as _,
        }
    };
}

/// Takes an `Option<&mut Vec<T>>` style buffer and gets its allocated length.
macro_rules! map_len {
    ($buffer:expr) => {
        match $buffer {
            Some(b) => b.capacity() as _,
            None => 0,
        }
    };
}

/// Takes an `Option<&mut Vec<T>>` style buffer and reserves space.
macro_rules! map_reserve {
    ($buffer:expr, $size:expr) => {
        match $buffer {
            Some(ref mut b) => crate::utils::map_reserve_inner(b, $size),
            _ => (),
        }
    };
}

pub(crate) fn map_reserve_inner<T>(b: &mut Vec<T>, size: usize) {
    let old_len = b.len();
    if size <= old_len {
        return;
    }
    b.reserve_exact(size - old_len);

    // `memset` to 0, at least so Valgrind doesn't complain
    unsafe {
        let ptr = b.as_mut_ptr().add(old_len) as *mut u8;
        ptr.write_bytes(0, (size - old_len) * std::mem::size_of::<T>());
    }
}

/// Takes an `Option<&mut Vec<T>>` style buffer and shrinks it.
macro_rules! map_set {
    ($buffer:expr, $min:expr) => {
        match $buffer {
            Some(ref mut b) => unsafe { b.set_len($min) },
            _ => (),
        }
    };
}
