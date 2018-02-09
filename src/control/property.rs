//! # Property
//!
//! A property is information and values about a control resource.

use control::{self, ResourceHandle, ResourceInfo};
use result::*;
use ffi;

use std::ffi::CStr;

/// The underlying value type of a property.
pub type RawValue = u64;

/// A `ResourceHandle` to a property.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
pub struct Handle(control::RawHandle);
impl ResourceHandle for Handle {}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// The `ResourceInfo` on a property.
pub struct Info {
    handle: Handle,
    name: control::RawName,
    mutable: bool,
    pending: bool,
    value_type: PropertyInfoType,
}

impl ResourceInfo for Info {
    type Handle = Handle;

    fn load_from_device<T>(device: &T, handle: Self::Handle) -> Result<Self>
    where
        T: control::Device,
    {
        let mut raw: ffi::drm_mode_get_property = Default::default();
        raw.prop_id = handle.into();
        unsafe {
            try!(ffi::ioctl_mode_getproperty(device.as_raw_fd(), &mut raw));
        }

        let info = Info {
            handle: handle,
            name: control::RawName(raw.name),
            mutable: raw.flags & (ffi::DRM_MODE_PROP_IMMUTABLE) == 0,
            pending: raw.flags & (ffi::DRM_MODE_PROP_PENDING) == 1,
            value_type: try!(PropertyInfoType::from_ffi_and_device(device, raw)),
        };

        Ok(info)
    }

    fn handle(&self) -> Self::Handle {
        self.handle
    }
}

