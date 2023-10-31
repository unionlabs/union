use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    cc::Build::new()
        .file("src/cxx.cc")
        .cpp(true)
        .cpp_link_stdlib(None) // linked via link-cplusplus crate
        .flag_if_supported(cxxbridge_flags::STD)
        .warnings_into_errors(cfg!(deny_warnings))
        .compile("cxxbridge1");

    println!("cargo:rerun-if-changed=src/cxx.cc");
    println!("cargo:rerun-if-changed=include/cxx.h");
    println!("cargo:rustc-cfg=built_with_cargo");

    if let Some(manifest_dir) = env::var_os("CARGO_MANIFEST_DIR") {
        let cxx_h = Path::new(&manifest_dir).join("include").join("cxx.h");
        println!("cargo:HEADER={}", cxx_h.to_string_lossy());
    }

    if let Some(rustc) = rustc_version() {
        if rustc.minor < 60 {
            println!("cargo:warning=The cxx crate requires a rustc version 1.60.0 or newer.");
            println!(
                "cargo:warning=You appear to be building with: {}",
                rustc.version,
            );
        }
    }
}

struct RustVersion {
    version: String,
    minor: u32,
}

fn rustc_version() -> Option<RustVersion> {
    let rustc = env::var_os("RUSTC")?;
    let output = Command::new(rustc).arg("--version").output().ok()?;
    let version = String::from_utf8(output.stdout).ok()?;
    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }
    let minor = pieces.next()?.parse().ok()?;
    Some(RustVersion { version, minor })
}
