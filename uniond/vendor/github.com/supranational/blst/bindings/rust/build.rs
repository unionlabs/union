#![allow(unused_imports)]

extern crate cc;

use std::env;
use std::path::{Path, PathBuf};

fn assembly(
    file_vec: &mut Vec<PathBuf>,
    base_dir: &Path,
    _arch: &str,
    _is_msvc: bool,
) {
    #[cfg(target_env = "msvc")]
    if _is_msvc {
        let sfx = match _arch {
            "x86_64" => "x86_64",
            "aarch64" => "armv8",
            _ => "unknown",
        };
        let files =
            glob::glob(&format!("{}/win64/*-{}.asm", base_dir.display(), sfx))
                .expect("unable to collect assembly files");
        for file in files {
            file_vec.push(file.unwrap());
        }
        return;
    }

    file_vec.push(base_dir.join("assembly.S"));
}

fn main() {
    if env::var("CARGO_FEATURE_SERDE_SECRET").is_ok() {
        println!(
            "cargo:warning=blst: non-production feature serde-secret enabled"
        );
    }

    // account for cross-compilation [by examining environment variables]
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_family = env::var("CARGO_CFG_TARGET_FAMILY").unwrap_or_default();

    let target_no_std = target_os.eq("none")
        || (target_os.eq("unknown") && target_arch.eq("wasm32"))
        || target_os.eq("uefi")
        || env::var("BLST_TEST_NO_STD").is_ok();

    if !target_no_std {
        println!("cargo:rustc-cfg=feature=\"std\"");
        if target_arch.eq("wasm32") || target_os.eq("unknown") {
            println!("cargo:rustc-cfg=feature=\"no-threads\"");
        }
    }
    println!("cargo:rerun-if-env-changed=BLST_TEST_NO_STD");

    /*
     * Use pre-built libblst.a if there is one. This is primarily
     * for trouble-shooting purposes. Idea is that libblst.a can be
     * compiled with flags independent from cargo defaults, e.g.
     * '../../build.sh -O1 ...'.
     */
    if Path::new("libblst.a").exists() {
        println!("cargo:rustc-link-search=.");
        println!("cargo:rustc-link-lib=blst");
        println!("cargo:rerun-if-changed=libblst.a");
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let mut blst_base_dir = manifest_dir.join("blst");
    if !blst_base_dir.exists() {
        // Reach out to ../.., which is the root of the blst repo.
        // Use an absolute path to avoid issues with relative paths
        // being treated as strings by `cc` and getting concatenated
        // in ways that reach out of the OUT_DIR.
        blst_base_dir = manifest_dir
            .parent()
            .and_then(|dir| dir.parent())
            .expect("can't access parent of parent of current directory")
            .into();
    }
    println!("Using blst source directory {}", blst_base_dir.display());

    // Set CC environment variable to choose alternative C compiler.
    // Optimization level depends on whether or not --release is passed
    // or implied.

    if target_os.eq("uefi") && env::var("CC").is_err() {
        match std::process::Command::new("clang")
            .arg("--version")
            .output()
        {
            Ok(_) => env::set_var("CC", "clang"),
            Err(_) => { /* no clang in sight, just ignore the error */ }
        }
    }

    if target_env.eq("sgx") && env::var("CC").is_err() {
        match std::process::Command::new("clang")
            .arg("--version")
            .output()
        {
            Ok(out) => {
                let version = String::from_utf8(out.stdout)
                    .unwrap_or("unintelligible".to_string());
                if let Some(x) = version.find("clang version ") {
                    let x = x + 14;
                    let y = version[x..].find('.').unwrap_or(0);
                    if version[x..x + y].parse::<i32>().unwrap_or(0) >= 11 {
                        env::set_var("CC", "clang");
                    }
                }
            }
            Err(_) => { /* no clang in sight, just ignore the error */ }
        }
    }

    if target_env.eq("msvc")
        && env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap().eq("32")
        && env::var("CC").is_err()
    {
        match std::process::Command::new("clang-cl")
            .args(["-m32", "--version"])
            .output()
        {
            Ok(out) => {
                if String::from_utf8(out.stdout)
                    .unwrap_or("unintelligible".to_string())
                    .contains("Target: i386-pc-windows-msvc")
                {
                    env::set_var("CC", "clang-cl");
                }
            }
            Err(_) => { /* no clang-cl in sight, just ignore the error */ }
        }
    }

    let mut cc = cc::Build::new();

    let c_src_dir = blst_base_dir.join("src");
    println!("cargo:rerun-if-changed={}", c_src_dir.display());
    let mut file_vec = vec![c_src_dir.join("server.c")];

    if target_arch.eq("x86_64") || target_arch.eq("aarch64") {
        let asm_dir = blst_base_dir.join("build");
        println!("cargo:rerun-if-changed={}", asm_dir.display());
        assembly(
            &mut file_vec,
            &asm_dir,
            &target_arch,
            cc.get_compiler().is_like_msvc(),
        );
    } else {
        cc.define("__BLST_NO_ASM__", None);
    }
    match (cfg!(feature = "portable"), cfg!(feature = "force-adx")) {
        (true, false) => {
            if target_arch.eq("x86_64") && target_env.eq("sgx") {
                panic!("'portable' is not supported on SGX target");
            }
            println!("Compiling in portable mode without ISA extensions");
            cc.define("__BLST_PORTABLE__", None);
        }
        (false, true) => {
            if target_arch.eq("x86_64") {
                println!("Enabling ADX support via `force-adx` feature");
                cc.define("__ADX__", None);
            } else {
                println!("`force-adx` is ignored for non-x86_64 targets");
            }
        }
        (false, false) => {
            if target_arch.eq("x86_64") {
                if target_env.eq("sgx") {
                    println!("Enabling ADX for Intel SGX target");
                    cc.define("__ADX__", None);
                } else if env::var("CARGO_ENCODED_RUSTFLAGS")
                    .unwrap_or_default()
                    .contains("target-cpu=")
                {
                    // If target-cpu is specified on the rustc command line,
                    // then obey the resulting target-features.
                    let feat_list = env::var("CARGO_CFG_TARGET_FEATURE")
                        .unwrap_or_default();
                    let features: Vec<_> = feat_list.split(',').collect();
                    if !features.contains(&"ssse3") {
                        println!(
                            "Compiling in portable mode without ISA extensions"
                        );
                        cc.define("__BLST_PORTABLE__", None);
                    } else if features.contains(&"adx") {
                        println!(
                            "Enabling ADX because it was set as target-feature"
                        );
                        cc.define("__ADX__", None);
                    }
                } else {
                    #[cfg(target_arch = "x86_64")]
                    if std::is_x86_feature_detected!("adx") {
                        println!(
                            "Enabling ADX because it was detected on the host"
                        );
                        cc.define("__ADX__", None);
                    }
                }
            }
        }
        (true, true) => panic!(
            "Cannot compile with both `portable` and `force-adx` features"
        ),
    }
    if target_env.eq("msvc") && cc.get_compiler().is_like_msvc() {
        cc.flag("-Zl");
    }
    cc.flag_if_supported("-mno-avx") // avoid costly transitions
        .flag_if_supported("-fno-builtin")
        .flag_if_supported("-Wno-unused-function")
        .flag_if_supported("-Wno-unused-command-line-argument");
    if target_arch.eq("wasm32") || target_family.is_empty() {
        cc.flag("-ffreestanding");
    }
    if target_arch.eq("wasm32") || target_no_std {
        cc.define("SCRATCH_LIMIT", "(45 * 1024)");
    }
    if target_env.eq("sgx") {
        cc.flag_if_supported("-mlvi-hardening");
        cc.define("__SGX_LVI_HARDENING__", None);
        cc.define("__BLST_NO_CPUID__", None);
        cc.define("__ELF__", None);
        cc.define("SCRATCH_LIMIT", "(45 * 1024)");
    }
    if !cfg!(debug_assertions) {
        cc.opt_level(2);
    }
    cc.files(&file_vec).compile("blst");

    // pass some DEP_BLST_* variables to dependents
    println!(
        "cargo:BINDINGS={}",
        blst_base_dir.join("bindings").to_string_lossy()
    );
    println!("cargo:C_SRC={}", c_src_dir.to_string_lossy());
}
