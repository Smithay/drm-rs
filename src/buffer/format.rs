//! Color formats using standard FourCC.

use drm_ffi::fourcc::*;

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

    pub fn from_raw(raw: u32) -> Option<PixelFormat> {
        use self::PixelFormat::*;
        Some(match raw {
           DRM_FORMAT_C8 => C8,
           DRM_FORMAT_R8 => R8,
           DRM_FORMAT_GR88 => GR88,

           DRM_FORMAT_RGB332 => RGB332,
           DRM_FORMAT_BGR233 => BGR233,

           DRM_FORMAT_XRGB4444 => XRGB4444,
           DRM_FORMAT_XBGR4444 => XBGR4444,
           DRM_FORMAT_RGBX4444 => RGBX4444,
           DRM_FORMAT_BGRX4444 => BGRX4444,

           DRM_FORMAT_ARGB4444 => ARGB4444,
           DRM_FORMAT_ABGR4444 => ABGR4444,
           DRM_FORMAT_RGBA4444 => RGBA4444,
           DRM_FORMAT_BGRA4444 => BGRA4444,

           DRM_FORMAT_XRGB1555 => XRGB1555,
           DRM_FORMAT_XBGR1555 => XBGR1555,
           DRM_FORMAT_RGBX5551 => RGBX5551,
           DRM_FORMAT_BGRX5551 => BGRX5551,

           DRM_FORMAT_ARGB1555 => ARGB1555,
           DRM_FORMAT_ABGR1555 => ABGR1555,
           DRM_FORMAT_RGBA5551 => RGBA4444,
           DRM_FORMAT_BGRA5551 => RGBA5551,

           DRM_FORMAT_RGB565 => RGB565,
           DRM_FORMAT_BGR565 => BGR565,

           DRM_FORMAT_XRGB8888 => XRGB8888,
           DRM_FORMAT_XBGR8888 => XBGR8888,
           DRM_FORMAT_RGBX8888 => RGBX8888,
           DRM_FORMAT_BGRX8888 => BGRX8888,

           DRM_FORMAT_ARGB8888 => ARGB8888,
           DRM_FORMAT_ABGR8888 => ABGR8888,
           DRM_FORMAT_RGBA8888 => RGBA8888,
           DRM_FORMAT_BGRA8888 => BGRA8888,

           DRM_FORMAT_XRGB2101010 => XRGB2101010,
           DRM_FORMAT_XBGR2101010 => XBGR2101010,
           DRM_FORMAT_RGBX1010102 => RGBX1010102,
           DRM_FORMAT_BGRX1010102 => BGRX1010102,

           DRM_FORMAT_ARGB2101010 => ARGB2101010,
           DRM_FORMAT_ABGR2101010 => ABGR2101010,
           DRM_FORMAT_RGBA1010102 => RGBA1010102,
           DRM_FORMAT_BGRA1010102 => BGRA1010102,

           DRM_FORMAT_YUYV => YUYV,
           DRM_FORMAT_YVYU => YVYU,
           DRM_FORMAT_UYVY => UYVY,
           DRM_FORMAT_VYUY => VYUY,

           DRM_FORMAT_AYUV => AYUV,

           _ => { return None; }
        })
    }

    /// The depth in bits per pixel.
    pub fn depth(&self) -> u32 {
        match *self {
            // TODO
            PixelFormat::C8 => unimplemented!(),
            PixelFormat::R8 => unimplemented!(),
            PixelFormat::GR88 => unimplemented!(),

            PixelFormat::RGB332 => 8,
            PixelFormat::BGR233 => 8,

            PixelFormat::XRGB4444 => 12,
            PixelFormat::XBGR4444 => 12,
            PixelFormat::RGBX4444 => 12,
            PixelFormat::BGRX4444 => 12,

            PixelFormat::ARGB4444 => 16,
            PixelFormat::ABGR4444 => 16,
            PixelFormat::RGBA4444 => 16,
            PixelFormat::BGRA4444 => 16,

            PixelFormat::XRGB1555 => 15,
            PixelFormat::XBGR1555 => 15,
            PixelFormat::RGBX5551 => 15,
            PixelFormat::BGRX5551 => 15,

            PixelFormat::ARGB1555 => 16,
            PixelFormat::ABGR1555 => 16,
            PixelFormat::RGBA5551 => 16,
            PixelFormat::BGRA5551 => 16,

            PixelFormat::RGB565 => 16,
            PixelFormat::BGR565 => 16,

            PixelFormat::XRGB8888 => 24,
            PixelFormat::XBGR8888 => 24,
            PixelFormat::RGBX8888 => 24,
            PixelFormat::BGRX8888 => 24,

            PixelFormat::ARGB8888 => 32,
            PixelFormat::ABGR8888 => 32,
            PixelFormat::RGBA8888 => 32,
            PixelFormat::BGRA8888 => 32,

            PixelFormat::XRGB2101010 => 30,
            PixelFormat::XBGR2101010 => 30,
            PixelFormat::RGBX1010102 => 30,
            PixelFormat::BGRX1010102 => 30,

            PixelFormat::ARGB2101010 => 32,
            PixelFormat::ABGR2101010 => 32,
            PixelFormat::RGBA1010102 => 32,
            PixelFormat::BGRA1010102 => 32,

            // TODO
            PixelFormat::YUYV => unimplemented!(),
            PixelFormat::YVYU => unimplemented!(),
            PixelFormat::UYVY => unimplemented!(),
            PixelFormat::VYUY => unimplemented!(),

            PixelFormat::AYUV => unimplemented!(),
        }
    }

    /// Bytes per pixel of the used format
    pub fn bpp(&self) -> u32 {
        match *self {
            PixelFormat::C8 => 8,
            PixelFormat::R8 => 8,
            PixelFormat::GR88 => 16,

            PixelFormat::RGB332 => 8,
            PixelFormat::BGR233 => 8,

            PixelFormat::XRGB4444 => 16,
            PixelFormat::XBGR4444 => 16,
            PixelFormat::RGBX4444 => 16,
            PixelFormat::BGRX4444 => 16,

            PixelFormat::ARGB4444 => 16,
            PixelFormat::ABGR4444 => 16,
            PixelFormat::RGBA4444 => 16,
            PixelFormat::BGRA4444 => 16,

            PixelFormat::XRGB1555 => 16,
            PixelFormat::XBGR1555 => 16,
            PixelFormat::RGBX5551 => 16,
            PixelFormat::BGRX5551 => 16,

            PixelFormat::ARGB1555 => 16,
            PixelFormat::ABGR1555 => 16,
            PixelFormat::RGBA5551 => 16,
            PixelFormat::BGRA5551 => 16,

            PixelFormat::RGB565 => 16,
            PixelFormat::BGR565 => 16,

            PixelFormat::XRGB8888 => 24,
            PixelFormat::XBGR8888 => 24,
            PixelFormat::RGBX8888 => 24,
            PixelFormat::BGRX8888 => 24,

            PixelFormat::ARGB8888 => 32,
            PixelFormat::ABGR8888 => 32,
            PixelFormat::RGBA8888 => 32,
            PixelFormat::BGRA8888 => 32,

            PixelFormat::XRGB2101010 => 32,
            PixelFormat::XBGR2101010 => 32,
            PixelFormat::RGBX1010102 => 32,
            PixelFormat::BGRX1010102 => 32,

            PixelFormat::ARGB2101010 => 32,
            PixelFormat::ABGR2101010 => 32,
            PixelFormat::RGBA1010102 => 32,
            PixelFormat::BGRA1010102 => 32,

            PixelFormat::YUYV => 16,
            PixelFormat::YVYU => 16,
            PixelFormat::UYVY => 16,
            PixelFormat::VYUY => 16,

            PixelFormat::AYUV => 32,
        }
    }
}
