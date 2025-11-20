# Repository Guidelines

## Project Structure & Module Organization
- `Cargo.toml` (workspace), `src/` (root binary `rast`).
- `crates/svt-av1-sys/`: Raw FFI (bindgen) to SVT‑AV1 v3.1.2.
- `crates/svt-av1/`: Thin safe wrappers and helpers; examples in `crates/svt-av1/examples/`.
- `vendor/SVT-AV1/`: Vendored upstream headers (v3.1.2) used for header‑only checks.
- `.github/workflows/bindgen.yml`: CI that runs bindgen against vendored headers.

## Build, Test, and Development Commands
- Build sys crate: `cargo build -p svt-av1-sys`
- Header‑only check (no linking):
  `SVT_AV1_NO_PKG_CONFIG=1 SVT_AV1_INCLUDE_DIR=vendor/SVT-AV1/Source/API cargo check -p svt-av1-sys`
- Safe wrapper: `cargo build -p svt-av1`
- Example (encoder):
  `SVT_AV1_NO_PKG_CONFIG=1 SVT_AV1_INCLUDE_DIR=vendor/SVT-AV1/Source/API cargo check -p svt-av1 --example encode`
- When system SVT‑AV1 headers/libs are present, they must be API‑compatible with v3.1.2; if not, prefer the vendored/header‑only commands above to avoid bindgen/type mismatches. Decoder requires a system install with `EbSvtAv1Dec.h`/`libSvtAv1Dec` (vendored copy is encoder-only); pkg-config is opt-in via `SVT_AV1_NO_PKG_CONFIG=0`.
- Format & lint: `cargo fmt --all`, `cargo clippy --workspace -D warnings`

## Coding Style & Naming Conventions
- Rust edition: root 2024; crates 2021. Indent 4 spaces.
- Prefer idiomatic Rust: `snake_case` for functions, `PascalCase` for types, `SCREAMING_SNAKE_CASE` for consts.
- In `svt-av1-sys`, don’t hand‑edit generated bindings; adjust `build.rs` instead.
- Keep wrapper minimal and close to the C API; document public items.

## Testing Guidelines
- Use Rust tests: `cargo test -p svt-av1`.
- Place unit tests inline with modules (`mod tests {}`); put integration tests under `crates/svt-av1/tests/` if added.
- Prefer small, deterministic tests; avoid requiring system SVT‑AV1 unless the test is feature‑gated.

## Commit & Pull Request Guidelines
- Keep commits focused; prefer Conventional Commit style (`feat:`, `fix:`, `docs:`) when possible.
- PRs should include: summary, motivation, linked issues, and any env/command notes to reproduce.
- Update docs (`README.md`, this file) when changing public API or build behavior.

## Security & Configuration Tips
- Linking: set `SVT_AV1_INCLUDE_DIR` and `SVT_AV1_LIB_DIR` or install via `pkg-config`.
- Do not commit generated bindings or secrets. Vendored headers are pinned to v3.1.2.

## Agent‑Specific Instructions
- Prefer minimal diffs; keep code style consistent. Use `rg` for search and read files in ≤250‑line chunks.
- Obey this AGENTS.md for all edits within the repo; update it if your changes alter the workflow.
