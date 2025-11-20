use std::{env, path::PathBuf};

fn parse_bool(val: &str) -> bool {
    val == "1" || val.eq_ignore_ascii_case("true") || val.eq_ignore_ascii_case("yes")
}

fn env_flag(var: &str) -> bool {
    env::var(var).map(|v| parse_bool(&v)).unwrap_or(false)
}

fn try_pkg_config(names: &[&str]) -> Option<pkg_config::Library> {
    for name in names {
        match pkg_config::Config::new().probe(name) {
            Ok(lib) => return Some(lib),
            Err(_) => continue,
        }
    }
    None
}

fn possible_include_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(dir) = env::var("SVT_AV1_INCLUDE_DIR") {
        dirs.push(PathBuf::from(dir));
    }
    // Common system paths
    dirs.push(PathBuf::from("/usr/include/svt-av1"));
    dirs.push(PathBuf::from("/usr/local/include/svt-av1"));
    dirs.push(PathBuf::from("/opt/homebrew/include/svt-av1"));
    dirs.push(PathBuf::from("/opt/local/include/svt-av1"));
    // Fallback: try bare include dirs (rare)
    dirs.push(PathBuf::from("/usr/include"));
    dirs.push(PathBuf::from("/usr/local/include"));
    dirs
}

fn link_with_provided(enc: bool, dec: bool) -> Option<Vec<PathBuf>> {
    let mut include_paths: Vec<PathBuf> = Vec::new();

    // Use pkg-config only when explicitly allowed.
    let no_pc_env = env::var("SVT_AV1_NO_PKG_CONFIG").ok();
    let use_pkg_config = no_pc_env.as_deref() == Some("0");
    if use_pkg_config {
        if let Some(lib) = if let Ok(override_name) = env::var("SVT_AV1_PKG_CONFIG_NAME") {
            pkg_config::Config::new().probe(&override_name).ok()
        } else {
            // Try a few common names
            try_pkg_config(&["svt-av1", "SvtAv1Enc", "SvtAv1Dec"]) // decoder may not be packaged separately
        } {
            if enc {
                println!("cargo:rustc-link-lib=SvtAv1Enc");
            }
            if dec {
                println!("cargo:rustc-link-lib=SvtAv1Dec");
            }
            include_paths.extend(lib.include_paths);
            return Some(include_paths);
        }
    }

    // Manual discovery via env var dir, only if lib dir is provided.
    let lib_dir = env::var("SVT_AV1_LIB_DIR").ok();
    if let Some(dir) = lib_dir.as_ref() {
        println!("cargo:rustc-link-search=native={}", dir);
        if enc {
            println!("cargo:rustc-link-lib=SvtAv1Enc");
        }
        if dec {
            println!("cargo:rustc-link-lib=SvtAv1Dec");
        }
        let include_dir = env::var("SVT_AV1_INCLUDE_DIR")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("vendor/SVT-AV1/Source/API"));
        include_paths.push(include_dir);
        return Some(include_paths);
    }

    None
}

fn build_vendored(enc: bool, dec: bool) -> Vec<PathBuf> {
    if dec {
        panic!("Decoder feature requested but EbSvtAv1Dec.h is not vendored. Provide a decoder-capable system install with SVT_AV1_NO_PKG_CONFIG=0 or SVT_AV1_LIB_DIR.");
    }

    let enable_lto = env::var("SVT_AV1_ENABLE_LTO")
        .map(|v| parse_bool(&v))
        .unwrap_or(true);

    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let vendor_dir = manifest_dir.join("../../vendor/SVT-AV1");
    if !vendor_dir.exists() {
        panic!(
            "Vendored SVT-AV1 source not found at {}",
            vendor_dir.display()
        );
    }

    let dst = cmake::Config::new(&vendor_dir)
        .profile("Release")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("BUILD_APPS", "OFF")
        .define("BUILD_TESTING", "OFF")
        .define("ENABLE_LTO", if enable_lto { "ON" } else { "OFF" })
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .build();

    let mut lib_dir = dst.join("lib");
    if !lib_dir.exists() {
        let alt = dst.join("lib64");
        if alt.exists() {
            lib_dir = alt;
        }
    }

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    if enc {
        println!("cargo:rustc-link-lib=static=SvtAv1Enc");
    }

    vec![vendor_dir.join("Source/API")]
}

