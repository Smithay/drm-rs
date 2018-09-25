//! Utilities used internally by this crate.

#![allow(dead_code)]
#![allow(missing_docs)]

use core::str::lossy::Utf8Lossy;
pub use std::ffi::OsStr;

use std::fmt;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct SmallOsString {
    data: [u8; 32],
    len: usize,
}

impl SmallOsString {
    pub fn new(data: [u8; 32], len: usize) -> Self {
        Self {
            data: data,
            len: len,
        }
    }
}

impl AsRef<[u8]> for SmallOsString {
    fn as_ref(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

impl AsRef<OsStr> for SmallOsString {
    fn as_ref(&self) -> &OsStr {
        use std::os::unix::ffi::OsStrExt;

        let slice: &[u8] = &self.data[..self.len];
        OsStr::from_bytes(slice)
    }
}

impl fmt::Debug for SmallOsString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let as_os_str: &OsStr = &self.as_ref();
        f.debug_struct("SmallOsString")
            .field("data", &self.data)
            .field("len", &self.len)
            .field("as_ref()", &as_os_str)
            .finish()
    }
}

impl fmt::Display for SmallOsString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let lossy_utf8 = Utf8Lossy::from_bytes(self.as_ref());
        write!(f, "{}", lossy_utf8)
    }
}

use std::marker::PhantomData;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct Buffer4x32<T> {
    data: [u32; 32],
    len: usize,
    _phantom: PhantomData<T>,
}

impl<T> Default for Buffer4x32<T> {
    fn default() -> Buffer4x32<T> {
        use std::mem;
        unsafe {
            Buffer4x32 {
                len: 32 as usize,
                ..mem::zeroed()
            }
        }
    }
}

impl<T> Buffer4x32<T> {
    /// Returns the data as an immutable `u32` slice.
    pub fn as_u32_slice(&self) -> &[u32] {
        &self.data[..self.len]
    }

    /// Returns the data as a mutable `u32` slice.
    pub fn as_mut_u32_slice(&mut self) -> &mut [u32] {
        &mut self.data[..self.len]
    }

    /// Returns the underlying data as an immutable slice.
    pub unsafe fn as_slice(&self) -> &[T] {
        use std::mem;
        mem::transmute(self.as_u32_slice())
    }

    /// Returns the underlying data as a mutable slice.
    pub unsafe fn as_mut_slice(&mut self) -> &mut [T] {
        use std::mem;
        mem::transmute(self.as_mut_u32_slice())
    }

    pub fn update_len(&mut self, len: usize) {
        self.len = len;
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct Buffer4x3<T> {
    data: [u32; 3],
    len: usize,
    _phantom: PhantomData<T>,
}

impl<T> Default for Buffer4x3<T> {
    fn default() -> Buffer4x3<T> {
        use std::mem;
        unsafe {
            Buffer4x3 {
                len: 3 as usize,
                ..mem::zeroed()
            }
        }
    }
}

impl<T> Buffer4x3<T> {
    /// Returns the data as an immutable `u32` slice.
    pub fn as_u32_slice(&self) -> &[u32] {
        &self.data[..self.len]
    }

    /// Returns the data as a mutable `u32` slice.
    pub fn as_mut_u32_slice(&mut self) -> &mut [u32] {
        &mut self.data[..self.len]
    }

    /// Returns the underlying data as an immutable slice.
    pub unsafe fn as_slice(&self) -> &[T] {
        use std::mem;
        mem::transmute(self.as_u32_slice())
    }

    /// Returns the underlying data as a mutable slice.
    pub unsafe fn as_mut_slice(&mut self) -> &mut [T] {
        use std::mem;
        mem::transmute(self.as_mut_u32_slice())
    }

    pub fn update_len(&mut self, len: usize) {
        self.len = len;
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct Buffer4x24<T> {
    data: [u32; 24],
    len: usize,
    _phantom: PhantomData<T>,
}

impl<T> Default for Buffer4x24<T> {
    fn default() -> Buffer4x24<T> {
        use std::mem;
        unsafe {
            Buffer4x24 {
                len: 24 as usize,
                ..mem::zeroed()
            }
        }
    }
}

impl<T> Buffer4x24<T> {
    /// Returns the data as an immutable `u32` slice.
    pub fn as_u32_slice(&self) -> &[u32] {
        &self.data[..self.len]
    }

    /// Returns the data as a mutable `u32` slice.
    pub fn as_mut_u32_slice(&mut self) -> &mut [u32] {
        &mut self.data[..self.len]
    }

    /// Returns the underlying data as an immutable slice.
    pub unsafe fn as_slice(&self) -> &[T] {
        use std::mem;
        mem::transmute(self.as_u32_slice())
    }

    /// Returns the underlying data as a mutable slice.
    pub unsafe fn as_mut_slice(&mut self) -> &mut [T] {
        use std::mem;
        mem::transmute(self.as_mut_u32_slice())
    }

    pub fn update_len(&mut self, len: usize) {
        self.len = len;
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct Buffer8x24<T> {
    data: [u64; 24],
    len: usize,
    _phantom: PhantomData<T>,
}

impl<T> Default for Buffer8x24<T> {
    fn default() -> Buffer8x24<T> {
        use std::mem;
        unsafe {
            Buffer8x24 {
                len: 24 as usize,
                ..mem::zeroed()
            }
        }
    }
}

impl<T> Buffer8x24<T> {
    /// Returns the data as an immutable `u64` slice.
    pub fn as_u64_slice(&self) -> &[u64] {
        &self.data[..self.len]
    }

    /// Returns the data as a mutable `u64` slice.
    pub fn as_mut_u64_slice(&mut self) -> &mut [u64] {
        &mut self.data[..self.len]
    }

    /// Returns the underlying data as an immutable slice.
    pub unsafe fn as_slice(&self) -> &[T] {
        use std::mem;
        mem::transmute(self.as_u64_slice())
    }

    /// Returns the underlying data as a mutable slice.
    pub unsafe fn as_mut_slice(&mut self) -> &mut [T] {
        use std::mem;
        mem::transmute(self.as_mut_u64_slice())
    }

    pub fn update_len(&mut self, len: usize) {
        self.len = len;
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct BufferNx32<T, U> {
    data: [U; 32],
    len: usize,
    _phantom: PhantomData<T>,
}

impl<T, U> Default for BufferNx32<T, U> {
    fn default() -> BufferNx32<T, U> {
        use std::mem;
        unsafe {
            BufferNx32 {
                len: 32 as usize,
                ..mem::zeroed()
            }
        }
    }
}

impl<T, U> BufferNx32<T, U> {
    /// Returns the data as an immutable `u32` slice.
    pub fn as_raw_slice(&self) -> &[U] {
        &self.data[..self.len]
    }

    /// Returns the data as a mutable `u32` slice.
    pub fn as_mut_raw_slice(&mut self) -> &mut [U] {
        &mut self.data[..self.len]
    }

    /// Returns the underlying data as an immutable slice.
    pub unsafe fn as_slice(&self) -> &[T] {
        use std::mem;
        mem::transmute(self.as_raw_slice())
    }

    /// Returns the underlying data as a mutable slice.
    pub unsafe fn as_mut_slice(&mut self) -> &mut [T] {
        use std::mem;
        mem::transmute(self.as_mut_raw_slice())
    }

    /// Updates the length for returning slices.
    pub fn update_len(&mut self, len: usize) {
        self.len = len;
    }
}
