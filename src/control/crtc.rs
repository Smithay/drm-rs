//! # CRTC
//!
//! A CRTC is a display controller provided by your device. It's primary job is
//! to take pixel data and send it to a connector with the proper resolution and
//! frequencies.
//!
//! Specific CRTCs can only be attached to connectors that have an encoder it
//! supports. For example, you can have a CRTC that can not output to analog
//! connectors. These are built in hardware limitations.
//!
//! Each CRTC has a built in plane, which can be attached to a framebuffer. It
//! can also use pixel data from other planes to perform hardware compositing.

use iPoint;
use buffer;
use control::{self, ResourceHandle, ResourceInfo};
use result::*;
use ffi;

use control::framebuffer::Handle as FBHandle;
use control::connector::Handle as ConHandle;

use std::any::Any;
use std::boxed::Box;
use std::io::Read;
use std::mem;
use std::time::Duration;

/// A [`ResourceHandle`] for a CRTC.
///
/// Like all control resources, every CRTC has a unique `Handle` associated with
/// it. This `Handle` can be used to acquire information about the CRTC (see
/// [`crtc::Info`]) or change the CRTC's state.
///
/// These can be retrieved by using [`ResourceIds::crtcs`].
///
/// [`ResourceHandle`]: ResourceHandle.t.html
/// [`crtc::Info`]: Info.t.html
/// [`ResourceIds::crtcs`]: ResourceIds.t.html#method.crtcs
#[derive(Handle, Clone, Copy, PartialEq, Eq, Hash)]
#[HandleType = "crtc"]
#[HandleTrait = "ResourceHandle"]
#[HandleRaw = "control::RawHandle"]
pub struct Handle(control::RawHandle);

/// A [`ResourceInfo`] for a CRTC.
///
/// [`ResourceInfo`]: ResourceInfo.t.html
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Info {
    handle: Handle,
    position: (u32, u32),
    // TODO: mode
    fb: control::framebuffer::Handle,
    gamma_length: u32
}

impl control::property::LoadProperties for Handle {
    const TYPE: u32 = ffi::DRM_MODE_OBJECT_CRTC;
}

impl ResourceInfo for Info {
    type Handle = Handle;

    fn load_from_device<T>(device: &T, handle: Handle) -> Result<Self>
        where T: control::Device {

        let crtc = {
            let mut raw: ffi::drm_mode_crtc = Default::default();
            raw.crtc_id = handle.0;
            unsafe {
                try!(ffi::ioctl_mode_getcrtc(device.as_raw_fd(), &mut raw));
            }

            Self {
                handle: handle,
                position: (raw.x, raw.y),
                fb: control::framebuffer::Handle::from_raw(raw.fb_id),
                gamma_length: raw.gamma_size
            }
        };

        Ok(crtc)
    }

    fn handle(&self) -> Self::Handle { self.handle }
}

/// Attaches a framebuffer to a CRTC's built-in plane, attaches the CRTC to
/// a connector, and sets the CRTC's mode to output the pixel data.
pub fn set<T>(device: &T, handle: Handle, fb: FBHandle, cons: &[ConHandle],
              position: (u32, u32), mode: Option<control::Mode>) -> Result<()>
    where T: control::Device {


    let mut raw: ffi::drm_mode_crtc = Default::default();
    raw.x = position.0;
    raw.y = position.1;
    raw.crtc_id = handle.as_raw();
    raw.fb_id = fb.as_raw();
    raw.set_connectors_ptr = cons.as_ptr() as u64;
    raw.count_connectors = cons.len() as u32;

    match mode {
        Some(m) => {
            raw.mode = m.mode;
            raw.mode_valid = 1;
        },
        _ => ()
    };

    unsafe {
        try!(ffi::ioctl_mode_setcrtc(device.as_raw_fd(), &mut raw));
    }

    Ok(())
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Flags to alter the behaviour of a page flip
pub enum PageFlipFlags {
    /// Request a vblank event on page flip
    PageFlipEvent = ffi::DRM_MODE_PAGE_FLIP_EVENT,
    /// Request page flip as soon as possible, not waiting for vblank
    PageFlipAsync = ffi::DRM_MODE_PAGE_FLIP_ASYNC,
}

struct FatPtrWrapper(Box<Any>);

/// Queue a page flip on the given crtc.
///
/// On the next vblank the given framebuffer will be attached to the
/// crtc and an event will be triggered by the device, which will be indicated by it's fd becoming
/// readable. The event can be received using `handle_event`.
pub fn page_flip<T, U>(device: &T, handle: Handle, fb: FBHandle, flags: &[PageFlipFlags], userdata: U) -> Result<()>
    where T: control::Device, U: 'static {

    let mut raw: ffi::drm_mode_crtc_page_flip = Default::default();
    raw.fb_id = fb.as_raw();
    raw.crtc_id = handle.as_raw();
    raw.flags = flags.into_iter().fold(0, |val, flag| val | *flag as u32);
    raw.user_data = Box::into_raw(Box::new(FatPtrWrapper(Box::new(userdata) as Box<Any>))) as u64;

    unsafe {
        try!(ffi::ioctl_mode_page_flip(device.as_raw_fd(), &mut raw));
    }

    Ok(())
}

