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
use ffi;

/// A handle to a property
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Handle(u32);

impl control::Handle for Handle {
    const OBJ_TYPE: u32 = ffi::DRM_MODE_OBJECT_PROPERTY;

    fn from_raw(raw: u32) -> Self {
        Handle(raw)
    }

    fn into_raw(self) -> u32 {
        let Handle(raw) = self;
        raw
    }
}

/// Information about a property
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Info {
    pub(crate) handle: Handle,
}