impl Info {
    /// Takes an `UnassociatedValue` and gives a specific `Value` based on this
    /// property.
    pub fn associate_value(&self, value: UnassociatedValue) -> Value {
        let raw_id = value.0 as control::RawHandle;
        match self.value_type {
            PropertyInfoType::Enum(_) => Value::Enum(EnumValue(value.0)),
            PropertyInfoType::URange(_) => Value::URange(value.0 as u64),
            PropertyInfoType::IRange(_) => Value::IRange(value.0 as i64),
            PropertyInfoType::Connector => {
                Value::Connector(control::connector::Handle::from(raw_id))
            }
            PropertyInfoType::Encoder => Value::Encoder(control::encoder::Handle::from(raw_id)),
            PropertyInfoType::Crtc => Value::Crtc(control::crtc::Handle::from(raw_id)),
            PropertyInfoType::Framebuffer => {
                Value::Framebuffer(control::framebuffer::Handle::from(raw_id))
            }
            PropertyInfoType::Plane => Value::Plane(control::plane::Handle::from(raw_id)),
            PropertyInfoType::Property => Value::Property(Handle::from(raw_id)),
            PropertyInfoType::Blob => unimplemented!(),
            PropertyInfoType::Unknown => Value::Unknown,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum PropertyInfoType {
    Enum(EnumInfo),
    URange(URangeInfo),
    IRange(IRangeInfo),
    Connector,
    Encoder,
    Crtc,
    Framebuffer,
    Plane,
    Property,
    Blob,
    Unknown,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// A `ResourceHandle` to a property with an associated resource and `Value`
pub struct AssociatedPropertyHandle {
    handle: Handle,
    value: UnassociatedValue,
    resource: control::RawHandle,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// The set of `AssociatedPropertyHandle`s for a specific property.
pub struct ResourceProperties {
    handles: ffi::Buffer<AssociatedPropertyHandle>,
}

impl ResourceProperties {
    pub fn load_from_device<T, U>(device: &T, handle: U) -> Result<Self>
    where
        T: control::Device,
        U: LoadProperties,
    {
        handle.load_resource_properties(device)
    }

    pub fn handles<'a>(&'a self) -> &'a [AssociatedPropertyHandle] {
        &self.handles
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
/// The value of an EnumEntry
pub struct EnumValue(RawValue);

#[derive(Copy, Clone, Hash, PartialEq, Eq, From, Into)]
/// The name of an EnumEntry
pub struct EnumName(control::RawName);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// A possible entry in an EnumInfo
pub struct EnumEntry(EnumValue, EnumName);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// The possible values of a particular enum.
pub struct EnumInfo {
    possible: ffi::Buffer<EnumEntry>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// The possible values of a particular unsigned range.
pub struct URangeInfo {
    possible: (u64, u64),
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// The possible values of a particular signed range.
pub struct IRangeInfo {
    possible: (i64, i64),
}

impl EnumEntry {
    pub fn value(&self) -> EnumValue {
        self.0
    }

    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr(::std::mem::transmute(&self.1)) }
    }
}

impl EnumInfo {
    fn continue_loading<T>(device: &T, mut raw: ffi::drm_mode_get_property) -> Result<Self>
    where
        T: control::Device,
    {
        let eblob = ffi_buf!(raw.enum_blob_ptr, raw.count_enum_blobs);

        // We set this to zero because an enum won't fill values_ptr
        // anyways. No need to create a buffer for it.
        raw.count_values = 0;

        unsafe {
            try!(ffi::ioctl_mode_getproperty(device.as_raw_fd(), &mut raw));
        }

        // Collect the enums into a list of EnumPropertyValues
        let enums = eblob
            .iter()
            .map(|en: &ffi::drm_mode_property_enum| {
                let val = EnumValue(en.value as RawValue);
                let name = EnumName(control::RawName(en.name));
                EnumEntry(val, name)
            })
            .collect();

        let en = EnumInfo { possible: enums };

        Ok(en)
    }

    pub fn entries(&self) -> &[EnumEntry] {
        &self.possible
    }
}

impl URangeInfo {
    fn continue_loading<T>(device: &T, mut raw: ffi::drm_mode_get_property) -> Result<Self>
    where
        T: control::Device,
    {
        let values: ffi::Buffer<u64> = ffi_buf!(raw.values_ptr, raw.count_values);

        unsafe {
            try!(ffi::ioctl_mode_getproperty(device.as_raw_fd(), &mut raw));
        }

        let &min = values.get(0).unwrap_or(&0);
        let &max = values.get(1).unwrap_or(&u64::max_value());

        let range = URangeInfo {
            possible: (min, max),
        };

        Ok(range)
    }
}

impl IRangeInfo {
    fn continue_loading<T>(device: &T, mut raw: ffi::drm_mode_get_property) -> Result<Self>
    where
        T: control::Device,
    {
        let values: ffi::Buffer<i64> = ffi_buf!(raw.values_ptr, raw.count_values);

        unsafe {
            try!(ffi::ioctl_mode_getproperty(device.as_raw_fd(), &mut raw));
        }

        let &min = values.get(0).unwrap_or(&i64::min_value());
        let &max = values.get(1).unwrap_or(&i64::max_value());

        let range = IRangeInfo {
            possible: (min, max),
        };

        Ok(range)
    }
}

impl PropertyInfoType {
    fn from_ffi_and_device<T>(device: &T, raw: ffi::drm_mode_get_property) -> Result<Self>
    where
        T: control::Device,
    {
        let info = if Self::is_enum(raw.flags) {
            PropertyInfoType::Enum(EnumInfo::continue_loading(device, raw)?)
        } else if Self::is_urange(raw.flags) {
            PropertyInfoType::URange(URangeInfo::continue_loading(device, raw)?)
        } else if Self::is_irange(raw.flags) {
            PropertyInfoType::IRange(IRangeInfo::continue_loading(device, raw)?)
        } else if Self::is_object(raw.flags) {
            // Object
            unimplemented!()
        } else if Self::is_blob(raw.flags) {
            PropertyInfoType::Blob
        } else {
            PropertyInfoType::Unknown
        };

        Ok(info)
    }

    fn is_enum(flag: u32) -> bool {
        flag & (ffi::DRM_MODE_PROP_ENUM | ffi::DRM_MODE_PROP_BITMASK) != 0
    }

    fn is_urange(flag: u32) -> bool {
        flag & ffi::DRM_MODE_PROP_RANGE != 0
    }

    fn is_irange(flag: u32) -> bool {
        flag & ffi::DRM_MODE_PROP_SIGNED_RANGE != 0
    }

    fn is_object(flag: u32) -> bool {
        flag & ffi::DRM_MODE_PROP_OBJECT != 0
    }

    fn is_blob(flag: u32) -> bool {
        flag & ffi::DRM_MODE_PROP_BLOB != 0
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
/// A generic value that is not associated with any particular type.
pub struct UnassociatedValue(u64);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// A value that has a type association.
pub enum Value {
    Enum(EnumValue),
    URange(u64),
    IRange(i64),
    Connector(control::connector::Handle),
    Encoder(control::encoder::Handle),
    Crtc(control::crtc::Handle),
    Framebuffer(control::framebuffer::Handle),
    Plane(control::plane::Handle),
    Property(Handle),
    // TODO: Blob,
    Unknown,
}

pub trait LoadProperties: ResourceHandle {
    const TYPE: u32;

    fn load_resource_properties<T>(self, device: &T) -> Result<ResourceProperties>
    where
        T: control::Device,
    {
        let mut raw: ffi::drm_mode_obj_get_properties = Default::default();
        raw.obj_id = self.into();
        raw.obj_type = Self::TYPE;
        unsafe {
            try!(ffi::ioctl_mode_obj_getproperties(
                device.as_raw_fd(),
                &mut raw
            ));
        }
        let ids: ffi::Buffer<Handle> = ffi_buf!(raw.props_ptr, raw.count_props);
        let vals: ffi::Buffer<UnassociatedValue> = ffi_buf!(raw.prop_values_ptr, raw.count_props);
        unsafe {
            try!(ffi::ioctl_mode_obj_getproperties(
                device.as_raw_fd(),
                &mut raw
            ));
        }
        let handles = ids.into_iter()
            .zip(vals.into_iter())
            .map(|(id, val)| AssociatedPropertyHandle {
                handle: id,
                value: val,
                resource: self.into(),
            })
            .collect();

        let props = ResourceProperties { handles: handles };

        Ok(props)
    }
}

impl ::std::fmt::Debug for EnumName {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let cstr = unsafe { CStr::from_ptr(::std::mem::transmute(&self.0)) };
        write!(f, "{:?}", cstr)
    }
}
