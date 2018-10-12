macro_rules! __fourcc_code {
    ($a:expr, $b:expr, $c:expr, $d:expr) => (
        ($a as u32) | (($b as u32) << 8) | (($c as u32) << 16) | (($d as u32) << 24)
    )
}

/* color index */
pub const DRM_FORMAT_C8 	:u32 = __fourcc_code!('C', '8', ' ', ' '); /* [7:0] C */

/* 8 bpp Red */
pub const DRM_FORMAT_R8		:u32 = __fourcc_code!('R', '8', ' ', ' '); /* [7:0] R */

/* 16 bpp RG */
pub const DRM_FORMAT_GR88		:u32 = __fourcc_code!('G', 'R', '8', '8'); /* [15:0] G:R 8:8 little endian */

/* 8 bpp RGB */
pub const DRM_FORMAT_RGB332	:u32 = __fourcc_code!('R', 'G', 'B', '8'); /* [7:0] R:G:B 3:3:2 */
pub const DRM_FORMAT_BGR233	:u32 = __fourcc_code!('B', 'G', 'R', '8'); /* [7:0] B:G:R 2:3:3 */

/* 16 bpp RGB */
pub const DRM_FORMAT_XRGB4444	:u32 = __fourcc_code!('X', 'R', '1', '2'); /* [15:0] x:R:G:B 4:4:4:4 little endian */
pub const DRM_FORMAT_XBGR4444	:u32 = __fourcc_code!('X', 'B', '1', '2'); /* [15:0] x:B:G:R 4:4:4:4 little endian */
pub const DRM_FORMAT_RGBX4444	:u32 = __fourcc_code!('R', 'X', '1', '2'); /* [15:0] R:G:B:x 4:4:4:4 little endian */
pub const DRM_FORMAT_BGRX4444	:u32 = __fourcc_code!('B', 'X', '1', '2'); /* [15:0] B:G:R:x 4:4:4:4 little endian */

pub const DRM_FORMAT_ARGB4444	:u32 = __fourcc_code!('A', 'R', '1', '2'); /* [15:0] A:R:G:B 4:4:4:4 little endian */
pub const DRM_FORMAT_ABGR4444	:u32 = __fourcc_code!('A', 'B', '1', '2'); /* [15:0] A:B:G:R 4:4:4:4 little endian */
pub const DRM_FORMAT_RGBA4444	:u32 = __fourcc_code!('R', 'A', '1', '2'); /* [15:0] R:G:B:A 4:4:4:4 little endian */
pub const DRM_FORMAT_BGRA4444	:u32 = __fourcc_code!('B', 'A', '1', '2'); /* [15:0] B:G:R:A 4:4:4:4 little endian */

pub const DRM_FORMAT_XRGB1555	:u32 = __fourcc_code!('X', 'R', '1', '5'); /* [15:0] x:R:G:B 1:5:5:5 little endian */
pub const DRM_FORMAT_XBGR1555	:u32 = __fourcc_code!('X', 'B', '1', '5'); /* [15:0] x:B:G:R 1:5:5:5 little endian */
pub const DRM_FORMAT_RGBX5551	:u32 = __fourcc_code!('R', 'X', '1', '5'); /* [15:0] R:G:B:x 5:5:5:1 little endian */
pub const DRM_FORMAT_BGRX5551	:u32 = __fourcc_code!('B', 'X', '1', '5'); /* [15:0] B:G:R:x 5:5:5:1 little endian */

pub const DRM_FORMAT_ARGB1555	:u32 = __fourcc_code!('A', 'R', '1', '5'); /* [15:0] A:R:G:B 1:5:5:5 little endian */
pub const DRM_FORMAT_ABGR1555	:u32 = __fourcc_code!('A', 'B', '1', '5'); /* [15:0] A:B:G:R 1:5:5:5 little endian */
pub const DRM_FORMAT_RGBA5551	:u32 = __fourcc_code!('R', 'A', '1', '5'); /* [15:0] R:G:B:A 5:5:5:1 little endian */
pub const DRM_FORMAT_BGRA5551	:u32 = __fourcc_code!('B', 'A', '1', '5'); /* [15:0] B:G:R:A 5:5:5:1 little endian */

pub const DRM_FORMAT_RGB565	:u32 = __fourcc_code!('R', 'G', '1', '6'); /* [15:0] R:G:B 5:6:5 little endian */
pub const DRM_FORMAT_BGR565	:u32 = __fourcc_code!('B', 'G', '1', '6'); /* [15:0] B:G:R 5:6:5 little endian */

