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

use {Dimensions, iPoint};
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
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
    gamma_length: u32,
}

impl ResourceHandle for Handle {
    fn from_raw(raw: control::RawHandle) -> Self {
        Handle(raw)
    }

    fn as_raw(&self) -> control::RawHandle {
        self.0
    }
}

impl control::property::LoadProperties for Handle {
    const TYPE: u32 = ffi::DRM_MODE_OBJECT_CRTC;
}

impl ResourceInfo for Info {
    type Handle = Handle;

    fn load_from_device<T>(device: &T, handle: Handle) -> Result<Self>
    where
        T: control::Device,
    {
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
                gamma_length: raw.gamma_size,
            }
        };

        Ok(crtc)
    }

    fn handle(&self) -> Self::Handle {
        self.handle
    }
}

/// Attaches a framebuffer to a CRTC's built-in plane, attaches the CRTC to
/// a connector, and sets the CRTC's mode to output the pixel data.
pub fn set<T>(
    device: &T,
    handle: Handle,
    fb: FBHandle,
    cons: &[ConHandle],
    position: (u32, u32),
    mode: Option<control::Mode>,
) -> Result<()>
where
    T: control::Device,
{
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
        }
        _ => (),
    };

    unsafe {
        try!(ffi::ioctl_mode_setcrtc(device.as_raw_fd(), &mut raw));
    }

    Ok(())
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageFlipFlags {
    PageFlipEvent = ffi::DRM_MODE_PAGE_FLIP_EVENT,
    PageFlipAsync = ffi::DRM_MODE_PAGE_FLIP_ASYNC,
}

struct FatPtrWrapper(Box<Any>);

