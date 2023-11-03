//! # Property
//!
//! A property of a modesetting resource.
//!
//! All modesetting resources have a set of properties that have values that
//! can be modified. These properties are modesetting resources themselves, and
//! may even have their own set of properties.
//!
//! Properties may have mutable values attached to them. These can be changed by
//! either changing the state of a resource (thereby affecting the property),
//! directly changing the property value itself, or by batching property changes
//! together and executing them all atomically.

use crate::control::{RawResourceHandle, ResourceHandle};
use drm_ffi as ffi;

/// A raw property value that does not have a specific property type
pub type RawValue = u64;

/// A handle to a property
#[repr(transparent)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Handle(RawResourceHandle);

// Safety: Handle is repr(transparent) over NonZeroU32
unsafe impl bytemuck::ZeroableInOption for Handle {}
unsafe impl bytemuck::PodInOption for Handle {}

impl From<Handle> for RawResourceHandle {
    fn from(handle: Handle) -> Self {
        handle.0
    }
}

impl From<Handle> for u32 {
    fn from(handle: Handle) -> Self {
        handle.0.into()
    }
}

impl From<RawResourceHandle> for Handle {
    fn from(handle: RawResourceHandle) -> Self {
        Handle(handle)
    }
}

impl ResourceHandle for Handle {
    const FFI_TYPE: u32 = ffi::DRM_MODE_OBJECT_PROPERTY;
}

impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("property::Handle").field(&self.0).finish()
    }
}

/// Information about a property
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Info {
    pub(crate) handle: Handle,
    pub(crate) val_type: ValueType,
    pub(crate) mutable: bool,
    pub(crate) atomic: bool,
    pub(crate) info: ffi::drm_mode_get_property,
}

impl Info {
    /// Returns the handle to this property.
    pub fn handle(&self) -> Handle {
        self.handle
    }

    /// Returns the name of this property.
    pub fn name(&self) -> &std::ffi::CStr {
        unsafe { std::ffi::CStr::from_ptr(&self.info.name[0] as _) }
    }

    /// Returns the ValueType of this property.
    pub fn value_type(&self) -> ValueType {
        self.val_type.clone()
    }

    /// Returns whether this property is mutable.
    pub fn mutable(&self) -> bool {
        self.mutable
    }

    /// Returns whether this property can be atomically updated.
    pub fn atomic(&self) -> bool {
        self.atomic
    }
}

/// Describes the types of value that a property uses.
#[allow(clippy::upper_case_acronyms)]
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ValueType {
    /// A catch-all for any unknown types
    Unknown,
    /// A True or False type
    Boolean,
    /// An unsigned integer that has a min and max value
    UnsignedRange(u64, u64),
    /// A signed integer that has a min and max value
    SignedRange(i64, i64),
    /// A set of values that are mutually exclusive
    Enum(EnumValues),
    /// A set of values that can be combined
    Bitmask,
    /// A chunk of binary data that must be acquired
    Blob,
    /// A non-specific DRM object
    Object,
    /// A CRTC object
    CRTC,
    /// A Connector object
    Connector,
    /// An Encoder object
    Encoder,
    /// A Framebuffer object
    Framebuffer,
    /// A Plane object
    Plane,
    /// A Property object
    Property,
}

impl ValueType {
    /// Given a [`RawValue`], convert it into a specific [`Value`]
    pub fn convert_value(&self, value: RawValue) -> Value {
        match self {
            ValueType::Unknown => Value::Unknown(value),
            ValueType::Boolean => Value::Boolean(value != 0),
            ValueType::UnsignedRange(_, _) => Value::UnsignedRange(value),
            ValueType::SignedRange(_, _) => Value::SignedRange(value as i64),
            ValueType::Enum(values) => Value::Enum(values.get_value_from_raw_value(value)),
            ValueType::Bitmask => Value::Bitmask(value),
            ValueType::Blob => Value::Blob(value),
            ValueType::Object => Value::Object(bytemuck::cast(value as u32)),
            ValueType::CRTC => Value::CRTC(bytemuck::cast(value as u32)),
            ValueType::Connector => Value::Connector(bytemuck::cast(value as u32)),
            ValueType::Encoder => Value::Encoder(bytemuck::cast(value as u32)),
            ValueType::Framebuffer => Value::Framebuffer(bytemuck::cast(value as u32)),
            ValueType::Plane => Value::Plane(bytemuck::cast(value as u32)),
            ValueType::Property => Value::Property(bytemuck::cast(value as u32)),
        }
    }
}

/// The value of a property, in a typed format
#[allow(missing_docs)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Value<'a> {
    /// Unknown value
    Unknown(RawValue),
    /// Boolean value
    Boolean(bool),
    /// Unsigned range value
    UnsignedRange(u64),
    /// Signed range value
    SignedRange(i64),
    /// Enum Value
    Enum(Option<&'a EnumValue>),
    /// Bitmask value
    Bitmask(u64),
    /// Opaque (blob) value
    Blob(u64),
    /// Unknown object value
    Object(Option<RawResourceHandle>),
    /// Crtc object value
    CRTC(Option<super::crtc::Handle>),
    /// Connector object value
    Connector(Option<super::connector::Handle>),
    /// Encoder object value
    Encoder(Option<super::encoder::Handle>),
    /// Framebuffer object value
    Framebuffer(Option<super::framebuffer::Handle>),
    /// Plane object value
    Plane(Option<super::plane::Handle>),
    /// Property object value
    Property(Option<Handle>),
}

