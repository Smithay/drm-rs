//! Modesetting operations that the DRM subsystem exposes.
//!
//! # Summary
//!
//! The DRM subsystem provides Kernel Modesetting (KMS) functionality by
//! exposing the following resource types:
//!
//! * FrameBuffer - Specific to an individual process, these wrap around generic
//! GPU buffers so that they can be attached to a Plane.
//!
//! * Planes - Dedicated memory objects which contain a buffer that can then be
//! scanned out by a CRTC. There exist a few different types of planes depending
//! on the use case.
//!
//! * CRTC - Scanout engines that read pixel data from a Plane and sends it to
//! a Connector. Each CRTC has at least one Primary Plane.
//!
//! * Connector - Represents the physical output, such as a DisplayPort or
//! VGA connector.
//!
//! * Encoder - Encodes pixel data from a CRTC into something a Connector can
//! understand.
//!
//! Further details on each resource can be found in their respective modules.
//!
//! # Usage
//!
//! To begin using modesetting functionality, the [`Device`] trait
//! must be implemented on top of the basic [`super::Device`] trait.

use drm_ffi as ffi;
use drm_fourcc::{DrmFourcc, DrmModifier, UnrecognizedFourcc};

use bytemuck::allocation::TransparentWrapperAlloc;
use rustix::io::Errno;

pub mod atomic;
pub mod connector;
pub mod crtc;
pub mod dumbbuffer;
pub mod encoder;
pub mod framebuffer;
pub mod plane;
pub mod syncobj;

pub mod property;

use self::dumbbuffer::*;
use crate::buffer;

use super::util::*;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::error;
use std::fmt;
use std::io;
use std::iter::Zip;
use std::mem;
use std::ops::RangeBounds;
use std::os::unix::io::{AsFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};
use std::time::Duration;

use core::num::NonZeroU32;

/// Raw handle for a drm resource
pub type RawResourceHandle = NonZeroU32;

/// Id of a Lease
pub type LeaseId = NonZeroU32;

/// Handle for a drm resource
pub trait ResourceHandle:
    From<RawResourceHandle> + Into<RawResourceHandle> + Into<u32> + Copy + Sized
{
    /// Associated encoded object type
    const FFI_TYPE: u32;
}

/// Convert from a raw drm object value to a typed Handle
///
/// Note: This does no verification on the validity of the original value
pub fn from_u32<T: From<RawResourceHandle>>(raw: u32) -> Option<T> {
    RawResourceHandle::new(raw).map(T::from)
}

/// Error from [`Device::get_planar_framebuffer`]
#[derive(Debug)]
pub enum GetPlanarFramebufferError {
    /// IO error
    Io(io::Error),
    /// Unrecognized fourcc format
    UnrecognizedFourcc(drm_fourcc::UnrecognizedFourcc),
}

impl fmt::Display for GetPlanarFramebufferError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{}", err),
            Self::UnrecognizedFourcc(err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for GetPlanarFramebufferError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::UnrecognizedFourcc(err) => Some(err),
        }
    }
}

impl From<io::Error> for GetPlanarFramebufferError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<UnrecognizedFourcc> for GetPlanarFramebufferError {
    fn from(err: UnrecognizedFourcc) -> Self {
        Self::UnrecognizedFourcc(err)
    }
}

/// This trait should be implemented by any object that acts as a DRM device and
/// provides modesetting functionality.
///
/// Like the parent [`super::Device`] trait, this crate does not
/// provide a concrete object for this trait.
///
/// # Example
/// ```ignore
/// use drm::control::Device as ControlDevice;
///
/// /// Assuming the [`Card`] wrapper already implements [`drm::Device`]
/// impl ControlDevice for Card {}
/// ```
pub trait Device: super::Device {
    /// Gets the set of resource handles that this device currently controls
    fn resource_handles(&self) -> io::Result<ResourceHandles> {
        let mut fbs = Vec::new();
        let mut crtcs = Vec::new();
        let mut connectors = Vec::new();
        let mut encoders = Vec::new();

        let ffi_res = ffi::mode::get_resources(
            self.as_fd(),
            Some(&mut fbs),
            Some(&mut crtcs),
            Some(&mut connectors),
            Some(&mut encoders),
        )?;

        let res = unsafe {
            ResourceHandles {
                fbs: transmute_vec_from_u32(fbs),
                crtcs: transmute_vec_from_u32(crtcs),
                connectors: transmute_vec_from_u32(connectors),
                encoders: transmute_vec_from_u32(encoders),
                width: (ffi_res.min_width, ffi_res.max_width),
                height: (ffi_res.min_height, ffi_res.max_height),
            }
        };

        Ok(res)
    }

    /// Gets the set of plane handles that this device currently has
    fn plane_handles(&self) -> io::Result<Vec<plane::Handle>> {
        let mut planes = Vec::new();
        let _ = ffi::mode::get_plane_resources(self.as_fd(), Some(&mut planes))?;
        Ok(unsafe { transmute_vec_from_u32(planes) })
    }

    /// Returns information about a specific connector
    ///
    /// ## Force-probing
    ///
    /// If `force_probe` is set to `true` and the DRM client is the current DRM master,
    /// the kernel will perform a forced probe on the connector to refresh the connector status, modes and EDID.
    /// A forced-probe can be slow, might cause flickering and the ioctl will block.
    ///
    /// - User needs to force-probe connectors to ensure their metadata is up-to-date at startup and after receiving a hot-plug event.
    /// - User may perform a forced-probe when the user explicitly requests it.
    /// - User shouldn’t perform a forced-probe in other situations.
    fn get_connector(
        &self,
        handle: connector::Handle,
        force_probe: bool,
    ) -> io::Result<connector::Info> {
        // Maximum number of encoders is 3 due to kernel restrictions
        let mut encoders = Vec::new();
        let mut modes = Vec::new();

        let ffi_info = ffi::mode::get_connector(
            self.as_fd(),
            handle.into(),
            None,
            None,
            Some(&mut modes),
            Some(&mut encoders),
            force_probe,
        )?;

        let connector = connector::Info {
            handle,
            interface: connector::Interface::from(ffi_info.connector_type),
            interface_id: ffi_info.connector_type_id,
            connection: connector::State::from(ffi_info.connection),
            size: match (ffi_info.mm_width, ffi_info.mm_height) {
                (0, 0) => None,
                (x, y) => Some((x, y)),
            },
            modes: Mode::wrap_vec(modes),
            encoders: unsafe { transmute_vec_from_u32(encoders) },
            curr_enc: unsafe { mem::transmute(ffi_info.encoder_id) },
            subpixel: connector::SubPixel::from_raw(ffi_info.subpixel),
        };

        Ok(connector)
    }

