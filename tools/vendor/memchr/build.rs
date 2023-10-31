use std::env;

fn main() {
    enable_simd_optimizations();
    enable_libc();
}

// This adds various simd cfgs if this compiler and target support it.
//
// This can be disabled with RUSTFLAGS="--cfg memchr_disable_auto_simd", but
// this is generally only intended for testing.
//
// On targets which don't feature SSE2, this is disabled, as LLVM wouln't know
// how to work with SSE2 operands. Enabling SSE4.2 and AVX on SSE2-only targets
// is not a problem. In that case, the fastest option will be chosen at
// runtime.
fn enable_simd_optimizations() {
    if is_env_set("CARGO_CFG_MEMCHR_DISABLE_AUTO_SIMD") {
        return;
    }
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    match &arch[..] {
        "x86_64" => {
            if !target_has_feature("sse2") {
                return;
            }
            println!("cargo:rustc-cfg=memchr_runtime_simd");
            println!("cargo:rustc-cfg=memchr_runtime_sse2");
            println!("cargo:rustc-cfg=memchr_runtime_sse42");
            println!("cargo:rustc-cfg=memchr_runtime_avx");
        }
        "wasm32" | "wasm64" => {
            if !target_has_feature("simd128") {
                return;
            }
            println!("cargo:rustc-cfg=memchr_runtime_simd");
            println!("cargo:rustc-cfg=memchr_runtime_wasm128");
        }
        _ => {}
    }
}

// This adds a `memchr_libc` cfg if and only if libc can be used, if no other
// better option is available.
//
// This could be performed in the source code, but it's simpler to do it once
// here and consolidate it into one cfg knob.
//
// Basically, we use libc only if its enabled and if we aren't targeting a
// known bad platform. For example, wasm32 doesn't have a libc and the
// performance of memchr on Windows is seemingly worse than the fallback
// implementation.
fn enable_libc() {
    const NO_ARCH: &'static [&'static str] = &["wasm32", "windows"];
    const NO_ENV: &'static [&'static str] = &["sgx"];

    if !is_feature_set("LIBC") {
        return;
    }

    let arch = match env::var("CARGO_CFG_TARGET_ARCH") {
        Err(_) => return,
        Ok(arch) => arch,
    };
    let env = match env::var("CARGO_CFG_TARGET_ENV") {
        Err(_) => return,
        Ok(env) => env,
    };
    if NO_ARCH.contains(&&*arch) || NO_ENV.contains(&&*env) {
        return;
    }

    println!("cargo:rustc-cfg=memchr_libc");
}

fn is_feature_set(name: &str) -> bool {
    is_env_set(&format!("CARGO_FEATURE_{}", name))
}

fn is_env_set(name: &str) -> bool {
    env::var_os(name).is_some()
}

fn target_has_feature(feature: &str) -> bool {
    env::var("CARGO_CFG_TARGET_FEATURE")
        .map(|features| features.contains(feature))
        .unwrap_or(false)
}
