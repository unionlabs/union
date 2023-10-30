#![allow(dead_code, unused_imports)]

macro_rules! convert_fn {
    (fn $name:ident($var:ident : $vartype:ty) -> $restype:ty {
            if feature("f16c") { $f16c:expr }
            else { $fallback:expr }}) => {
        #[inline]
        pub(crate) fn $name($var: $vartype) -> $restype {
            // Use CPU feature detection if using std
            #[cfg(all(
                feature = "use-intrinsics",
                feature = "std",
                any(target_arch = "x86", target_arch = "x86_64"),
                not(target_feature = "f16c")
            ))]
            {
                if is_x86_feature_detected!("f16c") {
                    $f16c
                } else {
                    $fallback
                }
            }
            // Use intrinsics directly when a compile target or using no_std
            #[cfg(all(
                feature = "use-intrinsics",
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "f16c"
            ))]
            {
                $f16c
            }
            // Fallback to software
            #[cfg(any(
                not(feature = "use-intrinsics"),
                not(any(target_arch = "x86", target_arch = "x86_64")),
                all(not(feature = "std"), not(target_feature = "f16c"))
            ))]
            {
                $fallback
            }
        }
    };
}

convert_fn! {
    fn f32_to_f16(f: f32) -> u16 {
        if feature("f16c") {
            unsafe { x86::f32_to_f16_x86_f16c(f) }
        } else {
            f32_to_f16_fallback(f)
        }
    }
}

convert_fn! {
    fn f64_to_f16(f: f64) -> u16 {
        if feature("f16c") {
            unsafe { x86::f32_to_f16_x86_f16c(f as f32) }
        } else {
            f64_to_f16_fallback(f)
        }
    }
}

convert_fn! {
    fn f16_to_f32(i: u16) -> f32 {
        if feature("f16c") {
            unsafe { x86::f16_to_f32_x86_f16c(i) }
        } else {
            f16_to_f32_fallback(i)
        }
    }
}

convert_fn! {
    fn f16_to_f64(i: u16) -> f64 {
        if feature("f16c") {
            unsafe { x86::f16_to_f32_x86_f16c(i) as f64 }
        } else {
            f16_to_f64_fallback(i)
        }
    }
}

// TODO: While SIMD versions are faster, further improvements can be made by doing runtime feature
// detection once at beginning of convert slice method, rather than per chunk

convert_fn! {
    fn f32x4_to_f16x4(f: &[f32]) -> [u16; 4] {
        if feature("f16c") {
            unsafe { x86::f32x4_to_f16x4_x86_f16c(f) }
        } else {
            f32x4_to_f16x4_fallback(f)
        }
    }
}

convert_fn! {
    fn f16x4_to_f32x4(i: &[u16]) -> [f32; 4] {
        if feature("f16c") {
            unsafe { x86::f16x4_to_f32x4_x86_f16c(i) }
        } else {
            f16x4_to_f32x4_fallback(i)
        }
    }
}

convert_fn! {
    fn f64x4_to_f16x4(f: &[f64]) -> [u16; 4] {
        if feature("f16c") {
            unsafe { x86::f64x4_to_f16x4_x86_f16c(f) }
        } else {
            f64x4_to_f16x4_fallback(f)
        }
    }
}

convert_fn! {
    fn f16x4_to_f64x4(i: &[u16]) -> [f64; 4] {
        if feature("f16c") {
            unsafe { x86::f16x4_to_f64x4_x86_f16c(i) }
        } else {
            f16x4_to_f64x4_fallback(i)
        }
    }
}

/////////////// Fallbacks ////////////////

// In the below functions, round to nearest, with ties to even.
// Let us call the most significant bit that will be shifted out the round_bit.
//
// Round up if either
//  a) Removed part > tie.
//     (mantissa & round_bit) != 0 && (mantissa & (round_bit - 1)) != 0
//  b) Removed part == tie, and retained part is odd.
//     (mantissa & round_bit) != 0 && (mantissa & (2 * round_bit)) != 0
// (If removed part == tie and retained part is even, do not round up.)
// These two conditions can be combined into one:
//     (mantissa & round_bit) != 0 && (mantissa & ((round_bit - 1) | (2 * round_bit))) != 0
// which can be simplified into
//     (mantissa & round_bit) != 0 && (mantissa & (3 * round_bit - 1)) != 0