/// Iterator over `Event`s of a device. Create via `receive_events`.
pub struct Events {
    event_buf: [u8; 1024],
    amount: usize,
    i: usize,
}

/// An event from a device.
pub enum Event {
    /// A vblank happened
    Vblank(VblankEvent),
    /// A page flip happened
    PageFlip(PageFlipEvent),
    /// Unknown event, raw data provided
    Unknown(Vec<u8>),
}

/// Vblank event
pub struct VblankEvent {
    /// sequence of the frame
    pub frame: u32,
    /// duration between events
    pub duration: Duration,
    /// userdata as passed into `page_flip`
    pub userdata: Box<Any>,
}

/// Page Flip event
pub struct PageFlipEvent {
    /// sequence of the frame
    pub frame: u32,
    /// duration between events
    pub duration: Duration,
    /// crtc that did throw the event, if available by the driver
    pub crtc: Option<Handle>,
    /// userdata as passed into `page_flip`
    pub userdata: Box<Any>,
}

impl Iterator for Events {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        if self.amount > 0 && self.i < self.amount {
            let event = unsafe { &*(self.event_buf.as_ptr().offset(self.i as isize) as *const ffi::drm_event) };
            self.i += event.length as usize;
            match event.type_ {
                x if x == ffi::DRM_EVENT_VBLANK => {
                    let vblank_event: &ffi::drm_event_vblank = unsafe { mem::transmute(event) };
                    let userdata = unsafe { Box::from_raw(vblank_event.user_data as *mut FatPtrWrapper).0 };
                    Some(Event::Vblank(VblankEvent {
                        frame: vblank_event.sequence,
                        duration: Duration::new(vblank_event.tv_sec as u64, vblank_event.tv_usec * 100),
                        userdata,
                    }))
                },
                x if x == ffi::DRM_EVENT_FLIP_COMPLETE => {
                    let vblank_event: &ffi::drm_event_vblank = unsafe { mem::transmute(event) };
                    let userdata = unsafe { Box::from_raw(vblank_event.user_data as *mut FatPtrWrapper).0 };
                    Some(Event::PageFlip(PageFlipEvent {
                        frame: vblank_event.sequence,
                        duration: Duration::new(vblank_event.tv_sec as u64, vblank_event.tv_usec * 1000),
                        crtc: if vblank_event.crtc_id != 0 { Some(Handle::from_raw(vblank_event.crtc_id)) } else { None },
                        userdata
                    }))
                },
                _ => Some(Event::Unknown(self.event_buf[self.i-(event.length as usize)..self.i].to_vec())),
            }
        } else {
            None
        }
    }
}