pub fn page_flip<T, U>(
    device: &T,
    handle: Handle,
    fb: FBHandle,
    flags: &[PageFlipFlags],
    userdata: U,
) -> Result<()>
where
    T: control::Device,
    U: 'static,
{
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

pub trait VblankHandler<T: control::Device> {
    fn handle_event(&mut self, device: &mut T, frame: u32, duration: Duration, userdata: Box<Any>);
}

impl<T, F> VblankHandler<T> for F
where
    T: control::Device,
    F: FnMut(&mut T, u32, Duration, Box<Any>),
{
    fn handle_event(&mut self, device: &mut T, frame: u32, duration: Duration, userdata: Box<Any>) {
        (*self)(device, frame, duration, userdata)
    }
}

impl<T> VblankHandler<T> for ()
where
    T: control::Device,
{
    fn handle_event(&mut self, _: &mut T, _: u32, _: Duration, _: Box<Any>) {}
}

pub trait PageFlipHandler<T: control::Device> {
    fn handle_event(&mut self, device: &mut T, frame: u32, duration: Duration, userdata: Box<Any>);
}

impl<T, F> PageFlipHandler<T> for F
where
    T: control::Device,
    F: FnMut(&mut T, u32, Duration, Box<Any>),
{
    fn handle_event(&mut self, device: &mut T, frame: u32, duration: Duration, userdata: Box<Any>) {
        (*self)(device, frame, duration, userdata)
    }
}

impl<T> PageFlipHandler<T> for ()
where
    T: control::Device,
{
    fn handle_event(&mut self, _: &mut T, _: u32, _: Duration, _: Box<Any>) {}
}

pub trait PageFlipHandler2<T: control::Device> {
    fn handle_event(
        &mut self,
        device: &mut T,
        frame: u32,
        duration: Duration,
        crtc: Handle,
        userdata: Box<Any>,
    );
}

impl<T, F> PageFlipHandler2<T> for F
where
    T: control::Device,
    F: FnMut(&mut T, u32, Duration, Handle, Box<Any>),
{
    fn handle_event(
        &mut self,
        device: &mut T,
        frame: u32,
        duration: Duration,
        crtc: Handle,
        userdata: Box<Any>,
    ) {
        (*self)(device, frame, duration, crtc, userdata)
    }
}

impl<T> PageFlipHandler2<T> for ()
where
    T: control::Device,
{
    fn handle_event(&mut self, _: &mut T, _: u32, _: Duration, _: Handle, _: Box<Any>) {}
}

pub fn handle_event<T, V, P, P2>(
    device: &mut T,
    version: u32,
    mut vblank_handler: Option<&mut V>,
    mut pageflip_handler: Option<&mut P>,
    mut pageflip_handler2: Option<&mut P2>,
) -> Result<()>
where
    T: control::Device,
    V: VblankHandler<T>,
    P: PageFlipHandler<T>,
    P2: PageFlipHandler2<T>,
{
    struct DeviceWrapper<'a, T: control::Device + 'a>(&'a mut T);
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
    if amount > 0 {
        let mut i = 0isize;
        while i < amount as isize {
            let event = unsafe { &*(event_buf.as_ptr().offset(i) as *const ffi::drm_event) };
            match event.type_ {
                x if x == ffi::DRM_EVENT_VBLANK => if version >= 1 {
                    if let Some(handler) = vblank_handler.as_mut() {
                        let vblank_event: &ffi::drm_event_vblank = unsafe { mem::transmute(event) };
                        let userdata = unsafe {
                            Box::from_raw(vblank_event.user_data as *mut FatPtrWrapper).0
                        };
                        (*handler).handle_event(
                            wrapper.0,
                            vblank_event.sequence,
                            Duration::new(vblank_event.tv_sec as u64, vblank_event.tv_usec * 1000),
                            userdata,
                        );
                    }
                },
                x if x == ffi::DRM_EVENT_FLIP_COMPLETE => {
                    let vblank_event: &ffi::drm_event_vblank = unsafe { mem::transmute(event) };
                    let userdata =
                        unsafe { Box::from_raw(vblank_event.user_data as *mut FatPtrWrapper).0 };
                    if let Some(handler) = pageflip_handler2.as_mut() {
                        if version >= 3 {
                            (*handler).handle_event(
                                wrapper.0,
                                vblank_event.sequence,
                                Duration::new(
                                    vblank_event.tv_sec as u64,
                                    vblank_event.tv_usec * 1000,
                                ),
                                Handle::from_raw(vblank_event.crtc_id),
                                userdata,
                            );
                        }
                    } else if let Some(handler) = pageflip_handler.as_mut() {
                        if version >= 2 {
                            (*handler).handle_event(
                                wrapper.0,
                                vblank_event.sequence,
                                Duration::new(
                                    vblank_event.tv_sec as u64,
                                    vblank_event.tv_usec * 1000,
                                ),
                                userdata,
                            );
                        }
                    }
                }
                _ => {}
            }
            i += event.length as isize;
        }
    }

    Ok(())
}

pub fn set_cursor<T>(
    device: &T,
    handle: Handle,
    bo: buffer::Id,
    dimensions: Dimensions,
) -> Result<()>
where
    T: control::Device,
{
    let mut raw: ffi::drm_mode_cursor = Default::default();
    raw.flags = ffi::DRM_MODE_CURSOR_BO;
    raw.crtc_id = handle.as_raw();
    raw.width = dimensions.0;
    raw.height = dimensions.1;
    raw.handle = bo.as_raw();

    unsafe {
        try!(ffi::ioctl_mode_cursor(device.as_raw_fd(), &mut raw));
    }

    Ok(())
}

pub fn set_cursor2<T>(
    device: &T,
    handle: Handle,
    bo: buffer::Id,
    dimensions: Dimensions,
    hotspot: iPoint,
) -> Result<()>
where
    T: control::Device,
{
    let mut raw: ffi::drm_mode_cursor2 = Default::default();
    raw.flags = ffi::DRM_MODE_CURSOR_BO;
    raw.crtc_id = handle.as_raw();
    raw.width = dimensions.0;
    raw.height = dimensions.1;
    raw.handle = bo.as_raw();
    raw.hot_x = hotspot.0;
    raw.hot_y = hotspot.1;

    unsafe {
        try!(ffi::ioctl_mode_cursor2(device.as_raw_fd(), &mut raw));
    }

    Ok(())
}

pub fn move_cursor<T>(device: &T, handle: Handle, to: iPoint) -> Result<()>
where
    T: control::Device,
{
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
pub struct GammaRamp {
    pub red: Box<[u16]>,
    pub green: Box<[u16]>,
    pub blue: Box<[u16]>,
}

pub fn gamma<T>(device: &T, handle: Handle) -> Result<GammaRamp>
where
    T: control::Device,
{
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
        red: red.into_boxed_slice(),
        green: green.into_boxed_slice(),
        blue: blue.into_boxed_slice(),
    })
}

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

impl ::std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "crtc::Handle({})", self.0)
    }
}