fn f32_to_f16_fallback(value: f32) -> u16 {
    // Convert to raw bytes
    let x = value.to_bits();

    // Extract IEEE754 components
    let sign = x & 0x8000_0000u32;
    let exp = x & 0x7F80_0000u32;
    let man = x & 0x007F_FFFFu32;

    // Check for all exponent bits being set, which is Infinity or NaN
    if exp == 0x7F80_0000u32 {
        // Set mantissa MSB for NaN (and also keep shifted mantissa bits)
        let nan_bit = if man == 0 { 0 } else { 0x0200u32 };
        return ((sign >> 16) | 0x7C00u32 | nan_bit | (man >> 13)) as u16;
    }

    // The number is normalized, start assembling half precision version
    let half_sign = sign >> 16;
    // Unbias the exponent, then bias for half precision
    let unbiased_exp = ((exp >> 23) as i32) - 127;
    let half_exp = unbiased_exp + 15;

    // Check for exponent overflow, return +infinity
    if half_exp >= 0x1F {
        return (half_sign | 0x7C00u32) as u16;
    }

    // Check for underflow
    if half_exp <= 0 {
        // Check mantissa for what we can do
        if 14 - half_exp > 24 {
            // No rounding possibility, so this is a full underflow, return signed zero
            return half_sign as u16;
        }
        // Don't forget about hidden leading mantissa bit when assembling mantissa
        let man = man | 0x0080_0000u32;
        let mut half_man = man >> (14 - half_exp);
        // Check for rounding (see comment above functions)
        let round_bit = 1 << (13 - half_exp);
        if (man & round_bit) != 0 && (man & (3 * round_bit - 1)) != 0 {
            half_man += 1;
        }
        // No exponent for subnormals
        return (half_sign | half_man) as u16;
    }

    // Rebias the exponent
    let half_exp = (half_exp as u32) << 10;
    let half_man = man >> 13;
    // Check for rounding (see comment above functions)
    let round_bit = 0x0000_1000u32;
    if (man & round_bit) != 0 && (man & (3 * round_bit - 1)) != 0 {
        // Round it
        ((half_sign | half_exp | half_man) + 1) as u16
    } else {
        (half_sign | half_exp | half_man) as u16
    }
}

fn f64_to_f16_fallback(value: f64) -> u16 {
    // Convert to raw bytes, truncating the last 32-bits of mantissa; that precision will always
    // be lost on half-precision.
    let val = value.to_bits();
    let x = (val >> 32) as u32;

    // Extract IEEE754 components
    let sign = x & 0x8000_0000u32;
    let exp = x & 0x7FF0_0000u32;
    let man = x & 0x000F_FFFFu32;

    // Check for all exponent bits being set, which is Infinity or NaN
    if exp == 0x7FF0_0000u32 {
        // Set mantissa MSB for NaN (and also keep shifted mantissa bits).
        // We also have to check the last 32 bits.
        let nan_bit = if man == 0 && (val as u32 == 0) {
            0
        } else {
            0x0200u32
        };
        return ((sign >> 16) | 0x7C00u32 | nan_bit | (man >> 10)) as u16;
    }

    // The number is normalized, start assembling half precision version
    let half_sign = sign >> 16;
    // Unbias the exponent, then bias for half precision
    let unbiased_exp = ((exp >> 20) as i64) - 1023;
    let half_exp = unbiased_exp + 15;

    // Check for exponent overflow, return +infinity
    if half_exp >= 0x1F {
        return (half_sign | 0x7C00u32) as u16;
    }

    // Check for underflow
    if half_exp <= 0 {
        // Check mantissa for what we can do
        if 10 - half_exp > 21 {
            // No rounding possibility, so this is a full underflow, return signed zero
            return half_sign as u16;
        }
        // Don't forget about hidden leading mantissa bit when assembling mantissa
        let man = man | 0x0010_0000u32;
        let mut half_man = man >> (11 - half_exp);
        // Check for rounding (see comment above functions)
        let round_bit = 1 << (10 - half_exp);
        if (man & round_bit) != 0 && (man & (3 * round_bit - 1)) != 0 {
            half_man += 1;
        }
        // No exponent for subnormals
        return (half_sign | half_man) as u16;
    }

    // Rebias the exponent
    let half_exp = (half_exp as u32) << 10;
    let half_man = man >> 10;
    // Check for rounding (see comment above functions)
    let round_bit = 0x0000_0200u32;
    if (man & round_bit) != 0 && (man & (3 * round_bit - 1)) != 0 {
        // Round it
        ((half_sign | half_exp | half_man) + 1) as u16
    } else {
        (half_sign | half_exp | half_man) as u16
    }
}