/// Receives all pending events of a given device and returns an Iterator for them.
pub fn receive_events<T>(device: &T) -> Result<Events>
    where T: control::Device,
{
    struct DeviceWrapper<'a, T: control::Device + 'a>(&'a T);
    impl<'a, T: control::Device> Read for DeviceWrapper<'a, T> {
        fn read(&mut self, buf: &mut [u8]) -> ::std::io::Result<usize> {
            ::nix::unistd::read(self.0.as_raw_fd(), buf).map_err(|err| {
                match err {
                    ::nix::Error::Sys(_) => ::std::io::Error::last_os_error(),
                    err => ::std::io::Error::new(::std::io::ErrorKind::Other, err),
                }
            })
        }
    }
    let mut wrapper = DeviceWrapper(device);

    let mut event_buf: [u8; 1024] = [0; 1024];
    let amount = try!(wrapper.read(&mut event_buf));

    Ok(Events {
        event_buf,
        amount,
        i: 0,
    })
}

/// Sets a hardware-cursor on the given crtc with the image of a given buffer
pub fn set_cursor<T, B>(device: &T, handle: Handle, buffer: &B) -> Result<()>
    where T: control::Device,
          B: buffer::Buffer   {

    let dimensions = buffer.size();

    let mut raw: ffi::drm_mode_cursor = Default::default();
    raw.flags = ffi::DRM_MODE_CURSOR_BO;
    raw.crtc_id = handle.as_raw();
    raw.width = dimensions.0;
    raw.height = dimensions.1;
    raw.handle = buffer.handle().as_raw();

    unsafe {
        try!(ffi::ioctl_mode_cursor(device.as_raw_fd(), &mut raw));
    }

    Ok(())
}

/// Sets a hardware-cursor on the given crtc with the image of a given buffer and a hotspot marking
/// the click point of the cursor
pub fn set_cursor2<T, B>(device: &T, handle: Handle, buffer: &B, hotspot: iPoint) -> Result<()>
    where T: control::Device,
          B: buffer::Buffer   {

    let dimensions = buffer.size();

    let mut raw: ffi::drm_mode_cursor2 = Default::default();
    raw.flags = ffi::DRM_MODE_CURSOR_BO;
    raw.crtc_id = handle.as_raw();
    raw.width = dimensions.0;
    raw.height = dimensions.1;
    raw.handle = buffer.handle().as_raw();
    raw.hot_x = hotspot.0;
    raw.hot_y = hotspot.1;

    unsafe {
        try!(ffi::ioctl_mode_cursor2(device.as_raw_fd(), &mut raw));
    }

    Ok(())
}

/// Moves a set cursor on a given crtc
pub fn move_cursor<T>(device: &T, handle: Handle, to: iPoint) -> Result<()>
    where T: control::Device {

    let mut raw: ffi::drm_mode_cursor = Default::default();
    raw.flags = ffi::DRM_MODE_CURSOR_MOVE;
    raw.crtc_id = handle.as_raw();
    raw.x = to.0;
    raw.y = to.1;

    unsafe {
        try!(ffi::ioctl_mode_cursor(device.as_raw_fd(), &mut raw));
    }

    Ok(())
}

#[derive(Debug, Clone)]
/// The hardware gamma ramp
pub struct GammaRamp {
    /// Red color component
    pub red: Box<[u16]>,
    /// Green color component
    pub green: Box<[u16]>,
    /// Blue color component
    pub blue: Box<[u16]>,
}

/// Receive the currently set gamma ramp of a crtc
pub fn gamma<T>(device: &T, handle: Handle) -> Result<GammaRamp>
    where T: control::Device {

    let info = Info::load_from_device(device, handle)?;

    let mut raw: ffi::drm_mode_crtc_lut = Default::default();
    raw.crtc_id = handle.as_raw();
    raw.gamma_size = info.gamma_length;
    let red = ffi_buf!(raw.red, info.gamma_length as usize);
    let green = ffi_buf!(raw.green, info.gamma_length as usize);
    let blue = ffi_buf!(raw.blue, info.gamma_length as usize);

    unsafe {
        try!(ffi::ioctl_mode_getgamma(device.as_raw_fd(), &mut raw));
    }

    Ok(GammaRamp {
        red:   red.into_boxed_slice(),
        green: green.into_boxed_slice(),
        blue:  blue.into_boxed_slice(),
    })
}

/// Set a gamma ramp for the given crtc
pub fn set_gamma<T>(device: &T, handle: Handle, mut gamma: GammaRamp) -> Result<()>
    where T: control::Device {

    let info = Info::load_from_device(device, handle)?;

    if gamma.red.len() as u32 != info.gamma_length {
        return Err(Error::from_kind(ErrorKind::InvalidGammaSize(gamma.red.len(), info.gamma_length)));
    }

    if gamma.green.len() as u32 != info.gamma_length {
        return Err(Error::from_kind(ErrorKind::InvalidGammaSize(gamma.green.len(), info.gamma_length).into()))
    }

    if gamma.blue.len() as u32 != info.gamma_length {
        return Err(Error::from_kind(ErrorKind::InvalidGammaSize(gamma.blue.len(), info.gamma_length).into()))
    }

    let mut raw: ffi::drm_mode_crtc_lut = Default::default();
    raw.crtc_id = handle.as_raw();
    raw.gamma_size = info.gamma_length;
    raw.red = gamma.red.as_mut_ptr() as u64;
    raw.green = gamma.green.as_mut_ptr() as u64;
    raw.blue = gamma.blue.as_mut_ptr() as u64;

    unsafe {
        try!(ffi::ioctl_mode_setgamma(device.as_raw_fd(), &mut raw));
    }

    Ok(())
}
