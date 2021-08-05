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
use drm_ffi::result::SystemError;
use drm_fourcc::DrmModifier;

pub mod atomic;
pub mod connector;
pub mod crtc;
pub mod dumbbuffer;
pub mod encoder;
pub mod framebuffer;
pub mod plane;

pub mod property;

use self::dumbbuffer::*;
use buffer;

use std::mem;
use std::os::unix::io::RawFd;
use std::time::Duration;

use core::num::NonZeroU32;

/// Raw handle for a drm resource
pub type RawResourceHandle = NonZeroU32;

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
pub fn from_u32<T: ResourceHandle>(raw: u32) -> Option<T> {
    RawResourceHandle::new(raw).map(T::from)
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
    fn resource_handles(&self) -> Result<ResourceHandles, SystemError> {
        let mut fbs = [0u32; 32];
        let mut crtcs = [0u32; 32];
        let mut connectors = [0u32; 32];
        let mut encoders = [0u32; 32];

        let mut fb_slice = &mut fbs[..];
        let mut crtc_slice = &mut crtcs[..];
        let mut conn_slice = &mut connectors[..];
        let mut enc_slice = &mut encoders[..];

        let ffi_res = ffi::mode::get_resources(
            self.as_raw_fd(),
            Some(&mut fb_slice),
            Some(&mut crtc_slice),
            Some(&mut conn_slice),
            Some(&mut enc_slice),
        )?;

        let fb_len = fb_slice.len();
        let crtc_len = crtc_slice.len();
        let conn_len = conn_slice.len();
        let enc_len = enc_slice.len();

        let res = ResourceHandles {
            fbs: unsafe { mem::transmute(fbs) },
            fb_len,
            crtcs: unsafe { mem::transmute(crtcs) },
            crtc_len,
            connectors: unsafe { mem::transmute(connectors) },
            conn_len,
            encoders: unsafe { mem::transmute(encoders) },
            enc_len,
            width: (ffi_res.min_width, ffi_res.max_width),
            height: (ffi_res.min_height, ffi_res.max_height),
        };

        Ok(res)
    }

    /// Gets the set of plane handles that this device currently has
    fn plane_handles(&self) -> Result<PlaneResourceHandles, SystemError> {
        let mut planes = [0u32; 32];
        let mut plane_slice = &mut planes[..];

        let _ffi_res = ffi::mode::get_plane_resources(self.as_raw_fd(), Some(&mut plane_slice))?;

        let plane_len = plane_slice.len();

        let res = PlaneResourceHandles {
            planes: unsafe { mem::transmute(planes) },
            plane_len,
        };

        Ok(res)
    }

    /// Returns information about a specific connector
    fn get_connector(&self, handle: connector::Handle) -> Result<connector::Info, SystemError> {
        // Maximum number of encoders is 3 due to kernel restrictions
        let mut encoders = [0u32; 3];
        let mut enc_slice = &mut encoders[..];
        let mut modes = Vec::new();

        let ffi_info = ffi::mode::get_connector(
            self.as_raw_fd(),
            handle.into(),
            None,
            None,
            Some(&mut modes),
            Some(&mut enc_slice),
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
            modes: unsafe { mem::transmute(modes) },
            encoders: unsafe { mem::transmute(encoders) },
            curr_enc: unsafe { mem::transmute(ffi_info.encoder_id) },
        };

        Ok(connector)
    }

    /// Returns information about a specific encoder
    fn get_encoder(&self, handle: encoder::Handle) -> Result<encoder::Info, SystemError> {
        let info = ffi::mode::get_encoder(self.as_raw_fd(), handle.into())?;

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
    fn get_crtc(&self, handle: crtc::Handle) -> Result<crtc::Info, SystemError> {
        let info = ffi::mode::get_crtc(self.as_raw_fd(), handle.into())?;

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
    ) -> Result<(), SystemError> {
        let _info = ffi::mode::set_crtc(
            self.as_raw_fd(),
            handle.into(),
            framebuffer.map(|x| x.into()).unwrap_or(0),
            pos.0,
            pos.1,
            unsafe { &*(conns as *const _ as *const [u32]) },
            unsafe { mem::transmute(mode) },
        )?;

        Ok(())
    }

    /// Returns information about a specific framebuffer
    fn get_framebuffer(
        &self,
        handle: framebuffer::Handle,
    ) -> Result<framebuffer::Info, SystemError> {
        let info = ffi::mode::get_framebuffer(self.as_raw_fd(), handle.into())?;

        let fb = framebuffer::Info {
            handle,
            size: (info.width, info.height),
            pitch: info.pitch,
            bpp: info.bpp,
            depth: info.depth,
            buffer: info.handle,
        };

        Ok(fb)
    }

    /// Add a new framebuffer
    fn add_framebuffer<B>(
        &self,
        buffer: &B,
        depth: u32,
        bpp: u32,
    ) -> Result<framebuffer::Handle, SystemError>
    where
        B: buffer::Buffer + ?Sized,
    {
        let (w, h) = buffer.size();
        let info = ffi::mode::add_fb(
            self.as_raw_fd(),
            w,
            h,
            buffer.pitch(),
            bpp,
            depth,
            buffer.handle().into(),
        )?;

        Ok(unsafe { mem::transmute(info.fb_id) })
    }

    /// Add framebuffer (with modifiers)
    fn add_planar_framebuffer<B>(
        &self,
        planar_buffer: &B,
        modifiers: &[Option<DrmModifier>; 4],
        flags: u32,
    ) -> Result<framebuffer::Handle, SystemError>
    where
        B: buffer::PlanarBuffer + ?Sized,
    {
        let (w, h) = planar_buffer.size();
        let opt_handles = planar_buffer.handles();
        let handles = [
            opt_handles[0].map(|x| x.into()).unwrap_or(0),
            opt_handles[1].map(|x| x.into()).unwrap_or(0),
            opt_handles[2].map(|x| x.into()).unwrap_or(0),
            opt_handles[3].map(|x| x.into()).unwrap_or(0),
        ];
        let mods = [
            modifiers[0].map(Into::<u64>::into).unwrap_or(0),
            modifiers[1].map(Into::<u64>::into).unwrap_or(0),
            modifiers[2].map(Into::<u64>::into).unwrap_or(0),
            modifiers[3].map(Into::<u64>::into).unwrap_or(0),
        ];

        let info = ffi::mode::add_fb2(
            self.as_raw_fd(),
            w,
            h,
            planar_buffer.format() as u32,
            &handles,
            &planar_buffer.pitches(),
            &planar_buffer.offsets(),
            &mods,
            flags,
        )?;

        Ok(unsafe { mem::transmute(info.fb_id) })
    }

    /// Mark parts of a framebuffer dirty
    fn dirty_framebuffer(
        &self,
        handle: framebuffer::Handle,
        clips: &[ClipRect],
    ) -> Result<(), SystemError> {
        ffi::mode::dirty_fb(self.as_raw_fd(), handle.into(), clips)?;
        Ok(())
    }

    /// Destroy a framebuffer
    fn destroy_framebuffer(&self, handle: framebuffer::Handle) -> Result<(), SystemError> {
        ffi::mode::rm_fb(self.as_raw_fd(), handle.into())
    }

    /// Returns information about a specific plane
    fn get_plane(&self, handle: plane::Handle) -> Result<plane::Info, SystemError> {
        let mut formats = [0u32; 8];
        let mut fmt_slice = &mut formats[..];

        let info = ffi::mode::get_plane(self.as_raw_fd(), handle.into(), Some(&mut fmt_slice))?;

        let fmt_len = fmt_slice.len();

        let plane = plane::Info {
            handle,
            crtc: from_u32(info.crtc_id),
            fb: from_u32(info.fb_id),
            pos_crtcs: info.possible_crtcs,
            formats,
            fmt_len,
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
    ) -> Result<(), SystemError> {
        let _info = ffi::mode::set_plane(
            self.as_raw_fd(),
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
    fn get_property(&self, handle: property::Handle) -> Result<property::Info, SystemError> {
        let mut values = [0u64; 24];
        let mut enums = [ffi::drm_mode_property_enum::default(); 24];

        let mut val_slice = &mut values[..];
        let mut enum_slice = &mut enums[..];

        let info = ffi::mode::get_property(
            self.as_raw_fd(),
            handle.into(),
            Some(&mut val_slice),
            Some(&mut enum_slice),
        )?;

        let val_len = val_slice.len();

        let val_type = {
            use self::property::ValueType;
            let flags = info.flags;

            if flags & ffi::DRM_MODE_PROP_RANGE != 0 {
                let min = values[0];
                let max = values[1];

                match (min, max) {
                    (0, 1) => ValueType::Boolean,
                    (min, max) => ValueType::UnsignedRange(min, max),
                }
            } else if flags & ffi::DRM_MODE_PROP_SIGNED_RANGE != 0 {
                let min = values[0];
                let max = values[1];

                ValueType::SignedRange(min as i64, max as i64)
            } else if flags & ffi::DRM_MODE_PROP_ENUM != 0 {
                let enum_values = self::property::EnumValues {
                    values,
                    enums: unsafe { mem::transmute(enums) },
                    length: val_len,
                };

                ValueType::Enum(enum_values)
            } else if flags & ffi::DRM_MODE_PROP_BLOB != 0 {
                ValueType::Blob
            } else if flags & ffi::DRM_MODE_PROP_BITMASK != 0 {
                ValueType::Bitmask
            } else if flags & ffi::DRM_MODE_PROP_OBJECT != 0 {
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
            mutable: info.flags & ffi::DRM_MODE_PROP_IMMUTABLE == 0,
            atomic: info.flags & ffi::DRM_MODE_PROP_ATOMIC == 0,
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
    ) -> Result<(), SystemError> {
        ffi::mode::set_property(
            self.as_raw_fd(),
            prop.into(),
            handle.into(),
            T::FFI_TYPE,
            value,
        )?;

        Ok(())
    }

    /// Create a property blob value from a given data blob
    fn create_property_blob<T>(&self, data: &T) -> Result<property::Value<'static>, SystemError> {
        let data = unsafe {
            std::slice::from_raw_parts_mut(data as *const _ as *mut u8, mem::size_of::<T>())
        };
        let blob = ffi::mode::create_property_blob(self.as_raw_fd(), data)?;

        Ok(property::Value::Blob(blob.blob_id.into()))
    }

    /// Get a property blob's data
    fn get_property_blob(&self, blob: u64) -> Result<Vec<u8>, SystemError> {
        let len = ffi::mode::get_property_blob(self.as_raw_fd(), blob as u32, None)?.length;
        let mut data = vec![0u8; len as usize];
        let _ = ffi::mode::get_property_blob(self.as_raw_fd(), blob as u32, Some(&mut &mut *data))?;
        Ok(data)
    }

    /// Destroy a given property blob value
    fn destroy_property_blob(&self, blob: u64) -> Result<(), SystemError> {
        ffi::mode::destroy_property_blob(self.as_raw_fd(), blob as u32)?;

        Ok(())
    }

    /// Returns the set of [`Mode`]s that a particular connector supports.
    fn get_modes(&self, handle: connector::Handle) -> Result<Vec<Mode>, SystemError> {
        let mut modes = Vec::new();

        let _ffi_info = ffi::mode::get_connector(
            self.as_raw_fd(),
            handle.into(),
            None,
            None,
            Some(&mut modes),
            None,
        )?;

        Ok(unsafe { mem::transmute(modes) })
    }

    /// Gets a list of property handles and values for this resource.
    fn get_properties<T: ResourceHandle>(
        &self,
        handle: T,
    ) -> Result<PropertyValueSet, SystemError> {
        let mut prop_ids = [0u32; 32];
        let mut prop_vals = [0u64; 32];

        let mut prop_id_slice = &mut prop_ids[..];
        let mut prop_val_slice = &mut prop_vals[..];

        ffi::mode::get_properties(
            self.as_raw_fd(),
            handle.into(),
            T::FFI_TYPE,
            Some(&mut prop_id_slice),
            Some(&mut prop_val_slice),
        )?;

        let prop_len = prop_id_slice.len();

        let prop_val_set = PropertyValueSet {
            prop_ids: unsafe { mem::transmute(prop_ids) },
            prop_vals: unsafe { mem::transmute(prop_vals) },
            len: prop_len,
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
    ) -> Result<(), SystemError> {
        let crtc_info = self.get_crtc(crtc)?;
        if crtc_info.gamma_length as usize > red.len()
            || crtc_info.gamma_length as usize > green.len()
            || crtc_info.gamma_length as usize > blue.len()
        {
            return Err(SystemError::InvalidArgument);
        }

        ffi::mode::get_gamma(
            self.as_raw_fd(),
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
    ) -> Result<(), SystemError> {
        let crtc_info = self.get_crtc(crtc)?;
        if crtc_info.gamma_length as usize > red.len()
            || crtc_info.gamma_length as usize > green.len()
            || crtc_info.gamma_length as usize > blue.len()
        {
            return Err(SystemError::InvalidArgument);
        }

        ffi::mode::set_gamma(
            self.as_raw_fd(),
            crtc.into(),
            crtc_info.gamma_length as usize,
            red,
            green,
            blue,
        )?;

        Ok(())
    }

    /// Open a GEM buffer handle by name
    fn open_buffer(&self, name: buffer::Name) -> Result<buffer::Handle, SystemError> {
        let info = drm_ffi::gem::open(self.as_raw_fd(), name.into())?;
        Ok(unsafe { mem::transmute(info.handle) })
    }

    /// Close a GEM buffer handle
    fn close_buffer(&self, handle: buffer::Handle) -> Result<(), SystemError> {
        let _info = drm_ffi::gem::close(self.as_raw_fd(), handle.into())?;
        Ok(())
    }

    /// Create a new dumb buffer with a given size and pixel format
    fn create_dumb_buffer(
        &self,
        size: (u32, u32),
        format: buffer::DrmFourcc,
        bpp: u32,
    ) -> Result<DumbBuffer, SystemError> {
        let info = drm_ffi::mode::dumbbuffer::create(self.as_raw_fd(), size.0, size.1, bpp, 0)?;

        let dumb = DumbBuffer {
            size: (info.width, info.height),
            length: info.size as usize,
            format,
            pitch: info.pitch,
            handle: unsafe { mem::transmute(info.handle) },
        };

        Ok(dumb)
    }
    /// Map the buffer for access
    fn map_dumb_buffer<'a>(
        &self,
        buffer: &'a mut DumbBuffer,
    ) -> Result<DumbMapping<'a>, SystemError> {
        let info = drm_ffi::mode::dumbbuffer::map(self.as_raw_fd(), buffer.handle.into(), 0, 0)?;

        let map = {
            use nix::sys::mman;
            let addr = core::ptr::null_mut();
            let prot = mman::ProtFlags::PROT_READ | mman::ProtFlags::PROT_WRITE;
            let flags = mman::MapFlags::MAP_SHARED;
            let length = buffer.length;
            let fd = self.as_raw_fd();
            let offset = info.offset as _;
            unsafe { mman::mmap(addr, length, prot, flags, fd, offset)? }
        };

        let mapping = DumbMapping {
            _phantom: ::std::marker::PhantomData,
            map: unsafe { ::std::slice::from_raw_parts_mut(map as *mut _, buffer.length) },
        };

        Ok(mapping)
    }

    /// Free the memory resources of a dumb buffer
    fn destroy_dumb_buffer(&self, buffer: DumbBuffer) -> Result<(), SystemError> {
        let _info = drm_ffi::mode::dumbbuffer::destroy(self.as_raw_fd(), buffer.handle.into())?;

        Ok(())
    }

    /// Sets a hardware-cursor on the given crtc with the image of a given buffer
    ///
    /// A buffer argument of [`None`] will clear the cursor.
    #[deprecated(note = "Usage of deprecated ioctl set_cursor: use a cursor plane instead")]
    #[allow(deprecated)]
    fn set_cursor<B>(&self, crtc: crtc::Handle, buffer: Option<&B>) -> Result<(), SystemError>
    where
        B: buffer::Buffer + ?Sized,
    {
        let (id, w, h) = buffer
            .map(|buf| {
                let (w, h) = buf.size();
                (buf.handle().into(), w, h)
            })
            .unwrap_or((0, 0, 0));
        drm_ffi::mode::set_cursor(self.as_raw_fd(), crtc.into(), id, w, h)?;

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
    ) -> Result<(), SystemError>
    where
        B: buffer::Buffer + ?Sized,
    {
        let (id, w, h) = buffer
            .map(|buf| {
                let (w, h) = buf.size();
                (buf.handle().into(), w, h)
            })
            .unwrap_or((0, 0, 0));
        drm_ffi::mode::set_cursor2(
            self.as_raw_fd(),
            crtc.into(),
            id,
            w,
            h,
            hotspot.0,
            hotspot.1,
        )?;

        Ok(())
    }

    /// Moves a set cursor on a given crtc
    #[deprecated(note = "Usage of deprecated ioctl move_cursor: use a cursor plane instead")]
    #[allow(deprecated)]
    fn move_cursor(&self, crtc: crtc::Handle, pos: (i32, i32)) -> Result<(), SystemError> {
        drm_ffi::mode::move_cursor(self.as_raw_fd(), crtc.into(), pos.0, pos.1)?;

        Ok(())
    }

    /// Request an atomic commit with given flags and property-value pair for a list of objects.
    fn atomic_commit(
        &self,
        flags: &[AtomicCommitFlags],
        mut req: atomic::AtomicModeReq,
    ) -> Result<(), SystemError> {
        drm_ffi::mode::atomic_commit(
            self.as_raw_fd(),
            flags.iter().fold(0, |acc, x| acc | *x as u32),
            unsafe { &mut *(&mut *req.objects as *mut _ as *mut [u32]) },
            &mut *req.count_props_per_object,
            unsafe { &mut *(&mut *req.props as *mut _ as *mut [u32]) },
            &mut *req.values,
        )
    }

    /// Convert a prime file descriptor to a GEM buffer handle
    fn prime_fd_to_buffer(&self, fd: RawFd) -> Result<buffer::Handle, SystemError> {
        let info = ffi::gem::fd_to_handle(self.as_raw_fd(), fd)?;
        Ok(unsafe { mem::transmute(info.handle) })
    }

    /// Convert a prime file descriptor to a GEM buffer handle
    fn buffer_to_prime_fd(&self, handle: buffer::Handle, flags: u32) -> Result<RawFd, SystemError> {
        let info = ffi::gem::handle_to_fd(self.as_raw_fd(), handle.into(), flags)?;
        Ok(info.fd)
    }

    /// Queue a page flip on the given crtc
    fn page_flip(
        &self,
        handle: crtc::Handle,
        framebuffer: framebuffer::Handle,
        flags: &[PageFlipFlags],
        target: Option<PageFlipTarget>,
    ) -> Result<(), SystemError> {
        let mut flags = flags.iter().fold(0, |val, flag| val | *flag as u32);
        if target.is_some() {
            flags |= ffi::drm_sys::DRM_MODE_PAGE_FLIP_TARGET;
        }

        let _info = ffi::mode::page_flip(
            self.as_raw_fd(),
            handle.into(),
            framebuffer.into(),
            flags,
            target.map(|x| x as _).unwrap_or(0),
        )?;

        Ok(())
    }

    /// Receive pending events
    fn receive_events(&self) -> Result<Events, SystemError>
    where
        Self: Sized,
    {
        let mut event_buf: [u8; 1024] = [0; 1024];
        let amount = ::nix::unistd::read(self.as_raw_fd(), &mut event_buf)?;

        Ok(Events {
            event_buf,
            amount,
            i: 0,
        })
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Flags to alter the behaviour of a page flip
pub enum PageFlipFlags {
    /// Request a vblank event on page flip
    PageFlipEvent = ffi::drm_sys::DRM_MODE_PAGE_FLIP_EVENT,
    /// Request page flip as soon as possible, not waiting for vblank
    PageFlipAsync = ffi::drm_sys::DRM_MODE_PAGE_FLIP_ASYNC,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Target to alter the sequence of page flips
pub enum PageFlipTarget {
    /// Absolute Vblank Sequence
    Absolute = ffi::drm_sys::DRM_MODE_PAGE_FLIP_TARGET_ABSOLUTE,
    /// Relative Vblank Sequence (to the current, when calling)
    Relative = ffi::drm_sys::DRM_MODE_PAGE_FLIP_TARGET_RELATIVE,
}

/// Iterator over [`Event`]s of a device. Create via [`Device::receive_events()`].
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
    /// crtc that did throw the event
    pub crtc: crtc::Handle,
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
            let event = unsafe { &*(self.event_buf.as_ptr().add(self.i) as *const ffi::drm_event) };
            self.i += event.length as usize;
            match event.type_ {
                x if x == ffi::DRM_EVENT_VBLANK => {
                    let vblank_event =
                        unsafe { &*(event as *const _ as *const ffi::drm_event_vblank) };
                    Some(Event::Vblank(VblankEvent {
                        frame: vblank_event.sequence,
                        duration: Duration::new(
                            vblank_event.tv_sec as u64,
                            vblank_event.tv_usec * 100,
                        ),
                        crtc: unsafe { mem::transmute(vblank_event.user_data as u32) },
                    }))
                }
                x if x == ffi::DRM_EVENT_FLIP_COMPLETE => {
                    let vblank_event =
                        unsafe { &*(event as *const _ as *const ffi::drm_event_vblank) };
                    Some(Event::PageFlip(PageFlipEvent {
                        frame: vblank_event.sequence,
                        duration: Duration::new(
                            vblank_event.tv_sec as u64,
                            vblank_event.tv_usec * 1000,
                        ),
                        crtc: unsafe {
                            mem::transmute(if vblank_event.crtc_id != 0 {
                                vblank_event.crtc_id
                            } else {
                                vblank_event.user_data as u32
                            })
                        },
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
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct ResourceHandles {
    fbs: [Option<framebuffer::Handle>; 32],
    fb_len: usize,
    crtcs: [Option<crtc::Handle>; 32],
    crtc_len: usize,
    connectors: [Option<connector::Handle>; 32],
    conn_len: usize,
    encoders: [Option<encoder::Handle>; 32],
    enc_len: usize,
    width: (u32, u32),
    height: (u32, u32),
}

impl ResourceHandles {
    /// Returns the set of [`connector::Handle`]
    pub fn connectors(&self) -> &[connector::Handle] {
        let buf_len = std::cmp::min(self.connectors.len(), self.conn_len);
        unsafe { &*(&self.connectors[..buf_len] as *const _ as *const [connector::Handle]) }
    }

    /// Returns the set of [`encoder::Handle`]
    pub fn encoders(&self) -> &[encoder::Handle] {
        let buf_len = std::cmp::min(self.encoders.len(), self.enc_len);
        unsafe { &*(&self.encoders[..buf_len] as *const _ as *const [encoder::Handle]) }
    }

    /// Returns the set of [`crtc::Handle`]
    pub fn crtcs(&self) -> &[crtc::Handle] {
        let buf_len = std::cmp::min(self.crtcs.len(), self.crtc_len);
        unsafe { &*(&self.crtcs[..buf_len] as *const _ as *const [crtc::Handle]) }
    }

    /// Returns the set of [`framebuffer::Handle`]
    pub fn framebuffers(&self) -> &[framebuffer::Handle] {
        let buf_len = std::cmp::min(self.fbs.len(), self.fb_len);
        unsafe { &*(&self.fbs[..buf_len] as *const _ as *const [framebuffer::Handle]) }
    }

    /// Apply a filter the all crtcs of these resources, resulting in a list of crtcs allowed.
    pub fn filter_crtcs(&self, filter: CrtcListFilter) -> Vec<crtc::Handle> {
        self.crtcs
            .iter()
            .enumerate()
            .filter(|&(n, _)| (1 << n) & filter.0 != 0)
            .flat_map(|(_, &e)| e)
            .collect()
    }
}

impl std::fmt::Debug for ResourceHandles {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ResourceHandles")
            .field("fbs", &self.framebuffers())
            .field("crtcs", &self.crtcs())
            .field("connectors", &self.connectors())
            .field("encoders", &self.encoders())
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

/// The set of [`plane::Handle`] that a
/// [`Device`] exposes.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct PlaneResourceHandles {
    planes: [Option<plane::Handle>; 32],
    plane_len: usize,
}

impl PlaneResourceHandles {
    /// Returns the set of [`plane::Handle`]
    pub fn planes(&self) -> &[plane::Handle] {
        let buf_len = std::cmp::min(self.planes.len(), self.plane_len);
        unsafe { &*(&self.planes[..buf_len] as *const _ as *const [plane::Handle]) }
    }
}

impl std::fmt::Debug for PlaneResourceHandles {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("PlaneResourceHandles")
            .field("planes", &self.planes())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A filter that can be used with a [`ResourceHandles`] to determine the set of
/// Crtcs that can attach to a specific encoder.
pub struct CrtcListFilter(u32);

/// Resolution and timing information for a display mode.
#[repr(transparent)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
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

    /// Returns the type flags bitmask of this mode
    pub fn type_(&self) -> u32 {
        self.mode.type_
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

impl std::fmt::Debug for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Mode")
            .field("name", &self.name())
            .field("clock", &self.clock())
            .field("size", &self.size())
            .field("hsync", &self.hsync())
            .field("vsync", &self.vsync())
            .field("hskew", &self.hskew())
            .field("vscan", &self.vscan())
            .field("vrefresh", &self.vrefresh())
            .field("type_", &self.type_())
            .finish()
    }
}

/// Display mode type flags
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModeType {
    /// Deprecated builtin mode type
    Builtin = ffi::DRM_MODE_TYPE_BUILTIN,
    /// Deprecated CLOCK_C mode type
    ClockC = ffi::DRM_MODE_TYPE_CLOCK_C,
    /// Deprecated CRTC_C mode type
    CrtcC = ffi::DRM_MODE_TYPE_CRTC_C,
    /// Preferred mode
    Preferred = ffi::DRM_MODE_TYPE_PREFERRED,
    /// Deprecated default mode
    Default = ffi::DRM_MODE_TYPE_DEFAULT,
    /// User defined mode type
    UserDef = ffi::DRM_MODE_TYPE_USERDEF,
    /// Mode is created by driver
    Driver = ffi::DRM_MODE_TYPE_DRIVER,
    /// Bitmask of all valid (non-deprecated) mode type flags
    All = ffi::DRM_MODE_TYPE_ALL,
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
#[derive(Debug, Copy, Clone)]
pub struct PropertyValueSet {
    prop_ids: [Option<property::Handle>; 32],
    prop_vals: [property::RawValue; 32],
    len: usize,
}

impl PropertyValueSet {
    /// Returns a pair representing a set of [`property::Handle`] and their raw values
    pub fn as_props_and_values(&self) -> (&[property::Handle], &[property::RawValue]) {
        unsafe {
            (
                &*(&self.prop_ids[..self.len] as *const _ as *const [property::Handle]),
                &*(&self.prop_vals[..self.len] as *const _ as *const [property::RawValue]),
            )
        }
    }
}

type ClipRect = ffi::drm_sys::drm_clip_rect;

/// Commit flags for atomic mode setting
#[allow(missing_docs)]
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Flags for an atomic commit
pub enum AtomicCommitFlags {
    /// Test only validity of the request, do not actually apply the requested changes.
    TestOnly = ffi::drm_sys::DRM_MODE_ATOMIC_TEST_ONLY,
    /// Do not block on the request and return early.
    Nonblock = ffi::drm_sys::DRM_MODE_ATOMIC_NONBLOCK,
    /// Allow the changes to trigger a modeset, if necessary.
    ///
    /// Changes requiring a modeset are rejected otherwise.
    AllowModeset = ffi::drm_sys::DRM_MODE_ATOMIC_ALLOW_MODESET,
    /// Generate a page flip event, when the changes are applied.
    PageFlipEvent = ffi::drm_sys::DRM_MODE_PAGE_FLIP_EVENT,
}
