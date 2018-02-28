//! Color formats using standard FourCC.

use ffi::fourcc::*;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(missing_docs)]
/// Possible pixel formats of a buffer
pub enum PixelFormat {
    C8,
    R8,
    GR88,

    RGB332,
    BGR233,

    XRGB4444,
    XBGR4444,
    RGBX4444,
    BGRX4444,

    ARGB4444,
    ABGR4444,
    RGBA4444,
    BGRA4444,

    XRGB1555,
    XBGR1555,
    RGBX5551,
    BGRX5551,

    ARGB1555,
    ABGR1555,
    RGBA5551,
    BGRA5551,

    RGB565,
    BGR565,

    XRGB8888,
    XBGR8888,
    RGBX8888,
    BGRX8888,

    ARGB8888,
    ABGR8888,
    RGBA8888,
    BGRA8888,

    XRGB2101010,
    XBGR2101010,
    RGBX1010102,
    BGRX1010102,

    ARGB2101010,
    ABGR2101010,
    RGBA1010102,
    BGRA1010102,

    YUYV,
    YVYU,
    UYVY,
    VYUY,

    AYUV,
}

impl PixelFormat {
    /// Convert into the raw fourcc code
    pub fn as_raw(&self) -> u32 {
        use self::PixelFormat::*;
        match *self {
            C8 => DRM_FORMAT_C8,
            R8 => DRM_FORMAT_R8,
            GR88 => DRM_FORMAT_GR88,

            RGB332 => DRM_FORMAT_RGB332,
            BGR233 => DRM_FORMAT_BGR233,

            XRGB4444 => DRM_FORMAT_XRGB4444,
            XBGR4444 => DRM_FORMAT_XBGR4444,
            RGBX4444 => DRM_FORMAT_RGBX4444,
            BGRX4444 => DRM_FORMAT_BGRX4444,

            ARGB4444 => DRM_FORMAT_ARGB4444,
            ABGR4444 => DRM_FORMAT_ABGR4444,
            RGBA4444 => DRM_FORMAT_RGBA4444,
            BGRA4444 => DRM_FORMAT_BGRA4444,

            XRGB1555 => DRM_FORMAT_XRGB1555,
            XBGR1555 => DRM_FORMAT_XBGR1555,
            RGBX5551 => DRM_FORMAT_RGBX5551,
            BGRX5551 => DRM_FORMAT_BGRX5551,

            ARGB1555 => DRM_FORMAT_ARGB1555,
            ABGR1555 => DRM_FORMAT_ABGR1555,
            RGBA5551 => DRM_FORMAT_RGBA4444,
            BGRA5551 => DRM_FORMAT_RGBA5551,

            RGB565 => DRM_FORMAT_RGB565,
            BGR565 => DRM_FORMAT_BGR565,

            XRGB8888 => DRM_FORMAT_XRGB8888,
            XBGR8888 => DRM_FORMAT_XBGR8888,
            RGBX8888 => DRM_FORMAT_RGBX8888,
            BGRX8888 => DRM_FORMAT_BGRX8888,

            ARGB8888 => DRM_FORMAT_ARGB8888,
            ABGR8888 => DRM_FORMAT_ABGR8888,
            RGBA8888 => DRM_FORMAT_RGBA8888,
            BGRA8888 => DRM_FORMAT_BGRA8888,

            XRGB2101010 => DRM_FORMAT_XRGB2101010,
            XBGR2101010 => DRM_FORMAT_XBGR2101010,
            RGBX1010102 => DRM_FORMAT_RGBX1010102,
            BGRX1010102 => DRM_FORMAT_BGRX1010102,

            ARGB2101010 => DRM_FORMAT_ARGB2101010,
            ABGR2101010 => DRM_FORMAT_ABGR2101010,
            RGBA1010102 => DRM_FORMAT_RGBA1010102,
            BGRA1010102 => DRM_FORMAT_BGRA1010102,

            YUYV => DRM_FORMAT_YUYV,
            YVYU => DRM_FORMAT_YVYU,
            UYVY => DRM_FORMAT_UYVY,
            VYUY => DRM_FORMAT_VYUY,

            AYUV => DRM_FORMAT_AYUV,
        }
    }

    /// Depth value of the format
    pub fn depth(&self) -> u32 {
        unimplemented!()
    }

    /// Bytes per pixel of the used format
    pub fn bpp(&self) -> u32 {
        unimplemented!()
    }
}
