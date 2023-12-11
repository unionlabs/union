use std::env;
use std::process::Command;
use std::str::{self, FromStr};

// The rustc-cfg strings below are *not* public API. Please let us know by
// opening a GitHub issue if your build environment requires some way to enable
// these cfgs other than by executing our build script.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let minor = match rustc_minor_version() {
        Some(minor) => minor,
        None => return,
    };

    let target = env::var("TARGET").unwrap();
    let emscripten = target == "asmjs-unknown-emscripten" || target == "wasm32-unknown-emscripten";

    // TryFrom, Atomic types, non-zero signed integers, and SystemTime::checked_add
    // stabilized in Rust 1.34:
    // https://blog.rust-lang.org/2019/04/11/Rust-1.34.0.html#tryfrom-and-tryinto
    // https://blog.rust-lang.org/2019/04/11/Rust-1.34.0.html#library-stabilizations
    if minor < 34 {
        println!("cargo:rustc-cfg=no_core_try_from");
        println!("cargo:rustc-cfg=no_num_nonzero_signed");
        println!("cargo:rustc-cfg=no_systemtime_checked_add");
        println!("cargo:rustc-cfg=no_relaxed_trait_bounds");
    }

    // Current minimum supported version of serde_derive crate is Rust 1.56.
    if minor < 56 {
        println!("cargo:rustc-cfg=no_serde_derive");
    }

    // Support for #[cfg(target_has_atomic = "...")] stabilized in Rust 1.60.
    if minor < 60 {
        println!("cargo:rustc-cfg=no_target_has_atomic");
        // Allowlist of archs that support std::sync::atomic module. This is
        // based on rustc's compiler/rustc_target/src/spec/*.rs.
        let has_atomic64 = target.starts_with("x86_64")
            || target.starts_with("i686")
            || target.starts_with("aarch64")
            || target.starts_with("powerpc64")
            || target.starts_with("sparc64")
            || target.starts_with("mips64el")
            || target.starts_with("riscv64");
        let has_atomic32 = has_atomic64 || emscripten;
        if minor < 34 || !has_atomic64 {
            println!("cargo:rustc-cfg=no_std_atomic64");
        }
        if minor < 34 || !has_atomic32 {
            println!("cargo:rustc-cfg=no_std_atomic");
        }
    }

    // Support for core::ffi::CStr and alloc::ffi::CString stabilized in Rust 1.64.
    // https://blog.rust-lang.org/2022/09/22/Rust-1.64.0.html#c-compatible-ffi-types-in-core-and-alloc
    if minor < 64 {
        println!("cargo:rustc-cfg=no_core_cstr");
    }
}

fn rustc_minor_version() -> Option<u32> {
    let rustc = match env::var_os("RUSTC") {
        Some(rustc) => rustc,
        None => return None,
    };

    let output = match Command::new(rustc).arg("--version").output() {
        Ok(output) => output,
        Err(_) => return None,
    };

    let version = match str::from_utf8(&output.stdout) {
        Ok(version) => version,
        Err(_) => return None,
    };

    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }

    let next = match pieces.next() {
        Some(next) => next,
        None => return None,
    };

    u32::from_str(next).ok()
}
