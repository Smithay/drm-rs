//!
//! Bindings to the DRM's modesetting capabilities.
//!

use drm_sys::*;
use ioctl;

use result::SystemError as Error;
use std::os::unix::io::RawFd;

use utils;

/// Enumerate most card resources.
pub fn get_resources(
    fd: RawFd,
    fbs: Option<&mut &mut [u32]>,
    crtcs: Option<&mut &mut [u32]>,
    connectors: Option<&mut &mut [u32]>,
    encoders: Option<&mut &mut [u32]>,
) -> Result<drm_mode_card_res, Error> {
    let mut res = drm_mode_card_res {
        fb_id_ptr: map_ptr!(&fbs),
        crtc_id_ptr: map_ptr!(&crtcs),
        connector_id_ptr: map_ptr!(&connectors),
        encoder_id_ptr: map_ptr!(&encoders),
        count_fbs: map_len!(&fbs),
        count_crtcs: map_len!(&crtcs),
        count_connectors: map_len!(&connectors),
        count_encoders: map_len!(&encoders),
        ..Default::default()
    };

    unsafe {
        ioctl::mode::get_resources(fd, &mut res)?;
    }

    map_shrink!(fbs, res.count_fbs as usize);
    map_shrink!(crtcs, res.count_crtcs as usize);
    map_shrink!(connectors, res.count_connectors as usize);
    map_shrink!(encoders, res.count_encoders as usize);

    Ok(res)
}

/// Enumerate plane resources.
pub fn get_plane_resources(
    fd: RawFd,
    planes: Option<&mut &mut [u32]>,
) -> Result<drm_mode_get_plane_res, Error> {
    let mut res = drm_mode_get_plane_res {
        plane_id_ptr: map_ptr!(&planes),
        count_planes: map_len!(&planes),
    };

    unsafe {
        ioctl::mode::get_plane_resources(fd, &mut res)?;
    }

    map_shrink!(planes, res.count_planes as usize);

    Ok(res)
}

/// Get info about a framebuffer.
pub fn get_framebuffer(fd: RawFd, id: u32) -> Result<drm_mode_fb_cmd, Error> {
    let mut info = drm_mode_fb_cmd {
        fb_id: id,
        ..Default::default()
    };

    unsafe {
        ioctl::mode::get_fb(fd, &mut info)?;
    }

    Ok(info)
}

/// Add a new framebuffer.
pub fn add_fb(
    fd: RawFd,
    w: u32,
    h: u32,
    pitch: u32,
    bpp: u32,
    depth: u32,
    handle: u32,
) -> Result<drm_mode_fb_cmd, Error> {
    let mut fb = drm_mode_fb_cmd {
        width: w,
        height: h,
        pitch: pitch,
        bpp: bpp,
        depth: depth,
        handle: handle,
        ..Default::default()
    };

    unsafe {
        ioctl::mode::add_fb(fd, &mut fb)?;
    }

    Ok(fb)
}

/// Add a new framebuffer (with modifiers)
pub fn add_fb2(
    fd: RawFd,
    w: u32,
    h: u32,
    fmt: u32,
    handles: &[u32; 4],
    pitches: &[u32; 4],
    offsets: &[u32; 4],
    modifier: &[u64; 4],
    flags: u32,
) -> Result<drm_mode_fb_cmd2, Error> {
    let mut fb = drm_mode_fb_cmd2 {
        width: w,
        height: h,
        pixel_format: fmt,
        flags: flags,
        handles: *handles,
        pitches: *pitches,
        offsets: *offsets,
        modifier: *modifier,
        ..Default::default()
    };

    unsafe {
        ioctl::mode::add_fb2(fd, &mut fb)?;
    }

    Ok(fb)
}

/// Remove a framebuffer.
pub fn rm_fb(fd: RawFd, mut id: u32) -> Result<(), Error> {
    unsafe {
        ioctl::mode::rm_fb(fd, &mut id)?;
    }

    Ok(())
}

/// Mark a framebuffer as dirty.
pub fn dirty_fb(fd: RawFd, id: u32, clips: &[drm_clip_rect]) -> Result<drm_mode_fb_dirty_cmd, Error> {
    let mut dirty = drm_mode_fb_dirty_cmd {
        fb_id: id,
        clips_ptr: clips.as_ptr() as _,
        num_clips: clips.len() as _,
        ..Default::default()
    };

    unsafe {
        ioctl::mode::dirty_fb(fd, &mut dirty)?;
    }

    Ok(dirty)
}

