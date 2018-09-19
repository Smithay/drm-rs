//! Utilities used internally by this crate.

use core::str::lossy::Utf8Lossy;
pub use std::ffi::OsStr;

use std::fmt;

/// Many different native DRM API calls will fill buffers with small amounts
/// of data. This object is used to eliminate the need for allocating buffer
/// dynamically.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct SmallBuffer<T: Sized> {
    data: [T; 32],
    len: usize,
}

impl<T: Sized> SmallBuffer<T> {
    pub fn new(data: [T; 32], len: usize) -> Self {
        Self {
            data: data,
            len: len,
        }
    }
}

impl<T: Sized> AsRef<[T]> for SmallBuffer<T> {
    fn as_ref(&self) -> &[T] {
        &self.data[..self.len]
    }
}

/// Same as a SmallBuffer<u8>, but specialized to hold an OsString
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
        Buffer4x32 {
            len: 32 as usize,
            ..Default::default()
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
        Buffer4x3 {
            len: 32 as usize,
            ..Default::default()
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
