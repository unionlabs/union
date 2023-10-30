#![allow(clippy::inconsistent_digit_grouping, clippy::unusual_byte_groupings)]

extern crate autocfg;
#[cfg(feature = "bindgen")]
extern crate bindgen;
extern crate cc;
#[cfg(feature = "vendored")]
extern crate openssl_src;
extern crate pkg_config;
#[cfg(target_env = "msvc")]
extern crate vcpkg;

use std::collections::HashSet;
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
mod cfgs;

mod find_normal;
#[cfg(feature = "vendored")]
mod find_vendored;
#[cfg(feature = "bindgen")]
mod run_bindgen;

#[derive(PartialEq)]
enum Version {
    Openssl3xx,
    Openssl11x,
    Openssl10x,
    Libressl,
}

fn env_inner(name: &str) -> Option<OsString> {
    let var = env::var_os(name);
    println!("cargo:rerun-if-env-changed={}", name);

    match var {
        Some(ref v) => println!("{} = {}", name, v.to_string_lossy()),
        None => println!("{} unset", name),
    }

    var
}

fn env(name: &str) -> Option<OsString> {
    let prefix = env::var("TARGET").unwrap().to_uppercase().replace('-', "_");
    let prefixed = format!("{}_{}", prefix, name);
    env_inner(&prefixed).or_else(|| env_inner(name))
}

fn find_openssl(target: &str) -> (Vec<PathBuf>, PathBuf) {
    #[cfg(feature = "vendored")]
    {
        // vendor if the feature is present, unless
        // OPENSSL_NO_VENDOR exists and isn't `0`
        if env("OPENSSL_NO_VENDOR").map_or(true, |s| s == "0") {
            return find_vendored::get_openssl(target);
        }
    }
    find_normal::get_openssl(target)
}

fn main() {
    check_rustc_versions();

    let target = env::var("TARGET").unwrap();

    let (lib_dirs, include_dir) = find_openssl(&target);

    if !lib_dirs.iter().all(|p| Path::new(p).exists()) {
        panic!("OpenSSL library directory does not exist: {:?}", lib_dirs);
    }
    if !Path::new(&include_dir).exists() {
        panic!(
            "OpenSSL include directory does not exist: {}",
            include_dir.to_string_lossy()
        );
    }

    for lib_dir in lib_dirs.iter() {
        println!(
            "cargo:rustc-link-search=native={}",
            lib_dir.to_string_lossy()
        );
    }
    println!("cargo:include={}", include_dir.to_string_lossy());

    let version = postprocess(&[include_dir]);

    let libs_env = env("OPENSSL_LIBS");
    let libs = match libs_env.as_ref().and_then(|s| s.to_str()) {
        Some(v) => {
            if v.is_empty() {
                vec![]
            } else {
                v.split(':').collect()
            }
        }
        None => match version {
            Version::Openssl10x if target.contains("windows") => vec!["ssleay32", "libeay32"],
            Version::Openssl3xx | Version::Openssl11x if target.contains("windows-msvc") => {
                vec!["libssl", "libcrypto"]
            }
            _ => vec!["ssl", "crypto"],
        },
    };

    let kind = determine_mode(&lib_dirs, &libs);
    for lib in libs.into_iter() {
        println!("cargo:rustc-link-lib={}={}", kind, lib);
    }

    // https://github.com/openssl/openssl/pull/15086
    if version == Version::Openssl3xx
        && kind == "static"
        && (env::var("CARGO_CFG_TARGET_OS").unwrap() == "linux"
            || env::var("CARGO_CFG_TARGET_OS").unwrap() == "android")
        && env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap() == "32"
    {
        println!("cargo:rustc-link-lib=dylib=atomic");
    }

    if kind == "static" && target.contains("windows") {
        println!("cargo:rustc-link-lib=dylib=gdi32");
        println!("cargo:rustc-link-lib=dylib=user32");
        println!("cargo:rustc-link-lib=dylib=crypt32");
        println!("cargo:rustc-link-lib=dylib=ws2_32");
        println!("cargo:rustc-link-lib=dylib=advapi32");
    }
}

fn check_rustc_versions() {
    let cfg = autocfg::new();

    if cfg.probe_rustc_version(1, 31) {
        println!("cargo:rustc-cfg=const_fn");
    }
}

#[allow(clippy::let_and_return)]
fn postprocess(include_dirs: &[PathBuf]) -> Version {
    let version = validate_headers(include_dirs);
    #[cfg(feature = "bindgen")]
    run_bindgen::run(&include_dirs);

    version
}