#[cfg(feature = "buildtime-bindgen")]
fn generate_bindings(enc: bool, dec: bool, mut include_dirs: Vec<PathBuf>) {
    // Also search common paths
    include_dirs.extend(possible_include_dirs());

    let clang_args: Vec<String> = include_dirs
        .iter()
        .map(|p| format!("-I{}", p.display()))
        .collect();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    if enc {
        let mut builder = bindgen::Builder::default()
            .header_contents("wrapper_enc.h", "#include <EbSvtAv1Enc.h>\n")
            .allowlist_recursively(true)
            .clang_args(&clang_args)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .layout_tests(false)
            .derive_debug(true)
            .generate_comments(true)
            .size_t_is_usize(true);

        // Some platforms require defining __STDC_CONSTANT_MACROS for stdint macros
        builder = builder.clang_arg("-D__STDC_CONSTANT_MACROS");

        let bindings = builder
            .generate()
            .expect("Unable to generate SVT-AV1 encoder bindings");

        let out_path = out_dir.join("bindings_encoder.rs");
        bindings
            .write_to_file(&out_path)
            .expect("Couldn't write encoder bindings");
        println!("cargo:rerun-if-changed=wrapper_enc.h");
    }

    if dec {
        let mut builder = bindgen::Builder::default()
            .header_contents(
                "wrapper_dec.h",
                "#include <stddef.h>\n#include <EbSvtAv1Dec.h>\n",
            )
            .allowlist_recursively(true)
            .clang_args(&clang_args)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .layout_tests(false)
            .derive_debug(true)
            .generate_comments(true)
            .size_t_is_usize(true);

        builder = builder.clang_arg("-D__STDC_CONSTANT_MACROS");

        let bindings = builder
            .generate()
            .expect("Unable to generate SVT-AV1 decoder bindings");

        let out_path = out_dir.join("bindings_decoder.rs");
        bindings
            .write_to_file(&out_path)
            .expect("Couldn't write decoder bindings");
        println!("cargo:rerun-if-changed=wrapper_dec.h");
    }
}

#[cfg(not(feature = "buildtime-bindgen"))]
fn generate_bindings(_enc: bool, _dec: bool, _include_dirs: Vec<PathBuf>) {}

fn main() {
    let enc = cfg!(feature = "encoder");
    let dec = cfg!(feature = "decoder");

    let force_vendor = env_flag("SVT_AV1_BUILD_FROM_SOURCE");
    let include_dirs = if force_vendor {
        None
    } else {
        link_with_provided(enc, dec)
    };
    let include_dirs = match include_dirs {
        Some(paths) => paths,
        None => build_vendored(enc, dec),
    };

    // Decoder headers are not present in the vendored tree; require an override for decoding.
    if dec {
        let has_dec_header = include_dirs
            .iter()
            .any(|p| p.join("EbSvtAv1Dec.h").exists());
        if !has_dec_header {
            panic!("Decoder feature requested but EbSvtAv1Dec.h not found. The vendored SVT-AV1 copy is encoder-only; set SVT_AV1_NO_PKG_CONFIG=0 and provide a decoder-capable install to enable decoding.");
        }
    }

    generate_bindings(enc, dec, include_dirs);

    // Always rerun if env hints change
    println!("cargo:rerun-if-env-changed=SVT_AV1_BUILD_FROM_SOURCE");
    println!("cargo:rerun-if-env-changed=SVT_AV1_ENABLE_LTO");
    println!("cargo:rerun-if-env-changed=SVT_AV1_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=SVT_AV1_LIB_DIR");
    println!("cargo:rerun-if-env-changed=SVT_AV1_PKG_CONFIG_NAME");
    println!("cargo:rerun-if-env-changed=SVT_AV1_NO_PKG_CONFIG");
}
