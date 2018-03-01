//! Module for pretty formatting of our control types.
//!
//! Since our structures are minimal wrappers around raw DRM types, the default
//! `Debug` derivation tends to print out unuseful information, such as pointer
//! locations and other raw types. This module implements `Debug` for these
//! types in a more human readable way.

use std::fmt;

use control::ResourceInfo;
use control::RawHandle;
use control;

use control::connector;
use control::encoder;
use control::crtc;
use control::framebuffer;
use control::plane;
use control::property;

/// This can automatically implement `Debug` for any RawHandle.
///
/// Unlike the other implementations, we don't use `DebugStruct` because it will
/// align it awkwardly if you use the "{:#?}" format specifier. Instead we just
/// write it all out on a single line. If you prefer it the other way, then
/// issue a bug report.
macro_rules! impl_newtype_debug {
    ($my_type:ty, $name:expr) => {
        impl fmt::Debug for $my_type {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}({})", $name, RawHandle::from(*self))
            }
        }
    }
}

impl_newtype_debug!(connector::Handle, "connector::Handle");
impl_newtype_debug!(encoder::Handle, "encoder::Handle");
impl_newtype_debug!(crtc::Handle, "crtc::Handle");
impl_newtype_debug!(framebuffer::Handle, "framebuffer::Handle");
impl_newtype_debug!(plane::Handle, "plane::Handle");
impl_newtype_debug!(property::Handle, "property::Handle");

/// Here's just a simple point type that can be used to debug tuples of
/// integral primitives. Same as `impl_newtype_debug`, this is done to
/// print on a single line when using the "{:#?}" format specifier.
struct DebugPoint<T: fmt::Display>(T, T);

impl<T: fmt::Display> fmt::Debug for DebugPoint<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

impl fmt::Debug for control::ResourceHandles {
    fn fmt<'a>(&'a self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ResourceHandles")
            .field("connectors", &self.connectors())
            .field("encoders", &self.encoders())
            .field("crtcs", &self.crtcs())
            .field("framebuffers", &self.framebuffers())
            .finish()
    }
}

impl fmt::Debug for control::PlaneResourceHandles {
    fn fmt<'a>(&'a self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PlaneResourceHandles")
            .field("planes", &self.planes())
            .finish()
    }
}

impl fmt::Debug for connector::Info {
    fn fmt<'a>(&'a self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("connector::Info")
            .field("handle", &self.handle())
            .field("type", &self.connector_type())
            .field("state", &self.connection_state())
            .field("encoder", &self.current_encoder())
            .field("possible_encoders", &self.possible_encoders())
            .field("property_handles", &self.property_handles())
            .field("property_values", &self.property_values())
            .finish()
    }
}

impl fmt::Debug for encoder::Info {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("encoder::Info")
            .field("handle", &self.handle())
            .field("type", &self.encoder_type())
            .field("crtc", &self.current_crtc())
            .finish()
    }
}

impl fmt::Debug for crtc::Info {
    fn fmt<'a>(&'a self, f: &mut fmt::Formatter) -> fmt::Result {
        let pos = DebugPoint(self.position().0, self.position().1);

        f.debug_struct("crtc::Info")
            .field("handle", &self.handle())
            .field("position", &pos)
            .field("framebuffer", &self.current_framebuffer())
            .field("gamma_size", &self.gamma_size())
            .finish()
    }
}

impl fmt::Debug for framebuffer::Info {
    fn fmt<'a>(&'a self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("framebuffer::Info")
            .field("handle", &self.handle())
            .finish()
    }
}

impl fmt::Debug for plane::Info {
    fn fmt<'a>(&'a self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("plane::Info")
            .field("handle", &self.handle())
            .field("crtc", &self.current_crtc())
            .field("framebuffer", &self.current_framebuffer())
            .field("possible_crtcs", &self.possible_crtcs())
            .field("gamma_size", &self.gamma_size())
            .field("formats", &self.formats())
            .finish()
    }
}

impl fmt::Debug for property::Info {
    fn fmt<'a>(&'a self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("property::Info")
            .field("handle", &self.handle())
            .field("name", &self.name())
            .finish()
    }
}
