//! # Framebuffer
//!
//! Process specific GPU buffers that can be attached to a plane.

use ffi::{self, mode::RawHandle};
use control::{ResourceHandle, ResourceInfo, Device};
use buffer;
use result::Result;

use std::ops::Deref;

#[derive(Copy, Clone, Hash, PartialEq, Eq, From, Into)]
/// A [ResourceHandle](../ResourceHandle.t.html) representing a framebuffer.
pub struct Handle(RawHandle);

impl ResourceHandle for Handle {
    type Info = Info;

    fn get_info<T: Device>(device: &T, handle: Self) -> Result<Info> {
        let mut t = ffi::mode::GetFB::default();
        t.as_mut().fb_id = handle.into();
        t.cmd(device.as_raw_fd())?;
        Ok(Info(t))
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
/// A [ResourceInfo](../ResourceInfo.t.html) object about a framebuffer.
pub struct Info(ffi::mode::GetFB);

impl ResourceInfo for Info {
    type Handle = Handle;

    fn handle(&self) -> Handle {
        Handle::from(self.0.as_ref().fb_id)
    }
}

/// Framebuffer related commands that can be executed by a
/// [control::Device](../Device.t.html).
pub trait Commands: super::Device {
    /// Creates a Framebuffer object from a specified buffer.
    fn create<B>(&self, buffer: &B) -> Result<Handle>
        where B: Deref<Target=buffer::Buffer>;

    /// Removes the specified Framebuffer from the device.
    fn destroy(&self, handle: Handle) -> Result<()>;

    fn mark_dirty(&self, handle: Handle) -> Result<()>;
}

impl<T: super::Device> Commands for T {
    fn create<B>(&self, buffer: &B) -> Result<Handle>
        where B: Deref<Target=buffer::Buffer> {

        let mut t = ffi::mode::AddFB::default();
        t.as_mut().width = buffer.size().0;
        t.as_mut().height = buffer.size().1;
        t.as_mut().pitch = buffer.pitch();
        t.as_mut().bpp = buffer.format().bpp();
        t.as_mut().depth = buffer.format().depth();
        t.as_mut().handle = buffer.handle().into();
        t.cmd(self.as_raw_fd())?;

        Ok(Handle(t.as_ref().fb_id))
    }

    fn destroy(&self, handle: Handle) -> Result<()> {
        let mut t = ffi::mode::RmFB::default();
        *t.as_mut() = handle.into();
        t.cmd(self.as_raw_fd())?;
        Ok(())
    }

    fn mark_dirty(&self, _handle: Handle) -> Result<()> {
        unimplemented!();
    }
}

/*
/// Rect inside the area of a framebuffer
pub type ClipRect = ffi::drm_clip_rect;

/// Mark areas of a framebuffer dirty
pub fn mark_dirty<T>(device: &T, fb: Handle, clips: &[ClipRect]) -> Result<()>
where
    T: control::Device,
{
    let mut raw: ffi::drm_mode_fb_dirty_cmd = Default::default();

    raw.fb_id = fb.into();
    raw.num_clips = clips.len() as u32;
    raw.clips_ptr = clips.as_ptr() as u64;

    unsafe {
        try!(ffi::ioctl_mode_dirtyfb(device.as_raw_fd(), &mut raw));
    }

    Ok(())
}
*/
