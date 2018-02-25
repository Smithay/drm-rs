//! Module for pretty formatting of our control types.
//!
//! Since our structures are minimal wrappers around raw DRM types, the default
//! `Debug` derivation tends to print out unuseful information, such as pointer
//! locations and other raw types. This module implements `Debug` for these
//! types in a more human readable way.

use std::fmt;

use control::ResourceHandle;
use control::ResourceInfo;
use control::RawHandle;
use control;

use control::connector;
use control::encoder;
use control::crtc;
use control::framebuffer;

/// This can automatically implement `Debug` for any RawHandle.
///
/// Unlike the other implementations, we don't use `DebugStruct` because it will
/// align it awkwardly if you use the "{:#?}" format specifier. Instead we just
/// write it all out on a single line. If you prefer it the other way, then
/// issue a bug report.
macro_rules! impl_newtype_debug {
    ($my_type:ty) => {
        impl fmt::Debug for $my_type {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}({})", Self::DEBUG_NAME, RawHandle::from(*self))
            }
        }
    }
}

impl_newtype_debug!(connector::Handle);
impl_newtype_debug!(encoder::Handle);
impl_newtype_debug!(crtc::Handle);
impl_newtype_debug!(framebuffer::Handle);

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
            .field("possible_encoders", &self.encoders())
            .field("properties", &self.properties())
            .field("property_values", &self.prop_values())
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
            .field("framebuffer", &self.fb())
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
