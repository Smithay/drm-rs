#![allow(dead_code)]

use nix::libc;
pub use drm_sys::*;

// The type to be used as an buffer.
pub type Buffer<T> = Vec<T>;

// Creates a buffer to be modified by an FFI function.
macro_rules! ffi_buf {
    ( $ptr:expr, $sz:expr) => (
        {
            let mut buf = unsafe { vec![mem::zeroed(); $sz as usize] };
            *(&mut $ptr) = unsafe { mem::transmute(buf.as_mut_ptr()) };
            buf
        }
    )
}

ioctl!(readwrite ioctl_version with DRM_IOCTL_BASE, 0x00; drm_version);
ioctl!(readwrite ioctl_get_unique with DRM_IOCTL_BASE, 0x01; drm_unique);

ioctl!(read ioctl_get_magic with DRM_IOCTL_BASE, 0x02; drm_auth);
ioctl!(readwrite ioctl_irq_busid with DRM_IOCTL_BASE, 0x03; drm_irq_busid);
ioctl!(readwrite ioctl_get_map with DRM_IOCTL_BASE, 0x04; drm_map);
ioctl!(readwrite ioctl_get_client with DRM_IOCTL_BASE, 0x05; drm_client);
ioctl!(read ioctl_get_stats with DRM_IOCTL_BASE, 0x06; drm_stats);
ioctl!(readwrite ioctl_set_version with DRM_IOCTL_BASE, 0x07; drm_set_version);
ioctl!(write ioctl_modeset_ctl with DRM_IOCTL_BASE, 0x08; drm_modeset_ctl);
ioctl!(write ioctl_gem_close with DRM_IOCTL_BASE, 0x09; drm_gem_close);
ioctl!(readwrite ioctl_gem_flink with DRM_IOCTL_BASE, 0x0a; drm_gem_flink);
ioctl!(readwrite ioctl_gem_open with DRM_IOCTL_BASE, 0x0b; drm_gem_open);
ioctl!(readwrite ioctl_get_cap with DRM_IOCTL_BASE, 0x0c; drm_get_cap);
ioctl!(write ioctl_set_client_cap with DRM_IOCTL_BASE, 0x0d; drm_set_client_cap);

ioctl!(write ioctl_set_unique with DRM_IOCTL_BASE, 0x10; drm_unique);
ioctl!(write ioctl_auth_magic with DRM_IOCTL_BASE, 0x11; drm_auth);
ioctl!(readwrite ioctl_block with DRM_IOCTL_BASE, 0x12; drm_block);
ioctl!(readwrite ioctl_unblock with DRM_IOCTL_BASE, 0x13; drm_block);
ioctl!(write ioctl_control with DRM_IOCTL_BASE, 0x14; drm_control);
ioctl!(readwrite ioctl_add_map with DRM_IOCTL_BASE, 0x15; drm_map);
ioctl!(readwrite ioctl_add_bufs with DRM_IOCTL_BASE, 0x16; drm_buf_desc);
ioctl!(write ioctl_mark_bufs with DRM_IOCTL_BASE, 0x17; drm_buf_desc);
ioctl!(readwrite ioctl_info_bufs with DRM_IOCTL_BASE, 0x18; drm_buf_info);
ioctl!(readwrite ioctl_map_bufs with DRM_IOCTL_BASE, 0x19; drm_buf_map);
ioctl!(write ioctl_free_bufs with DRM_IOCTL_BASE, 0x1a; drm_buf_free);

ioctl!(write ioctl_rm_map with DRM_IOCTL_BASE, 0x1b; drm_map);

ioctl!(write ioctl_set_sarea_ctx with DRM_IOCTL_BASE, 0x1c; drm_ctx_priv_map);
ioctl!(readwrite ioctl_get_sarea_ctx
       with DRM_IOCTL_BASE, 0x1d; drm_ctx_priv_map);

ioctl!(none ioctl_set_master with DRM_IOCTL_BASE, 0x1e);
ioctl!(none ioctl_drop_master with DRM_IOCTL_BASE, 0x1f);

ioctl!(readwrite ioctl_add_ctx with DRM_IOCTL_BASE, 0x20; drm_ctx);
ioctl!(readwrite ioctl_rm_ctx with DRM_IOCTL_BASE, 0x21; drm_ctx);
ioctl!(write ioctl_mod_ctx with DRM_IOCTL_BASE, 0x22; drm_ctx);
ioctl!(readwrite ioctl_get_ctx with DRM_IOCTL_BASE, 0x23; drm_ctx);
ioctl!(write ioctl_switch_ctx with DRM_IOCTL_BASE, 0x24; drm_ctx);
ioctl!(write ioctl_new_ctx with DRM_IOCTL_BASE, 0x25; drm_ctx);
ioctl!(readwrite ioctl_res_ctx with DRM_IOCTL_BASE, 0x26; drm_ctx_res);
ioctl!(readwrite ioctl_add_draw with DRM_IOCTL_BASE, 0x27; drm_draw);
ioctl!(readwrite ioctl_rm_draw with DRM_IOCTL_BASE, 0x28; drm_draw);
ioctl!(readwrite ioctl_dma with DRM_IOCTL_BASE, 0x29; drm_dma);
ioctl!(write ioctl_lock with DRM_IOCTL_BASE, 0x2a; drm_lock);
ioctl!(write ioctl_unlock with DRM_IOCTL_BASE, 0x2b; drm_lock);
ioctl!(write ioctl_finish with DRM_IOCTL_BASE, 0x2c; drm_lock);

ioctl!(readwrite ioctl_prime_handle_to_fd
       with DRM_IOCTL_BASE, 0x2d; drm_prime_handle);
