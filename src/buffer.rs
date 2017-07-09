use drm_sys::*;

/// The underlying handle for a buffer
pub type RawHandle = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)];
pub struct GemHandle(RawHandle);

pub trait GemBuffer {
    fn handle(&self) -> GemHandle;
}
