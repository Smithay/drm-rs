//! # Framebuffer
//!
//! A framebuffer is pixel data that can be attached to a plane.

use control::{self, ResourceHandle, ResourceInfo};
use result::*;
use ffi;

/// A [`ResourceHandle`] for a framebuffer.
///
/// Like all control resources, every framebuffer has a unique `Handle`
/// associated with it. This `Handle` can be used to acquire information about
/// the framebuffer (see [`framebuffer::Info`]) or change the framebuffer's
/// state.
///
/// These can be retrieved by using [`ResourceIds::framebuffers`], but most
/// often you will create your own using [`Device::create_framebuffer`].
///
/// [`ResourceHandle`]: ResourceHandle.t.html
/// [`framebuffer::Info`]: Info.t.html
/// [`ResourceIds::framebuffers`]: ResourceIds.t.html#method.framebuffers
#[derive(Handle, Clone, Copy, PartialEq, Eq, Hash)]
#[HandleType = "framebuffer"]
#[HandleTrait = "ResourceHandle"]
#[HandleRaw = "control::RawHandle"]
pub struct Handle(control::RawHandle);

/// A [`ResourceInfo`] for a framebuffer.
///
/// [`ResourceInfo`]: ResourceInfo.t.html
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Info {
    handle: Handle,
    size: (u32, u32),
    pitch: u32,
    bpp: u8,
    // TODO: Gem handle?
    depth: u8,
}

impl control::property::LoadProperties for Handle {
    const TYPE: u32 = ffi::DRM_MODE_OBJECT_FB;
}

impl ResourceInfo for Info {
    type Handle = Handle;

    fn load_from_device<T>(device: &T, handle: Self::Handle) -> Result<Self>
    where
        T: control::Device,
    {
        let framebuffer = {
            let mut raw: ffi::drm_mode_fb_cmd = Default::default();
            raw.fb_id = handle.as_raw();
            unsafe {
                try!(ffi::ioctl_mode_getfb(device.as_raw_fd(), &mut raw));
            }

            Self {
                handle: handle,
                size: (raw.width, raw.height),
                pitch: raw.pitch,
                bpp: raw.bpp as u8,
                depth: raw.depth as u8,
            }
        };

        Ok(framebuffer)
    }

    fn handle(&self) -> Self::Handle {
        self.handle
    }
}

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
        raw.handles[0] = buffer.handle().as_raw();
        raw.pitches[0] = buffer.pitch();
        raw.offsets[0] = 0; //TODO
        raw.modifier[0]; //TODO

        match unsafe { ffi::ioctl_mode_addfb2(device.as_raw_fd(), &mut raw) } {
            Ok(_) => try!(Info::load_from_device(device, Handle::from_raw(raw.fb_id))),
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
                raw_old.handle = buffer.handle().as_raw();

                unsafe {
                    try!(ffi::ioctl_mode_addfb(device.as_raw_fd(), &mut raw_old));
                }

                Info {
                    handle: Handle::from_raw(raw_old.fb_id),
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

    raw.fb_id = fb.as_raw();
    raw.num_clips = clips.len() as u32;
    raw.clips_ptr = clips.as_ptr() as u64;

    unsafe {
        try!(ffi::ioctl_mode_dirtyfb(device.as_raw_fd(), &mut raw));
    }

    Ok(())
}

/// Destroy a framebuffer
pub fn destroy<T>(device: &T, fb: Handle) -> Result<()>
where
    T: control::Device,
{
    unsafe {
        try!(ffi::ioctl_mode_rmfb(
            device.as_raw_fd(),
            &mut fb.as_raw() as *mut _
        ));
    }

    Ok(())
}