/// Get info about a CRTC
pub fn get_crtc(fd: RawFd, id: u32) -> Result<drm_mode_crtc, Error> {
    let mut info = drm_mode_crtc {
        crtc_id: id,
        ..Default::default()
    };

    unsafe {
        ioctl::mode::get_crtc(fd, &mut info)?;
    }

    Ok(info)
}

/// Set CRTC state
pub fn set_crtc(
    fd: RawFd,
    id: u32,
    fb_id: u32,
    x: u32,
    y: u32,
    conns: &[u32],
    mode: Option<drm_mode_modeinfo>,
) -> Result<drm_mode_crtc, Error> {
    let mut crtc = drm_mode_crtc {
        x: x,
        y: y,
        crtc_id: id,
        fb_id: fb_id,
        set_connectors_ptr: conns.as_ptr() as _,
        count_connectors: conns.len() as _,
        mode: match mode {
            Some(m) => m,
            None => Default::default(),
        },
        mode_valid: match mode {
            Some(_) => 1,
            None => 0,
        },
        ..Default::default()
    };

    unsafe {
        ioctl::mode::set_crtc(fd, &mut crtc)?;
    }

    Ok(crtc)
}

/// Get CRTC gamma ramp
pub fn get_gamma(
    fd: RawFd,
    id: u32,
    size: usize,
    red: &mut [u16],
    green: &mut [u16],
    blue: &mut [u16],
) -> Result<drm_mode_crtc_lut, Error> {
    let mut lut = drm_mode_crtc_lut {
        crtc_id: id,
        gamma_size: size as _,
        red: red.as_ptr() as _,
        green: green.as_ptr() as _,
        blue: blue.as_ptr() as _,
    };

    unsafe {
        ioctl::mode::get_gamma(fd, &mut lut)?;
    }

    Ok(lut)
}

/// Set CRTC gamma ramp
pub fn set_gamma(
    fd: RawFd,
    id: u32,
    size: usize,
    red: &[u16],
    green: &[u16],
    blue: &[u16],
) -> Result<drm_mode_crtc_lut, Error> {
    let mut lut = drm_mode_crtc_lut {
        crtc_id: id,
        gamma_size: size as _,
        red: red.as_ptr() as _,
        green: green.as_ptr() as _,
        blue: blue.as_ptr() as _,
    };

    unsafe {
        ioctl::mode::set_gamma(fd, &mut lut)?;
    }

    Ok(lut)
}

/// Set cursor state
///
/// The buffer must be allocated using the buffer manager of the driver (GEM or TTM). It is not
/// allowed to be a dumb buffer.
#[deprecated = "use a cursor plane instead"]
pub fn set_cursor(fd: RawFd, id: u32, buf_id: u32, w: u32, h: u32) -> Result<drm_mode_cursor, Error> {
    let mut cursor = drm_mode_cursor {
        flags: DRM_MODE_CURSOR_BO,
        crtc_id: id,
        width: w,
        height: h,
        handle: buf_id,
        ..Default::default()
    };

    unsafe {
        ioctl::mode::cursor(fd, &mut cursor)?;
    }

    Ok(cursor)
}

/// Set cursor state (with hotspot position)
///
/// The buffer must be allocated using the buffer manager of the driver (GEM or TTM). It is not
/// allowed to be a dumb buffer.
///
/// The hotspot position is used to coordinate the guest and host cursor location in case of
/// virtualization.
#[deprecated = "use a cursor plane instead"]
pub fn set_cursor2(
    fd: RawFd,
    id: u32,
    buf_id: u32,
    w: u32,
    h: u32,
    hot_x: i32,
    hot_y: i32,
) -> Result<drm_mode_cursor2, Error> {
    let mut cursor = drm_mode_cursor2 {
        flags: DRM_MODE_CURSOR_BO,
        crtc_id: id,
        width: w,
        height: h,
        handle: buf_id,
        hot_x,
        hot_y,
        ..Default::default()
    };

    unsafe {
        ioctl::mode::cursor2(fd, &mut cursor)?;
    }

    Ok(cursor)
}

/// Move cursor
#[deprecated = "use a cursor plane instead"]
pub fn move_cursor(fd: RawFd, id: u32, x: i32, y: i32) -> Result<drm_mode_cursor, Error> {
    let mut cursor = drm_mode_cursor {
        flags: DRM_MODE_CURSOR_MOVE,
        crtc_id: id,
        x: x,
        y: y,
        ..Default::default()
    };

    unsafe {
        ioctl::mode::cursor(fd, &mut cursor)?;
    }

    Ok(cursor)
}