    /// Returns information about a specific encoder
    fn get_encoder(&self, handle: encoder::Handle) -> io::Result<encoder::Info> {
        let info = ffi::mode::get_encoder(self.as_fd(), handle.into())?;

        let enc = encoder::Info {
            handle,
            enc_type: encoder::Kind::from(info.encoder_type),
            crtc: from_u32(info.crtc_id),
            pos_crtcs: info.possible_crtcs,
            pos_clones: info.possible_clones,
        };

        Ok(enc)
    }

    /// Returns information about a specific CRTC
    fn get_crtc(&self, handle: crtc::Handle) -> io::Result<crtc::Info> {
        let info = ffi::mode::get_crtc(self.as_fd(), handle.into())?;

        let crtc = crtc::Info {
            handle,
            position: (info.x, info.y),
            mode: match info.mode_valid {
                0 => None,
                _ => Some(Mode::from(info.mode)),
            },
            fb: from_u32(info.fb_id),
            gamma_length: info.gamma_size,
        };

        Ok(crtc)
    }

    /// Set CRTC state
    fn set_crtc(
        &self,
        handle: crtc::Handle,
        framebuffer: Option<framebuffer::Handle>,
        pos: (u32, u32),
        conns: &[connector::Handle],
        mode: Option<Mode>,
    ) -> io::Result<()> {
        let _info = ffi::mode::set_crtc(
            self.as_fd(),
            handle.into(),
            framebuffer.map(Into::into).unwrap_or(0),
            pos.0,
            pos.1,
            unsafe { &*(conns as *const _ as *const [u32]) },
            mode.map(|m| m.into()),
        )?;

        Ok(())
    }

    /// Returns information about a specific framebuffer
    fn get_framebuffer(&self, handle: framebuffer::Handle) -> io::Result<framebuffer::Info> {
        let info = ffi::mode::get_framebuffer(self.as_fd(), handle.into())?;

        let fb = framebuffer::Info {
            handle,
            size: (info.width, info.height),
            pitch: info.pitch,
            bpp: info.bpp,
            depth: info.depth,
            buffer: from_u32(info.handle),
        };

        Ok(fb)
    }

    /// Returns information about a specific framebuffer (with modifiers)
    fn get_planar_framebuffer(
        &self,
        handle: framebuffer::Handle,
    ) -> Result<framebuffer::PlanarInfo, GetPlanarFramebufferError> {
        let info = ffi::mode::get_framebuffer2(self.as_fd(), handle.into())?;

        let pixel_format = DrmFourcc::try_from(info.pixel_format)?;

        let flags = FbCmd2Flags::from_bits_truncate(info.flags);
        let modifier = flags
            .contains(FbCmd2Flags::MODIFIERS)
            .then(|| DrmModifier::from(info.modifier[0]));

        let fb = framebuffer::PlanarInfo {
            handle,
            size: (info.width, info.height),
            pixel_format,
            flags,
            buffers: bytemuck::cast(info.handles),
            pitches: info.pitches,
            offsets: info.offsets,
            modifier,
        };

        Ok(fb)
    }

    /// Add a new framebuffer
    fn add_framebuffer<B>(
        &self,
        buffer: &B,
        depth: u32,
        bpp: u32,
    ) -> io::Result<framebuffer::Handle>
    where
        B: buffer::Buffer + ?Sized,
    {
        let (w, h) = buffer.size();
        let info = ffi::mode::add_fb(
            self.as_fd(),
            w,
            h,
            buffer.pitch(),
            bpp,
            depth,
            buffer.handle().into(),
        )?;

        Ok(from_u32(info.fb_id).unwrap())
    }

    /// Add framebuffer (with modifiers)
    fn add_planar_framebuffer<B>(
        &self,
        planar_buffer: &B,
        flags: FbCmd2Flags,
    ) -> io::Result<framebuffer::Handle>
    where
        B: buffer::PlanarBuffer + ?Sized,
    {
        let modifier = planar_buffer
            .modifier()
            .filter(|modifier| !matches!(modifier, DrmModifier::Invalid));
        let has_modifier = flags.contains(FbCmd2Flags::MODIFIERS);
        assert!((has_modifier && modifier.is_some()) || (!has_modifier && modifier.is_none()));
        let modifier = if let Some(modifier) = modifier {
            u64::from(modifier)
        } else {
            0
        };

        let (w, h) = planar_buffer.size();
        let opt_handles = planar_buffer.handles();

        let handles = bytemuck::cast(opt_handles);
        let mods = [
            opt_handles[0].map_or(0, |_| modifier),
            opt_handles[1].map_or(0, |_| modifier),
            opt_handles[2].map_or(0, |_| modifier),
            opt_handles[3].map_or(0, |_| modifier),
        ];

        let info = ffi::mode::add_fb2(
            self.as_fd(),
            w,
            h,
            planar_buffer.format() as u32,
            &handles,
            &planar_buffer.pitches(),
            &planar_buffer.offsets(),
            &mods,
            flags.bits(),
        )?;

        Ok(from_u32(info.fb_id).unwrap())
    }

    /// Mark parts of a framebuffer dirty
    fn dirty_framebuffer(&self, handle: framebuffer::Handle, clips: &[ClipRect]) -> io::Result<()> {
        ffi::mode::dirty_fb(self.as_fd(), handle.into(), unsafe {
            // SAFETY: ClipRect is repr(transparent) for drm_clip_rect
            core::slice::from_raw_parts(clips.as_ptr() as *const ffi::drm_clip_rect, clips.len())
        })?;
        Ok(())
    }

    /// Destroy a framebuffer
    fn destroy_framebuffer(&self, handle: framebuffer::Handle) -> io::Result<()> {
        ffi::mode::rm_fb(self.as_fd(), handle.into())
    }

    /// Returns information about a specific plane
    fn get_plane(&self, handle: plane::Handle) -> io::Result<plane::Info> {
        let mut formats = Vec::new();

        let info = ffi::mode::get_plane(self.as_fd(), handle.into(), Some(&mut formats))?;

        let plane = plane::Info {
            handle,
            crtc: from_u32(info.crtc_id),
            fb: from_u32(info.fb_id),
            pos_crtcs: info.possible_crtcs,
            formats: unsafe { transmute_vec_from_u32(formats) },
        };

        Ok(plane)
    }

    /// Set plane state.
    ///
    /// Providing no framebuffer clears the plane.
    fn set_plane(
        &self,
        handle: plane::Handle,
        crtc: crtc::Handle,
        framebuffer: Option<framebuffer::Handle>,
        flags: u32,
        crtc_rect: (i32, i32, u32, u32),
        src_rect: (u32, u32, u32, u32),
    ) -> io::Result<()> {
        let _info = ffi::mode::set_plane(
            self.as_fd(),
            handle.into(),
            crtc.into(),
            framebuffer.map(Into::into).unwrap_or(0),
            flags,
            crtc_rect.0,
            crtc_rect.1,
            crtc_rect.2,
            crtc_rect.3,
            src_rect.0,
            src_rect.1,
            src_rect.2,
            src_rect.3,
        )?;

        Ok(())
    }

