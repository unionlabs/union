fn main() {
    // This does not enable `RUSTC_BOOTSTRAP=1` for `packed_simd`.
    // You still need to knowingly have a setup that makes
    // `packed_simd` compile. Therefore, having this file on
    // crates.io is harmless in terms of users of `encoding_rs`
    // accidentally depending on nightly features. Having this
    // here means that if you knowingly want this, you only
    // need to maintain a fork of `packed_simd` without _also_
    // having to maintain a fork of `encoding_rs`.
    #[cfg(feature = "simd-accel")]
    println!("cargo:rustc-env=RUSTC_BOOTSTRAP=1");
}
