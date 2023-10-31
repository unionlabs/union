//! x86/x86-64 CPU feature detection support.
//!
//! Portable, `no_std`-friendly implementation that relies on the x86 `CPUID`
//! instruction for feature detection.

// Evaluate the given `$body` expression any of the supplied target features
// are not enabled. Otherwise returns true.
//
// The `$body` expression is not evaluated on SGX targets, and returns false
// on these targets unless *all* supplied target features are enabled.
#[macro_export]
#[doc(hidden)]
macro_rules! __unless_target_features {
    ($($tf:tt),+ => $body:expr ) => {{
        #[cfg(not(all($(target_feature=$tf,)*)))]
        {
            #[cfg(not(target_env = "sgx"))]
            $body

            // CPUID is not available on SGX targets
            #[cfg(target_env = "sgx")]
            false
        }

        #[cfg(all($(target_feature=$tf,)*))]
        true
    }};
}

// Use CPUID to detect the presence of all supplied target features.
#[macro_export]
#[doc(hidden)]
macro_rules! __detect_target_features {
    ($($tf:tt),+) => {{
        #[cfg(target_arch = "x86")]
        use core::arch::x86::{__cpuid, __cpuid_count};
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::{__cpuid, __cpuid_count};

        let cr = unsafe {
            [__cpuid(1), __cpuid_count(7, 0)]
        };

        $($crate::check!(cr, $tf) & )+ true
    }};
}

macro_rules! __expand_check_macro {
    ($(($name:tt, $i:expr, $reg:ident, $offset:expr)),* $(,)?) => {
        #[macro_export]
        #[doc(hidden)]
        macro_rules! check {
            $(
                ($cr:expr, $name) => { ($cr[$i].$reg & (1 << $offset) != 0) };
            )*
        }
    };
}

__expand_check_macro! {
    ("mmx", 0, edx, 23),
    ("sse", 0, edx, 25),
    ("sse2", 0, edx, 26),
    ("sse3", 0, ecx, 0),
    ("pclmulqdq", 0, ecx, 1),
    ("ssse3", 0, ecx, 9),
    ("fma", 0, ecx, 12),
    ("sse4.1", 0, ecx, 19),
    ("sse4.2", 0, ecx, 20),
    ("popcnt", 0, ecx, 23),
    ("aes", 0, ecx, 25),
    ("avx", 0, ecx, 28),
    ("rdrand", 0, ecx, 30),
    ("sgx", 1, ebx, 2),
    ("bmi1", 1, ebx, 3),
    ("avx2", 1, ebx, 5),
    ("bmi2", 1, ebx, 8),
    ("rdseed", 1, ebx, 18),
    ("adx", 1, ebx, 19),
    ("sha", 1, ebx, 29),
}