fn f16_to_f32_fallback(i: u16) -> f32 {
    // Check for signed zero
    if i & 0x7FFFu16 == 0 {
        return f32::from_bits((i as u32) << 16);
    }

    let half_sign = (i & 0x8000u16) as u32;
    let half_exp = (i & 0x7C00u16) as u32;
    let half_man = (i & 0x03FFu16) as u32;

    // Check for an infinity or NaN when all exponent bits set
    if half_exp == 0x7C00u32 {
        // Check for signed infinity if mantissa is zero
        if half_man == 0 {
            return f32::from_bits((half_sign << 16) | 0x7F80_0000u32);
        } else {
            // NaN, keep current mantissa but also set most significiant mantissa bit
            return f32::from_bits((half_sign << 16) | 0x7FC0_0000u32 | (half_man << 13));
        }
    }

    // Calculate single-precision components with adjusted exponent
    let sign = half_sign << 16;
    // Unbias exponent
    let unbiased_exp = ((half_exp as i32) >> 10) - 15;

    // Check for subnormals, which will be normalized by adjusting exponent
    if half_exp == 0 {
        // Calculate how much to adjust the exponent by
        let e = (half_man as u16).leading_zeros() - 6;

        // Rebias and adjust exponent
        let exp = (127 - 15 - e) << 23;
        let man = (half_man << (14 + e)) & 0x7F_FF_FFu32;
        return f32::from_bits(sign | exp | man);
    }

    // Rebias exponent for a normalized normal
    let exp = ((unbiased_exp + 127) as u32) << 23;
    let man = (half_man & 0x03FFu32) << 13;
    f32::from_bits(sign | exp | man)
}

fn f16_to_f64_fallback(i: u16) -> f64 {
    // Check for signed zero
    if i & 0x7FFFu16 == 0 {
        return f64::from_bits((i as u64) << 48);
    }

    let half_sign = (i & 0x8000u16) as u64;
    let half_exp = (i & 0x7C00u16) as u64;
    let half_man = (i & 0x03FFu16) as u64;

    // Check for an infinity or NaN when all exponent bits set
    if half_exp == 0x7C00u64 {
        // Check for signed infinity if mantissa is zero
        if half_man == 0 {
            return f64::from_bits((half_sign << 48) | 0x7FF0_0000_0000_0000u64);
        } else {
            // NaN, keep current mantissa but also set most significiant mantissa bit
            return f64::from_bits((half_sign << 48) | 0x7FF8_0000_0000_0000u64 | (half_man << 42));
        }
    }

    // Calculate double-precision components with adjusted exponent
    let sign = half_sign << 48;
    // Unbias exponent
    let unbiased_exp = ((half_exp as i64) >> 10) - 15;

    // Check for subnormals, which will be normalized by adjusting exponent
    if half_exp == 0 {
        // Calculate how much to adjust the exponent by
        let e = (half_man as u16).leading_zeros() - 6;

        // Rebias and adjust exponent
        let exp = ((1023 - 15 - e) as u64) << 52;
        let man = (half_man << (43 + e)) & 0xF_FFFF_FFFF_FFFFu64;
        return f64::from_bits(sign | exp | man);
    }

    // Rebias exponent for a normalized normal
    let exp = ((unbiased_exp + 1023) as u64) << 52;
    let man = (half_man & 0x03FFu64) << 42;
    f64::from_bits(sign | exp | man)
}

#[inline]
fn f16x4_to_f32x4_fallback(v: &[u16]) -> [f32; 4] {
    debug_assert!(v.len() >= 4);

    [
        f16_to_f32_fallback(v[0]),
        f16_to_f32_fallback(v[1]),
        f16_to_f32_fallback(v[2]),
        f16_to_f32_fallback(v[3]),
    ]
}

#[inline]
fn f32x4_to_f16x4_fallback(v: &[f32]) -> [u16; 4] {
    debug_assert!(v.len() >= 4);

    [
        f32_to_f16_fallback(v[0]),
        f32_to_f16_fallback(v[1]),
        f32_to_f16_fallback(v[2]),
        f32_to_f16_fallback(v[3]),
    ]
}

#[inline]
fn f16x4_to_f64x4_fallback(v: &[u16]) -> [f64; 4] {
    debug_assert!(v.len() >= 4);

    [
        f16_to_f64_fallback(v[0]),
        f16_to_f64_fallback(v[1]),
        f16_to_f64_fallback(v[2]),
        f16_to_f64_fallback(v[3]),
    ]
}

