use drm_sys::*;
use ffi::ioctl;

use nix::libc::*;
use nix::Error;
use std::os::unix::io::RawFd;

/// Enumerate most card resources.
pub fn get_resources(
    fd: RawFd,
    fbs: &mut [u32; 32],
    crtcs: &mut [u32; 32],
    connectors: &mut [u32; 32],
    encoders: &mut [u32; 32],
) -> Result<drm_mode_card_res, Error> {
    let mut res = drm_mode_card_res {
        fb_id_ptr: fbs.as_ptr() as _,
        crtc_id_ptr: crtcs.as_ptr() as _,
        connector_id_ptr: connectors.as_ptr() as _,
        encoder_id_ptr: encoders.as_ptr() as _,
        count_fbs: fbs.len() as _,
        count_crtcs: crtcs.len() as _,
        count_connectors: connectors.len() as _,
        count_encoders: encoders.len() as _,
        ..Default::default()
    };

    unsafe {
        ioctl::mode::get_resources(fd, &mut res)?;
    }

    Ok(res)
}

/// Enumerate plane resources.
pub fn get_plane_resources(
    fd: RawFd,
    planes: &mut [u32; 32],
) -> Result<drm_mode_get_plane_res, Error> {
    let mut res = drm_mode_get_plane_res {
        plane_id_ptr: planes.as_ptr() as _,
        count_planes: planes.len() as _,
    };

    unsafe {
        ioctl::mode::get_plane_resources(fd, &mut res)?;
    }

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
) -> Result<(), Error> {
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

    Ok(())
}

/// Set cursor state
pub fn set_cursor(fd: RawFd, id: u32, buf_id: u32, w: u32, h: u32) -> Result<(), Error> {
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

    Ok(())
}

/// Set cursor state (with position)
pub fn set_cursor2(
    fd: RawFd,
    id: u32,
    buf_id: u32,
    w: u32,
    h: u32,
    x: i32,
    y: i32,
) -> Result<(), Error> {
    let mut cursor = drm_mode_cursor2 {
        flags: DRM_MODE_CURSOR_BO,
        crtc_id: id,
        width: w,
        height: h,
        handle: buf_id,
        hot_x: x,
        hot_y: y,
        ..Default::default()
    };

    unsafe {
        ioctl::mode::cursor2(fd, &mut cursor)?;
    }

    Ok(())
}

/// Move cursor
pub fn move_cursor(fd: RawFd, id: u32, x: i32, y: i32) -> Result<(), Error> {
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

    Ok(())
}

/// Get info about a connector
pub fn get_connector(
    fd: RawFd,
    id: u32,
    props: &mut [u32; 32],
    prop_values: &mut [u64; 32],
    modes: &mut [drm_mode_modeinfo; 32],
    encoders: &mut [u32; 32],
) -> Result<drm_mode_get_connector, Error> {
    let mut info = drm_mode_get_connector {
        connector_id: id,
        props_ptr: props.as_ptr() as _,
        prop_values_ptr: prop_values.as_ptr() as _,
        modes_ptr: modes.as_ptr() as _,
        encoders_ptr: encoders.as_ptr() as _,
        count_props: props.len() as _,
        count_modes: modes.len() as _,
        count_encoders: encoders.len() as _,
        ..Default::default()
    };

    unsafe {
        ioctl::mode::get_connector(fd, &mut info)?;
    }

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
pub fn get_plane(fd: RawFd, id: u32, formats: &mut [u32; 32]) -> Result<drm_mode_get_plane, Error> {
    let mut info = drm_mode_get_plane {
        plane_id: id,
        format_type_ptr: formats.as_ptr() as _,
        count_format_types: formats.len() as _,
        ..Default::default()
    };

    unsafe {
        ioctl::mode::get_plane(fd, &mut info)?;
    }

    Ok(info)
}
