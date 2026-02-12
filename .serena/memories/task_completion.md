# Task Completion Checklist
- Run `cargo fmt --all` then `cargo clippy --workspace -- -D warnings`.
- Execute relevant tests: typically `cargo test -p svt-av1` (encoder default).
- If changing the CLI/examples, also validate the E2E CLI test harness:
  `cargo test -p svt-av1 --test e2e_cli` (decode is opt-in via `SVT_AV1_E2E_DECODER=1`).
- If changing sys binding/build behavior, validate vendored build + bindgen:
  `SVT_AV1_NO_PKG_CONFIG=1 SVT_AV1_INCLUDE_DIR=vendor/SVT-AV1/Source/API cargo check -p svt-av1-sys`
- If touching examples/wrapper APIs, also check the examples:
  - `SVT_AV1_NO_PKG_CONFIG=1 SVT_AV1_INCLUDE_DIR=vendor/SVT-AV1/Source/API cargo check -p svt-av1 --example encode`
  - `SVT_AV1_NO_PKG_CONFIG=1 SVT_AV1_INCLUDE_DIR=vendor/SVT-AV1/Source/API cargo check -p svt-av1 --example encode_roi`
- If touching decoder paths, ensure a decoder-capable system install is present and verify with pkg-config enabled:
  `SVT_AV1_NO_PKG_CONFIG=0 cargo test -p svt-av1 --features decoder`
- Keep diffs minimal and focused; update docs (`README.md`, `AGENTS.md`) when public API/build workflow changes.