/// Validates the header files found in `include_dir` and then returns the
/// version string of OpenSSL.
#[allow(clippy::manual_strip)] // we need to support pre-1.45.0
fn validate_headers(include_dirs: &[PathBuf]) -> Version {
    // This `*-sys` crate only works with OpenSSL 1.0.1, 1.0.2, 1.1.0, 1.1.1 and 3.0.0.
    // To correctly expose the right API from this crate, take a look at
    // `opensslv.h` to see what version OpenSSL claims to be.
    //
    // OpenSSL has a number of build-time configuration options which affect
    // various structs and such. Since OpenSSL 1.1.0 this isn't really a problem
    // as the library is much more FFI-friendly, but 1.0.{1,2} suffer this problem.
    //
    // To handle all this conditional compilation we slurp up the configuration
    // file of OpenSSL, `opensslconf.h`, and then dump out everything it defines
    // as our own #[cfg] directives. That way the `ossl10x.rs` bindings can
    // account for compile differences and such.
    println!("cargo:rerun-if-changed=build/expando.c");
    let mut gcc = cc::Build::new();
    for include_dir in include_dirs {
        gcc.include(include_dir);
    }
    let expanded = match gcc.file("build/expando.c").try_expand() {
        Ok(expanded) => expanded,
        Err(e) => {
            panic!(
                "
Header expansion error:
{:?}

Failed to find OpenSSL development headers.

You can try fixing this setting the `OPENSSL_DIR` environment variable
pointing to your OpenSSL installation or installing OpenSSL headers package
specific to your distribution:

    # On Ubuntu
    sudo apt-get install libssl-dev
    # On Arch Linux
    sudo pacman -S openssl
    # On Fedora
    sudo dnf install openssl-devel
    # On Alpine Linux
    apk add openssl-dev

See rust-openssl README for more information:

    https://github.com/sfackler/rust-openssl#linux
",
                e
            );
        }
    };
    let expanded = String::from_utf8(expanded).unwrap();

    let mut enabled = vec![];
    let mut openssl_version = None;
    let mut libressl_version = None;
    for line in expanded.lines() {
        let line = line.trim();

        let openssl_prefix = "RUST_VERSION_OPENSSL_";
        let new_openssl_prefix = "RUST_VERSION_NEW_OPENSSL_";
        let libressl_prefix = "RUST_VERSION_LIBRESSL_";
        let conf_prefix = "RUST_CONF_";
        if line.starts_with(openssl_prefix) {
            let version = &line[openssl_prefix.len()..];
            openssl_version = Some(parse_version(version));
        } else if line.starts_with(new_openssl_prefix) {
            let version = &line[new_openssl_prefix.len()..];
            openssl_version = Some(parse_new_version(version));
        } else if line.starts_with(libressl_prefix) {
            let version = &line[libressl_prefix.len()..];
            libressl_version = Some(parse_version(version));
        } else if line.starts_with(conf_prefix) {
            enabled.push(&line[conf_prefix.len()..]);
        }
    }

    for enabled in &enabled {
        println!("cargo:rustc-cfg=osslconf=\"{}\"", enabled);
    }
    println!("cargo:conf={}", enabled.join(","));

    for cfg in cfgs::get(openssl_version, libressl_version) {
        println!("cargo:rustc-cfg={}", cfg);
    }

    if let Some(libressl_version) = libressl_version {
        println!("cargo:libressl_version_number={:x}", libressl_version);

        let major = (libressl_version >> 28) as u8;
        let minor = (libressl_version >> 20) as u8;
        let fix = (libressl_version >> 12) as u8;
        let (major, minor, fix) = match (major, minor, fix) {
            (2, 5, 0) => ('2', '5', '0'),
            (2, 5, 1) => ('2', '5', '1'),
            (2, 5, 2) => ('2', '5', '2'),
            (2, 5, _) => ('2', '5', 'x'),
            (2, 6, 0) => ('2', '6', '0'),
            (2, 6, 1) => ('2', '6', '1'),
            (2, 6, 2) => ('2', '6', '2'),
            (2, 6, _) => ('2', '6', 'x'),
            (2, 7, _) => ('2', '7', 'x'),
            (2, 8, 0) => ('2', '8', '0'),
            (2, 8, 1) => ('2', '8', '1'),
            (2, 8, _) => ('2', '8', 'x'),
            (2, 9, 0) => ('2', '9', '0'),
            (2, 9, _) => ('2', '9', 'x'),
            (3, 0, 0) => ('3', '0', '0'),
            (3, 0, 1) => ('3', '0', '1'),
            (3, 0, _) => ('3', '0', 'x'),
            (3, 1, 0) => ('3', '1', '0'),
            (3, 1, _) => ('3', '1', 'x'),
            (3, 2, 0) => ('3', '2', '0'),
            (3, 2, 1) => ('3', '2', '1'),
            (3, 2, _) => ('3', '2', 'x'),
            (3, 3, 0) => ('3', '3', '0'),
            (3, 3, 1) => ('3', '3', '1'),
            (3, 3, _) => ('3', '3', 'x'),
            (3, 4, 0) => ('3', '4', '0'),
            (3, 4, _) => ('3', '4', 'x'),
            (3, 5, _) => ('3', '5', 'x'),
            _ => version_error(),
        };

        println!("cargo:libressl=true");
        println!("cargo:libressl_version={}{}{}", major, minor, fix);
        println!("cargo:version=101");
        Version::Libressl
    } else {
        let openssl_version = openssl_version.unwrap();
        println!("cargo:version_number={:x}", openssl_version);

        if openssl_version >= 0x4_00_00_00_0 {
            version_error()
        } else if openssl_version >= 0x3_00_00_00_0 {
            Version::Openssl3xx
        } else if openssl_version >= 0x1_01_01_00_0 {
            println!("cargo:version=111");
            Version::Openssl11x
        } else if openssl_version >= 0x1_01_00_06_0 {
            println!("cargo:version=110");
            println!("cargo:patch=f");
            Version::Openssl11x
        } else if openssl_version >= 0x1_01_00_00_0 {
            println!("cargo:version=110");
            Version::Openssl11x
        } else if openssl_version >= 0x1_00_02_00_0 {
            println!("cargo:version=102");
            Version::Openssl10x
        } else if openssl_version >= 0x1_00_01_00_0 {
            println!("cargo:version=101");
            Version::Openssl10x
        } else {
            version_error()
        }
    }
}

