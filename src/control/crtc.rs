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
//! Each CRTC has a built in plane, which can have a framebuffer attached to it,
//! but they can also use pixel data from other planes to perform hardware
//! compositing.

use ffi::{self, Wrapper, mode::RawHandle};
use control::{ResourceHandle, ResourceInfo, Device};
use control::framebuffer;
use control::connector;
use buffer;
use result::Result;

use std::ops::Deref;

#[derive(Copy, Clone, Hash, PartialEq, Eq, From, Into)]
/// A [ResourceHandle](../ResourceHandle.t.html) representing a CRTC.
pub struct Handle(RawHandle);

impl ResourceHandle for Handle {
    const DEBUG_NAME: &'static str = "crtc::Handle";
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
/// A [ResourceInfo](../ResourceInfo.t.html) object about a CRTC.
pub struct Info(ffi::mode::GetCrtc);

impl ResourceInfo for Info {
    type Handle = Handle;

    fn load_from_device<T: Device>(device: &T, id: Handle) -> Result<Self> {
        let mut t = ffi::mode::GetCrtc::default();
        t.raw_mut_ref().crtc_id = id.into();
        t.ioctl(device.as_raw_fd())?;
        Ok(Info(t))
    }

    fn handle(&self) -> Handle {
        Handle::from(self.0.raw_ref().crtc_id)
    }
}

impl Info {
    /// Returns the origin this CRTC starts to scan from.
    pub fn position(&self) -> (u32, u32) {
        (self.0.raw_ref().x, self.0.raw_ref().y)
    }

    /// Returns the current mode this CRTC is outputting at.
    pub fn mode(&self) {
        unimplemented!()
    }

    /// Returns the handle associated with the currently attached framebuffer.
    pub fn fb(&self) -> Option<framebuffer::Handle> {
        framebuffer::Handle::from_checked(self.0.raw_ref().fb_id)
    }

    /// Returns the size of the gamma buffers.
    pub fn gamma_size(&self) -> u32 {
        self.0.raw_ref().gamma_size
    }
}

/// CRTC related commands that can be executed by a
/// [control::Device](../Device.t.html).
pub trait Commands: super::Device {
    /// Sets the specified CRTC's output connectors, input framebuffer,
    /// position, and mode.
    fn set(&self, handle: Handle, connectors: &[connector::Handle],
           fb: framebuffer::Handle, position: (u32, u32),
           mode: Option<()>) -> Result<()>;

    /// Updates a cursor image.
    fn set_cursor<B>(&self, handle: Handle, buffer: &B, size: (u32, u32),
                     hot: Option<(i32, i32)>) -> Result<()>
        where B: Deref<Target=buffer::Buffer>;

    /// Moves the cursor to a new position.
    fn move_cursor(&self, handle: Handle, position: (i32, i32)) -> Result<()>;

    /// Clears the cursor's buffer, resetting it.
    fn clear_cursor(&self, handle: Handle) -> Result<()>;

    /// Requests a page flip on the given CRTC.
    fn page_flip(&self, handle: Handle, fb: framebuffer::Handle,
                 flags: PageFlipFlag, target: Option<()>) -> Result<()>;

    /// Returns the currently set gamma ramp for the given CRTC.
    fn get_gamma(&self, handle: Handle) -> Result<()>;

    /// Sets a new gamma ramp for the given CRTC.
    fn set_gamma(&self, handle: Handle) -> Result<()>;
}

impl<T: super::Device> Commands for T {
    fn set(&self, handle: Handle, connectors: &[connector::Handle],
           fb: framebuffer::Handle, position: (u32, u32),
           mode: Option<()>) -> Result<()> {

        let mut t = ffi::mode::SetCrtc::default();
        t.raw_mut_ref().crtc_id = handle.into();
        t.raw_mut_ref().set_connectors_ptr = connectors.as_ptr() as u64;
        t.raw_mut_ref().count_connectors = connectors.len() as u32;
        t.raw_mut_ref().fb_id = fb.into();
        t.raw_mut_ref().x = position.0;
        t.raw_mut_ref().y = position.1;

        match mode {
            Some(_m) => {
                // TODO: Get Mode type working first
                unimplemented!();
                //t.raw_mut_ref().mode = m;
                //t.raw_mut_ref().mode_valid = 1;
            },
            None => {
                t.raw_mut_ref().mode_valid = 0;
            }
        }

        t.ioctl(self.as_raw_fd())?;
        Ok(())
    }

