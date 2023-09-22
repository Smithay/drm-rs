# Usage

```toml
[dependencies]
drm-sys = "..."
```

# Platforms

The following platforms have prebuilt bindings available:

* Linux/Android
  * x86_64
  * x86
  * arm
  * aarch64
* FreeBSD
  * x86_64

If bindings for your target platform are not available, you can attempt to
generate them by enabling the `use_bindgen` feature:

```toml
[dependencies.drm-sys]
version = "..."
features = ["use_bindgen"]
```
