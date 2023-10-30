use pkg_config;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

use super::env;

pub fn get_openssl(target: &str) -> (Vec<PathBuf>, PathBuf) {
    let lib_dir = env("OPENSSL_LIB_DIR").map(PathBuf::from);
    let include_dir = env("OPENSSL_INCLUDE_DIR").map(PathBuf::from);

    match (lib_dir, include_dir) {
        (Some(lib_dir), Some(include_dir)) => (vec![lib_dir], include_dir),
        (lib_dir, include_dir) => {
            let openssl_dir = env("OPENSSL_DIR").unwrap_or_else(|| find_openssl_dir(target));
            let openssl_dir = Path::new(&openssl_dir);
            let lib_dir = lib_dir.map(|d| vec![d]).unwrap_or_else(|| {
                let mut lib_dirs = vec![];
                // OpenSSL 3.0 now puts it's libraries in lib64/ by default,
                // check for both it and lib/.
                if openssl_dir.join("lib64").exists() {
                    lib_dirs.push(openssl_dir.join("lib64"));
                }
                if openssl_dir.join("lib").exists() {
                    lib_dirs.push(openssl_dir.join("lib"));
                }
                lib_dirs
            });
            let include_dir = include_dir.unwrap_or_else(|| openssl_dir.join("include"));
            (lib_dir, include_dir)
        }
    }
}

fn resolve_with_wellknown_homebrew_location(dir: &str) -> Option<PathBuf> {
    let versions = ["openssl@3", "openssl@1.1"];

    // Check up default aarch 64 Homebrew installation location first
    // for quick resolution if possible.
    //  `pkg-config` on brew doesn't necessarily contain settings for openssl apparently.
    for version in &versions {
        let homebrew = Path::new(dir).join(format!("opt/{}", version));
        if homebrew.exists() {
            return Some(homebrew);
        }
    }

    for version in &versions {
        // Calling `brew --prefix <package>` command usually slow and
        // takes seconds, and will be used only as a last resort.
        let output = execute_command_and_get_output("brew", &["--prefix", version]);
        if let Some(ref output) = output {
            let homebrew = Path::new(&output);
            if homebrew.exists() {
                return Some(homebrew.to_path_buf());
            }
        }
    }

    None
}

fn resolve_with_wellknown_location(dir: &str) -> Option<PathBuf> {
    let root_dir = Path::new(dir);
    let include_openssl = root_dir.join("include/openssl");
    if include_openssl.exists() {
        Some(root_dir.to_path_buf())
    } else {
        None
    }
}

