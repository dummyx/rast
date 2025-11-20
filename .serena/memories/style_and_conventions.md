# Style and Conventions
- Rust style: workspace root edition 2024; crates use 2021; 4-space indent; idiomatic Rust naming (snake_case fns, PascalCase types, SCREAMING_SNAKE_CASE consts).
- Sys crate: treat bindings as generated; do not hand-edit generated outputs—change `build.rs` or bindgen config instead. Keep wrappers minimal and close to the C API; document public items when exposed.
- Wrapper crate (`svt-av1`): keep abstractions light/RAII-focused for init/teardown and I/O; maintain API parity with SVT-AV1; avoid heavy abstractions.
- Headers/libs: vendored headers pinned to v3.1.2; ensure external installs are API-compatible when enabling decoder/pkg-config.
- Development habits: prefer `rg` for search; minimal diffs; keep code comments succinct and only when clarifying non-obvious logic.
- Git/PRs: Conventional Commit style preferred (e.g., feat:, fix:); update README/AGENTS when public API or build workflow changes.