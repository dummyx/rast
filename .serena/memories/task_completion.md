# Task Completion Checklist
- Run `cargo fmt --all` then `cargo clippy --workspace -D warnings`.
- Execute relevant tests: typically `cargo test -p svt-av1`; add feature flags/env vars if touching decoder or sys binding setup.
- If changing binding/build behavior, validate header-only check with vendored headers: `SVT_AV1_NO_PKG_CONFIG=1 SVT_AV1_INCLUDE_DIR=vendor/SVT-AV1/Source/API cargo check -p svt-av1-sys` (and wrapper/examples as needed).
- Ensure docs/README/AGENTS.md updated when altering public API, build workflows, or prerequisites.
- Confirm env var expectations noted when relying on system SVT-AV1 (decoder/pkg-config paths).
- Keep diffs minimal and focused; follow Conventional Commit style when committing.