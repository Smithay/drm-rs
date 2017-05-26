#[cfg(feature = "use_bindgen")]
mod use_bindgen {
    extern crate bindgen;
    use self::bindgen::Builder;
    use std::env::var;
    use std::path::PathBuf;
    use std::fs::File;
    use std::io::Write;

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
        let new_bind = "const nsigned int ".to_string() + name + " = _" + name + ";\n";

        temp_bind + &undef + &new_bind
    }

    pub fn generate_header() {
        let out_path = String::from(var("OUT_DIR").unwrap());
        let header = out_path.clone() + "/bindings.h";

        let mut f = File::create(header).expect("Could not create header");
        let includes = "#include <drm.h>\n#include <drm_mode.h>\n".to_string();
        f.write(includes.as_bytes()).expect("Could not write header.");

        for m in MACROS {
            f.write(bind_function_macro(m).as_bytes())
                .expect("Could not write header");
        }
    }

    pub fn generate_bindings() {
        let out_path = String::from(var("OUT_DIR").unwrap());
        let header = out_path.clone() + "/bindings.h";

        let bindings = Builder::default()
            .no_unstable_rust()
            .header(header)
            .clang_arg("-I/usr/include/drm")
            .ctypes_prefix("libc")
            .emit_builtins()
            .emit_clang_ast()
            .emit_ir()
            .derive_debug(true)
            .derive_default(true)
            .generate()
            .expect("Unable to generate libdrm bindings");

        let bind_file = PathBuf::from(out_path).join("bindings.rs");

        bindings.write_to_file(bind_file).expect("Could not write bindings");
    }
}

#[cfg(feature = "use_bindgen")]
pub fn main() {
    use_bindgen::generate_header();
    use_bindgen::generate_bindings();
}

#[cfg(not(feature = "use_bindgen"))]
pub fn main() {}