#[inline]
fn f64x4_to_f16x4_fallback(v: &[f64]) -> [u16; 4] {
    debug_assert!(v.len() >= 4);

    [
        f64_to_f16_fallback(v[0]),
        f64_to_f16_fallback(v[1]),
        f64_to_f16_fallback(v[2]),
        f64_to_f16_fallback(v[3]),
    ]
}

/////////////// x86/x86_64 f16c ////////////////
#[cfg(all(
    feature = "use-intrinsics",
    any(target_arch = "x86", target_arch = "x86_64")
))]
mod x86 {
    use core::{mem::MaybeUninit, ptr};

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{__m128, __m128i, _mm_cvtph_ps, _mm_cvtps_ph, _MM_FROUND_TO_NEAREST_INT};
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128, __m128i, _mm_cvtph_ps, _mm_cvtps_ph, _MM_FROUND_TO_NEAREST_INT,
    };

    #[target_feature(enable = "f16c")]
    #[inline]
    pub(super) unsafe fn f16_to_f32_x86_f16c(i: u16) -> f32 {
        let mut vec = MaybeUninit::<__m128i>::zeroed();
        vec.as_mut_ptr().cast::<u16>().write(i);
        let retval = _mm_cvtph_ps(vec.assume_init());
        *(&retval as *const __m128).cast()
    }

    #[target_feature(enable = "f16c")]
    #[inline]
    pub(super) unsafe fn f32_to_f16_x86_f16c(f: f32) -> u16 {
        let mut vec = MaybeUninit::<__m128>::zeroed();
        vec.as_mut_ptr().cast::<f32>().write(f);
        let retval = _mm_cvtps_ph(vec.assume_init(), _MM_FROUND_TO_NEAREST_INT);
        *(&retval as *const __m128i).cast()
    }

    #[target_feature(enable = "f16c")]
    #[inline]
    pub(super) unsafe fn f16x4_to_f32x4_x86_f16c(v: &[u16]) -> [f32; 4] {
        debug_assert!(v.len() >= 4);

        let mut vec = MaybeUninit::<__m128i>::zeroed();
        ptr::copy_nonoverlapping(v.as_ptr(), vec.as_mut_ptr().cast(), 4);
        let retval = _mm_cvtph_ps(vec.assume_init());
        *(&retval as *const __m128).cast()
    }

    #[target_feature(enable = "f16c")]
    #[inline]
    pub(super) unsafe fn f32x4_to_f16x4_x86_f16c(v: &[f32]) -> [u16; 4] {
        debug_assert!(v.len() >= 4);

        let mut vec = MaybeUninit::<__m128>::uninit();
        ptr::copy_nonoverlapping(v.as_ptr(), vec.as_mut_ptr().cast(), 4);
        let retval = _mm_cvtps_ph(vec.assume_init(), _MM_FROUND_TO_NEAREST_INT);
        *(&retval as *const __m128i).cast()
    }

    #[target_feature(enable = "f16c")]
    #[inline]
    pub(super) unsafe fn f16x4_to_f64x4_x86_f16c(v: &[u16]) -> [f64; 4] {
        debug_assert!(v.len() >= 4);

        let mut vec = MaybeUninit::<__m128i>::zeroed();
        ptr::copy_nonoverlapping(v.as_ptr(), vec.as_mut_ptr().cast(), 4);
        let retval = _mm_cvtph_ps(vec.assume_init());
        let array = *(&retval as *const __m128).cast::<[f32; 4]>();
        // Let compiler vectorize this regular cast for now.
        // TODO: investigate auto-detecting sse2/avx convert features
        [
            array[0] as f64,
            array[1] as f64,
            array[2] as f64,
            array[3] as f64,
        ]
    }

    #[target_feature(enable = "f16c")]
    #[inline]
    pub(super) unsafe fn f64x4_to_f16x4_x86_f16c(v: &[f64]) -> [u16; 4] {
        debug_assert!(v.len() >= 4);

        // Let compiler vectorize this regular cast for now.
        // TODO: investigate auto-detecting sse2/avx convert features
        let v = [v[0] as f32, v[1] as f32, v[2] as f32, v[3] as f32];

        let mut vec = MaybeUninit::<__m128>::uninit();
        ptr::copy_nonoverlapping(v.as_ptr(), vec.as_mut_ptr().cast(), 4);
        let retval = _mm_cvtps_ph(vec.assume_init(), _MM_FROUND_TO_NEAREST_INT);
        *(&retval as *const __m128i).cast()
    }
}