fn find_openssl_dir(target: &str) -> OsString {
    let host = env::var("HOST").unwrap();

    if host == target && target.ends_with("-apple-darwin") {
        let homebrew_dir = match target {
            "aarch64-apple-darwin" => "/opt/homebrew",
            _ => "/usr/local",
        };

        if let Some(dir) = resolve_with_wellknown_homebrew_location(homebrew_dir) {
            return dir.into();
        } else if let Some(dir) = resolve_with_wellknown_location("/opt/pkg") {
            // pkgsrc
            return dir.into();
        } else if let Some(dir) = resolve_with_wellknown_location("/opt/local") {
            // MacPorts
            return dir.into();
        }
    }

    try_pkg_config();
    try_vcpkg();

    // FreeBSD ships with OpenSSL but doesn't include a pkg-config file :(
    if host == target && target.contains("freebsd") {
        return OsString::from("/usr");
    }

    // DragonFly has libressl (or openssl) in ports, but this doesn't include a pkg-config file
    if host == target && target.contains("dragonfly") {
        return OsString::from("/usr/local");
    }

    let mut msg = format!(
        "

Could not find directory of OpenSSL installation, and this `-sys` crate cannot
proceed without this knowledge. If OpenSSL is installed and this crate had
trouble finding it,  you can set the `OPENSSL_DIR` environment variable for the
compilation process.

Make sure you also have the development packages of openssl installed.
For example, `libssl-dev` on Ubuntu or `openssl-devel` on Fedora.

If you're in a situation where you think the directory *should* be found
automatically, please open a bug at https://github.com/sfackler/rust-openssl
and include information about your system as well as this message.

$HOST = {}
$TARGET = {}
openssl-sys = {}

",
        host,
        target,
        env!("CARGO_PKG_VERSION")
    );

    if host.contains("apple-darwin") && target.contains("apple-darwin") {
        let system = Path::new("/usr/lib/libssl.0.9.8.dylib");
        if system.exists() {
            msg.push_str(
                "

openssl-sys crate build failed: no supported version of OpenSSL found.

Ways to fix it:
- Use the `vendored` feature of openssl-sys crate to build OpenSSL from source.
- Use Homebrew to install the `openssl` package.

",
            );
        }
    }

    if host.contains("unknown-linux")
        && target.contains("unknown-linux-gnu")
        && Command::new("pkg-config").output().is_err()
    {
        msg.push_str(
            "
It looks like you're compiling on Linux and also targeting Linux. Currently this
requires the `pkg-config` utility to find OpenSSL but unfortunately `pkg-config`
could not be found. If you have OpenSSL installed you can likely fix this by
installing `pkg-config`.

",
        );
    }

    if host.contains("windows") && target.contains("windows-gnu") {
        msg.push_str(
            "
It looks like you're compiling for MinGW but you may not have either OpenSSL or
pkg-config installed. You can install these two dependencies with:

pacman -S openssl-devel pkg-config

and try building this crate again.

",
        );
    }

    if host.contains("windows") && target.contains("windows-msvc") {
        msg.push_str(
            "
It looks like you're compiling for MSVC but we couldn't detect an OpenSSL
installation. If there isn't one installed then you can try the rust-openssl
README for more information about how to download precompiled binaries of
OpenSSL:

https://github.com/sfackler/rust-openssl#windows

",
        );
    }

    panic!("{}", msg);
}

/// Attempt to find OpenSSL through pkg-config.
///
/// Note that if this succeeds then the function does not return as pkg-config
/// typically tells us all the information that we need.
fn try_pkg_config() {
    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();

    // If we're going to windows-gnu we can use pkg-config, but only so long as
    // we're coming from a windows host.
    //
    // Otherwise if we're going to windows we probably can't use pkg-config.
    if target.contains("windows-gnu") && host.contains("windows") {
        env::set_var("PKG_CONFIG_ALLOW_CROSS", "1");
    } else if target.contains("windows") {
        return;
    }

    let lib = match pkg_config::Config::new()
        .print_system_libs(false)
        .find("openssl")
    {
        Ok(lib) => lib,
        Err(e) => {
            println!("run pkg_config fail: {:?}", e);
            return;
        }
    };

    super::postprocess(&lib.include_paths);

    for include in lib.include_paths.iter() {
        println!("cargo:include={}", include.display());
    }

    process::exit(0);
}

/// Attempt to find OpenSSL through vcpkg.
///
/// Note that if this succeeds then the function does not return as vcpkg
/// should emit all of the cargo metadata that we need.
#[cfg(target_env = "msvc")]
fn try_vcpkg() {
    // vcpkg will not emit any metadata if it can not find libraries
    // appropriate for the target triple with the desired linkage.

    let lib = match vcpkg::Config::new()
        .emit_includes(true)
        .find_package("openssl")
    {
        Ok(lib) => lib,
        Err(e) => {
            println!("note: vcpkg did not find openssl: {}", e);
            return;
        }
    };

    super::postprocess(&lib.include_paths);

    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=gdi32");
    println!("cargo:rustc-link-lib=crypt32");

    process::exit(0);
}

#[cfg(not(target_env = "msvc"))]
fn try_vcpkg() {}

fn execute_command_and_get_output(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output();
    if let Ok(ref r1) = out {
        if r1.status.success() {
            let r2 = String::from_utf8(r1.stdout.clone());
            if let Ok(r3) = r2 {
                return Some(r3.trim().to_string());
            }
        }
    }

    None
}
