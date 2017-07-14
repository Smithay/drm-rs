use control::{self, ResourceHandle, ResourceInfo};
use buffer::Buffer;
use result::*;
use ffi;

#[derive(Clone, Copy, PartialEq, Eq)]
/// A `ResourceHandle` to a framebuffer.
pub struct Id(control::RawId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The `ResourceInfo` on a framebuffer.
pub struct Info {
    handle: Id,
    size: (u32, u32),
    pitch: u32,
    bpp: u8,
    // TODO: Gem handle?
    depth: u8
}

impl Info {
    /// Attempts to create a new framebuffer object using the provided Buffer.
    pub fn create_from_buffer<T, U>(device: &T, buf: &U) -> Result<Info>
        where T: control::Device, U: Buffer {

        let mut raw: ffi::drm_mode_fb_cmd = Default::default();
        let (w, h) = buf.size();
        raw.width = w;
        raw.height = h;
        raw.pitch = buf.pitch();
        raw.bpp = buf.bpp() as u32;
        raw.depth = buf.depth() as u32;
        raw.handle = buf.handle().as_raw();

        unsafe {
            try!(ffi::ioctl_mode_addfb(device.as_raw_fd(), &mut raw));
        }

        let fb = Info {
            handle: Id::from_raw(raw.fb_id),
            size: (raw.width, raw.height),
            pitch: raw.pitch,
            bpp: raw.bpp as u8,
            depth: raw.depth as u8
        };

        Ok(fb)
    }
}

impl ResourceHandle for Id {
    type RawHandle = control::RawId;

    fn from_raw(raw: Self::RawHandle) -> Self {
        Id(raw)
    }

    fn as_raw(&self) -> Self::RawHandle {
        self.0
    }
}

impl control::property::LoadProperties for Id {
    const TYPE: u32 = ffi::DRM_MODE_OBJECT_FB;
}

impl ResourceInfo for Info {
    type Handle = Id;

    fn load_from_device<T>(device: &T, handle: Self::Handle) -> Result<Self>
        where T: control::Device {

        let mut raw: ffi::drm_mode_fb_cmd = Default::default();
        raw.fb_id = handle.0;
        unsafe {
            try!(ffi::ioctl_mode_getfb(device.as_raw_fd(), &mut raw));
        }

        let fb = Self {
            handle: handle,
            size: (raw.width, raw.height),
            pitch: raw.pitch,
            bpp: raw.bpp as u8,
            depth: raw.depth as u8
        };

        Ok(fb)
    }

    fn handle(&self) -> Self::Handle { self.handle }
}

impl From<Id> for control::RawId {
    fn from(id: Id) -> Self {
        id.0
    }
}

impl From<Info> for Id {
    fn from(info: Info) -> Self {
        info.handle
    }
}

impl ::std::fmt::Debug for Id {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "framebuffer::Id({})", self.0)
    }
}