    /// Returns information about a specific property.
    fn get_property(&self, handle: property::Handle) -> io::Result<property::Info> {
        let mut values = Vec::new();
        let mut enums = Vec::new();

        let info = ffi::mode::get_property(
            self.as_fd(),
            handle.into(),
            Some(&mut values),
            Some(&mut enums),
        )?;

        let flags = ModePropFlags::from_bits_truncate(info.flags);

        let val_type = {
            use self::property::ValueType;

            if flags.contains(ModePropFlags::RANGE) {
                let min = values[0];
                let max = values[1];

                match (min, max) {
                    (0, 1) => ValueType::Boolean,
                    (min, max) => ValueType::UnsignedRange(min, max),
                }
            } else if flags.contains(ModePropFlags::SIGNED_RANGE) {
                let min = values[0];
                let max = values[1];

                ValueType::SignedRange(min as i64, max as i64)
            } else if flags.contains(ModePropFlags::ENUM) {
                let enum_values = self::property::EnumValues {
                    values,
                    enums: property::EnumValue::wrap_vec(enums),
                };

                ValueType::Enum(enum_values)
            } else if flags.contains(ModePropFlags::BLOB) {
                ValueType::Blob
            } else if flags.contains(ModePropFlags::BITMASK) {
                ValueType::Bitmask
            } else if flags.contains(ModePropFlags::OBJECT) {
                match values[0] as u32 {
                    ffi::DRM_MODE_OBJECT_CRTC => ValueType::CRTC,
                    ffi::DRM_MODE_OBJECT_CONNECTOR => ValueType::Connector,
                    ffi::DRM_MODE_OBJECT_ENCODER => ValueType::Encoder,
                    ffi::DRM_MODE_OBJECT_FB => ValueType::Framebuffer,
                    ffi::DRM_MODE_OBJECT_PLANE => ValueType::Plane,
                    ffi::DRM_MODE_OBJECT_PROPERTY => ValueType::Property,
                    ffi::DRM_MODE_OBJECT_BLOB => ValueType::Blob,
                    ffi::DRM_MODE_OBJECT_ANY => ValueType::Object,
                    _ => ValueType::Unknown,
                }
            } else {
                ValueType::Unknown
            }
        };

        let property = property::Info {
            handle,
            val_type,
            mutable: !flags.contains(ModePropFlags::IMMUTABLE),
            atomic: flags.contains(ModePropFlags::ATOMIC),
            info,
        };

        Ok(property)
    }

    /// Sets a property for a specific resource.
    fn set_property<T: ResourceHandle>(
        &self,
        handle: T,
        prop: property::Handle,
        value: property::RawValue,
    ) -> io::Result<()> {
        ffi::mode::set_property(self.as_fd(), prop.into(), handle.into(), T::FFI_TYPE, value)?;

        Ok(())
    }

    /// Create a property blob value from a given data blob
    fn create_property_blob<T>(&self, data: &T) -> io::Result<property::Value<'static>> {
        let data = unsafe {
            std::slice::from_raw_parts_mut(data as *const _ as *mut u8, mem::size_of::<T>())
        };
        let blob = ffi::mode::create_property_blob(self.as_fd(), data)?;

