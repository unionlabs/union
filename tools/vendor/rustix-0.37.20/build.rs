#[cfg(feature = "cc")]
use cc::Build;
use std::env::var;
use std::io::Write;

/// The directory for out-of-line (“outline”) libraries.
const OUTLINE_PATH: &str = "src/backend/linux_raw/arch/outline";

fn main() {
    // Don't rerun this on changes other than build.rs, as we only depend on
    // the rustc version.
    println!("cargo:rerun-if-changed=build.rs");

    use_feature_or_nothing("rustc_attrs");

    // Features only used in no-std configurations.
    #[cfg(not(feature = "std"))]
    {
        use_feature_or_nothing("const_raw_ptr_deref");
        use_feature_or_nothing("core_ffi_c");
        use_feature_or_nothing("core_c_str");
        use_feature_or_nothing("alloc_c_string");
    }

    // Gather target information.
    let arch = var("CARGO_CFG_TARGET_ARCH").unwrap();
    let vendor = var("CARGO_CFG_TARGET_VENDOR").unwrap();
    let asm_name = format!("{}/{}.s", OUTLINE_PATH, arch);
    let asm_name_present = std::fs::metadata(&asm_name).is_ok();
    let target_os = var("CARGO_CFG_TARGET_OS").unwrap();
    let pointer_width = var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap();
    let endian = var("CARGO_CFG_TARGET_ENDIAN").unwrap();

    // Check for special target variants.
    let is_x32 = arch == "x86_64" && pointer_width == "32";
    let is_arm64_ilp32 = arch == "aarch64" && pointer_width == "32";
    let is_powerpc64be = arch == "powerpc64" && endian == "big";
    let is_mipseb = arch == "mips" && endian == "big";
    let is_mips64eb = arch == "mips64" && endian == "big";
    let is_unsupported_abi = is_x32 || is_arm64_ilp32 || is_powerpc64be || is_mipseb || is_mips64eb;

    // Check for `--features=use-libc`. This allows crate users to enable the
    // libc backend.
    let feature_use_libc = var("CARGO_FEATURE_USE_LIBC").is_ok();

    // Check for `--features=rustc-dep-of-std`. This is used when rustix is
    // being used to build std, in which case `can_compile` doesn't work
    // because `core` isn't available yet, but also, we can assume we have a
    // recent compiler.
    let feature_rustc_dep_of_std = var("CARGO_FEATURE_RUSTC_DEP_OF_STD").is_ok();

    // Check for `RUSTFLAGS=--cfg=rustix_use_libc`. This allows end users to
    // enable the libc backend even if rustix is depended on transitively.
    let cfg_use_libc = var("CARGO_CFG_RUSTIX_USE_LIBC").is_ok();

    // Check for eg. `RUSTFLAGS=--cfg=rustix_use_experimental_asm`. This is a
    // rustc flag rather than a cargo feature flag because it's experimental
    // and not something we want accidentally enabled via `--all-features`.
    let rustix_use_experimental_asm = var("CARGO_CFG_RUSTIX_USE_EXPERIMENTAL_ASM").is_ok();

    // Miri doesn't support inline asm, and has builtin support for recognizing
    // libc FFI calls, so if we're running under miri, use the libc backend.
    let miri = var("CARGO_CFG_MIRI").is_ok();

    // If the libc backend is requested, or if we're not on a platform for
    // which we have linux_raw support, use the libc backend.
    //
    // For now Android uses the libc backend; in theory it could use the
    // linux_raw backend, but to do that we'll need to figure out how to
    // install the toolchain for it.
    if feature_use_libc
        || cfg_use_libc
        || target_os != "linux"
        || !asm_name_present
        || is_unsupported_abi
        || miri
    {
        // Use the libc backend.
        use_feature("libc");
    } else {
        // Use the linux_raw backend.
        use_feature("linux_raw");
        use_feature_or_nothing("core_intrinsics");

        // Use inline asm if we have it, or outline asm otherwise. On 32-bit
        // x86 our asm support requires naked functions. On PowerPC and MIPS,
        // Rust's inline asm is considered experimental, so only use it if
        // `--cfg=rustix_use_experimental_asm` is given.
        if (feature_rustc_dep_of_std || vendor == "mustang" || can_compile("use std::arch::asm;"))
            && (arch != "x86" || has_feature("naked_functions"))
            && ((arch != "powerpc64" && arch != "mips" && arch != "mips64")
                || rustix_use_experimental_asm)
        {
            use_feature("asm");
            if arch == "x86" {
                use_feature("naked_functions");
            }
            if rustix_use_experimental_asm {
                use_feature("asm_experimental_arch");
            }
        } else {
            link_in_librustix_outline(&arch, &asm_name);
        }
    }

    // Detect whether the compiler requires us to use thumb mode on ARM.
    if arch == "arm" && use_thumb_mode() {
        use_feature("thumb_mode");
    }

    // Rust's libc crate groups some OS's together which have similar APIs;
    // create similarly-named features to make `cfg` tests more concise.
    if target_os == "freebsd" || target_os == "dragonfly" {
        use_feature("freebsdlike");
    }
    if target_os == "openbsd" || target_os == "netbsd" {
        use_feature("netbsdlike");
    }
    if target_os == "macos" || target_os == "ios" || target_os == "tvos" || target_os == "watchos" {
        use_feature("apple");
    }
    if target_os == "linux"
        || target_os == "l4re"
        || target_os == "android"
        || target_os == "emscripten"
    {
        use_feature("linux_like");
    }
    if target_os == "solaris" || target_os == "illumos" {
        use_feature("solarish");
    }
    if target_os == "macos"
        || target_os == "ios"
        || target_os == "tvos"
        || target_os == "watchos"
        || target_os == "freebsd"
        || target_os == "dragonfly"
        || target_os == "openbsd"
        || target_os == "netbsd"
    {
        use_feature("bsd");
    }

    // Add some additional common target combinations.
    if target_os == "android" || target_os == "linux" {
        use_feature("linux_kernel");
    }

    if target_os == "wasi" {
        use_feature_or_nothing("wasi_ext");
    }

    println!("cargo:rerun-if-env-changed=CARGO_CFG_RUSTIX_USE_EXPERIMENTAL_ASM");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_RUSTIX_USE_LIBC");

    // Rerun this script if any of our features or configuration flags change,
    // or if the toolchain we used for feature detection changes.
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_USE_LIBC");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_RUSTC_DEP_OF_STD");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_MIRI");
}