/// Get info about a connector
pub fn get_connector(
    fd: RawFd,
    id: u32,
    props: Option<&mut &mut [u32]>,
    prop_values: Option<&mut &mut [u64]>,
    modes: Option<&mut &mut [drm_mode_modeinfo]>,
    encoders: Option<&mut &mut [u32]>,
) -> Result<drm_mode_get_connector, Error> {
    let mut info = drm_mode_get_connector {
        connector_id: id,
        props_ptr: map_ptr!(&props),
        prop_values_ptr: map_ptr!(&prop_values),
        modes_ptr: map_ptr!(&modes),
        encoders_ptr: map_ptr!(&encoders),
        count_props: map_len!(&props),
        count_modes: map_len!(&modes),
        count_encoders: map_len!(&encoders),
        ..Default::default()
    };

    unsafe {
        ioctl::mode::get_connector(fd, &mut info)?;
    }

    map_shrink!(props, info.count_props as usize);
    map_shrink!(prop_values, info.count_props as usize);
    map_shrink!(modes, info.count_modes as usize);
    map_shrink!(encoders, info.count_encoders as usize);

    Ok(info)
}

/// Get info about an encoder
pub fn get_encoder(fd: RawFd, id: u32) -> Result<drm_mode_get_encoder, Error> {
    let mut info = drm_mode_get_encoder {
        encoder_id: id,
        ..Default::default()
    };

    unsafe {
        ioctl::mode::get_encoder(fd, &mut info)?;
    }

    Ok(info)
}

/// Get info about a plane.
pub fn get_plane(
    fd: RawFd,
    id: u32,
    formats: Option<&mut &mut [u32]>,
) -> Result<drm_mode_get_plane, Error> {
    let mut info = drm_mode_get_plane {
        plane_id: id,
        format_type_ptr: map_ptr!(&formats),
        count_format_types: map_len!(&formats),
        ..Default::default()
    };

    unsafe {
        ioctl::mode::get_plane(fd, &mut info)?;
    }

    map_shrink!(formats, info.count_format_types as usize);

    Ok(info)
}

/// Set plane state.
pub fn set_plane(
    fd: RawFd,
    plane_id: u32,
    crtc_id: u32,
    fb_id: u32,
    flags: u32,
    crtc_x: i32,
    crtc_y: i32,
    crtc_w: u32,
    crtc_h: u32,
    src_x: u32,
    src_y: u32,
    src_w: u32,
    src_h: u32,
) -> Result<drm_mode_set_plane, Error> {
    let mut plane = drm_mode_set_plane {
        plane_id: plane_id,
        crtc_id: crtc_id,
        fb_id: fb_id,
        flags: flags,
        crtc_x: crtc_x,
        crtc_y: crtc_y,
        crtc_w: crtc_w,
        crtc_h: crtc_h,
        src_x: src_x,
        src_y: src_y,
        src_w: src_w,
        src_h: src_h,
    };

    unsafe {
        ioctl::mode::set_plane(fd, &mut plane)?;
    }

    Ok(plane)
}

/// Get property
pub fn get_property(
    fd: RawFd,
    id: u32,
    values: Option<&mut &mut [u64]>,
    enums: Option<&mut &mut [drm_mode_property_enum]>,
) -> Result<drm_mode_get_property, Error> {
    let mut prop = drm_mode_get_property {
        values_ptr: map_ptr!(&values),
        enum_blob_ptr: map_ptr!(&enums),
        prop_id: id,
        count_values: map_len!(&values),
        count_enum_blobs: map_len!(&enums),
        ..Default::default()
    };

    unsafe {
        ioctl::mode::get_property(fd, &mut prop)?;
    }

    map_shrink!(values, prop.count_values as usize);
    map_shrink!(enums, prop.count_enum_blobs as usize);

    Ok(prop)
}

/// Set property
pub fn set_connector_property(
    fd: RawFd,
    conn_id: u32,
    prop_id: u32,
    value: u64,
) -> Result<drm_mode_connector_set_property, Error> {
    let mut prop = drm_mode_connector_set_property {
        value: value,
        prop_id: prop_id,
        connector_id: conn_id,
    };

    unsafe {
        ioctl::mode::connector_set_property(fd, &mut prop)?;
    }

    Ok(prop)
}

/// Get the value of a property blob
pub fn get_property_blob(fd: RawFd, id: u32, data: Option<&mut &mut [u64]>) -> Result<drm_mode_get_blob, Error> {
    let mut blob = drm_mode_get_blob {
        blob_id: id,
        length: map_len!(&data),
        data: map_ptr!(&data),
    };

    unsafe {
        ioctl::mode::get_blob(fd, &mut blob)?;
    }

    map_shrink!(data, blob.length as usize);

    Ok(blob)
}