fn version_error() -> ! {
    panic!(
        "

This crate is only compatible with OpenSSL (version 1.0.1 through 1.1.1, or 3.0.0), or LibreSSL 2.5
through 3.5, but a different version of OpenSSL was found. The build is now aborting
due to this version mismatch.

"
    );
}

// parses a string that looks like "0x100020cfL"
#[allow(deprecated)] // trim_right_matches is now trim_end_matches
#[allow(clippy::match_like_matches_macro)] // matches macro requires rust 1.42.0
fn parse_version(version: &str) -> u64 {
    // cut off the 0x prefix
    assert!(version.starts_with("0x"));
    let version = &version[2..];

    // and the type specifier suffix
    let version = version.trim_right_matches(|c: char| match c {
        '0'..='9' | 'a'..='f' | 'A'..='F' => false,
        _ => true,
    });

    u64::from_str_radix(version, 16).unwrap()
}

// parses a string that looks like 3_0_0
fn parse_new_version(version: &str) -> u64 {
    println!("version: {}", version);
    let mut it = version.split('_');
    let major = it.next().unwrap().parse::<u64>().unwrap();
    let minor = it.next().unwrap().parse::<u64>().unwrap();
    let patch = it.next().unwrap().parse::<u64>().unwrap();

    (major << 28) | (minor << 20) | (patch << 4)
}

/// Given a libdir for OpenSSL (where artifacts are located) as well as the name
/// of the libraries we're linking to, figure out whether we should link them
/// statically or dynamically.
fn determine_mode(libdirs: &[PathBuf], libs: &[&str]) -> &'static str {
    // First see if a mode was explicitly requested
    let kind = env("OPENSSL_STATIC");
    match kind.as_ref().and_then(|s| s.to_str()) {
        Some("0") => return "dylib",
        Some(_) => return "static",
        None => {}
    }

    // Next, see what files we actually have to link against, and see what our
    // possibilities even are.
    let mut files = HashSet::new();
    for dir in libdirs {
        for path in dir
            .read_dir()
            .unwrap()
            .map(|e| e.unwrap())
            .map(|e| e.file_name())
            .filter_map(|e| e.into_string().ok())
        {
            files.insert(path);
        }
    }
    let can_static = libs
        .iter()
        .all(|l| files.contains(&format!("lib{}.a", l)) || files.contains(&format!("{}.lib", l)));
    let can_dylib = libs.iter().all(|l| {
        files.contains(&format!("lib{}.so", l))
            || files.contains(&format!("{}.dll", l))
            || files.contains(&format!("lib{}.dylib", l))
    });
    match (can_static, can_dylib) {
        (true, false) => return "static",
        (false, true) => return "dylib",
        (false, false) => {
            panic!(
                "OpenSSL libdir at `{:?}` does not contain the required files \
                 to either statically or dynamically link OpenSSL",
                libdirs
            );
        }
        (true, true) => {}
    }

    // Ok, we've got not explicit preference and can *either* link statically or
    // link dynamically. In the interest of "security upgrades" and/or "best
    // practices with security libs", let's link dynamically.
    "dylib"
}
