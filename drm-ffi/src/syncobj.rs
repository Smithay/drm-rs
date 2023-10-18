//!
//! Bindings for DRM sync objects
//!

use drm_sys::*;
use ioctl;

use result::SystemError as Error;
use std::os::unix::io::{AsRawFd, BorrowedFd};

/// Creates a syncobj.
pub fn create(fd: BorrowedFd<'_>, signaled: bool) -> Result<drm_syncobj_create, Error> {
    let mut args = drm_syncobj_create {
        handle: 0,
        flags: if signaled {
            DRM_SYNCOBJ_CREATE_SIGNALED
        } else {
            0
        },
    };

    unsafe {
        ioctl::syncobj::create(fd.as_raw_fd(), &mut args)?;
    }

    Ok(args)
}

/// Destroys a syncobj.
pub fn destroy(fd: BorrowedFd<'_>, handle: u32) -> Result<drm_syncobj_destroy, Error> {
    let mut args = drm_syncobj_destroy { handle, pad: 0 };

    unsafe {
        ioctl::syncobj::destroy(fd.as_raw_fd(), &mut args)?;
    }

    Ok(args)
}

/// Exports a syncobj as an inter-process file descriptor or as a poll()-able sync file.
pub fn handle_to_fd(
    fd: BorrowedFd<'_>,
    handle: u32,
    export_sync_file: bool,
) -> Result<drm_syncobj_handle, Error> {
    let mut args = drm_syncobj_handle {
        handle,
        flags: if export_sync_file {
            DRM_SYNCOBJ_HANDLE_TO_FD_FLAGS_EXPORT_SYNC_FILE
        } else {
            0
        },
        fd: 0,
        pad: 0,
    };

    unsafe {
        ioctl::syncobj::handle_to_fd(fd.as_raw_fd(), &mut args)?;
    }

    Ok(args)
}

/// Imports a file descriptor exported by [`handle_to_fd`] back into a process-local handle.
pub fn fd_to_handle(
    fd: BorrowedFd<'_>,
    syncobj_fd: BorrowedFd<'_>,
    import_sync_file: bool,
) -> Result<drm_syncobj_handle, Error> {
    let mut args = drm_syncobj_handle {
        handle: 0,
        flags: if import_sync_file {
            DRM_SYNCOBJ_FD_TO_HANDLE_FLAGS_IMPORT_SYNC_FILE
        } else {
            0
        },
        fd: syncobj_fd.as_raw_fd(),
        pad: 0,
    };

    unsafe {
        ioctl::syncobj::fd_to_handle(fd.as_raw_fd(), &mut args)?;
    }

    Ok(args)
}

/// Waits for one or more syncobjs to become signalled.
pub fn wait(
    fd: BorrowedFd<'_>,
    handles: &[u32],
    timeout_nsec: i64,
    wait_all: bool,
    wait_for_submit: bool,
) -> Result<drm_syncobj_wait, Error> {
    let mut args = drm_syncobj_wait {
        handles: handles.as_ptr() as _,
        timeout_nsec,
        count_handles: handles.len() as _,
        flags: if wait_all {
            DRM_SYNCOBJ_WAIT_FLAGS_WAIT_ALL
        } else {
            0
        } | if wait_for_submit {
            DRM_SYNCOBJ_WAIT_FLAGS_WAIT_FOR_SUBMIT
        } else {
            0
        },
        first_signaled: 0,
        pad: 0,
    };

    unsafe {
        ioctl::syncobj::wait(fd.as_raw_fd(), &mut args)?;
    }

    Ok(args)
}

/// Resets (un-signals) one or more syncobjs.
pub fn reset(fd: BorrowedFd<'_>, handles: &[u32]) -> Result<drm_syncobj_array, Error> {
    let mut args = drm_syncobj_array {
        handles: handles.as_ptr() as _,
        count_handles: handles.len() as _,
        pad: 0,
    };

    unsafe {
        ioctl::syncobj::reset(fd.as_raw_fd(), &mut args)?;
    }

    Ok(args)
}

/// Signals one or more syncobjs.
pub fn signal(fd: BorrowedFd<'_>, handles: &[u32]) -> Result<drm_syncobj_array, Error> {
    let mut args = drm_syncobj_array {
        handles: handles.as_ptr() as _,
        count_handles: handles.len() as _,
        pad: 0,
    };

    unsafe {
        ioctl::syncobj::signal(fd.as_raw_fd(), &mut args)?;
    }

    Ok(args)
}

/// Waits for one or more specific timeline syncobj points.
pub fn timeline_wait(
    fd: BorrowedFd<'_>,
    handles: &[u32],
    points: &[u64],
    timeout_nsec: i64,
    wait_all: bool,
    wait_for_submit: bool,
    wait_available: bool,
) -> Result<drm_syncobj_timeline_wait, Error> {
    debug_assert_eq!(handles.len(), points.len());

    let mut args = drm_syncobj_timeline_wait {
        handles: handles.as_ptr() as _,
        points: points.as_ptr() as _,
        timeout_nsec,
        count_handles: handles.len() as _,
        flags: if wait_all {
            DRM_SYNCOBJ_WAIT_FLAGS_WAIT_ALL
        } else {
            0
        } | if wait_for_submit {
            DRM_SYNCOBJ_WAIT_FLAGS_WAIT_FOR_SUBMIT
        } else {
            0
        } | if wait_available {
            DRM_SYNCOBJ_WAIT_FLAGS_WAIT_AVAILABLE
        } else {
            0
        },
        first_signaled: 0,
        pad: 0,
    };

    unsafe {
        ioctl::syncobj::timeline_wait(fd.as_raw_fd(), &mut args)?;
    }

    Ok(args)
}

/// Queries for state of one or more timeline syncobjs.
pub fn query(
    fd: BorrowedFd<'_>,
    handles: &[u32],
    points: &mut [u64],
    last_submitted: bool,
) -> Result<drm_syncobj_timeline_array, Error> {
    debug_assert_eq!(handles.len(), points.len());

    let mut args = drm_syncobj_timeline_array {
        handles: handles.as_ptr() as _,
        points: points.as_mut_ptr() as _,
        count_handles: handles.len() as _,
        flags: if last_submitted {
            DRM_SYNCOBJ_QUERY_FLAGS_LAST_SUBMITTED
        } else {
            0
        },
    };

    unsafe {
        ioctl::syncobj::query(fd.as_raw_fd(), &mut args)?;
    }

    Ok(args)
}

/// Transfers one timeline syncobj point to another.
pub fn transfer(
    fd: BorrowedFd<'_>,
    src_handle: u32,
    dst_handle: u32,
    src_point: u64,
    dst_point: u64,
) -> Result<drm_syncobj_transfer, Error> {
    let mut args = drm_syncobj_transfer {
        src_handle,
        dst_handle,
        src_point,
        dst_point,
        flags: 0,
        pad: 0,
    };

    unsafe {
        ioctl::syncobj::transfer(fd.as_raw_fd(), &mut args)?;
    }

    Ok(args)
}

/// Signals one or more specific timeline syncobj points.
pub fn timeline_signal(
    fd: BorrowedFd<'_>,
    handles: &[u32],
    points: &[u64],
) -> Result<drm_syncobj_timeline_array, Error> {
    debug_assert_eq!(handles.len(), points.len());

    let mut args = drm_syncobj_timeline_array {
        handles: handles.as_ptr() as _,
        points: points.as_ptr() as _,
        count_handles: handles.len() as _,
        flags: 0,
    };

    unsafe {
        ioctl::syncobj::timeline_signal(fd.as_raw_fd(), &mut args)?;
    }

    Ok(args)
}
