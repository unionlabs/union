use std::{env, str};

const VALUE_BAG_CAPTURE_CONST_TYPE_ID: &'static str = "VALUE_BAG_CAPTURE_CONST_TYPE_ID";
const VALUE_BAG_CAPTURE_CTOR: &'static str = "VALUE_BAG_CAPTURE_CTOR";
const VALUE_BAG_CAPTURE_FALLBACK: &'static str = "VALUE_BAG_CAPTURE_FALLBACK";

const CTOR_ARCHS: &'static [&'static str] = &["x86_64", "aarch64"];

const CTOR_OSES: &'static [&'static str] = &["windows", "linux", "macos"];

fn main() {
    if env_is_set(VALUE_BAG_CAPTURE_CONST_TYPE_ID) {
        println!(
            "cargo:rustc-cfg={}",
            VALUE_BAG_CAPTURE_CONST_TYPE_ID.to_lowercase()
        );
    } else if env_is_set(VALUE_BAG_CAPTURE_CTOR) {
        println!("cargo:rustc-cfg={}", VALUE_BAG_CAPTURE_CTOR.to_lowercase());
    } else if env_is_set(VALUE_BAG_CAPTURE_FALLBACK) {
        println!(
            "cargo:rustc-cfg={}",
            VALUE_BAG_CAPTURE_FALLBACK.to_lowercase()
        );
    } else if rustc::is_feature_flaggable().unwrap_or(false) {
        println!(
            "cargo:rustc-cfg={}",
            VALUE_BAG_CAPTURE_CONST_TYPE_ID.to_lowercase()
        );
    } else if target_arch_is_any(CTOR_ARCHS) && target_os_is_any(CTOR_OSES) {
        println!("cargo:rustc-cfg={}", VALUE_BAG_CAPTURE_CTOR.to_lowercase());
    } else {
        println!(
            "cargo:rustc-cfg={}",
            VALUE_BAG_CAPTURE_FALLBACK.to_lowercase()
        );
    }

    println!(
        "cargo:rerun-if-env-changed={}",
        VALUE_BAG_CAPTURE_CONST_TYPE_ID
    );
    println!("cargo:rerun-if-env-changed={}", VALUE_BAG_CAPTURE_CTOR);
    println!("cargo:rerun-if-env-changed={}", VALUE_BAG_CAPTURE_FALLBACK);
}

fn target_arch_is_any(archs: &[&str]) -> bool {
    cargo_env_is_any("CARGO_CFG_TARGET_ARCH", archs)
}

fn target_os_is_any(families: &[&str]) -> bool {
    cargo_env_is_any("CARGO_CFG_TARGET_OS", families)
}

fn cargo_env_is_any(env: &str, values: &[&str]) -> bool {
    match env::var(env) {
        Ok(var) if values.contains(&&*var) => true,
        _ => false,
    }
}

fn env_is_set(env: &str) -> bool {
    match env::var(env) {
        Ok(var) if var == "1" => true,
        _ => false,
    }
}
