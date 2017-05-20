#[cfg(feature = "use_bindgen")]
mod use_bindgen {
    extern crate bindgen;
    use self::bindgen::Builder;
    use std::env::var;
    use std::path::PathBuf;

    pub fn generate_bindings() {
        let bindings = Builder::default()
            .no_unstable_rust()
            .header("src/headers/drm_api.h")
            .clang_arg("-I/usr/include/drm")
            .ctypes_prefix("libc")
            .emit_builtins()
            .emit_clang_ast()
            .emit_ir()
            .derive_debug(true)
            .derive_default(true)
            .generate()
            .expect("Unable to generate libdrm bindings");

        let out_path = PathBuf::from(var("OUT_DIR").unwrap()).join("bindings.rs");


        bindings.write_to_file(out_path).expect("Could not write bindings");
    }
}

#[cfg(feature = "use_bindgen")]
pub fn main() {
    use_bindgen::generate_bindings();
}

#[cfg(not(feature = "use_bindgen"))]
pub fn main() {}