impl<'a> From<Value<'a>> for RawValue {
    fn from(value: Value<'a>) -> Self {
        match value {
            Value::Unknown(x) => x,
            Value::Boolean(true) => 1,
            Value::Boolean(false) => 0,
            Value::UnsignedRange(x) => x,
            Value::SignedRange(x) => x as u64,
            Value::Enum(val) => val.map_or(0, EnumValue::value),
            Value::Bitmask(x) => x,
            Value::Blob(x) => x,
            Value::Object(x) => bytemuck::cast::<_, u32>(x) as u64,
            Value::CRTC(x) => bytemuck::cast::<_, u32>(x) as u64,
            Value::Connector(x) => bytemuck::cast::<_, u32>(x) as u64,
            Value::Encoder(x) => bytemuck::cast::<_, u32>(x) as u64,
            Value::Framebuffer(x) => bytemuck::cast::<_, u32>(x) as u64,
            Value::Plane(x) => bytemuck::cast::<_, u32>(x) as u64,
            Value::Property(x) => bytemuck::cast::<_, u32>(x) as u64,
        }
    }
}

macro_rules! match_variant {
    ($this:ident, $variant:ident) => {
        if let Self::$variant(v) = *$this {
            Some(v)
        } else {
            None
        }
    };
}

impl<'a> Value<'a> {
    /// Boolean value
    pub fn as_boolean(&self) -> Option<bool> {
        match_variant!(self, Boolean)
    }

    /// Unsigned range value
    pub fn as_unsigned_range(&self) -> Option<u64> {
        match_variant!(self, UnsignedRange)
    }

    /// Signed range value
    pub fn as_signed_range(&self) -> Option<i64> {
        match_variant!(self, SignedRange)
    }

    /// Enum Value
    pub fn as_enum(&self) -> Option<&'a EnumValue> {
        match_variant!(self, Enum).flatten()
    }

    /// Bitmask value
    pub fn as_bitmask(&self) -> Option<u64> {
        match_variant!(self, Bitmask)
    }

    /// Opaque (blob) value
    pub fn as_blob(&self) -> Option<u64> {
        match_variant!(self, Blob)
    }

    /// Unknown object value
    pub fn as_object(&self) -> Option<RawResourceHandle> {
        match_variant!(self, Object).flatten()
    }

    /// Crtc object value
    pub fn as_crtc(&self) -> Option<super::crtc::Handle> {
        match_variant!(self, CRTC).flatten()
    }

    /// Connector object value
    pub fn as_connector(&self) -> Option<super::connector::Handle> {
        match_variant!(self, Connector).flatten()
    }

    /// Encoder object value
    pub fn as_encoder(&self) -> Option<super::encoder::Handle> {
        match_variant!(self, Encoder).flatten()
    }

    /// Framebuffer object value
    pub fn as_framebuffer(&self) -> Option<super::framebuffer::Handle> {
        match_variant!(self, Framebuffer).flatten()
    }

    /// Plane object value
    pub fn as_plane(&self) -> Option<super::plane::Handle> {
        match_variant!(self, Plane).flatten()
    }

    /// Property object value
    pub fn as_property(&self) -> Option<Handle> {
        match_variant!(self, Property).flatten()
    }
}

/// A single value of [`ValueType::Enum`] type
#[repr(transparent)]
#[derive(Copy, Clone, Hash, PartialEq, Eq, bytemuck::TransparentWrapper)]
pub struct EnumValue(ffi::drm_mode_property_enum);

impl EnumValue {
    /// Returns the [`RawValue`] of this value
    pub fn value(&self) -> RawValue {
        self.0.value
    }

    /// Returns the name of this value
    pub fn name(&self) -> &std::ffi::CStr {
        unsafe { std::ffi::CStr::from_ptr(&self.0.name[0] as _) }
    }
}

impl From<ffi::drm_mode_property_enum> for EnumValue {
    fn from(inner: ffi::drm_mode_property_enum) -> Self {
        EnumValue(inner)
    }
}

impl std::fmt::Debug for EnumValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("EnumValue")
            .field("value", &self.value())
            .field("name", &self.name())
            .finish()
    }
}

/// A set of [`EnumValue`]s for a single property
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct EnumValues {
    pub(crate) values: Vec<u64>,
    pub(crate) enums: Vec<EnumValue>,
}

impl EnumValues {
    /// Returns a tuple containing slices to the [`RawValue`]s and the [`EnumValue`]s
    pub fn values(&self) -> (&[RawValue], &[EnumValue]) {
        (&self.values, &self.enums)
    }

    /// Returns an [`EnumValue`] for a [`RawValue`], or [`None`] if `value` is
    /// not part of this [`EnumValues`].
    pub fn get_value_from_raw_value(&self, value: RawValue) -> Option<&EnumValue> {
        let (values, enums) = self.values();
        let index = if values.get(value as usize) == Some(&value) {
            // Early-out: indices match values
            value as usize
        } else {
            values.iter().position(|&v| v == value)?
        };
        Some(&enums[index])
    }
}
