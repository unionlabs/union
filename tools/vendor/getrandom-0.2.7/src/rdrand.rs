// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for SGX using RDRAND instruction
use crate::Error;
use core::mem;

cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        use core::arch::x86_64 as arch;
        use arch::_rdrand64_step as rdrand_step;
    } else if #[cfg(target_arch = "x86")] {
        use core::arch::x86 as arch;
        use arch::_rdrand32_step as rdrand_step;
    }
}

// Recommendation from "Intel® Digital Random Number Generator (DRNG) Software
// Implementation Guide" - Section 5.2.1 and "Intel® 64 and IA-32 Architectures
// Software Developer’s Manual" - Volume 1 - Section 7.3.17.1.
const RETRY_LIMIT: usize = 10;
const WORD_SIZE: usize = mem::size_of::<usize>();

#[target_feature(enable = "rdrand")]
unsafe fn rdrand() -> Result<[u8; WORD_SIZE], Error> {
    for _ in 0..RETRY_LIMIT {
        let mut el = mem::zeroed();
        if rdrand_step(&mut el) == 1 {
            // AMD CPUs from families 14h to 16h (pre Ryzen) sometimes fail to
            // set CF on bogus random data, so we check these values explicitly.
            // See https://github.com/systemd/systemd/issues/11810#issuecomment-489727505
            // We perform this check regardless of target to guard against
            // any implementation that incorrectly fails to set CF.
            if el != 0 && el != !0 {
                return Ok(el.to_ne_bytes());
            }
            // Keep looping in case this was a false positive.
        }
    }
    Err(Error::FAILED_RDRAND)
}

// "rdrand" target feature requires "+rdrnd" flag, see https://github.com/rust-lang/rust/issues/49653.
#[cfg(all(target_env = "sgx", not(target_feature = "rdrand")))]
compile_error!(
    "SGX targets require 'rdrand' target feature. Enable by using -C target-feature=+rdrnd."
);

#[cfg(target_feature = "rdrand")]
fn is_rdrand_supported() -> bool {
    true
}

// TODO use is_x86_feature_detected!("rdrand") when that works in core. See:
// https://github.com/rust-lang-nursery/stdsimd/issues/464
#[cfg(not(target_feature = "rdrand"))]
fn is_rdrand_supported() -> bool {
    use crate::util::LazyBool;

    // SAFETY: All Rust x86 targets are new enough to have CPUID, and if CPUID
    // is supported, CPUID leaf 1 is always supported.
    const FLAG: u32 = 1 << 30;
    static HAS_RDRAND: LazyBool = LazyBool::new();
    HAS_RDRAND.unsync_init(|| unsafe { (arch::__cpuid(1).ecx & FLAG) != 0 })
}

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    if !is_rdrand_supported() {
        return Err(Error::NO_RDRAND);
    }

    // SAFETY: After this point, rdrand is supported, so calling the rdrand
    // functions is not undefined behavior.
    unsafe { rdrand_exact(dest) }
}

#[target_feature(enable = "rdrand")]
unsafe fn rdrand_exact(dest: &mut [u8]) -> Result<(), Error> {
    // We use chunks_exact_mut instead of chunks_mut as it allows almost all
    // calls to memcpy to be elided by the compiler.
    let mut chunks = dest.chunks_exact_mut(WORD_SIZE);
    for chunk in chunks.by_ref() {
        chunk.copy_from_slice(&rdrand()?);
    }

    let tail = chunks.into_remainder();
    let n = tail.len();
    if n > 0 {
        tail.copy_from_slice(&rdrand()?[..n]);
    }
    Ok(())
}
