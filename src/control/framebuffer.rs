//! # Framebuffer
//!
//! Process specific GPU buffers that can be attached to a plane.

use ffi::{self, Wrapper, mode::RawHandle};
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
        t.raw_mut_ref().fb_id = handle.into();
        t.ioctl(device.as_raw_fd())?;
        Ok(Info(t))
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
/// A [ResourceInfo](../ResourceInfo.t.html) object about a framebuffer.
pub struct Info(ffi::mode::GetFB);

impl ResourceInfo for Info {
    type Handle = Handle;

    fn handle(&self) -> Handle {
        Handle::from(self.0.raw_ref().fb_id)
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
        t.raw_mut_ref().width = buffer.size().0;
        t.raw_mut_ref().height = buffer.size().1;
        t.raw_mut_ref().pitch = buffer.pitch();
        t.raw_mut_ref().bpp = buffer.format().bpp();
        t.raw_mut_ref().depth = buffer.format().depth();
        t.raw_mut_ref().handle = buffer.handle().into();
        t.ioctl(self.as_raw_fd())?;

        Ok(Handle(t.raw_ref().fb_id))
    }

    fn destroy(&self, handle: Handle) -> Result<()> {
        let mut t = ffi::mode::RmFB::default();
        *t.raw_mut_ref() = handle.into();
        t.ioctl(self.as_raw_fd())?;
        Ok(())
    }

    fn mark_dirty(&self, _handle: Handle) -> Result<()> {
        unimplemented!();
    }
}

/*
/// Creates a framebuffer from a [`Buffer`], returning
/// [`framebuffer::Info`].
///
/// [`framebuffer::Info`]: framebuffer.Handle.html
pub fn create<T, U>(device: &T, buffer: &U) -> Result<Info>
where
    T: control::Device,
    U: super::super::buffer::Buffer,
{
    let framebuffer = {
        let mut raw: ffi::drm_mode_fb_cmd2 = Default::default();
        let (w, h) = buffer.size();
        raw.width = w;
        raw.height = h;
        raw.pixel_format = buffer.format().as_raw();
        raw.flags = 0; //TODO
        raw.handles[0] = buffer.handle().into();
        raw.pitches[0] = buffer.pitch();
        raw.offsets[0] = 0; //TODO
        raw.modifier[0]; //TODO

        match unsafe { ffi::ioctl_mode_addfb2(device.as_raw_fd(), &mut raw) } {
            Ok(_) => try!(Info::load_from_device(device, Handle::from(raw.fb_id))),
            Err(_) => {
                //ioctl addfd2 unsupported
                let mut raw_old: ffi::drm_mode_fb_cmd = Default::default();
                raw_old.width = w;
                raw_old.height = h;
                raw_old.pitch = buffer.pitch();
                let depth = try!(
                    buffer
                        .format()
                        .depth()
                        .ok_or(Error::from_kind(ErrorKind::UnsupportedPixelFormat))
                );
                let bpp = try!(
                    buffer
                        .format()
                        .bpp()
                        .ok_or(Error::from_kind(ErrorKind::UnsupportedPixelFormat))
                );
                raw_old.bpp = bpp as u32;
                raw_old.depth = depth as u32;
                raw_old.handle = buffer.handle().into();

                unsafe {
                    try!(ffi::ioctl_mode_addfb(device.as_raw_fd(), &mut raw_old));
                }

                Info {
                    handle: Handle::from(raw_old.fb_id),
                    size: (raw_old.width, raw_old.height),
                    pitch: raw_old.pitch,
                    depth: raw_old.depth as u8,
                    bpp: raw_old.bpp as u8,
                }
            }
        }
    };

    Ok(framebuffer)
}

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
