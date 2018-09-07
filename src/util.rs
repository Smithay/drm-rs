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
    len: usize
}

impl<T: Sized> SmallBuffer<T> {
    pub fn new(data: [T; 32], len: usize) -> Self {
        Self {
            data: data,
            len: len
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
    len: usize
}

impl SmallOsString {
    pub fn new(data: [u8; 32], len: usize) -> Self {
        Self {
            data: data,
            len: len
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
