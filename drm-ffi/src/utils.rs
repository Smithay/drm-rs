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

/// Takes an `Option<&mut Vec<T>>` style buffer and shrinks it.
macro_rules! map_reserve {
    ($buffer:expr, $size:expr) => {
        match $buffer {
            Some(ref mut b) => b.reserve_exact($size - b.len()),
            _ => (),
        }
    };
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