        Ok(property::Value::Blob(blob.blob_id.into()))
    }

    /// Get a property blob's data
    fn get_property_blob(&self, blob: u64) -> io::Result<Vec<u8>> {
        let mut data = Vec::new();
        let _ = ffi::mode::get_property_blob(self.as_fd(), blob as u32, Some(&mut data))?;
        Ok(data)
    }

    /// Destroy a given property blob value
    fn destroy_property_blob(&self, blob: u64) -> io::Result<()> {
        ffi::mode::destroy_property_blob(self.as_fd(), blob as u32)?;

        Ok(())
    }

    /// Returns the set of [`Mode`]s that a particular connector supports.
    fn get_modes(&self, handle: connector::Handle) -> io::Result<Vec<Mode>> {
        let mut modes = Vec::new();

        let _ffi_info = ffi::mode::get_connector(
            self.as_fd(),
            handle.into(),
            None,
            None,
            Some(&mut modes),
            None,
            false,
        )?;

        Ok(Mode::wrap_vec(modes))
    }

    /// Gets a list of property handles and values for this resource.
    fn get_properties<T: ResourceHandle>(&self, handle: T) -> io::Result<PropertyValueSet> {
        let mut prop_ids = Vec::new();
        let mut prop_vals = Vec::new();

        ffi::mode::get_properties(
            self.as_fd(),
            handle.into(),
            T::FFI_TYPE,
            Some(&mut prop_ids),
            Some(&mut prop_vals),
        )?;

        let prop_val_set = PropertyValueSet {
            prop_ids: unsafe { transmute_vec_from_u32(prop_ids) },
            prop_vals,
        };

        Ok(prop_val_set)
    }

    /// Receive the currently set gamma ramp of a crtc
    fn get_gamma(
        &self,
        crtc: crtc::Handle,
        red: &mut [u16],
        green: &mut [u16],
        blue: &mut [u16],
    ) -> io::Result<()> {
        let crtc_info = self.get_crtc(crtc)?;
        if crtc_info.gamma_length as usize > red.len()
            || crtc_info.gamma_length as usize > green.len()
            || crtc_info.gamma_length as usize > blue.len()
        {
            return Err(Errno::INVAL.into());
        }

        ffi::mode::get_gamma(
            self.as_fd(),
            crtc.into(),
            crtc_info.gamma_length as usize,
            red,
            green,
            blue,
        )?;

        Ok(())
    }

    /// Set a gamma ramp for the given crtc
    fn set_gamma(
        &self,
        crtc: crtc::Handle,
        red: &[u16],
        green: &[u16],
        blue: &[u16],
    ) -> io::Result<()> {
        let crtc_info = self.get_crtc(crtc)?;
        if crtc_info.gamma_length as usize > red.len()
            || crtc_info.gamma_length as usize > green.len()
            || crtc_info.gamma_length as usize > blue.len()
        {
            return Err(Errno::INVAL.into());
        }

        ffi::mode::set_gamma(
            self.as_fd(),
            crtc.into(),
            crtc_info.gamma_length as usize,
            red,
            green,
            blue,
        )?;

        Ok(())
    }

    /// Open a GEM buffer handle by name
    fn open_buffer(&self, name: buffer::Name) -> io::Result<buffer::Handle> {
        let info = drm_ffi::gem::open(self.as_fd(), name.into())?;
        Ok(from_u32(info.handle).unwrap())
    }

    /// Close a GEM buffer handle
    fn close_buffer(&self, handle: buffer::Handle) -> io::Result<()> {
        let _info = drm_ffi::gem::close(self.as_fd(), handle.into())?;
        Ok(())
    }

    /// Create a new dumb buffer with a given size and pixel format
    fn create_dumb_buffer(
        &self,
        size: (u32, u32),
        format: buffer::DrmFourcc,
        bpp: u32,
    ) -> io::Result<DumbBuffer> {
        let info = drm_ffi::mode::dumbbuffer::create(self.as_fd(), size.0, size.1, bpp, 0)?;

        let dumb = DumbBuffer {
            size: (info.width, info.height),
            length: info.size as usize,
            format,
            pitch: info.pitch,
            handle: from_u32(info.handle).unwrap(),
        };

        Ok(dumb)
    }
    /// Map the buffer for access
    fn map_dumb_buffer<'a>(&self, buffer: &'a mut DumbBuffer) -> io::Result<DumbMapping<'a>> {
        let info = drm_ffi::mode::dumbbuffer::map(self.as_fd(), buffer.handle.into(), 0, 0)?;

        let map = {
            use rustix::mm;
            let prot = mm::ProtFlags::READ | mm::ProtFlags::WRITE;
            let flags = mm::MapFlags::SHARED;
            let fd = self.as_fd();
            let offset = info.offset as _;
            unsafe { mm::mmap(std::ptr::null_mut(), buffer.length, prot, flags, fd, offset)? }
        };

        let mapping = DumbMapping {
            _phantom: std::marker::PhantomData,
            map: unsafe { std::slice::from_raw_parts_mut(map as *mut _, buffer.length) },
        };

        Ok(mapping)
    }

    /// Free the memory resources of a dumb buffer
    fn destroy_dumb_buffer(&self, buffer: DumbBuffer) -> io::Result<()> {
        let _info = drm_ffi::mode::dumbbuffer::destroy(self.as_fd(), buffer.handle.into())?;

        Ok(())
    }

    /// Sets a hardware-cursor on the given crtc with the image of a given buffer
    ///
    /// A buffer argument of [`None`] will clear the cursor.
    #[deprecated(note = "Usage of deprecated ioctl set_cursor: use a cursor plane instead")]
    #[allow(deprecated)]
    fn set_cursor<B>(&self, crtc: crtc::Handle, buffer: Option<&B>) -> io::Result<()>
    where
        B: buffer::Buffer + ?Sized,
    {
        let (id, w, h) = buffer
            .map(|buf| {
                let (w, h) = buf.size();
                (buf.handle().into(), w, h)
            })
            .unwrap_or((0, 0, 0));
        drm_ffi::mode::set_cursor(self.as_fd(), crtc.into(), id, w, h)?;

        Ok(())
    }

    /// Sets a hardware-cursor on the given crtc with the image of a given buffer
    /// and a hotspot marking the click point of the cursor.
    ///
    /// A buffer argument of [`None`] will clear the cursor.
    #[deprecated(note = "Usage of deprecated ioctl set_cursor2: use a cursor plane instead")]
    #[allow(deprecated)]
    fn set_cursor2<B>(
        &self,
        crtc: crtc::Handle,
        buffer: Option<&B>,
        hotspot: (i32, i32),
    ) -> io::Result<()>
    where
        B: buffer::Buffer + ?Sized,
    {
        let (id, w, h) = buffer
            .map(|buf| {
                let (w, h) = buf.size();
                (buf.handle().into(), w, h)
            })
            .unwrap_or((0, 0, 0));
        drm_ffi::mode::set_cursor2(self.as_fd(), crtc.into(), id, w, h, hotspot.0, hotspot.1)?;

        Ok(())
    }

    /// Moves a set cursor on a given crtc
    #[deprecated(note = "Usage of deprecated ioctl move_cursor: use a cursor plane instead")]
    #[allow(deprecated)]
    fn move_cursor(&self, crtc: crtc::Handle, pos: (i32, i32)) -> io::Result<()> {
        drm_ffi::mode::move_cursor(self.as_fd(), crtc.into(), pos.0, pos.1)?;

        Ok(())
    }

    /// Request an atomic commit with given flags and property-value pair for a list of objects.
    fn atomic_commit(
        &self,
        flags: AtomicCommitFlags,
        mut req: atomic::AtomicModeReq,
    ) -> io::Result<()> {
        drm_ffi::mode::atomic_commit(
            self.as_fd(),
            flags.bits(),
            unsafe { &mut *(&mut *req.objects as *mut _ as *mut [u32]) },
            &mut req.count_props_per_object,
            unsafe { &mut *(&mut *req.props as *mut _ as *mut [u32]) },
            &mut req.values,
        )
    }

    /// Convert a prime file descriptor to a GEM buffer handle
    fn prime_fd_to_buffer(&self, fd: BorrowedFd<'_>) -> io::Result<buffer::Handle> {
        let info = ffi::gem::fd_to_handle(self.as_fd(), fd)?;
        Ok(from_u32(info.handle).unwrap())
    }

    /// Convert a GEM buffer handle to a prime file descriptor
    fn buffer_to_prime_fd(&self, handle: buffer::Handle, flags: u32) -> io::Result<OwnedFd> {
        let info = ffi::gem::handle_to_fd(self.as_fd(), handle.into(), flags)?;
        Ok(unsafe { OwnedFd::from_raw_fd(info.fd) })
    }

    /// Queue a page flip on the given crtc
    fn page_flip(
        &self,
        handle: crtc::Handle,
        framebuffer: framebuffer::Handle,
        flags: PageFlipFlags,
        target_sequence: Option<PageFlipTarget>,
    ) -> io::Result<()> {
        let mut flags = flags.bits();

        let sequence = match target_sequence {
            Some(PageFlipTarget::Absolute(n)) => {
                flags |= ffi::drm_sys::DRM_MODE_PAGE_FLIP_TARGET_ABSOLUTE;
                n
            }
            Some(PageFlipTarget::Relative(n)) => {
                flags |= ffi::drm_sys::DRM_MODE_PAGE_FLIP_TARGET_RELATIVE;
                n
            }
            None => 0,
        };

        ffi::mode::page_flip(
            self.as_fd(),
            handle.into(),
            framebuffer.into(),
            flags,
            sequence,
        )?;

        Ok(())
    }

    /// Creates a syncobj.
    fn create_syncobj(&self, signalled: bool) -> io::Result<syncobj::Handle> {
        let info = ffi::syncobj::create(self.as_fd(), signalled)?;
        Ok(from_u32(info.handle).unwrap())
    }

    /// Destroys a syncobj.
    fn destroy_syncobj(&self, handle: syncobj::Handle) -> io::Result<()> {
        ffi::syncobj::destroy(self.as_fd(), handle.into())?;
        Ok(())
    }

    /// Exports a syncobj as an inter-process file descriptor or as a poll()-able sync file.
    fn syncobj_to_fd(
        &self,
        handle: syncobj::Handle,
        export_sync_file: bool,
    ) -> io::Result<OwnedFd> {
        let info = ffi::syncobj::handle_to_fd(self.as_fd(), handle.into(), export_sync_file)?;
        Ok(unsafe { OwnedFd::from_raw_fd(info.fd) })
    }

    /// Imports a file descriptor exported by [`Self::syncobj_to_fd`] back into a process-local handle.
    fn fd_to_syncobj(
        &self,
        fd: BorrowedFd<'_>,
        import_sync_file: bool,
    ) -> io::Result<syncobj::Handle> {
        let info = ffi::syncobj::fd_to_handle(self.as_fd(), fd, import_sync_file)?;
        Ok(from_u32(info.handle).unwrap())
    }

    /// Waits for one or more syncobjs to become signalled.
    fn syncobj_wait(
        &self,
        handles: &[syncobj::Handle],
        timeout_nsec: i64,
        wait_all: bool,
        wait_for_submit: bool,
    ) -> io::Result<u32> {
        let info = ffi::syncobj::wait(
            self.as_fd(),
            bytemuck::cast_slice(handles),
            timeout_nsec,
            wait_all,
            wait_for_submit,
        )?;
        Ok(info.first_signaled)
    }

    /// Resets (un-signals) one or more syncobjs.
    fn syncobj_reset(&self, handles: &[syncobj::Handle]) -> io::Result<()> {
        ffi::syncobj::reset(self.as_fd(), bytemuck::cast_slice(handles))?;
        Ok(())
    }

    /// Signals one or more syncobjs.
    fn syncobj_signal(&self, handles: &[syncobj::Handle]) -> io::Result<()> {
        ffi::syncobj::signal(self.as_fd(), bytemuck::cast_slice(handles))?;
        Ok(())
    }

    /// Waits for one or more specific timeline syncobj points.
    fn syncobj_timeline_wait(
        &self,
        handles: &[syncobj::Handle],
        points: &[u64],
        timeout_nsec: i64,
        wait_all: bool,
        wait_for_submit: bool,
        wait_available: bool,
    ) -> io::Result<u32> {
        let info = ffi::syncobj::timeline_wait(
            self.as_fd(),
            bytemuck::cast_slice(handles),
            points,
            timeout_nsec,
            wait_all,
            wait_for_submit,
            wait_available,
        )?;
        Ok(info.first_signaled)
    }

    /// Queries for state of one or more timeline syncobjs.
    fn syncobj_timeline_query(
        &self,
        handles: &[syncobj::Handle],
        points: &mut [u64],
        last_submitted: bool,
    ) -> io::Result<()> {
        ffi::syncobj::query(
            self.as_fd(),
            bytemuck::cast_slice(handles),
            points,
            last_submitted,
        )?;
        Ok(())
    }

    /// Transfers one timeline syncobj point to another.
    fn syncobj_timeline_transfer(
        &self,
        src_handle: syncobj::Handle,
        dst_handle: syncobj::Handle,
        src_point: u64,
        dst_point: u64,
    ) -> io::Result<()> {
        ffi::syncobj::transfer(
            self.as_fd(),
            src_handle.into(),
            dst_handle.into(),
            src_point,
            dst_point,
        )?;
        Ok(())
    }

    /// Signals one or more specific timeline syncobj points.
    fn syncobj_timeline_signal(
        &self,
        handles: &[syncobj::Handle],
        points: &[u64],
    ) -> io::Result<()> {
        ffi::syncobj::timeline_signal(self.as_fd(), bytemuck::cast_slice(handles), points)?;
        Ok(())
    }

    /// Register an eventfd to be signalled by a syncobj.
    fn syncobj_eventfd(
        &self,
        handle: syncobj::Handle,
        point: u64,
        eventfd: BorrowedFd<'_>,
        wait_available: bool,
    ) -> io::Result<()> {
        ffi::syncobj::eventfd(self.as_fd(), handle.into(), point, eventfd, wait_available)?;
        Ok(())
    }

    /// Create a drm lease
    fn create_lease(
        &self,
        objects: &[RawResourceHandle],
        flags: u32,
    ) -> io::Result<(LeaseId, OwnedFd)> {
        let lease = ffi::mode::create_lease(self.as_fd(), bytemuck::cast_slice(objects), flags)?;
        Ok((
            unsafe { NonZeroU32::new_unchecked(lease.lessee_id) },
            unsafe { OwnedFd::from_raw_fd(lease.fd as RawFd) },
        ))
    }

    /// List active lessees
    fn list_lessees(&self) -> io::Result<Vec<LeaseId>> {
        let mut lessees = Vec::new();
        ffi::mode::list_lessees(self.as_fd(), Some(&mut lessees))?;
        Ok(unsafe { transmute_vec_from_u32(lessees) })
    }

    /// Revoke a previously issued drm lease
    fn revoke_lease(&self, lessee_id: LeaseId) -> io::Result<()> {
        ffi::mode::revoke_lease(self.as_fd(), lessee_id.get())
    }

    /// Receive pending events
    fn receive_events(&self) -> io::Result<Events>
    where
        Self: Sized,
    {
        let mut event_buf: [u8; 1024] = [0; 1024];
        let amount = rustix::io::read(self.as_fd(), &mut event_buf)?;

        Ok(Events::with_event_buf(event_buf, amount))
    }
}

