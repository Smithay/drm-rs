#[cfg(feature = "use_bindgen")]
mod use_bindgen {
    extern crate bindgen;
    extern crate pkg_config;

    use self::bindgen::Builder;
    use std::env::var;
    use std::path::PathBuf;

    const TMP_BIND_PREFIX: &str = "__BINDGEN_TMP_";
    const TMP_BIND_PREFIX_REG: &str = "__BINDGEN_TMP_.*";

    const INCLUDES: &'static [&str] = &[
        "drm.h",
        "drm_mode.h"
    ];

    const MACROS: &'static [&str] = &[
        "DRM_MODE_PROP_SIGNED_RANGE",
        "DRM_MODE_PROP_OBJECT"
    ];

    const IOCTLS: &'static [&str] = &[
        "DRM_IOCTL_BASE",
        "DRM_IOCTL_VERSION",
        "DRM_IOCTL_GET_UNIQUE",
        "DRM_IOCTL_GET_MAGIC",
        "DRM_IOCTL_IRQ_BUSID",
        "DRM_IOCTL_GET_MAP",
        "DRM_IOCTL_GET_CLIENT",
        "DRM_IOCTL_GET_STATS",
        "DRM_IOCTL_SET_VERSION",
        "DRM_IOCTL_MODESET_CTL",
        "DRM_IOCTL_GEM_CLOSE",
        "DRM_IOCTL_GEM_FLINK",
        "DRM_IOCTL_GEM_OPEN",
        "DRM_IOCTL_GET_CAP",
        "DRM_IOCTL_SET_CLIENT_CAP",
        "DRM_IOCTL_SET_UNIQUE",
        "DRM_IOCTL_AUTH_MAGIC",
        "DRM_IOCTL_BLOCK",
        "DRM_IOCTL_UNBLOCK",
        "DRM_IOCTL_CONTROL",
        "DRM_IOCTL_ADD_MAP",
        "DRM_IOCTL_ADD_BUFS",
        "DRM_IOCTL_MARK_BUFS",
        "DRM_IOCTL_INFO_BUFS",
        "DRM_IOCTL_MAP_BUFS",
        "DRM_IOCTL_FREE_BUFS",
        "DRM_IOCTL_RM_MAP",
        "DRM_IOCTL_SET_SAREA_CTX",
        "DRM_IOCTL_GET_SAREA_CTX",
        "DRM_IOCTL_SET_MASTER",
        "DRM_IOCTL_DROP_MASTER",
        "DRM_IOCTL_ADD_CTX",
        "DRM_IOCTL_RM_CTX",
        "DRM_IOCTL_MOD_CTX",
        "DRM_IOCTL_GET_CTX",
        "DRM_IOCTL_SWITCH_CTX",
        "DRM_IOCTL_NEW_CTX",
        "DRM_IOCTL_RES_CTX",
        "DRM_IOCTL_ADD_DRAW",
        "DRM_IOCTL_RM_DRAW",
        "DRM_IOCTL_DMA",
        "DRM_IOCTL_LOCK",
        "DRM_IOCTL_UNLOCK",
        "DRM_IOCTL_FINISH",
        "DRM_IOCTL_PRIME_HANDLE_TO_FD",
        "DRM_IOCTL_PRIME_FD_TO_HANDLE",
        "DRM_IOCTL_AGP_ACQUIRE",
        "DRM_IOCTL_AGP_RELEASE",
        "DRM_IOCTL_AGP_ENABLE",
        "DRM_IOCTL_AGP_INFO",
        "DRM_IOCTL_AGP_ALLOC",
        "DRM_IOCTL_AGP_FREE",
        "DRM_IOCTL_AGP_BIND",
        "DRM_IOCTL_AGP_UNBIND",
        "DRM_IOCTL_SG_ALLOC",
        "DRM_IOCTL_SG_FREE",
        "DRM_IOCTL_WAIT_VBLANK",
        "DRM_IOCTL_CRTC_GET_SEQUENCE",
        "DRM_IOCTL_CRTC_QUEUE_SEQUENCE",
        "DRM_IOCTL_UPDATE_DRAW",
        "DRM_IOCTL_MODE_GETRESOURCES",
        "DRM_IOCTL_MODE_GETCRTC",
        "DRM_IOCTL_MODE_SETCRTC",
        "DRM_IOCTL_MODE_CURSOR",
        "DRM_IOCTL_MODE_GETGAMMA",
        "DRM_IOCTL_MODE_SETGAMMA",
        "DRM_IOCTL_MODE_GETENCODER",
        "DRM_IOCTL_MODE_GETCONNECTOR",
        "DRM_IOCTL_MODE_ATTACHMODE",
        "DRM_IOCTL_MODE_DETACHMODE",
        "DRM_IOCTL_MODE_GETPROPERTY",
        "DRM_IOCTL_MODE_SETPROPERTY",
        "DRM_IOCTL_MODE_GETPROPBLOB",
        "DRM_IOCTL_MODE_GETFB",
        "DRM_IOCTL_MODE_ADDFB",
        "DRM_IOCTL_MODE_RMFB",
        "DRM_IOCTL_MODE_PAGE_FLIP",
        "DRM_IOCTL_MODE_DIRTYFB",
        "DRM_IOCTL_MODE_CREATE_DUMB",
        "DRM_IOCTL_MODE_MAP_DUMB",
        "DRM_IOCTL_MODE_DESTROY_DUMB",
        "DRM_IOCTL_MODE_GETPLANERESOURCES",
        "DRM_IOCTL_MODE_GETPLANE",
        "DRM_IOCTL_MODE_SETPLANE",
        "DRM_IOCTL_MODE_ADDFB2",
        "DRM_IOCTL_MODE_OBJ_GETPROPERTIES",
        "DRM_IOCTL_MODE_OBJ_SETPROPERTY",
        "DRM_IOCTL_MODE_CURSOR2",
        "DRM_IOCTL_MODE_ATOMIC",
        "DRM_IOCTL_MODE_CREATEPROPBLOB",
        "DRM_IOCTL_MODE_DESTROYPROPBLOB",
        "DRM_IOCTL_SYNCOBJ_CREATE",
        "DRM_IOCTL_SYNCOBJ_DESTROY",
        "DRM_IOCTL_SYNCOBJ_HANDLE_TO_FD",
        "DRM_IOCTL_SYNCOBJ_FD_TO_HANDLE",
        "DRM_IOCTL_SYNCOBJ_WAIT",
        "DRM_IOCTL_SYNCOBJ_RESET",
        "DRM_IOCTL_SYNCOBJ_SIGNAL",
        "DRM_IOCTL_MODE_CREATE_LEASE",
        "DRM_IOCTL_MODE_LIST_LESSEES",
        "DRM_IOCTL_MODE_GET_LEASE",
        "DRM_IOCTL_MODE_REVOKE_LEASE",
    ];


    // Applies a formatting function over a slice of strings,
    // concatenating them on separate lines into a single String
    fn apply_formatting<I, F>(iter: I, f: F) -> String
        where
        I: Iterator,
        I::Item: AsRef<str>,
        F: Fn(&str) -> String
    {
        iter.fold(String::new(), | acc, x | {
            acc + &f(x.as_ref()) + "\n"
        })
    }

    // Create a name for a temporary value
    fn tmp_val(name: &str) -> String {
        format!("{}{}", TMP_BIND_PREFIX, name)
    }

    // Create a C include directive
    fn include(header: &str) -> String {
        format!("#include <{}>", header)
    }

    // Create a C constant
    fn decl_const(ty: &str, name: &str, value: &str) -> String {
        format!("const {} {} = {};", ty, name, value)
    }

    // Create a C macro definition
    fn define_macro(name: &str, val: &str) -> String {
        format!("#define {} {}", name, val)
    }

    // Create a C undefinition
    fn undefine_macro(name: &str) -> String {
        format!("#undef {}", name)
    }

    // Rebind a C macro as a constant
    // Required for some macros that won't get generated
    fn rebind_macro(name: &str) -> String {
        let tmp_name = tmp_val(name);
        format!("{}\n{}\n{}\n",
                decl_const("unsigned int", &tmp_name, name),
                undefine_macro(name),
                decl_const("unsigned int", name, &tmp_name)
        )
    }

    // Fully create the header
    fn create_header() -> String {
        apply_formatting(INCLUDES.iter(), include) +
            &apply_formatting(MACROS.iter(), rebind_macro) +
            &apply_formatting(IOCTLS.iter(), rebind_macro)
    }


    pub fn generate_bindings() {
        let pkgconf = pkg_config::Config::new();
        let lib = pkgconf.probe("libdrm").unwrap();

        println!("{}", &create_header());

        let bindings = Builder::default()
            .header_contents("bindings.h", &create_header())
            .ctypes_prefix("libc")
            .bitfield_enum("drm_ctx_flags")
            .bitfield_enum("drm_dma_flags")
            .bitfield_enum("drm_lock_flags")
            .bitfield_enum("drm_map_flags")
            .bitfield_enum("drm_vblan_seq_flags")
            .constified_enum_module("drm_control__bindgen_ty_1")
            .constified_enum_module("drm_mode_subconnector")
            .constified_enum_module("drm_map_type")
            .constified_enum_module("drm_stat_type")
            .constified_enum_module("drm_vblan_seq_type")
            .prepend_enum_name(false)
            .layout_tests(false)
            .rustfmt_bindings(true)
            .derive_copy(true)
            .derive_debug(true)
            .derive_default(true)
            .derive_hash(true)
            .derive_eq(true)
            .whitelist_recursively(false)
            .blacklist_type(TMP_BIND_PREFIX_REG)
            .clang_args(lib.include_paths.into_iter().map(| path | {
                "-I".to_string() + &path.into_os_string().into_string().unwrap()
            }))
            .generate()
            .expect("Unable to generate libdrm bindings");

        let out_path = String::from(var("OUT_DIR").unwrap());
        let bind_file = PathBuf::from(out_path).join("bindings.rs");

        bindings.write_to_file(bind_file).expect("Could not write bindings");
    }
}

#[cfg(feature = "use_bindgen")]
pub fn main() {
    use_bindgen::generate_bindings();
}

#[cfg(not(feature = "use_bindgen"))]
pub fn main() {}