/* 24 bpp RGB */
pub const DRM_FORMAT_RGB888	:u32 = __fourcc_code!('R', 'G', '2', '4'); /* [23:0] R:G:B little endian */
pub const DRM_FORMAT_BGR888	:u32 = __fourcc_code!('B', 'G', '2', '4'); /* [23:0] B:G:R little endian */

/* 32 bpp RGB */
pub const DRM_FORMAT_XRGB8888	:u32 = __fourcc_code!('X', 'R', '2', '4'); /* [31:0] x:R:G:B 8:8:8:8 little endian */
pub const DRM_FORMAT_XBGR8888	:u32 = __fourcc_code!('X', 'B', '2', '4'); /* [31:0] x:B:G:R 8:8:8:8 little endian */
pub const DRM_FORMAT_RGBX8888	:u32 = __fourcc_code!('R', 'X', '2', '4'); /* [31:0] R:G:B:x 8:8:8:8 little endian */
pub const DRM_FORMAT_BGRX8888	:u32 = __fourcc_code!('B', 'X', '2', '4'); /* [31:0] B:G:R:x 8:8:8:8 little endian */

pub const DRM_FORMAT_ARGB8888	:u32 = __fourcc_code!('A', 'R', '2', '4'); /* [31:0] A:R:G:B 8:8:8:8 little endian */
pub const DRM_FORMAT_ABGR8888	:u32 = __fourcc_code!('A', 'B', '2', '4'); /* [31:0] A:B:G:R 8:8:8:8 little endian */
pub const DRM_FORMAT_RGBA8888	:u32 = __fourcc_code!('R', 'A', '2', '4'); /* [31:0] R:G:B:A 8:8:8:8 little endian */
pub const DRM_FORMAT_BGRA8888	:u32 = __fourcc_code!('B', 'A', '2', '4'); /* [31:0] B:G:R:A 8:8:8:8 little endian */

pub const DRM_FORMAT_XRGB2101010	:u32 = __fourcc_code!('X', 'R', '3', '0'); /* [31:0] x:R:G:B 2:10:10:10 little endian */
pub const DRM_FORMAT_XBGR2101010	:u32 = __fourcc_code!('X', 'B', '3', '0'); /* [31:0] x:B:G:R 2:10:10:10 little endian */
pub const DRM_FORMAT_RGBX1010102	:u32 = __fourcc_code!('R', 'X', '3', '0'); /* [31:0] R:G:B:x 10:10:10:2 little endian */
pub const DRM_FORMAT_BGRX1010102	:u32 = __fourcc_code!('B', 'X', '3', '0'); /* [31:0] B:G:R:x 10:10:10:2 little endian */

pub const DRM_FORMAT_ARGB2101010	:u32 = __fourcc_code!('A', 'R', '3', '0'); /* [31:0] A:R:G:B 2:10:10:10 little endian */
pub const DRM_FORMAT_ABGR2101010	:u32 = __fourcc_code!('A', 'B', '3', '0'); /* [31:0] A:B:G:R 2:10:10:10 little endian */
pub const DRM_FORMAT_RGBA1010102	:u32 = __fourcc_code!('R', 'A', '3', '0'); /* [31:0] R:G:B:A 10:10:10:2 little endian */
pub const DRM_FORMAT_BGRA1010102	:u32 = __fourcc_code!('B', 'A', '3', '0'); /* [31:0] B:G:R:A 10:10:10:2 little endian */

/* packed YCbCr */
pub const DRM_FORMAT_YUYV		:u32 = __fourcc_code!('Y', 'U', 'Y', 'V'); /* [31:0] Cr0:Y1:Cb0:Y0 8:8:8:8 little endian */
pub const DRM_FORMAT_YVYU		:u32 = __fourcc_code!('Y', 'V', 'Y', 'U'); /* [31:0] Cb0:Y1:Cr0:Y0 8:8:8:8 little endian */
pub const DRM_FORMAT_UYVY		:u32 = __fourcc_code!('U', 'Y', 'V', 'Y'); /* [31:0] Y1:Cr0:Y0:Cb0 8:8:8:8 little endian */
pub const DRM_FORMAT_VYUY		:u32 = __fourcc_code!('V', 'Y', 'U', 'Y'); /* [31:0] Y1:Cb0:Y0:Cr0 8:8:8:8 little endian */

pub const DRM_FORMAT_AYUV		:u32 = __fourcc_code!('A', 'Y', 'U', 'V'); /* [31:0] A:Y:Cb:Cr 8:8:8:8 little endian */