/// List of leased resources
pub struct LeaseResources {
    /// leased crtcs
    pub crtcs: Vec<crtc::Handle>,
    /// leased connectors
    pub connectors: Vec<connector::Handle>,
    /// leased planes
    pub planes: Vec<plane::Handle>,
}

/// Query lease resources
pub fn get_lease<D: AsFd>(lease: D) -> io::Result<LeaseResources> {
    let mut crtcs = Vec::new();
    let mut connectors = Vec::new();
    let mut planes = Vec::new();
    let mut objects = Vec::new();

    ffi::mode::get_lease(lease.as_fd(), Some(&mut objects))?;

    let _ = ffi::mode::get_resources(
        lease.as_fd(),
        None,
        Some(&mut crtcs),
        Some(&mut connectors),
        None,
    )?;
    let _ = ffi::mode::get_plane_resources(lease.as_fd(), Some(&mut planes))?;

    unsafe {
        Ok(LeaseResources {
            crtcs: transmute_vec_from_u32::<crtc::Handle>(
                crtcs
                    .into_iter()
                    .filter(|handle| objects.contains(handle))
                    .collect(),
            ),
            connectors: transmute_vec_from_u32::<connector::Handle>(
                connectors
                    .into_iter()
                    .filter(|handle| objects.contains(handle))
                    .collect(),
            ),
            planes: transmute_vec_from_u32::<plane::Handle>(
                planes
                    .into_iter()
                    .filter(|handle| objects.contains(handle))
                    .collect(),
            ),
        })
    }
}

