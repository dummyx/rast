# Repository Guidelines

## Project Structure & Module Organization
- `Cargo.toml` (workspace), `src/` (root binary `rast` - currently placeholder).
- `crates/svt-av1-sys/`: Raw FFI (bindgen) to SVT‑AV1 v3.1.2.
- `crates/svt-av1/`: Thin safe wrappers and helpers; examples in `crates/svt-av1/examples/`.
- `vendor/SVT-AV1/`: Vendored upstream source (v3.1.2) used for header‑only checks and building.

## Build, Test, and Development Commands
- Build sys crate: `cargo build -p svt-av1-sys`
- Header‑only check (no linking):
  `SVT_AV1_NO_PKG_CONFIG=1 SVT_AV1_INCLUDE_DIR=vendor/SVT-AV1/Source/API cargo check -p svt-av1-sys`
- Safe wrapper: `cargo build -p svt-av1`
- Examples:
  - Encoder: `SVT_AV1_NO_PKG_CONFIG=1 SVT_AV1_INCLUDE_DIR=vendor/SVT-AV1/Source/API cargo check -p svt-av1 --example encode`
  - ROI encoding: `SVT_AV1_NO_PKG_CONFIG=1 SVT_AV1_INCLUDE_DIR=vendor/SVT-AV1/Source/API cargo check -p svt-av1 --example encode_roi`
  - Decoder: `cargo check -p svt-av1 --example decode` (requires system install)
- When system SVT‑AV1 headers/libs are present, they must be API‑compatible with v3.1.2; if not, prefer the vendored commands above to avoid bindgen/type mismatches. Decoder requires a system install with `EbSvtAv1Dec.h`/`libSvtAv1Dec` (vendored copy is encoder-only); pkg-config is opt-in via `SVT_AV1_NO_PKG_CONFIG=0`.
- Format & lint: `cargo fmt --all`, `cargo clippy --workspace -- -D warnings`

### Build Process Notes
- By default, the build compiles the vendored SVT-AV1 encoder via CMake when pkg-config is disabled and no `SVT_AV1_LIB_DIR` is provided
- LTO is disabled by default for broader linker compatibility (rust-lld cannot consume GCC LTO objects)
- Users will see a warning about LTO being disabled; set `SVT_AV1_ENABLE_LTO=1` to force enable

## Coding Style & Naming Conventions
- Rust edition: root 2024; crates 2021. Indent 4 spaces.
- Prefer idiomatic Rust: `snake_case` for functions, `PascalCase` for types, `SCREAMING_SNAKE_CASE` for consts.
- In `svt-av1-sys`, don’t hand‑edit generated bindings; adjust `build.rs` instead.
- Keep wrapper minimal and close to the C API; document public items.

## Testing Guidelines
- Use Rust tests: `cargo test -p svt-av1`.
- Place unit tests inline with modules (`mod tests {}`); current tests are in `src/tests.rs`.
- Prefer small, deterministic tests; avoid requiring system SVT‑AV1 unless the test is feature‑gated.
- Test with specific features: `cargo test -p svt-av1 --features encoder,decoder` (when decoder headers available)

## Commit & Pull Request Guidelines
- Keep commits focused; prefer Conventional Commit style (`feat:`, `fix:`, `docs:`) when possible.
- PRs should include: summary, motivation, linked issues, and any env/command notes to reproduce.
- Update docs (`README.md`, this file) when changing public API or build behavior.

## Security & Configuration Tips
- Linking: set `SVT_AV1_INCLUDE_DIR` and `SVT_AV1_LIB_DIR` or install via `pkg-config`.
- Do not commit generated bindings or secrets. Vendored headers are pinned to v3.1.2.

### Environment Variables
- `SVT_AV1_INCLUDE_DIR`: directory containing headers.
- `SVT_AV1_LIB_DIR`: directory containing libraries.
- `SVT_AV1_PKG_CONFIG_NAME`: override pkg-config package name.
- `SVT_AV1_NO_PKG_CONFIG=1`: disable pkg-config probing (default if unset; pkg-config is opt-in).
- `SVT_AV1_BUILD_FROM_SOURCE=1`: force building the vendored SVT-AV1 codec.
- `SVT_AV1_ENABLE_LTO=0/1`: toggle link-time optimization for vendored builds (default off).

## Agent‑Specific Instructions
- Prefer minimal diffs; keep code style consistent. Use `rg` for search and read files in ≤250‑line chunks.
- Obey this AGENTS.md for all edits within the repo; update it if your changes alter the workflow.
- Build warnings from bindgen-generated code are suppressed with `#![allow(unnecessary_transmutes)]` in `svt-av1-sys/src/lib.rs`.
