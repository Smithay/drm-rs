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
//! * Connector - Respresents the physical output, such as a DisplayPort or
//! VGA connector.
//!
//! * Encoder - Encodes pixel data from a CRTC into something a Connector can
//! understand.
//!
//! Further details on each resource can be found in their respective modules.
//!
//! # Usage
//!
//! To begin using modesetting functionality, the [Device trait](Device.t.html)
//! must be implemented on top of the [basic Device trait](../Device.t.html).

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

use core::num::NonZeroU32;
pub type RawResourceHandle = NonZeroU32;

pub trait ResourceHandle : From<RawResourceHandle> + Into<RawResourceHandle> + Into<u32> + Copy + Sized {
    const FFI_TYPE: u32;
}

pub fn from_u32<T: ResourceHandle>(raw: u32) -> Option<T> {
    RawResourceHandle::new(raw).map(|n| T::from(n))
}

/// This trait should be implemented by any object that acts as a DRM device and
/// provides modesetting functionality.
///
/// Like the parent [Device](../Device.t.html) trait, this crate does not
/// provide a concrete object for this trait.
///
/// # Example
/// ```
/// use drm::control::Device as ControlDevice;
///
/// // Assuming the `Card` wrapper already implements drm::Device
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
            fb_len: fb_len,
            crtcs: unsafe { mem::transmute(crtcs) },
            crtc_len: crtc_len,
            connectors: unsafe { mem::transmute(connectors) },
            conn_len: conn_len,
            encoders: unsafe { mem::transmute(encoders) },
            enc_len: enc_len,
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
            plane_len: plane_len,
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
            handle: handle,
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
        let info = ffi::mode::get_encoder(
            self.as_raw_fd(),
            handle.into(),
            )?;

        let enc = encoder::Info {
            handle: handle,
            enc_type: encoder::Kind::from(info.encoder_type),
            crtc: from_u32(info.crtc_id),
            pos_crtcs: info.possible_crtcs,
            pos_clones: info.possible_clones,
        };

        Ok(enc)
    }

    /// Returns information about a specific CRTC
    fn get_crtc(&self, handle: crtc::Handle) -> Result<crtc::Info, SystemError> {
        let info = ffi::mode::get_crtc(
            self.as_raw_fd(),
            handle.into(),
            )?;

        let crtc = crtc::Info {
            handle: handle,
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
            pos.0, pos.1,
            unsafe { mem::transmute(conns) },
            unsafe { mem::transmute(mode) },
        )?;

        Ok(())
    }

    /// Returns information about a specific framebuffer
    fn get_framebuffer(
        &self,
        handle: framebuffer::Handle,
        ) -> Result<framebuffer::Info, SystemError> {
        let info = ffi::mode::get_framebuffer(
            self.as_raw_fd(),
            handle.into(),
            )?;

        let fb = framebuffer::Info {
            handle: handle,
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
            w, h,
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
            w, h,
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
    fn dirty_framebuffer(&self, handle: framebuffer::Handle, clips: &[ClipRect]) -> Result<(), SystemError> {
        ffi::mode::dirty_fb(self.as_raw_fd(), handle.into(), &clips)?;
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

        let info = ffi::mode::get_plane(
            self.as_raw_fd(),
            handle.into(),
            Some(&mut fmt_slice)
            )?;

        let fmt_len = fmt_slice.len();

        let plane = plane::Info {
            handle: handle,
            crtc: from_u32(info.crtc_id),
            fb: from_u32(info.fb_id),
            pos_crtcs: info.possible_crtcs,
            formats: formats,
            fmt_len: fmt_len
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
            crtc_rect.0, crtc_rect.1, crtc_rect.2, crtc_rect.3,
            src_rect.0, src_rect.1, src_rect.2, src_rect.3,
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
            Some(&mut enum_slice)
            )?;

        let val_len = val_slice.len();
        let enum_len = enum_slice.len();

        let val_type = {
            use self::property::ValueType;
            let flags = info.flags;

            if flags & ffi::DRM_MODE_PROP_RANGE != 0 {
                let min = values[0];
                let max = values[1];

                match (min, max) {
                    (0, 1) => ValueType::Boolean,
                    (min, max) => ValueType::UnsignedRange(min, max)
                }
            } else if flags & ffi::DRM_MODE_PROP_SIGNED_RANGE != 0 {
                let min = values[0];
                let max = values[1];

                ValueType::SignedRange(min as i64, max as i64)
            } else if flags & ffi::DRM_MODE_PROP_ENUM != 0 {
                let enum_values = self::property::EnumValues {
                    values: values,
                    enums: unsafe { mem::transmute(enums) },
                    length: val_len
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
            handle: handle,
            val_type: val_type,
            mutable: info.flags & ffi::DRM_MODE_PROP_IMMUTABLE == 0,
            atomic: info.flags & ffi::DRM_MODE_PROP_ATOMIC == 0,
            info: info
        };

        Ok(property)
    }

    /// Sets a property for a specific resource.
    fn set_property<T: ResourceHandle>(
        &self,
        handle: T,
        prop: property::Handle,
        value: property::RawValue
        ) -> Result<(), SystemError> {

        ffi::mode::set_property(
            self.as_raw_fd(),
            prop.into(),
            handle.into(),
            T::FFI_TYPE,
            value
            )?;

        Ok(())
    }

    fn create_property_blob(&self, mode: Mode) -> Result<property::Value<'static>, SystemError> {
        let mut raw_mode: ffi::drm_mode_modeinfo = mode.into();
        let data = unsafe {
            std::slice::from_raw_parts_mut(
                mem::transmute(&mut raw_mode as *mut ffi::drm_mode_modeinfo),
                mem::size_of::<ffi::drm_mode_modeinfo>()
            )
        };
        let blob = ffi::mode::create_property_blob(
            self.as_raw_fd(),
            data,
        )?;

        Ok(property::Value::Blob(blob.blob_id.into()))
    }

    fn destroy_property_blob(&self, blob: u64) -> Result<(), SystemError> {
        ffi::mode::destroy_property_blob(self.as_raw_fd(), blob as u32)?;

        Ok(())
    }

    /// Returns the set of `Mode`s that a particular connector supports.
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
    fn get_properties<T: ResourceHandle>(&self, handle: T) -> Result<PropertyValueSet, SystemError> {
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
            len: prop_len
        };

        Ok(prop_val_set)
    }
    
    /// Receive the currently set gamma ramp of a crtc
    fn get_gamma(&self, crtc: crtc::Handle, red: &mut [u16], green: &mut [u16], blue: &mut [u16]) -> Result<(), SystemError> {
        let crtc_info = self.get_crtc(crtc)?;
        if crtc_info.gamma_length as usize > red.len() ||
           crtc_info.gamma_length as usize > green.len() ||
           crtc_info.gamma_length as usize > blue.len()
        {
            return Err(SystemError::InvalidArgument);
        }

        ffi::mode::get_gamma(
            self.as_raw_fd(),
            crtc.into(),
            crtc_info.gamma_length as usize,
            red,
            green,
            blue
        )?;

        Ok(())
    }

    /// Set a gamma ramp for the given crtc
    fn set_gamma(&self, crtc: crtc::Handle, red: &[u16], green: &[u16], blue: &[u16]) -> Result<(), SystemError> {
        let crtc_info = self.get_crtc(crtc)?;
        if crtc_info.gamma_length as usize > red.len() ||
           crtc_info.gamma_length as usize > green.len() ||
           crtc_info.gamma_length as usize > blue.len()
        {
            return Err(SystemError::InvalidArgument);
        }
        
        ffi::mode::set_gamma(
            self.as_raw_fd(),
            crtc.into(),
            crtc_info.gamma_length as usize,
            red,
            green,
            blue
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
    fn map_dumb_buffer<'a>(&self, buffer: &'a mut DumbBuffer) -> Result<DumbMapping<'a>, SystemError> {
        let info = drm_ffi::mode::dumbbuffer::map(self.as_raw_fd(), buffer.handle.into(), 0, 0)?;

        let map = {
            use ::nix::sys::mman;
            let addr = core::ptr::null_mut();
            let prot = mman::ProtFlags::PROT_READ | mman::ProtFlags::PROT_WRITE;
            let flags = mman::MapFlags::MAP_SHARED;
            let length = buffer.length;
            let fd = self.as_raw_fd();
            let offset = info.offset as i64;
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
    /// A buffer argument of `None` will clear the cursor.
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
    /// A buffer argument of `None` will clear the cursor.
    fn set_cursor2<B>(&self, crtc: crtc::Handle, buffer: Option<&B>, hotspot: (i32, i32)) -> Result<(), SystemError>
    where
        B: buffer::Buffer + ?Sized,
    {
        let (id, w, h) = buffer
            .map(|buf| {
                let (w, h) = buf.size();
                (buf.handle().into(), w, h)
            })
            .unwrap_or((0, 0, 0));
        drm_ffi::mode::set_cursor2(self.as_raw_fd(), crtc.into(), id, w, h, hotspot.0, hotspot.1)?;

        Ok(())
    }

    /// Moves a set cursor on a given crtc
    fn move_cursor(&self, crtc: crtc::Handle, pos: (i32, i32)) -> Result<(), SystemError> {
        drm_ffi::mode::move_cursor(self.as_raw_fd(), crtc.into(), pos.0, pos.1)?;

        Ok(())
    }

    fn atomic_commit(&self, flags: &[AtomicCommitFlags], mut req: atomic::AtomicModeReq) -> Result<(), SystemError> {
        use std::mem::transmute as tm;

        drm_ffi::mode::atomic_commit(
            self.as_raw_fd(),
            flags.iter().fold(0, |acc, x| acc | *x as u32),
            unsafe { tm(&mut *req.objects) },
            &mut *req.count_props_per_object,
            unsafe { tm(&mut *req.props) },
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
}

/// The set of [ResourceHandles](ResourceHandle.t.html) that a
/// [Device](Device.t.html) exposes. Excluding Plane resources.
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
    /// Returns the set of [connector::Handles](connector/Handle.t.html)
    pub fn connectors(&self) -> &[connector::Handle] {
        let buf_len = std::cmp::min(self.connectors.len(), self.conn_len);
        unsafe { mem::transmute(&self.connectors[..buf_len]) }
    }

    /// Returns the set of [encoder::Handles](encoder/Handle.t.html)
    pub fn encoders(&self) -> &[encoder::Handle] {
        let buf_len = std::cmp::min(self.encoders.len(), self.enc_len);
        unsafe { mem::transmute(&self.encoders[..buf_len]) }
    }

    /// Returns the set of [crtc::Handles](crtc/Handle.t.html)
    pub fn crtcs(&self) -> &[crtc::Handle] {
        let buf_len = std::cmp::min(self.crtcs.len(), self.crtc_len);
        unsafe { mem::transmute(&self.crtcs[..buf_len]) }
    }

    /// Returns the set of [framebuffer::Handles](framebuffer/Handle.t.html)
    pub fn framebuffers(&self) -> &[framebuffer::Handle] {
        let buf_len = std::cmp::min(self.fbs.len(), self.fb_len);
        unsafe { mem::transmute(&self.fbs[..buf_len]) }
    }

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

/// The set of [plane::Handles](plane/Handle.t.html) that a
/// [Device](Device.t.html) exposes.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct PlaneResourceHandles {
    planes: [Option<plane::Handle>; 32],
    plane_len: usize,
}

impl PlaneResourceHandles {
    /// Returns the set of [plane::Handles](plane/Handle.t.html)
    pub fn planes(&self) -> &[plane::Handle] {
        let buf_len = std::cmp::min(self.planes.len(), self.plane_len);
        unsafe { mem::transmute(&self.planes[..buf_len]) }
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
/// A filter that can be used with a ResourceHandles to determine the set of
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
}

impl From<ffi::drm_mode_modeinfo> for Mode {
    fn from(raw: ffi::drm_mode_modeinfo) -> Mode {
        Mode { mode: raw }
    }
}

impl Into<ffi::drm_mode_modeinfo> for Mode {
    fn into(self) -> ffi::drm_mode_modeinfo {
        self.mode
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
            .finish()
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaneType {
    Overlay = ffi::DRM_PLANE_TYPE_OVERLAY,
    Primary = ffi::DRM_PLANE_TYPE_PRIMARY,
    Cursor = ffi::DRM_PLANE_TYPE_CURSOR,
}

/// Wrapper around a set of property IDs and their raw values.
#[derive(Debug, Copy, Clone)]
pub struct PropertyValueSet {
    prop_ids: [Option<property::Handle>; 32],
    prop_vals: [property::RawValue; 32],
    len: usize
}

impl PropertyValueSet {
    /// Returns a pair representing a set of [property::Handles](property/Handle.t.html) and their raw values
    pub fn as_props_and_values(&self) -> (&[property::Handle], &[property::RawValue]) {
        unsafe {
            mem::transmute((&self.prop_ids[..self.len], &self.prop_vals[..self.len]))
        }
    }
}

type ClipRect = ffi::drm_sys::drm_clip_rect;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AtomicCommitFlags {
    TestOnly = ffi::drm_sys::DRM_MODE_ATOMIC_TEST_ONLY,
    Nonblock =  ffi::drm_sys::DRM_MODE_ATOMIC_NONBLOCK,
    AllowModeset = ffi::drm_sys::DRM_MODE_ATOMIC_ALLOW_MODESET,
    PageFlipEvent = ffi::drm_sys::DRM_MODE_PAGE_FLIP_EVENT,
}