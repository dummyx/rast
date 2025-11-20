# Suggested Commands
- Build sys crate: `cargo build -p svt-av1-sys`
- Header-only check using vendored headers (preferred deterministic path): `SVT_AV1_NO_PKG_CONFIG=1 SVT_AV1_INCLUDE_DIR=vendor/SVT-AV1/Source/API cargo check -p svt-av1-sys`
- Build safe wrapper: `cargo build -p svt-av1`
- Run encoder example with vendored headers: `SVT_AV1_NO_PKG_CONFIG=1 SVT_AV1_INCLUDE_DIR=vendor/SVT-AV1/Source/API cargo check -p svt-av1 --example encode`
- Opt into pkg-config discovery (requires system install, for decoder too): `SVT_AV1_NO_PKG_CONFIG=0 cargo build -p svt-av1-sys` (override package name via `SVT_AV1_PKG_CONFIG_NAME`)
- Tests: `cargo test -p svt-av1`
- Lint/format: `cargo fmt --all` and `cargo clippy --workspace -D warnings`
- Root binary (demo): `cargo run -p rast`