//! # Colorop
//!
//! A color transform operation applied on a plane.
//!
//! They form fixed chains through the NEXT property of each colorop.
//! The COLOR_PIPELINE property of eligible planes can be set to the starting
//! colorop of a chain.
//!
//! [`ClientCapability::PlaneColorPipeline`] must be set for these properties to
//! be available.

use crate::control;
use drm_ffi as ffi;

/// A handle to a specific Colorop
#[repr(transparent)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Handle(control::RawResourceHandle);

// Safety: Handle is repr(transparent) over NonZeroU32
unsafe impl bytemuck::ZeroableInOption for Handle {}
unsafe impl bytemuck::PodInOption for Handle {}

impl From<Handle> for control::RawResourceHandle {
    fn from(handle: Handle) -> Self {
        handle.0
    }
}

impl From<Handle> for u32 {
    fn from(handle: Handle) -> Self {
        handle.0.into()
    }
}

impl From<control::RawResourceHandle> for Handle {
    fn from(handle: control::RawResourceHandle) -> Self {
        Handle(handle)
    }
}

impl control::ResourceHandle for Handle {
    const FFI_TYPE: u32 = ffi::DRM_MODE_OBJECT_COLOROP;
}

impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("colorop::Handle").field(&self.0).finish()
    }
}

/// 32-bit per channel color LUT entry.
///
/// Values are mapped linearly to 0.0 - 1.0 range, with 0x0 == 0.0 and
/// 0xffffffff == 1.0.
pub type LUT32 = ffi::drm_color_lut32;

/// Conversion matrix with 3x4 dimensions in S31.32 sign-magnitude (not two's
/// complement!) format.
///```text
/// out   matrix          in
/// |R|   |0  1  2  3 |   | R |
/// |G| = |4  5  6  7 | x | G |
/// |B|   |8  9  10 11|   | B |
///                       |1.0|
///```
pub type CTM3x4 = ffi::drm_color_ctm_3x4;

/// Values for the TYPE colorop property
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    /// A 1D curve that is being applied to all color channels. The curve is
    /// specified via the CURVE_1D_TYPE colorop property.
    _1DCurve = ffi::DRM_COLOROP_1D_CURVE,
    /// A simple 1D LUT of uniformly spaced [`LUT32`] entries, packed into a
    /// blob via the DATA property.
    /// The driver's expected LUT size is advertised via the SIZE property.
    ///
    /// The DATA blob is an array of [`LUT32`] with size of "size".
    _1DLUT = ffi::DRM_COLOROP_1D_LUT,
    /// A 3x4 matrix. Its values are specified via the [`CTM3x4`] provided via
    /// the DATA property
    Ctm3x4 = ffi::DRM_COLOROP_CTM_3X4,
    /// A simple multiplier, applied to all color values. The multiplier is
    /// specified as a S31.32 via the MULTIPLIER property.
    Multiplier = ffi::DRM_COLOROP_MULTIPLIER,
    /// A 3D LUT of [`LUT32`] entries, packed into a blob via the DATA property.
    /// The driver's expected LUT size is advertised via the SIZE property,
    /// i.e., a 3D LUT with 17x17x17 entries will have SIZE set to 17.
    ///
    /// The DATA blob is a 3D array of struct [`LUT32`] with
    /// dimension length of "size". The LUT elements are traversed like so:
    ///```text
    ///   for B in range 0..n
    ///     for G in range 0..n
    ///       for R in range 0..n
    ///        index = R + n * (G + n * B)
    ///          color = lut3d[index]
    ///```
    _3DLut = ffi::DRM_COLOROP_3D_LUT,
}

/// Values for the LUT3D_INTERPOLATION colorop property
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LUT3DInterpolationType {
    /// Tetrahedral 3DLUT interpolation
    Tetrahedral = ffi::DRM_COLOROP_LUT3D_INTERPOLATION_TETRAHEDRAL,
}

/// Values for the LUT1D_INTERPOLATION colorop property
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LUT1DInterpolationType {
    /// Linear interpolation. Values between points of the LUT will be linearly
    /// interpolated.
    Linear = ffi::DRM_COLOROP_LUT1D_INTERPOLATION_LINEAR,
}