ioctl!(readwrite ioctl_prime_fd_to_handle
       with DRM_IOCTL_BASE, 0x2e; drm_prime_handle);

ioctl!(none ioctl_agp_acquire with DRM_IOCTL_BASE, 0x30);
ioctl!(none ioctl_agp_release with DRM_IOCTL_BASE, 0x31);
ioctl!(write ioctl_agp_enable with DRM_IOCTL_BASE, 0x32; drm_agp_mode);
ioctl!(read ioctl_agp_info with DRM_IOCTL_BASE, 0x33; drm_agp_info);
ioctl!(readwrite ioctl_agp_alloc with DRM_IOCTL_BASE, 0x34; drm_agp_buffer);
ioctl!(write ioctl_agp_free with DRM_IOCTL_BASE, 0x35; drm_agp_buffer);
ioctl!(write ioctl_agp_bind with DRM_IOCTL_BASE, 0x36; drm_agp_binding);
ioctl!(write ioctl_agp_unbind with DRM_IOCTL_BASE, 0x37; drm_agp_binding);

ioctl!(readwrite ioctl_sg_alloc with DRM_IOCTL_BASE, 0x38; drm_scatter_gather);
ioctl!(write ioctl_sg_free with DRM_IOCTL_BASE, 0x39; drm_scatter_gather);

ioctl!(readwrite ioctl_wait_vblank with DRM_IOCTL_BASE, 0x3a; drm_wait_vblank);

ioctl!(write ioctl_update_draw with DRM_IOCTL_BASE, 0x3f; drm_update_draw);

ioctl!(readwrite ioctl_mode_getresources
       with DRM_IOCTL_BASE, 0xA0; drm_mode_card_res);
ioctl!(readwrite ioctl_mode_getcrtc with DRM_IOCTL_BASE, 0xA1; drm_mode_crtc);
ioctl!(readwrite ioctl_mode_setcrtc with DRM_IOCTL_BASE, 0xA2; drm_mode_crtc);
ioctl!(readwrite ioctl_mode_cursor with DRM_IOCTL_BASE, 0xA3; drm_mode_cursor);
ioctl!(readwrite ioctl_mode_getgamma
       with DRM_IOCTL_BASE, 0xA4; drm_mode_crtc_lut);
ioctl!(readwrite ioctl_mode_setgamma
       with DRM_IOCTL_BASE, 0xA5; drm_mode_crtc_lut);
ioctl!(readwrite ioctl_mode_getencoder
       with DRM_IOCTL_BASE, 0xA6; drm_mode_get_encoder);
ioctl!(readwrite ioctl_mode_getconnector
       with DRM_IOCTL_BASE, 0xA7; drm_mode_get_connector);

ioctl!(readwrite ioctl_mode_getproperty
       with DRM_IOCTL_BASE, 0xAA; drm_mode_get_property);
ioctl!(readwrite ioctl_mode_setproperty
       with DRM_IOCTL_BASE, 0xAB; drm_mode_connector_set_property);
ioctl!(readwrite ioctl_mode_getpropblob
       with DRM_IOCTL_BASE, 0xAC; drm_mode_get_blob);
ioctl!(readwrite ioctl_mode_getfb with DRM_IOCTL_BASE, 0xAD; drm_mode_fb_cmd);
ioctl!(readwrite ioctl_mode_addfb with DRM_IOCTL_BASE, 0xAE; drm_mode_fb_cmd);
ioctl!(readwrite ioctl_mode_rmfb with DRM_IOCTL_BASE, 0xAF; libc::c_uint);
ioctl!(readwrite ioctl_mode_page_flip
       with DRM_IOCTL_BASE, 0xB0; drm_mode_crtc_page_flip);
ioctl!(readwrite ioctl_mode_dirtyfb
       with DRM_IOCTL_BASE, 0xB1; drm_mode_fb_dirty_cmd);

ioctl!(readwrite ioctl_mode_create_dumb
       with DRM_IOCTL_BASE, 0xB2; drm_mode_create_dumb);
ioctl!(readwrite ioctl_mode_map_dumb
       with DRM_IOCTL_BASE, 0xB3; drm_mode_map_dumb);
ioctl!(readwrite ioctl_mode_destroy_dumb
       with DRM_IOCTL_BASE, 0xB4; drm_mode_destroy_dumb);
ioctl!(readwrite ioctl_mode_getplaneresources
       with DRM_IOCTL_BASE, 0xB5; drm_mode_get_plane_res);
ioctl!(readwrite ioctl_mode_getplane
       with DRM_IOCTL_BASE, 0xB6; drm_mode_get_plane);
ioctl!(readwrite ioctl_mode_setplane
       with DRM_IOCTL_BASE, 0xB7; drm_mode_set_plane);
ioctl!(readwrite ioctl_mode_addfb2 with DRM_IOCTL_BASE, 0xB8; drm_mode_fb_cmd2);
ioctl!(readwrite ioctl_mode_obj_getproperties
       with DRM_IOCTL_BASE, 0xB9; drm_mode_obj_get_properties);
ioctl!(readwrite ioctl_mode_obj_setproperty
       with DRM_IOCTL_BASE, 0xBA; drm_mode_obj_set_property);
ioctl!(readwrite ioctl_mode_cursor2 with DRM_IOCTL_BASE, 0xBB; drm_mode_cursor2);
ioctl!(readwrite ioctl_mode_atomic with DRM_IOCTL_BASE, 0xBC; drm_mode_atomic);
ioctl!(readwrite ioctl_mode_createpropblob
       with DRM_IOCTL_BASE, 0xBD; drm_mode_create_blob);
ioctl!(readwrite ioctl_mode_destroypropblob
       with DRM_IOCTL_BASE, 0xBE; drm_mode_destroy_blob);
