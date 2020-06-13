#[cfg(feature = "use_bindgen")]
mod use_bindgen {
    extern crate bindgen;
    extern crate pkg_config;

    use self::bindgen::{Builder, CodegenConfig};
    use std::env::var;
    use std::path::PathBuf;

    fn create_builder(contents: &str) -> Builder {
        println!("{}", contents);

        let pkgconf = pkg_config::Config::new();
        let lib = pkgconf.probe("libdrm").unwrap();

        let config = CodegenConfig::all();

        Builder::default()
            .clang_args(lib.include_paths.into_iter().map(| path | {
                "-I".to_string() + &path.into_os_string().into_string().unwrap()
            }))
            .header_contents("bindings.h", contents)
            .ctypes_prefix("libc")
            .with_codegen_config(config)
            .prepend_enum_name(false)
            .layout_tests(false)
            .generate_comments(false)
            .rustfmt_bindings(true)
            .derive_copy(true)
            .derive_debug(true)
            .derive_default(true)
            .derive_hash(true)
            .derive_eq(true)
            .whitelist_recursively(true)
            .use_core()
    }

    const TMP_BIND_PREFIX: &str = "__BINDGEN_TMP_";
    const TMP_BIND_PREFIX_REG: &str = "_BINDGEN_TMP_.*";

    const INCLUDES: &'static [&str] = &[
        "drm.h",
        "drm_mode.h"
    ];

    const MACROS: &'static [&str] = &[
        "DRM_MODE_PROP_SIGNED_RANGE",
        "DRM_MODE_PROP_OBJECT"
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
        format!("{}\n{}\n{}\n{}",
                decl_const("unsigned int", &tmp_name, name),
                undefine_macro(name),
                decl_const("unsigned int", name, &tmp_name),
                define_macro(name, name)
        )
    }

    // Fully create the header
    fn create_header() -> String {
        apply_formatting(INCLUDES.iter(), include) +
            &apply_formatting(MACROS.iter(), rebind_macro)
    }


    pub fn generate_bindings() {
        let bindings = create_builder(&create_header())
            .blacklist_type(TMP_BIND_PREFIX_REG)
            .blacklist_type("drm_control_DRM_ADD_COMMAND")
            .whitelist_type("DRM_.*|drm_.*")
            .whitelist_var("DRM_.*|drm_.*")
            .constified_enum_module("drm_control_.*")
            .constified_enum_module("drm_buf_desc_.*")
            .constified_enum_module("drm_map_type")
            .constified_enum_module("drm_map_flags")
            .constified_enum_module("drm_stat_type")
            .constified_enum_module("drm_lock_flags")
            .constified_enum_module("drm_dma_flags")
            .constified_enum_module("drm_ctx_flags")
            .constified_enum_module("drm_drawable_info_type_t")
            .constified_enum_module("drm_vblank_seq_type")
            .constified_enum_module("drm_mode_subconnector")
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
