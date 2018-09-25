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

use util::*;

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

/// A value not yet bound to any particular property.
///
/// For simplicity, a property value is initially unbound to any particular
/// property handle. It is the user's responsibility to keep track of which
/// resource and property handle this value needs to be bound to.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct NonBoundedValue {
    pub(crate) raw: u64
}

/// A set of property handles and their values.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct BoundedValueList {
    pub(crate) handles: Buffer4x24<Handle>,
    pub(crate) values: Buffer8x24<NonBoundedValue>,
}

impl BoundedValueList {
    /// Returns a list of handles for each property in the list.
    pub fn handles(&self) -> &[Handle] {
        unsafe {
            self.handles.as_slice()
        }
    }

    /// Returns a list of non-bounded values for each property in the list.
    pub fn nonbounded_values(&self) -> &[NonBoundedValue] {
        unsafe {
            self.values.as_slice()
        }
    }
}
