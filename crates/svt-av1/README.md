svt-av1
=======

Thin safe wrappers over the raw `svt-av1-sys` bindings for SVT-AV1.
Targeted for SVT-AV1 v3.1.2 encoder API. The goal is ergonomics with RAII for initialization/teardown while keeping API parity close to the C interface.

Features
- `encoder` feature (default) mirrors the underlying sys crate.
- `decoder` feature is available but not enabled by default; decoder headers are not present in v3.1.2.
- RAII wrappers for encoder handles with `Drop` safety.
- Minimal, composable safe methods for init, parameter setting, and frame/packet I/O.

This crate intentionally avoids heavy abstractions to stay close to the native API and make it easy to map to SVT-AV1 docs.
