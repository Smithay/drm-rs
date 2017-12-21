//!
//! # DumbBuffer
//!
//! Memory-supported, slow, but easy & cross-platform buffer implementation
//!

use ffi;
use result::*;
use control;
use buffer;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Slow, but generic `Buffer` implementation
pub struct DumbBuffer {
    size: (u32, u32),
    length: usize,
    format: buffer::PixelFormat,
    pitch: u32,
    handle: buffer::Id,
}

/// Mapping of a dumbbuffer
pub struct DumbMapping<'a> {
    _phantom: ::std::marker::PhantomData<&'a ()>,
    map: &'a mut [u8],
}

impl DumbBuffer {
    /// Create a new dumb buffer with a given size and pixel format
    pub fn create_from_device<T>(
        device: &T,
        size: (u32, u32),
        format: buffer::PixelFormat,
    ) -> Result<Self>
    where
        T: control::Device,
    {
        let mut raw: ffi::drm_mode_create_dumb = Default::default();
        raw.width = size.0;
        raw.height = size.1;
        raw.bpp = try!(
            format
                .bpp()
                .ok_or(Error::from_kind(ErrorKind::UnsupportedPixelFormat))
        ) as u32;

        unsafe {
            try!(ffi::ioctl_mode_create_dumb(device.as_raw_fd(), &mut raw));
        }

        let dumb = Self {
            size: (raw.width, raw.height),
            length: raw.size as usize,
            format: format,
            pitch: raw.pitch,
            handle: buffer::Id::from_raw(raw.handle),
        };

        Ok(dumb)
    }

    /// Free the memory resources of a dumb buffer
    pub fn destroy<T>(self, device: &T) -> Result<()>
    where
        T: control::Device,
    {
        let mut raw: ffi::drm_mode_destroy_dumb = Default::default();
        raw.handle = self.handle.as_raw();

        unsafe {
            try!(ffi::ioctl_mode_destroy_dumb(device.as_raw_fd(), &mut raw));
        }

        Ok(())
    }

    /// Map the buffer for access
    pub fn map<'a, T>(&'a mut self, device: &T) -> Result<DumbMapping<'a>>
    where
        T: control::Device,
    {
        let mut raw: ffi::drm_mode_map_dumb = Default::default();
        raw.handle = self.handle.as_raw();

        unsafe {
            try!(ffi::ioctl_mode_map_dumb(device.as_raw_fd(), &mut raw));
        }

        let map = {
            use nix::sys::mman;
            let addr = ::std::ptr::null_mut();
            let prot = mman::PROT_READ | mman::PROT_WRITE;
            let flags = mman::MAP_SHARED;
            let length = self.length;
            let fd = device.as_raw_fd();
            let offset = raw.offset as i64;
            try!(mman::mmap(addr, length, prot, flags, fd, offset))
        };

        let mapping = DumbMapping {
            _phantom: ::std::marker::PhantomData,
            map: unsafe { ::std::slice::from_raw_parts_mut(map as *mut _, self.length) },
        };

        Ok(mapping)
    }
}

impl<'a> AsMut<[u8]> for DumbMapping<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.map
    }
}

impl<'a> Drop for DumbMapping<'a> {
    fn drop(&mut self) {
        use nix::sys::mman;

        mman::munmap(self.map.as_mut_ptr() as *mut _, self.map.len()).expect("Unmap failed");
    }
}

impl buffer::Buffer for DumbBuffer {
    fn size(&self) -> (u32, u32) {
        self.size
    }
    fn format(&self) -> buffer::PixelFormat {
        self.format
    }
    fn pitch(&self) -> u32 {
        self.pitch
    }
    fn handle(&self) -> buffer::Id {
        self.handle
    }
}
