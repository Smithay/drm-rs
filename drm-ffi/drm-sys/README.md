# Usage

```toml
[dependencies]
drm-sys = "..."
```

# Platforms

The following platforms have prebuilt bindings available:

* Linux
* \*BSD

The bindings are not architecture dependant, but CI testing only happens for:

* arm
* armv7
* aarch64
* riscv64gc
* i686
* x86\_64

If bindings for your target platform are not available, you can attempt to
generate them by enabling the `use_bindgen` feature:

```toml
[dependencies.drm-sys]
version = "..."
features = ["use_bindgen"]
```