bitflags::bitflags! {
    /// Flags to alter the behaviour of a page flip
    ///
    /// Limited to the values in [`ffi::drm_sys::DRM_MODE_PAGE_FLIP_FLAGS`],
    /// minus [`ffi::drm_sys::DRM_MODE_PAGE_FLIP_TARGET`] bits which are
    /// passed through [`PageFlipTarget`].
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct PageFlipFlags : u32 {
        /// Request a vblank event on page flip
        const EVENT = ffi::drm_sys::DRM_MODE_PAGE_FLIP_EVENT;
        /// Request page flip as soon as possible, not waiting for vblank
        const ASYNC = ffi::drm_sys::DRM_MODE_PAGE_FLIP_ASYNC;
    }
}

/// Target to alter the sequence of page flips
///
/// These represent the [`ffi::drm_sys::DRM_MODE_PAGE_FLIP_TARGET`] bits
/// of [`PageFlipFlags`] wrapped in a regular `enum` due to their
/// mutual-exclusiveness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageFlipTarget {
    /// Absolute Vblank Sequence
    Absolute(u32),
    /// Relative Vblank Sequence (to the current, when calling)
    Relative(u32),
}

/// Iterator over [`Event`]s of a device. Create via [`Device::receive_events()`].
pub struct Events {
    event_buf: [u8; 1024],
    amount: usize,
    i: usize,
}

impl Events {
    /// Create [`Event`]s iterator from buffer read using something other than
    /// [`Device::receive_events()`].
    pub fn with_event_buf(event_buf: [u8; 1024], amount: usize) -> Self {
        Events {
            event_buf,
            amount,
            i: 0,
        }
    }
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
    /// time at which the vblank occurred
    pub time: Duration,
    /// crtc that did throw the event
    pub crtc: crtc::Handle,
    /// user data that was passed to wait_vblank
    pub user_data: usize,
}

/// Page Flip event
pub struct PageFlipEvent {
    /// sequence of the frame
    pub frame: u32,
    /// duration between events
    pub duration: Duration,
    /// crtc that did throw the event
    pub crtc: crtc::Handle,
}

