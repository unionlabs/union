//! SHA-256 `aarch64` backend.

// TODO: stdarch intrinsics: RustCrypto/hashes#257

cpufeatures::new!(sha2_hwcap, "sha2");

pub fn compress(state: &mut [u32; 8], blocks: &[[u8; 64]]) {
    // TODO: Replace with https://github.com/rust-lang/rfcs/pull/2725
    // after stabilization
    if sha2_hwcap::get() {
        sha2_asm::compress256(state, blocks);
    } else {
        super::soft::compress(state, blocks);
    }
}
