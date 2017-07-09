use drm_sys;
use control::{self, ResourceHandle, ResourceInfo};
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
    bpp: u32,
    // TODO: Gem handle?
    depth: u32
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

impl ResourceInfo for Info {
    type Handle = Id;

    fn load_from_device<T>(device: &T, handle: Self::Handle) -> Result<Self>
        where T: control::Device {

        let mut raw: drm_sys::drm_mode_fb_cmd = Default::default();
        raw.fb_id = handle.0;
        unsafe {
            try!(ffi::ioctl_mode_getfb(device.as_raw_fd(), &mut raw));
        }

        let fb = Self {
            handle: handle,
            size: (raw.width, raw.height),
            pitch: raw.pitch,
            bpp: raw.bpp,
            depth: raw.depth
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
