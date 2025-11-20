#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::all)]

// When the buildtime-bindgen feature is enabled (default), bindings are emitted to OUT_DIR.
// Otherwise, the crate can be adapted to include prebuilt bindings from src/.

#[cfg(feature = "encoder")]
#[cfg(feature = "encoder")]
pub mod enc_bindings {
    #[cfg(feature = "buildtime-bindgen")]
    include!(concat!(env!("OUT_DIR"), "/bindings_encoder.rs"));

    #[cfg(not(feature = "buildtime-bindgen"))]
    compile_error!("Prebuilt encoder bindings not provided. Enable 'buildtime-bindgen' or vendor prebuilt files.");
}

#[cfg(feature = "decoder")]
pub mod dec_bindings {
    #[cfg(feature = "buildtime-bindgen")]
    include!(concat!(env!("OUT_DIR"), "/bindings_decoder.rs"));

    #[cfg(not(feature = "buildtime-bindgen"))]
    compile_error!("Prebuilt decoder bindings not provided. Enable 'buildtime-bindgen' or vendor prebuilt files.");
}

// Public re-exports to present a flat module surface
// Removed to avoid ambiguous glob re-exports. Use `enc_bindings` or `dec_bindings` directly.