    fn set_cursor<B>(&self, handle: Handle, _buffer: &B, size: (u32, u32),
                     hot: Option<(i32, i32)>) -> Result<()>
        where B: Deref<Target=buffer::Buffer> {

        // Determines if we use Cursor or Cursor2
        match hot {
            None => {
                let mut t = ffi::mode::Cursor::default();
                t.raw_mut_ref().flags = ffi::DRM_MODE_CURSOR_BO;
                t.raw_mut_ref().crtc_id = handle.into();
                t.raw_mut_ref().width = size.0;
                // TODO: Get buffer working first
                unimplemented!();
                //t.raw_mut_ref().handle = buffer.???;
                t.ioctl(self.as_raw_fd())?;
            },
            Some(h) => {
                let mut t = ffi::mode::Cursor2::default();
                t.raw_mut_ref().flags = ffi::DRM_MODE_CURSOR_BO;
                t.raw_mut_ref().crtc_id = handle.into();
                t.raw_mut_ref().width = size.0;
                t.raw_mut_ref().height = size.1;
                t.raw_mut_ref().hot_x = h.0;
                t.raw_mut_ref().hot_y = h.1;
                // TODO: Get buffer working first
                unimplemented!();
                //t.raw_mut_ref().handle = buffer.???;
                t.ioctl(self.as_raw_fd())?;
            }
        }

        Ok(())
    }

    fn move_cursor(&self, handle: Handle, position: (i32, i32)) -> Result<()> {
        let mut t = ffi::mode::Cursor::default();
        t.raw_mut_ref().flags = ffi::DRM_MODE_CURSOR_MOVE;
        t.raw_mut_ref().crtc_id = handle.into();
        t.raw_mut_ref().x = position.0;
        t.raw_mut_ref().y = position.1;
        t.ioctl(self.as_raw_fd())?;
        Ok(())
    }

    fn clear_cursor(&self, handle: Handle) -> Result<()> {
        let mut t = ffi::mode::Cursor::default();
        t.raw_mut_ref().flags = ffi::DRM_MODE_CURSOR_BO;
        t.raw_mut_ref().crtc_id = handle.into();
        t.ioctl(self.as_raw_fd())?;
        Ok(())
    }

    fn page_flip(&self, handle: Handle, fb: framebuffer::Handle,
                 flags: PageFlipFlag, target: Option<()>) -> Result<()> {

        match target {
            Some(_) => unimplemented!(),
            None => {
                let mut t = ffi::mode::CrtcPageFlip::default();
                t.raw_mut_ref().crtc_id = handle.into();
                t.raw_mut_ref().fb_id = fb.into();
                t.raw_mut_ref().flags = flags as u32;
                t.ioctl(self.as_raw_fd())?;
            }
        }

        Ok(())
    }

    fn get_gamma(&self, _handle: Handle) -> Result<()> {
        unimplemented!();
    }