/// Create a property blob
pub fn create_property_blob(fd: RawFd, data: &mut [u64]) -> Result<drm_mode_create_blob, Error> {
    let mut blob = drm_mode_create_blob {
        length: data.len() as _,
        data: data.as_ptr() as _,
        ..Default::default()
    };

    unsafe {
        ioctl::mode::create_blob(fd, &mut blob)?;
    }

    Ok(blob)
}

/// Destroy a property blob
pub fn destroy_property_blob(fd: RawFd, id: u32) -> Result<drm_mode_destroy_blob, Error> {
    let mut blob = drm_mode_destroy_blob { blob_id: id };

    unsafe {
        ioctl::mode::destroy_blob(fd, &mut blob)?;
    }

    Ok(blob)
}

/// Get properties from an object
pub fn get_properties(
    fd: RawFd,
    id: u32,
    obj_type: u32,
    props: Option<&mut &mut [u32]>,
    values: Option<&mut &mut [u64]>,
) -> Result<drm_mode_obj_get_properties, Error> {
    let mut info = drm_mode_obj_get_properties {
        props_ptr: map_ptr!(&props),
        prop_values_ptr: map_ptr!(&values),
        count_props: map_len!(&props),
        obj_id: id,
        obj_type: obj_type,
    };

    unsafe {
        ioctl::mode::obj_get_properties(fd, &mut info)?;
    }

    map_shrink!(props, info.count_props as usize);
    map_shrink!(values, info.count_props as usize);

    Ok(info)
}

/// Set the properties of an object
pub fn set_property(
    fd: RawFd,
    prop_id: u32,
    obj_id: u32,
    obj_type: u32,
    value: u64,
) -> Result<(), Error> {
    let mut prop = drm_mode_obj_set_property {
        value: value,
        prop_id: prop_id,
        obj_id: obj_id,
        obj_type: obj_type,
    };

    unsafe {
        ioctl::mode::obj_set_property(fd, &mut prop)?;
    }

    Ok(())
}

/// Schedule a page flip
pub fn page_flip(
    fd: RawFd,
    crtc_id: u32,
    fb_id: u32,
    flags: u32,
    sequence: u32,
) -> Result<(), Error> {
    let mut flip = drm_mode_crtc_page_flip {
        crtc_id: crtc_id,
        fb_id: fb_id,
        flags: flags,
        reserved: sequence,
        user_data: crtc_id as _,
    };

    unsafe {
        ioctl::mode::crtc_page_flip(fd, &mut flip)?;
    }

    Ok(())
}

/// Atomically set properties
pub fn atomic_commit(
    fd: RawFd,
    flags: u32,
    objs: &mut [u32],
    prop_counts: &mut [u32],
    props: &mut [u32],
    values: &mut [u64],
) -> Result<(), Error> {
    let mut atomic = drm_mode_atomic {
        flags: flags,
        count_objs: objs.len() as _,
        objs_ptr: objs.as_ptr() as _,
        count_props_ptr: prop_counts.as_ptr() as _,
        props_ptr: props.as_ptr() as _,
        prop_values_ptr: values.as_ptr() as _,
        ..Default::default()
    };

    unsafe {
        ioctl::mode::atomic(fd, &mut atomic)?;
    }

    Ok(())
}

///
/// Dumbbuffers are basic buffers that can be used for scanout.
///
pub mod dumbbuffer {
    use drm_sys::*;
    use ioctl;

    use result::SystemError as Error;
    use std::os::unix::io::RawFd;

    /// Create a dumb buffer
    pub fn create(
        fd: RawFd,
        w: u32,
        h: u32,
        bpp: u32,
        flags: u32,
    ) -> Result<drm_mode_create_dumb, Error> {
        let mut db = drm_mode_create_dumb {
            height: h,
            width: w,
            bpp: bpp,
            flags: flags,
            ..Default::default()
        };

        unsafe {
            ioctl::mode::create_dumb(fd, &mut db)?;
        }

        Ok(db)
    }

    /// Destroy a dumb buffer
    pub fn destroy(fd: RawFd, handle: u32) -> Result<drm_mode_destroy_dumb, Error> {
        let mut db = drm_mode_destroy_dumb { handle: handle };

        unsafe {
            ioctl::mode::destroy_dumb(fd, &mut db)?;
        }

        Ok(db)
    }

    /// Map a dump buffer and prep it for an mmap
    pub fn map(fd: RawFd, handle: u32, pad: u32, offset: u64) -> Result<drm_mode_map_dumb, Error> {
        let mut map = drm_mode_map_dumb {
            handle: handle,
            pad: pad,
            offset: offset,
        };

        unsafe {
            ioctl::mode::map_dumb(fd, &mut map)?;
        }

        Ok(map)
    }
}
