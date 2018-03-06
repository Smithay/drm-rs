//!
//! Foreign function interface
//!

#![allow(dead_code)]
#![allow(missing_docs)]

use nix::libc::{c_int, c_char};
use nix::Error;
pub use drm_sys::*;

use std::cmp;

pub mod ioctl;

macro_rules! impl_refs {
    ($type:ty, $rty:ty, $field:tt) => {
        impl AsRef<$rty> for $type {
            fn as_ref(&self) -> &$rty {
                &self.$field
            }
        }

        impl AsMut<$rty> for $type {
            fn as_mut(&mut self) -> &mut $rty {
                &mut self.$field
            }
        }
    }
}

/// Creates a wrapper around a type.
///
/// - Implements AsRef<$raw> and AsMut<$raw> using.
///
/// - Created function $type::cmd as a wrapper around $cmd
///
/// - If there are buffers listed, prepares them in $type::cmd automatically,
/// and then creates getter functions for them.
macro_rules! wrapper {
    (
        struct $type:ident ( $rty:ty );

        fn $cmd:expr;
    ) => {
        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
        pub(crate) struct $type ($rty);

        impl_refs!($type, $rty, 0);

        impl $type {
            /// Command to execute.
            pub fn cmd(&mut self, fd: c_int) -> Result<(), Error>
            {
                // Run the command.
                unsafe {
                    $cmd(fd, self.as_mut())?
                };
                Ok(())
            }
        }
    };
    (
        struct $type:ident {
            $rn:tt : $rty:ty,
            $(
                $buf:tt : [$bty:ty; $max:expr] = [raw.$ptr:tt; raw.$sz:tt]
            ),*
        }

        fn $cmd:expr;
    ) => {
        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
        pub(crate) struct $type {
            $rn : $rty,
            $(
                $buf: [$bty; $max],
            )*
        }

        impl_refs!($type, $rty, $rn);

        impl $type {
            /// Command to execute.
            pub fn cmd(&mut self, fd: c_int) -> Result<(), Error>
            {
                // Prepare buffers if there are any.
                $(
                    self.as_mut().$ptr = (&mut self.$buf).as_mut_ptr() as _;
                    self.as_mut().$sz  = $max;
                )*

                // Run the command.
                    unsafe {
                        $cmd(fd, self.as_mut())?
                    };

                // Coerce the buffer sizes if they went over the limit.
                $(
                    self.as_mut().$sz = cmp::min(self.as_ref().$sz, $max);
                )*

                    Ok(())
            }

            $(
                pub fn $buf(&self) -> &[$bty] {
                    {
                        use std::slice;

                        let ptr = self.$buf.as_ptr() as *const _;
                        let len = self.as_ref().$sz as usize;
                        unsafe {
                            slice::from_raw_parts(ptr, len)
                        }
                    }
                }
            )*
        }
    };
}

/// Many DRM structures have fields that act as pointers to buffers. In libdrm,
/// these buffers are allocated at runtime using `drmMalloc` after determining
/// the size of the buffer.
///
/// However, these buffers tend to be extremely tiny in nature. Therefore, we
/// wrap the DRM structures in a new type that also owns these buffers as
/// fixed-sized arrays. This provides us with two benefits:
///
/// 1. We only need to make the ioctl call once.
/// 2. We do not need to allocate memory on the heap.
///
/// If the actual number of elements exceeds our fixed-length array though, then
/// we will only fill the number of elements we can contain. If this happens on
/// a particular system, it's recommended to increase the length of these buffers
/// and consider filing a bug report.
pub(crate) type Buffer<T> = [T; 32];

wrapper! {
    struct BusID {
        raw: drm_unique,
        unique: [c_char; 32] = [raw.unique; raw.unique_len]
    }

    fn ioctl::get_bus_id;
}

wrapper! {
    struct Client(drm_client);
    fn ioctl::get_client;
}

wrapper! {
    struct Stats(drm_stats);
    fn ioctl::get_stats;
}

wrapper! {
    struct GetCap(drm_get_cap);
    fn ioctl::get_cap;
}

wrapper! {
    struct SetCap(drm_set_client_cap);
    fn ioctl::set_cap;
}

wrapper! {
    struct SetVersion(drm_set_version);
    fn ioctl::set_version;
}

wrapper! {
    struct GetVersion {
        raw: drm_version,
        name: [c_char; 32] = [raw.name; raw.name_len],
        date: [c_char; 32] = [raw.date; raw.date_len],
        desc: [c_char; 32] = [raw.desc; raw.desc_len]
    }

    fn ioctl::get_version;
}

wrapper! {
    struct GetToken(drm_auth);
    fn ioctl::get_token;
}

wrapper! {
    struct AuthToken(drm_auth);
    fn ioctl::auth_token;
}

wrapper! {
    struct IRQControl(drm_control);
    fn ioctl::irq_control;
}

wrapper! {
    struct WaitVBlank(drm_wait_vblank);
    fn ioctl::wait_vblank;
}

wrapper! {
    struct ModesetCtl(drm_modeset_ctl);
    fn ioctl::modeset_ctl;
}

pub(crate) mod mode {
    use nix::libc::{c_int, c_uint, uint32_t, uint64_t};
    use nix::Error;
    use super::*;

    pub(crate) type RawHandle = uint32_t;

    wrapper! {
        struct CardRes {
            raw: drm_mode_card_res,
            connectors: [RawHandle; 32] = [raw.connector_id_ptr; raw.count_connectors],
            encoders: [RawHandle; 32] = [raw.encoder_id_ptr; raw.count_encoders],
            crtcs: [RawHandle; 32] = [raw.crtc_id_ptr; raw.count_crtcs],
            framebuffers: [RawHandle; 32] = [raw.fb_id_ptr; raw.count_fbs]
        }

        fn ioctl::mode::get_resources;
    }

