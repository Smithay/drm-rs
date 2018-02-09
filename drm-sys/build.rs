#[cfg(feature = "use_bindgen")]
mod use_bindgen {
    extern crate bindgen;
    extern crate pkg_config;

    use self::bindgen::Builder;
    use std::env::var;
    use std::path::PathBuf;

    const MACROS: &'static [&str] = &[
        "DRM_MODE_PROP_SIGNED_RANGE",
        "DRM_MODE_PROP_OBJECT"
    ];

    // Unfortunately the cexpr crate (and as such, bindgen) does not support C
    // functional macros (https://github.com/jethrogb/rust-cexpr/issues/3).
    // Therefore we must create them ourselves.
    fn bind_function_macro(name: &str) -> String {
        let temp_bind = "const unsigned int _".to_string() + name + " = " + name + ";\n";
        let undef = "#undef ".to_string() + name + "\n";
        let new_bind = "const unsigned int ".to_string() + name + " = _" + name + ";\n";

        temp_bind + &undef + &new_bind
    }

    pub fn generate_header_content() -> String {
        let includes = "#include <drm.h>\n#include <drm_mode.h>\n".to_string();
        let mvec: Vec<_> = MACROS.iter().map(| m | {
            bind_function_macro(m)
        }).collect();

        includes + &mvec.concat()
    }

    pub fn generate_bindings() {
        let pkgconf = pkg_config::Config::new();
        let lib = pkgconf.probe("libdrm").unwrap();

        let header_content = generate_header_content();
        let bindings = Builder::default()
            .header_contents("bindings.h", &header_content)
            .ctypes_prefix("libc")
            .constified_enum_module("DRM_CAP_*")
            .layout_tests(false)
            .rustfmt_bindings(true)
            .blacklist_type("drm_set_client_cap")
            .derive_copy(true)
            .derive_debug(true)
            .derive_default(true)
            .derive_hash(true)
            .derive_eq(true)
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

