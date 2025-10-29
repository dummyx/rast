svt-av1-sys
===========

Raw Rust FFI bindings to the SVT-AV1 C API (encoder; decoder optional), generated with bindgen at build time. Targeted for SVT-AV1 v3.1.2 which exposes encoder headers.

How it works
- By default, the build script uses `pkg-config` to discover include/lib paths for SVT-AV1 and runs `bindgen` on the public headers to generate Rust FFI bindings.
- You can control discovery with environment variables if `pkg-config` isn’t available.
- Feature flags: `encoder` and `decoder` enable their respective APIs (only `encoder` enabled by default for v3.1.2).

Environment variables
- `SVT_AV1_INCLUDE_DIR`: path to directory containing SVT-AV1 headers (e.g., `EbSvtAv1Enc.h`, `EbSvtAv1Dec.h`).
- `SVT_AV1_LIB_DIR`: path to directory containing `libSvtAv1Enc.*` and/or `libSvtAv1Dec.*`.
- `SVT_AV1_PKG_CONFIG_NAME`: override pkg-config name (defaults: tries `svt-av1`, `SvtAv1Enc`, `SvtAv1Dec`).
- `SVT_AV1_NO_PKG_CONFIG=1`: skip pkg-config probing entirely.

Linking
- On Linux the libraries are typically `SvtAv1Enc` (and `SvtAv1Dec` if decoder is available).
- The build script emits the appropriate `cargo:rustc-link-lib` lines as needed.

Headers
- Typical include directories: `/usr/include/svt-av1`, `/usr/local/include/svt-av1`.

Notes
- Bindgen is used at build time (`buildtime-bindgen` feature enabled by default).
- If you prefer vendored/prebuilt bindings, add them to `src/` and adjust the build/features accordingly.
