---
name: rast-svt-av1
description: Workflow for developing the rast SVT-AV1 Rust bindings workspace (svt-av1-sys + svt-av1): build flags, env vars, and validation commands.
---

# rast SVT-AV1 Workspace

Use this skill when making changes to the Rust SVT-AV1 bindings in this repo.

## Key Facts

- The vendored SVT-AV1 copy (`vendor/SVT-AV1`) is encoder-only (v4.0.1). Enabling the Rust `decoder` feature requires a system install that provides `EbSvtAv1Dec.h` and `libSvtAv1Dec.a`.
- `svt-av1-sys` links `SvtAv1Enc` (and `SvtAv1Dec` when enabled) as **static** libraries.
- pkg-config is opt-in: set `SVT_AV1_NO_PKG_CONFIG=0` to enable it.
- Vendored builds disable LTO by default; set `SVT_AV1_ENABLE_LTO=1` to force enable.

## What To Edit

- `crates/svt-av1-sys/src/lib.rs`: keep it minimal; do not hand-edit generated bindings (they come from `OUT_DIR` when `buildtime-bindgen` is enabled).
- `crates/svt-av1-sys/build.rs`: adjust bindgen inputs/flags, include path discovery, and vendored/system linking behavior.
- `crates/svt-av1/src/lib.rs`: safe(ish) wrappers; keep API close to the C API.

## Common Commands

```bash
# Sys crate (vendored build + bindgen; deterministic)
SVT_AV1_NO_PKG_CONFIG=1 SVT_AV1_INCLUDE_DIR=vendor/SVT-AV1/Source/API cargo check -p svt-av1-sys

# Wrapper crate
cargo build -p svt-av1

# Encoder example check (vendored headers)
SVT_AV1_NO_PKG_CONFIG=1 SVT_AV1_INCLUDE_DIR=vendor/SVT-AV1/Source/API cargo check -p svt-av1 --example encode

# ROI example check (vendored headers)
SVT_AV1_NO_PKG_CONFIG=1 SVT_AV1_INCLUDE_DIR=vendor/SVT-AV1/Source/API cargo check -p svt-av1 --example encode_roi

# Decode example (system decoder install via pkg-config)
SVT_AV1_NO_PKG_CONFIG=0 cargo run -p svt-av1 --features decoder --example decode -- <file.ivf>

# E2E CLI tests (encode always; decode is opt-in)
cargo test -p svt-av1 --test e2e_cli
SVT_AV1_E2E_DECODER=1 SVT_AV1_NO_PKG_CONFIG=0 cargo test -p svt-av1 --test e2e_cli
```

## Validation Checklist

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test -p svt-av1`
  - E2E CLI: `cargo test -p svt-av1 --test e2e_cli` (decode opt-in via `SVT_AV1_E2E_DECODER=1`)
  - If touching decoder code, also: `SVT_AV1_NO_PKG_CONFIG=0 cargo test -p svt-av1 --features decoder`
