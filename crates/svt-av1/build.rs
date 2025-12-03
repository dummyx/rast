use std::env;
use std::path::PathBuf;

fn main() {
    let enc = env::var("CARGO_FEATURE_ENCODER").is_ok();
    let dec = env::var("CARGO_FEATURE_DECODER").is_ok();

    // Propagate the native search path from svt-av1-sys if available.
    if let Ok(root) = env::var("DEP_SVTAV1_ROOT") {
        let lib_dir = PathBuf::from(root).join("lib");
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
    }

    if enc {
        println!("cargo:rustc-link-lib=static=SvtAv1Enc");
    }
    if dec {
        println!("cargo:rustc-link-lib=static=SvtAv1Dec");
    }
}
