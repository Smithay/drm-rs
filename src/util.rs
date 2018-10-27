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
    pub fn from_u8_buffer(data: [u8; 32], len: usize) -> Self {
        Self {
            data: data,
            len: len,
        }
    }

    pub fn from_i8_buffer(data: [i8; 32], len: usize) -> Self {
        unsafe { Self::from_u8_buffer(std::mem::transmute(data), len) }
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
