#![allow(unused_imports)]

extern crate cc;

use std::env;
use std::path::{Path, PathBuf};

#[cfg(target_env = "msvc")]
fn assembly(file_vec: &mut Vec<PathBuf>, base_dir: &Path, arch: &String) {
    let sfx = match arch.as_str() {
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
}

#[cfg(not(target_env = "msvc"))]
fn assembly(file_vec: &mut Vec<PathBuf>, base_dir: &Path, _: &String) {
    file_vec.push(base_dir.join("assembly.S"))
}

fn main() {
    // account for cross-compilation [by examining environment variable]
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    if target_arch.eq("wasm32") {
        println!("cargo:rustc-cfg=feature=\"no-threads\"");
    }

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
    #[cfg(target_env = "msvc")]
    if env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap().eq("32")
        && !env::var("CC").is_ok()
        && which::which("clang-cl").is_ok()
    {
        env::set_var("CC", "clang-cl");
    }

    let mut cc = cc::Build::new();

    let c_src_dir = blst_base_dir.join("src");
    println!("cargo:rerun-if-changed={}", c_src_dir.display());
    let mut file_vec = vec![c_src_dir.join("server.c")];

    if target_arch.eq("x86_64") || target_arch.eq("aarch64") {
        let asm_dir = blst_base_dir.join("build");
        println!("cargo:rerun-if-changed={}", asm_dir.display());
        assembly(&mut file_vec, &asm_dir, &target_arch);
    } else {
        cc.define("__BLST_NO_ASM__", None);
    }
    match (cfg!(feature = "portable"), cfg!(feature = "force-adx")) {
        (true, false) => {
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
            #[cfg(target_arch = "x86_64")]
            if target_arch.eq("x86_64") && std::is_x86_feature_detected!("adx")
            {
                println!("Enabling ADX because it was detected on the host");
                cc.define("__ADX__", None);
            }
        }
        (true, true) => panic!(
            "Cannot compile with both `portable` and `force-adx` features"
        ),
    }
    cc.flag_if_supported("-mno-avx") // avoid costly transitions
        .flag_if_supported("-fno-builtin")
        .flag_if_supported("-Wno-unused-function")
        .flag_if_supported("-Wno-unused-command-line-argument");
    if !cfg!(debug_assertions) {
        cc.opt_level(2);
    }
    cc.files(&file_vec).compile("libblst.a");

    // pass some DEP_BLST_* variables to dependents
    println!(
        "cargo:BINDINGS={}",
        blst_base_dir.join("bindings").to_string_lossy()
    );
    println!("cargo:C_SRC={}", c_src_dir.to_string_lossy());
}
