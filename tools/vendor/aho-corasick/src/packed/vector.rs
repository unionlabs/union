// This file contains a set of fairly generic utility functions when working
// with SIMD vectors.
//
// SAFETY: All of the routines below are unsafe to call because they assume
// the necessary CPU target features in order to use particular vendor
// intrinsics. Calling these routines when the underlying CPU does not support
// the appropriate target features is NOT safe. Callers must ensure this
// themselves.
//
// Note that it may not look like this safety invariant is being upheld when
// these routines are called. Namely, the CPU feature check is typically pretty
// far away from when these routines are used. Instead, we rely on the fact
// that certain types serve as a guaranteed receipt that pertinent target
// features are enabled. For example, the only way TeddySlim3Mask256 can be
// constructed is if the AVX2 CPU feature is available. Thus, any code running
// inside of TeddySlim3Mask256 can use any of the functions below without any
// additional checks: its very existence *is* the check.

use std::arch::x86_64::*;

/// Shift `a` to the left by two bytes (removing its two most significant
/// bytes), and concatenate it with the the two most significant bytes of `b`.
#[target_feature(enable = "avx2")]
pub unsafe fn alignr256_14(a: __m256i, b: __m256i) -> __m256i {
    // Credit goes to jneem for figuring this out:
    // https://github.com/jneem/teddy/blob/9ab5e899ad6ef6911aecd3cf1033f1abe6e1f66c/src/x86/teddy_simd.rs#L145-L184
    //
    // TL;DR avx2's PALIGNR instruction is actually just two 128-bit PALIGNR
    // instructions, which is not what we want, so we need to do some extra
    // shuffling.

    // This permute gives us the low 16 bytes of a concatenated with the high
    // 16 bytes of b, in order of most significant to least significant. So
    // `v = a[15:0] b[31:16]`.
    let v = _mm256_permute2x128_si256(b, a, 0x21);
    // This effectively does this (where we deal in terms of byte-indexing
    // and byte-shifting, and use inclusive ranges):
    //
    //   ret[15:0]  := ((a[15:0] << 16) | v[15:0]) >> 14
    //               = ((a[15:0] << 16) | b[31:16]) >> 14
    //   ret[31:16] := ((a[31:16] << 16) | v[31:16]) >> 14
    //               = ((a[31:16] << 16) | a[15:0]) >> 14
    //
    // Which therefore results in:
    //
    //   ret[31:0]  := a[29:16] a[15:14] a[13:0] b[31:30]
    //
    // The end result is that we've effectively done this:
    //
    //   (a << 2) | (b >> 30)
    //
    // When `A` and `B` are strings---where the beginning of the string is in
    // the least significant bits---we effectively result in the following
    // semantic operation:
    //
    //   (A >> 2) | (B << 30)
    //
    // The reversal being attributed to the fact that we are in little-endian.
    _mm256_alignr_epi8(a, v, 14)
}

/// Shift `a` to the left by one byte (removing its most significant byte), and
/// concatenate it with the the most significant byte of `b`.
#[target_feature(enable = "avx2")]
pub unsafe fn alignr256_15(a: __m256i, b: __m256i) -> __m256i {
    // For explanation, see alignr256_14.
    let v = _mm256_permute2x128_si256(b, a, 0x21);
    _mm256_alignr_epi8(a, v, 15)
}

/// Unpack the given 128-bit vector into its 64-bit components. The first
/// element of the array returned corresponds to the least significant 64-bit
/// lane in `a`.
#[target_feature(enable = "ssse3")]
pub unsafe fn unpack64x128(a: __m128i) -> [u64; 2] {
    [
        _mm_cvtsi128_si64(a) as u64,
        _mm_cvtsi128_si64(_mm_srli_si128(a, 8)) as u64,
    ]
}

/// Unpack the given 256-bit vector into its 64-bit components. The first
/// element of the array returned corresponds to the least significant 64-bit
/// lane in `a`.
#[target_feature(enable = "avx2")]
pub unsafe fn unpack64x256(a: __m256i) -> [u64; 4] {
    // Using transmute here is precisely equivalent, but actually slower. It's
    // not quite clear why.
    let lo = _mm256_extracti128_si256(a, 0);
    let hi = _mm256_extracti128_si256(a, 1);
    [
        _mm_cvtsi128_si64(lo) as u64,
        _mm_cvtsi128_si64(_mm_srli_si128(lo, 8)) as u64,
        _mm_cvtsi128_si64(hi) as u64,
        _mm_cvtsi128_si64(_mm_srli_si128(hi, 8)) as u64,
    ]
}

