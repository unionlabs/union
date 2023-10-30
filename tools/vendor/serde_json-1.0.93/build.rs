use std::env;
use std::process::Command;
use std::str::{self, FromStr};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Decide ideal limb width for arithmetic in the float parser. Refer to
    // src/lexical/math.rs for where this has an effect.
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    match target_arch.as_str() {
        "aarch64" | "mips64" | "powerpc64" | "x86_64" => {
            println!("cargo:rustc-cfg=limb_width_64");
        }
        _ => {
            println!("cargo:rustc-cfg=limb_width_32");
        }
    }

    let minor = match rustc_minor_version() {
        Some(minor) => minor,
        None => return,
    };

    // BTreeMap::get_key_value
    // https://blog.rust-lang.org/2019/12/19/Rust-1.40.0.html#additions-to-the-standard-library
    if minor < 40 {
        println!("cargo:rustc-cfg=no_btreemap_get_key_value");
    }

    // BTreeMap::remove_entry
    // https://blog.rust-lang.org/2020/07/16/Rust-1.45.0.html#library-changes
    if minor < 45 {
        println!("cargo:rustc-cfg=no_btreemap_remove_entry");
    }

    // BTreeMap::retain
    // https://blog.rust-lang.org/2021/06/17/Rust-1.53.0.html#stabilized-apis
    if minor < 53 {
        println!("cargo:rustc-cfg=no_btreemap_retain");
    }
}

fn rustc_minor_version() -> Option<u32> {
    let rustc = env::var_os("RUSTC")?;
    let output = Command::new(rustc).arg("--version").output().ok()?;
    let version = str::from_utf8(&output.stdout).ok()?;
    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }
    let next = pieces.next()?;
    u32::from_str(next).ok()
}