/// Link in the desired version of librustix_outline_{arch}.a, containing the
/// outline assembly code for making syscalls.
fn link_in_librustix_outline(arch: &str, asm_name: &str) {
    let name = format!("rustix_outline_{}", arch);
    let profile = var("PROFILE").unwrap();
    let to = format!("{}/{}/lib{}.a", OUTLINE_PATH, profile, name);
    println!("cargo:rerun-if-changed={}", to);

    // If "cc" is not enabled, use a pre-built library.
    #[cfg(not(feature = "cc"))]
    {
        let _ = asm_name;
        println!("cargo:rustc-link-search={}/{}", OUTLINE_PATH, profile);
        println!("cargo:rustc-link-lib=static={}", name);
    }

    // If "cc" is enabled, build the library from source, update the pre-built
    // version, and assert that the pre-built version is checked in.
    #[cfg(feature = "cc")]
    {
        let out_dir = var("OUT_DIR").unwrap();
        // Add `-gdwarf-3` so that we always get the same output, regardless of
        // the Rust version we're using. DWARF3 is the version used in
        // Rust 1.48 and is entirely adequate for our simple needs here.
        let mut build = Build::new();
        if profile == "debug" {
            build.flag("-gdwarf-3");
        }
        build.file(&asm_name);
        build.compile(&name);
        println!("cargo:rerun-if-changed={}", asm_name);
        if std::fs::metadata(".git").is_ok() {
            let from = format!("{}/lib{}.a", out_dir, name);
            let prev_metadata = std::fs::metadata(&to);
            std::fs::copy(&from, &to).unwrap();
            assert!(
                prev_metadata.is_ok(),
                "{} didn't previously exist; please inspect the new file and `git add` it",
                to
            );
            assert!(
                std::process::Command::new("git")
                    .arg("diff")
                    .arg("--quiet")
                    .arg(&to)
                    .status()
                    .unwrap()
                    .success(),
                "{} changed; please inspect the change and `git commit` it",
                to
            );
        }
    }
}

fn use_thumb_mode() -> bool {
    // In thumb mode, r7 is reserved.
    !can_compile("pub unsafe fn f() { core::arch::asm!(\"udf #16\", in(\"r7\") 0); }")
}

fn use_feature_or_nothing(feature: &str) {
    if has_feature(feature) {
        use_feature(feature);
    }
}

fn use_feature(feature: &str) {
    println!("cargo:rustc-cfg={}", feature);
}

/// Test whether the rustc at `var("RUSTC")` supports the given feature.
fn has_feature(feature: &str) -> bool {
    can_compile(format!(
        "#![allow(stable_features)]\n#![feature({})]",
        feature
    ))
}

/// Test whether the rustc at `var("RUSTC")` can compile the given code.
fn can_compile<T: AsRef<str>>(test: T) -> bool {
    use std::process::Stdio;

    let out_dir = var("OUT_DIR").unwrap();
    let rustc = var("RUSTC").unwrap();
    let target = var("TARGET").unwrap();

    // Use `RUSTC_WRAPPER` if it's set, unless it's set to an empty string,
    // as documented [here].
    // [here]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-reads
    let wrapper = var("RUSTC_WRAPPER")
        .ok()
        .and_then(|w| if w.is_empty() { None } else { Some(w) });

    let mut cmd = if let Some(wrapper) = wrapper {
        let mut cmd = std::process::Command::new(wrapper);
        // The wrapper's first argument is supposed to be the path to rustc.
        cmd.arg(rustc);
        cmd
    } else {
        std::process::Command::new(rustc)
    };

    cmd.arg("--crate-type=rlib") // Don't require `main`.
        .arg("--emit=metadata") // Do as little as possible but still parse.
        .arg("--target")
        .arg(target)
        .arg("--out-dir")
        .arg(out_dir); // Put the output somewhere inconsequential.

    // If Cargo wants to set RUSTFLAGS, use that.
    if let Ok(rustflags) = var("CARGO_ENCODED_RUSTFLAGS") {
        if !rustflags.is_empty() {
            for arg in rustflags.split('\x1f') {
                cmd.arg(arg);
            }
        }
    }

    let mut child = cmd
        .arg("-") // Read from stdin.
        .stdin(Stdio::piped()) // Stdin is a pipe.
        .stderr(Stdio::null()) // Errors from feature detection aren't interesting and can be confusing.
        .spawn()
        .unwrap();

    writeln!(child.stdin.take().unwrap(), "{}", test.as_ref()).unwrap();

    child.wait().unwrap().success()
}