    wrapper! {
        struct PlaneRes {
            raw: drm_mode_get_plane_res,
            planes: [RawHandle; 32] = [raw.plane_id_ptr; raw.count_planes]
        }

        fn ioctl::mode::get_plane_resources;
    }

    wrapper! {
        struct GetConnector {
            raw: drm_mode_get_connector,
            encoders: [RawHandle; 32] = [raw.encoders_ptr; raw.count_encoders],
            properties: [RawHandle; 32] = [raw.props_ptr; raw.count_props],
            prop_values: [uint64_t; 32] = [raw.prop_values_ptr; raw.count_props],
            modes: [drm_mode_modeinfo; 32] = [raw.modes_ptr; raw.count_modes]
        }

        fn ioctl::mode::get_connector;
    }

    wrapper! {
        struct GetEncoder(drm_mode_get_encoder);
        fn ioctl::mode::get_encoder;
    }

    wrapper! {
        struct GetCrtc {
            raw: drm_mode_crtc,
            connectors: [RawHandle; 32] = [raw.set_connectors_ptr; raw.count_connectors]
        }

        fn ioctl::mode::get_crtc;
    }

    wrapper! {
        struct SetCrtc {
            raw: drm_mode_crtc,
            connectors: [RawHandle; 32] = [raw.set_connectors_ptr; raw.count_connectors]
        }

        fn ioctl::mode::get_crtc;
    }

    wrapper! {
        struct GetFB(drm_mode_fb_cmd);
        fn ioctl::mode::get_fb;
    }

    wrapper! {
        struct AddFB(drm_mode_fb_cmd);
        fn ioctl::mode::add_fb;
    }

    wrapper! {
        struct AddFB2(drm_mode_fb_cmd2);
        fn ioctl::mode::add_fb2;
    }

    wrapper! {
        struct RmFB(c_uint);
        fn ioctl::mode::rm_fb;
    }

    wrapper! {
        struct GetPlane {
            raw: drm_mode_get_plane,
            formats: [uint32_t; 32] = [raw.format_type_ptr; raw.count_format_types]
        }

        fn ioctl::mode::get_plane;
    }

    wrapper! {
        struct SetPlane(drm_mode_set_plane);
        fn ioctl::mode::set_plane;
    }

    wrapper! {
        struct CreateDumb(drm_mode_create_dumb);
        fn ioctl::mode::create_dumb;
    }

    wrapper! {
        struct MapDumb(drm_mode_map_dumb);
        fn ioctl::mode::map_dumb;
    }

    wrapper! {
        struct DestroyDumb(drm_mode_destroy_dumb);
        fn ioctl::mode::destroy_dumb;
    }

    wrapper! {
        struct Cursor(drm_mode_cursor);
        fn ioctl::mode::cursor;
    }

    wrapper! {
        struct Cursor2(drm_mode_cursor2);
        fn ioctl::mode::cursor2;
    }

    // TODO: Requires some extra work for setting up buffers
    #[derive(Debug, Default, Copy, Clone, Hash)]
    pub(crate) struct GetProperty {
        raw: drm_mode_get_property,
    }

    wrapper! {
        struct ConnectorSetProperty(drm_mode_connector_set_property);
        fn ioctl::mode::connector_set_property;
    }

    // TODO: Requires some extra work for setting up buffers
    #[derive(Debug, Default, Copy, Clone, Hash)]
    pub(crate) struct ObjGetProperties {
        raw: drm_mode_obj_get_properties,
        pub prop_buf: Buffer<uint32_t>,
        pub vals_buf: Buffer<uint64_t>
    }

    wrapper! {
        struct ObjSetProperty(drm_mode_obj_set_property);
        fn ioctl::mode::obj_set_property;
    }

    wrapper! {
        struct CreateBlob(drm_mode_create_blob);
        fn ioctl::mode::create_blob;
    }

    wrapper! {
        struct DestroyBlob(drm_mode_destroy_blob);
        fn ioctl::mode::destroy_blob;
    }

    wrapper! {
        struct CrtcPageFlip(drm_mode_crtc_page_flip);
        fn ioctl::mode::crtc_page_flip;
    }

    wrapper! {
        struct FBDirtyCmd(drm_mode_fb_dirty_cmd);
        fn ioctl::mode::dirty_fb;
    }

    wrapper! {
        struct Atomic {
            raw: drm_mode_atomic,
            objects: [uint32_t; 32] = [raw.objs_ptr; raw.count_objs],
            count_properties: [uint32_t; 32] = [raw.count_props_ptr; raw.count_objs],
            properties: [uint32_t; 32] = [raw.props_ptr; raw.count_objs],
            prop_values: [uint64_t; 32] = [raw.prop_values_ptr; raw.count_objs]
        }

        fn ioctl::mode::atomic;
    }
}

pub(crate) mod gem {
    use nix::libc::c_int;
    use nix::Error;
    use super::*;

    // Underlying type for a GEM handle.
    pub(crate) type RawHandle = u32;

    wrapper! {
        struct Open(drm_gem_open);
        fn ioctl::gem::open;
    }

    wrapper! {
        struct Close(drm_gem_close);
        fn ioctl::gem::close;
    }

    wrapper! {
        struct Flink(drm_gem_flink);
        fn ioctl::gem::flink;
    }

    wrapper! {
        struct PrimeHandleToFD(drm_prime_handle);
        fn ioctl::gem::prime_handle_to_fd;
    }

    wrapper! {
        struct PrimeFDToHandle(drm_prime_handle);
        fn ioctl::gem::prime_fd_to_handle;
    }
}
