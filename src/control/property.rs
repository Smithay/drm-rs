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

use control;
use drm_ffi as ffi;
use util::*;

/// A handle to a property
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Handle(control::ResourceHandle);

impl AsRef<control::ResourceHandle> for Handle {
    fn as_ref(&self) -> &control::ResourceHandle {
        &self.0
    }
}

impl control::ResourceType for Handle {
    const FFI_TYPE: u32 = ffi::DRM_MODE_OBJECT_PROPERTY;
}

/// Information about a property
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Info {
    pub(crate) handle: Handle,
}

/// A raw property value that does not have a specific property type
pub type RawPropertyValue = u64;

pub struct BetaInfo {
    pub(crate) handle: Handle,
    pub(crate) name: SmallOsString,
    pub(crate) value_type: ValueType
}

pub enum ValueType {
    UnsignedRange,
    SignedRange,
    Enum,
    Bitmask,
    Object,
    Blob,
}