impl Iterator for Events {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        if self.amount > 0 && self.i < self.amount {
            let event_ptr = unsafe { self.event_buf.as_ptr().add(self.i) as *const ffi::drm_event };
            let event = unsafe { std::ptr::read_unaligned(event_ptr) };
            self.i += event.length as usize;
            match event.type_ {
                ffi::DRM_EVENT_VBLANK => {
                    let vblank_event = unsafe {
                        std::ptr::read_unaligned(event_ptr as *const ffi::drm_event_vblank)
                    };
                    Some(Event::Vblank(VblankEvent {
                        frame: vblank_event.sequence,
                        time: Duration::new(
                            vblank_event.tv_sec as u64,
                            vblank_event.tv_usec * 1000,
                        ),
                        #[allow(clippy::unnecessary_cast)]
                        crtc: from_u32(vblank_event.crtc_id as u32).unwrap(),
                        user_data: vblank_event.user_data as usize,
                    }))
                }
                ffi::DRM_EVENT_FLIP_COMPLETE => {
                    let vblank_event = unsafe {
                        std::ptr::read_unaligned(event_ptr as *const ffi::drm_event_vblank)
                    };
                    Some(Event::PageFlip(PageFlipEvent {
                        frame: vblank_event.sequence,
                        duration: Duration::new(
                            vblank_event.tv_sec as u64,
                            vblank_event.tv_usec * 1000,
                        ),
                        crtc: from_u32(if vblank_event.crtc_id != 0 {
                            vblank_event.crtc_id
                        } else {
                            vblank_event.user_data as u32
                        })
                        .unwrap(),
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

/// The set of [`ResourceHandles`] that a
/// [`Device`] exposes. Excluding Plane resources.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ResourceHandles {
    /// Set of [`framebuffer::Handle`]
    pub fbs: Vec<framebuffer::Handle>,
    /// Set of [`crtc::Handle`]
    pub crtcs: Vec<crtc::Handle>,
    /// Set of [`connector::Handle`]
    pub connectors: Vec<connector::Handle>,
    /// Set of [`encoder::Handle`]
    pub encoders: Vec<encoder::Handle>,
    width: (u32, u32),
    height: (u32, u32),
}

impl ResourceHandles {
    /// Returns the set of [`connector::Handle`]
    pub fn connectors(&self) -> &[connector::Handle] {
        &self.connectors
    }

    /// Returns the set of [`encoder::Handle`]
    pub fn encoders(&self) -> &[encoder::Handle] {
        &self.encoders
    }

    /// Returns the set of [`crtc::Handle`]
    pub fn crtcs(&self) -> &[crtc::Handle] {
        &self.crtcs
    }

    /// Returns the set of [`framebuffer::Handle`]
    pub fn framebuffers(&self) -> &[framebuffer::Handle] {
        &self.fbs
    }

    /// Returns the supported minimum and maximum width for framebuffers
    pub fn supported_fb_width(&self) -> impl RangeBounds<u32> {
        self.width.0..=self.width.1
    }

    /// Returns the supported minimum and maximum height for framebuffers
    pub fn supported_fb_height(&self) -> impl RangeBounds<u32> {
        self.height.0..=self.height.1
    }

    /// Apply a filter the all crtcs of these resources, resulting in a list of crtcs allowed.
    pub fn filter_crtcs(&self, filter: CrtcListFilter) -> Vec<crtc::Handle> {
        self.crtcs
            .iter()
            .enumerate()
            .filter(|&(n, _)| (1 << n) & filter.0 != 0)
            .map(|(_, &e)| e)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A filter that can be used with a [`ResourceHandles`] to determine the set of
/// Crtcs that can attach to a specific encoder.
pub struct CrtcListFilter(u32);

/// Resolution and timing information for a display mode.
#[repr(transparent)]
#[derive(Copy, Clone, Hash, PartialEq, Eq, bytemuck::TransparentWrapper)]
pub struct Mode {
    // We're using the FFI struct because the DRM API expects it when giving it
    // to a CRTC or creating a blob from it. Rather than rearranging the fields
    // to convert to/from an abstracted type, just use the raw object.
    mode: ffi::drm_mode_modeinfo,
}

impl Mode {
    /// Returns the name of this mode.
    pub fn name(&self) -> &std::ffi::CStr {
        unsafe { std::ffi::CStr::from_ptr(&self.mode.name[0] as _) }
    }

    /// Returns the clock speed of this mode.
    pub fn clock(&self) -> u32 {
        self.mode.clock
    }

    /// Returns the size (resolution) of the mode.
    pub fn size(&self) -> (u16, u16) {
        (self.mode.hdisplay, self.mode.vdisplay)
    }

    /// Returns the horizontal sync start, end, and total.
    pub fn hsync(&self) -> (u16, u16, u16) {
        (self.mode.hsync_start, self.mode.hsync_end, self.mode.htotal)
    }

    /// Returns the vertical sync start, end, and total.
    pub fn vsync(&self) -> (u16, u16, u16) {
        (self.mode.vsync_start, self.mode.vsync_end, self.mode.vtotal)
    }

    /// Returns the horizontal skew of this mode.
    pub fn hskew(&self) -> u16 {
        self.mode.hskew
    }

    /// Returns the vertical scan of this mode.
    pub fn vscan(&self) -> u16 {
        self.mode.vscan
    }

    /// Returns the vertical refresh rate of this mode
    pub fn vrefresh(&self) -> u32 {
        self.mode.vrefresh
    }

    /// Returns the bitmask of this mode
    pub fn mode_type(&self) -> ModeTypeFlags {
        ModeTypeFlags::from_bits_truncate(self.mode.type_)
    }

    /// Returns the flags of this mode
    pub fn flags(&self) -> ModeFlags {
        ModeFlags::from_bits_truncate(self.mode.flags)
    }
}

impl From<ffi::drm_mode_modeinfo> for Mode {
    fn from(raw: ffi::drm_mode_modeinfo) -> Mode {
        Mode { mode: raw }
    }
}

impl From<Mode> for ffi::drm_mode_modeinfo {
    fn from(mode: Mode) -> Self {
        mode.mode
    }
}

impl fmt::Debug for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Mode")
            .field("name", &self.name())
            .field("clock", &self.clock())
            .field("size", &self.size())
            .field("hsync", &self.hsync())
            .field("vsync", &self.vsync())
            .field("hskew", &self.hskew())
            .field("vscan", &self.vscan())
            .field("vrefresh", &self.vrefresh())
            .field("mode_type", &self.mode_type())
            .finish()
    }
}

bitflags::bitflags! {
    /// Display mode type flags
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ModeTypeFlags : u32 {
        /// Builtin mode type
        #[deprecated]
        const BUILTIN = ffi::DRM_MODE_TYPE_BUILTIN;
        /// CLOCK_C mode type
        #[deprecated]
        const CLOCK_C = ffi::DRM_MODE_TYPE_CLOCK_C;
        /// CRTC_C mode type
        #[deprecated]
        const CRTC_C = ffi::DRM_MODE_TYPE_CRTC_C;
        /// Preferred mode
        const PREFERRED = ffi::DRM_MODE_TYPE_PREFERRED;
        /// Default mode
        #[deprecated]
        const DEFAULT = ffi::DRM_MODE_TYPE_DEFAULT;
        /// User defined mode type
        const USERDEF = ffi::DRM_MODE_TYPE_USERDEF;
        /// Mode created by driver
        const DRIVER = ffi::DRM_MODE_TYPE_DRIVER;
        /// Bitmask of all valid (non-deprecated) mode type flags
        const ALL = ffi::DRM_MODE_TYPE_ALL;
    }
}

bitflags::bitflags! {
    /// Display mode flags
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ModeFlags: u32 {
        /// PHSYNC flag
        const PHSYNC = ffi::DRM_MODE_FLAG_PHSYNC;
        /// NHSYNC flag
        const NHSYNC = ffi::DRM_MODE_FLAG_NHSYNC;
        /// PVSYNC flag
        const PVSYNC = ffi::DRM_MODE_FLAG_PVSYNC;
        /// NVSYNC flag
        const NVSYNC = ffi::DRM_MODE_FLAG_NVSYNC;
        /// Interlace flag
        const INTERLACE = ffi::DRM_MODE_FLAG_INTERLACE;
        /// DBLSCAN flag
        const DBLSCAN = ffi::DRM_MODE_FLAG_DBLSCAN;
        /// CSYNC flag
        const CSYNC = ffi::DRM_MODE_FLAG_CSYNC;
        /// PCSYNC flag
        const PCSYNC = ffi::DRM_MODE_FLAG_PCSYNC;
        /// NCSYNC flag
        const NCSYNC = ffi::DRM_MODE_FLAG_NCSYNC;
        /// HSKEW flag
        const HSKEW = ffi::DRM_MODE_FLAG_HSKEW;
        #[deprecated]
        /// BCAST flag
        const BCAST = ffi::DRM_MODE_FLAG_BCAST;
        #[deprecated]
        /// PIXMUX flag
        const PIXMUX = ffi::DRM_MODE_FLAG_PIXMUX;
        /// DBLCLK flag
        const DBLCLK = ffi::DRM_MODE_FLAG_DBLCLK;
        /// CLKDIV2 flag
        const CLKDIV2 = ffi::DRM_MODE_FLAG_CLKDIV2;
        /// Stereo 3D mode utilizing frame packing
        const _3D_FRAME_PACKING = ffi::DRM_MODE_FLAG_3D_FRAME_PACKING;
        /// Stereo 3D mode utilizing alternating fields
        const _3D_FIELD_ALTERNATIVE = ffi::DRM_MODE_FLAG_3D_FIELD_ALTERNATIVE;
        /// Stereo 3D mode utilizing alternating lines
        const _3D_LINE_ALTERNATIVE = ffi::DRM_MODE_FLAG_3D_LINE_ALTERNATIVE;
        /// Stereo 3D mode utilizing side by side full size image
        const _3D_SIDE_BY_SIDE_FULL = ffi::DRM_MODE_FLAG_3D_SIDE_BY_SIDE_FULL;
        /// Stereo 3D mode utilizing depth images
        const _3D_L_DEPTH = ffi::DRM_MODE_FLAG_3D_L_DEPTH;
        /// Stereo 3D mode utilizing depth images
        const _3D_L_DEPTH_GFX_GFX_DEPTH = ffi::DRM_MODE_FLAG_3D_L_DEPTH_GFX_GFX_DEPTH;
        /// Stereo 3D mode utilizing top and bottom images
        const _3D_TOP_AND_BOTTOM = ffi::DRM_MODE_FLAG_3D_TOP_AND_BOTTOM;
        /// Stereo 3D mode utilizing side by side half size image
        const _3D_SIDE_BY_SIDE_HALF = ffi::DRM_MODE_FLAG_3D_SIDE_BY_SIDE_HALF;
    }
}

/// Type of a plane
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaneType {
    /// Overlay plane
    Overlay = ffi::DRM_PLANE_TYPE_OVERLAY,
    /// Primary plane
    Primary = ffi::DRM_PLANE_TYPE_PRIMARY,
    /// Cursor plane
    Cursor = ffi::DRM_PLANE_TYPE_CURSOR,
}

/// Wrapper around a set of property IDs and their raw values.
#[derive(Debug, Clone)]
pub struct PropertyValueSet {
    prop_ids: Vec<property::Handle>,
    prop_vals: Vec<property::RawValue>,
}

impl PropertyValueSet {
    /// Returns a HashMap mapping property names to info
    pub fn as_hashmap(&self, device: &impl Device) -> io::Result<HashMap<String, property::Info>> {
        let mut map = HashMap::new();
        for id in self.prop_ids.iter() {
            let info = device.get_property(*id)?;
            let name = info.name().to_str().unwrap().to_owned();
            map.insert(name, info);
        }
        Ok(map)
    }

    /// Returns a pair representing a set of [`property::Handle`] and their raw values
    pub fn as_props_and_values(&self) -> (&[property::Handle], &[property::RawValue]) {
        (&self.prop_ids, &self.prop_vals)
    }

    /// Returns iterator over pairs representing a set of [`property::Handle`] and their raw values
    pub fn iter(&self) -> impl Iterator<Item = (&property::Handle, &property::RawValue)> {
        self.into_iter()
    }
}

impl<'a> IntoIterator for &'a PropertyValueSet {
    type Item = (&'a property::Handle, &'a property::RawValue);
    type IntoIter =
        Zip<std::slice::Iter<'a, property::Handle>, std::slice::Iter<'a, property::RawValue>>;

    fn into_iter(self) -> Self::IntoIter {
        self.prop_ids.iter().zip(self.prop_vals.iter())
    }
}

impl IntoIterator for PropertyValueSet {
    type Item = (property::Handle, property::RawValue);
    type IntoIter =
        Zip<std::vec::IntoIter<property::Handle>, std::vec::IntoIter<property::RawValue>>;

    fn into_iter(self) -> Self::IntoIter {
        self.prop_ids.into_iter().zip(self.prop_vals)
    }
}

/// Describes a rectangular region of a buffer
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct ClipRect(ffi::drm_sys::drm_clip_rect);

impl ClipRect {
    /// Create a new clipping rectangle.
    pub fn new(x1: u16, y1: u16, x2: u16, y2: u16) -> Self {
        Self(ffi::drm_sys::drm_clip_rect { x1, y1, x2, y2 })
    }

    /// Get the X coordinate of the top left corner of the rectangle.
    pub fn x1(self) -> u16 {
        self.0.x1
    }

    /// Get the Y coordinate of the top left corner of the rectangle.
    pub fn y1(self) -> u16 {
        self.0.y1
    }

    /// Get the X coordinate of the bottom right corner of the rectangle
    pub fn x2(self) -> u16 {
        self.0.x2
    }

    /// Get the Y coordinate of the bottom right corner of the rectangle.
    pub fn y2(self) -> u16 {
        self.0.y2
    }
}

bitflags::bitflags! {
    /// Commit flags for atomic mode setting
    ///
    /// Limited to the values in [`ffi::drm_sys::DRM_MODE_ATOMIC_FLAGS`].
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct AtomicCommitFlags : u32 {
        /// Generate a page flip event, when the changes are applied
        const PAGE_FLIP_EVENT = ffi::drm_sys::DRM_MODE_PAGE_FLIP_EVENT;
        /// Request page flip when the changes are applied, not waiting for vblank
        const PAGE_FLIP_ASYNC = ffi::drm_sys::DRM_MODE_PAGE_FLIP_ASYNC;
        /// Test only validity of the request, do not actually apply the requested changes
        const TEST_ONLY = ffi::drm_sys::DRM_MODE_ATOMIC_TEST_ONLY;
        /// Do not block on the request and return early
        const NONBLOCK = ffi::drm_sys::DRM_MODE_ATOMIC_NONBLOCK;
        /// Allow the changes to trigger a modeset, if necessary
        ///
        /// Changes requiring a modeset are rejected otherwise.
        const ALLOW_MODESET = ffi::drm_sys::DRM_MODE_ATOMIC_ALLOW_MODESET;
    }
}

bitflags::bitflags! {
    /// Mode property flags
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ModePropFlags : u32 {
        /// Do not use
        #[deprecated]
        const PENDING = ffi::DRM_MODE_PROP_PENDING;

        /// Non-extended types: legacy bitmask, one bit per type:
        const LEGACY_TYPE = ffi::DRM_MODE_PROP_LEGACY_TYPE;
        /// An unsigned integer that has a min and max value
        const RANGE = ffi::DRM_MODE_PROP_RANGE;
        /// Set when this property is informational only and cannot be modified
        const IMMUTABLE = ffi::DRM_MODE_PROP_IMMUTABLE;
        /// Enumerated type with text strings
        const ENUM = ffi::DRM_MODE_PROP_ENUM;
        /// A chunk of binary data that must be acquired
        const BLOB = ffi::DRM_MODE_PROP_BLOB;
        /// Bitmask of enumerated types
        const BITMASK = ffi::DRM_MODE_PROP_BITMASK;

        /// Extended-types: rather than continue to consume a bit per type,
        /// grab a chunk of the bits to use as integer type id.
        const EXTENDED_TYPE = ffi::DRM_MODE_PROP_EXTENDED_TYPE;
        /// A DRM object that can have a specific type
        ///
        /// See `ffi::DRM_MODE_OBJECT_*` for specific types.
        const OBJECT = ffi::DRM_MODE_PROP_OBJECT;
        /// A signed integer that has a min and max value
        const SIGNED_RANGE = ffi::DRM_MODE_PROP_SIGNED_RANGE;
        /// the [`Self::ATOMIC`] flag is used to hide properties from userspace that
        /// is not aware of atomic properties.  This is mostly to work around
        /// older userspace (DDX drivers) that read/write each prop they find,
        /// witout being aware that this could be triggering a lengthy modeset.
        const ATOMIC = ffi::DRM_MODE_PROP_ATOMIC;
    }
}

bitflags::bitflags! {
    /// Planar framebuffer flags
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct FbCmd2Flags : u32 {
        /// For interlaced framebuffers
        const INTERLACED = ffi::DRM_MODE_FB_INTERLACED;
        /// Enables .modifier
        const MODIFIERS = ffi::DRM_MODE_FB_MODIFIERS;
    }
}