    fn set_gamma(&self, _handle: Handle) -> Result<()> {
        unimplemented!();
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// Flags to alter the behaviour of a page flip
pub enum PageFlipFlag {
    /// Request a vblank event on page flip
    Event = ffi::DRM_MODE_PAGE_FLIP_EVENT,
    /// Request page flip as soon as possible, not waiting for vblank
    Async = ffi::DRM_MODE_PAGE_FLIP_ASYNC,
}

/*
/// Iterator over `Event`s of a device. Create via `receive_events`.
#[derive(Copy, Clone)]
pub struct Events {
    event_buf: [u8; 1024],
    amount: usize,
    i: usize,
}

/// An event from a device.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Event {
    /// A vblank happened
    Vblank(VblankEvent),
    /// A page flip happened
    PageFlip(PageFlipEvent),
    /// Unknown event, raw data provided
    Unknown(Vec<u8>),
}

/// Vblank event
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct VblankEvent {
    /// sequence of the frame
    pub frame: u32,
    /// duration between events
    pub duration: Duration,
    /// crtc that did throw the event
    pub crtc: Handle,
}

/// Page Flip event
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct PageFlipEvent {
    /// sequence of the frame
    pub frame: u32,
    /// duration between events
    pub duration: Duration,
    /// crtc that did throw the event
    pub crtc: Handle,
}

impl Iterator for Events {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        if self.amount > 0 && self.i < self.amount {
            let event = unsafe {
                &*(self.event_buf.as_ptr().offset(self.i as isize) as *const ffi::drm_event)
            };
            self.i += event.length as usize;
            match event.type_ {
                x if x == ffi::DRM_EVENT_VBLANK => {
                    let vblank_event: &ffi::drm_event_vblank = unsafe { mem::transmute(event) };
                    Some(Event::Vblank(VblankEvent {
                        frame: vblank_event.sequence,
                        duration: Duration::new(
                            vblank_event.tv_sec as u64,
                            vblank_event.tv_usec * 100,
                        ),
                        crtc: Handle::from(vblank_event.user_data as u32),
                    }))
                }
                x if x == ffi::DRM_EVENT_FLIP_COMPLETE => {
                    let vblank_event: &ffi::drm_event_vblank = unsafe { mem::transmute(event) };
                    Some(Event::PageFlip(PageFlipEvent {
                        frame: vblank_event.sequence,
                        duration: Duration::new(
                            vblank_event.tv_sec as u64,
                            vblank_event.tv_usec * 1000,
                        ),
                        crtc: Handle::from(if vblank_event.crtc_id != 0 {
                            vblank_event.crtc_id
                        } else {
                            vblank_event.user_data as u32
                        }),
                    }))
                }
                _ => Some(Event::Unknown(
                    self.event_buf[self.i - (event.length as usize)..self.i].to_vec(),
                )),
            }
        } else {
            None
        }
    }
}

/// Receives all pending events of a given device and returns an Iterator for them.
pub fn receive_events<T>(device: &T) -> Result<Events>
where
    T: control::Device,
{
    struct DeviceWrapper<'a, T: control::Device + 'a>(&'a T);
    impl<'a, T: control::Device> Read for DeviceWrapper<'a, T> {
        fn read(&mut self, buf: &mut [u8]) -> ::std::io::Result<usize> {
            ::nix::unistd::read(self.0.as_raw_fd(), buf).map_err(|err| match err {
                ::nix::Error::Sys(_) => ::std::io::Error::last_os_error(),
                err => ::std::io::Error::new(::std::io::ErrorKind::Other, err),
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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
where
    T: control::Device,
{
    let info = Info::load_from_device(device, handle)?;

    let mut raw: ffi::drm_mode_crtc_lut = Default::default();
    raw.crtc_id = handle.into();
    raw.gamma_size = info.gamma_length;
    let red = ffi_buf!(raw.red, info.gamma_length as usize);
    let green = ffi_buf!(raw.green, info.gamma_length as usize);
    let blue = ffi_buf!(raw.blue, info.gamma_length as usize);

    unsafe {
        try!(ffi::ioctl_mode_getgamma(device.as_raw_fd(), &mut raw));
    }

    Ok(GammaRamp {
        red: red.into_boxed_slice(),
        green: green.into_boxed_slice(),
        blue: blue.into_boxed_slice(),
    })
}

/// Set a gamma ramp for the given crtc
pub fn set_gamma<T>(device: &T, handle: Handle, mut gamma: GammaRamp) -> Result<()>
where
    T: control::Device,
{
    let info = Info::load_from_device(device, handle)?;

    if gamma.red.len() as u32 != info.gamma_length {
        return Err(Error::from_kind(ErrorKind::InvalidGammaSize(
            gamma.red.len(),
            info.gamma_length,
        )));
    }

    if gamma.green.len() as u32 != info.gamma_length {
        return Err(Error::from_kind(
            ErrorKind::InvalidGammaSize(gamma.green.len(), info.gamma_length).into(),
        ));
    }

    if gamma.blue.len() as u32 != info.gamma_length {
        return Err(Error::from_kind(
            ErrorKind::InvalidGammaSize(gamma.blue.len(), info.gamma_length).into(),
        ));
    }

    let mut raw: ffi::drm_mode_crtc_lut = Default::default();
    raw.crtc_id = handle.into();
    raw.gamma_size = info.gamma_length;
    raw.red = gamma.red.as_mut_ptr() as u64;
    raw.green = gamma.green.as_mut_ptr() as u64;
    raw.blue = gamma.blue.as_mut_ptr() as u64;

    unsafe {
        try!(ffi::ioctl_mode_setgamma(device.as_raw_fd(), &mut raw));
    }

    Ok(())
}
*/