/// Unpack the low 128-bits of `a` and `b`, and return them as 4 64-bit
/// integers.
///
/// More precisely, if a = a4 a3 a2 a1 and b = b4 b3 b2 b1, where each element
/// is a 64-bit integer and a1/b1 correspond to the least significant 64 bits,
/// then the return value is `b2 b1 a2 a1`.
#[target_feature(enable = "avx2")]
pub unsafe fn unpacklo64x256(a: __m256i, b: __m256i) -> [u64; 4] {
    let lo = _mm256_castsi256_si128(a);
    let hi = _mm256_castsi256_si128(b);
    [
        _mm_cvtsi128_si64(lo) as u64,
        _mm_cvtsi128_si64(_mm_srli_si128(lo, 8)) as u64,
        _mm_cvtsi128_si64(hi) as u64,
        _mm_cvtsi128_si64(_mm_srli_si128(hi, 8)) as u64,
    ]
}

/// Returns true if and only if all bits in the given 128-bit vector are 0.
#[target_feature(enable = "ssse3")]
pub unsafe fn is_all_zeroes128(a: __m128i) -> bool {
    let cmp = _mm_cmpeq_epi8(a, zeroes128());
    _mm_movemask_epi8(cmp) as u32 == 0xFFFF
}

/// Returns true if and only if all bits in the given 256-bit vector are 0.
#[target_feature(enable = "avx2")]
pub unsafe fn is_all_zeroes256(a: __m256i) -> bool {
    let cmp = _mm256_cmpeq_epi8(a, zeroes256());
    _mm256_movemask_epi8(cmp) as u32 == 0xFFFFFFFF
}

/// Load a 128-bit vector from slice at the given position. The slice does
/// not need to be unaligned.
///
/// Since this code assumes little-endian (there is no big-endian x86), the
/// bytes starting in `slice[at..]` will be at the least significant bits of
/// the returned vector. This is important for the surrounding code, since for
/// example, shifting the resulting vector right is equivalent to logically
/// shifting the bytes in `slice` left.
#[target_feature(enable = "sse2")]
pub unsafe fn loadu128(slice: &[u8], at: usize) -> __m128i {
    let ptr = slice.get_unchecked(at..).as_ptr();
    _mm_loadu_si128(ptr as *const u8 as *const __m128i)
}

/// Load a 256-bit vector from slice at the given position. The slice does
/// not need to be unaligned.
///
/// Since this code assumes little-endian (there is no big-endian x86), the
/// bytes starting in `slice[at..]` will be at the least significant bits of
/// the returned vector. This is important for the surrounding code, since for
/// example, shifting the resulting vector right is equivalent to logically
/// shifting the bytes in `slice` left.
#[target_feature(enable = "avx2")]
pub unsafe fn loadu256(slice: &[u8], at: usize) -> __m256i {
    let ptr = slice.get_unchecked(at..).as_ptr();
    _mm256_loadu_si256(ptr as *const u8 as *const __m256i)
}

/// Returns a 128-bit vector with all bits set to 0.
#[target_feature(enable = "sse2")]
pub unsafe fn zeroes128() -> __m128i {
    _mm_set1_epi8(0)
}

/// Returns a 256-bit vector with all bits set to 0.
#[target_feature(enable = "avx2")]
pub unsafe fn zeroes256() -> __m256i {
    _mm256_set1_epi8(0)
}

/// Returns a 128-bit vector with all bits set to 1.
#[target_feature(enable = "sse2")]
pub unsafe fn ones128() -> __m128i {
    _mm_set1_epi8(0xFF as u8 as i8)
}

/// Returns a 256-bit vector with all bits set to 1.
#[target_feature(enable = "avx2")]
pub unsafe fn ones256() -> __m256i {
    _mm256_set1_epi8(0xFF as u8 as i8)
}
