use super::fallback;

// We only use AVX when we can detect at runtime whether it's available, which
// requires std.
#[cfg(feature = "std")]
mod avx;
mod sse2;

/// This macro employs a gcc-like "ifunc" trick where by upon first calling
/// `memchr` (for example), CPU feature detection will be performed at runtime
/// to determine the best implementation to use. After CPU feature detection
/// is done, we replace `memchr`'s function pointer with the selection. Upon
/// subsequent invocations, the CPU-specific routine is invoked directly, which
/// skips the CPU feature detection and subsequent branch that's required.
///
/// While this typically doesn't matter for rare occurrences or when used on
/// larger haystacks, `memchr` can be called in tight loops where the overhead
/// of this branch can actually add up *and is measurable*. This trick was
/// necessary to bring this implementation up to glibc's speeds for the 'tiny'
/// benchmarks, for example.
///
/// At some point, I expect the Rust ecosystem will get a nice macro for doing
/// exactly this, at which point, we can replace our hand-jammed version of it.
///
/// N.B. The ifunc strategy does prevent function inlining of course, but
/// on modern CPUs, you'll probably end up with the AVX2 implementation,
/// which probably can't be inlined anyway---unless you've compiled your
/// entire program with AVX2 enabled. However, even then, the various memchr
/// implementations aren't exactly small, so inlining might not help anyway!
///
/// # Safety
///
/// Callers must ensure that fnty is function pointer type.
#[cfg(feature = "std")]
macro_rules! unsafe_ifunc {
    ($fnty:ty, $name:ident, $haystack:ident, $($needle:ident),+) => {{
        use std::{mem, sync::atomic::{AtomicPtr, Ordering}};

        type FnRaw = *mut ();

        static FN: AtomicPtr<()> = AtomicPtr::new(detect as FnRaw);

        fn detect($($needle: u8),+, haystack: &[u8]) -> Option<usize> {
            let fun =
                if cfg!(memchr_runtime_avx) && is_x86_feature_detected!("avx2") {
                    avx::$name as FnRaw
                } else if cfg!(memchr_runtime_sse2) {
                    sse2::$name as FnRaw
                } else {
                    fallback::$name as FnRaw
                };
            FN.store(fun as FnRaw, Ordering::Relaxed);
            // SAFETY: By virtue of the caller contract, $fnty is a function
            // pointer, which is always safe to transmute with a *mut ().
            // Also, if 'fun is the AVX routine, then it is guaranteed to be
            // supported since we checked the avx2 feature.
            unsafe {
                mem::transmute::<FnRaw, $fnty>(fun)($($needle),+, haystack)
            }
        }

        // SAFETY: By virtue of the caller contract, $fnty is a function
        // pointer, which is always safe to transmute with a *mut (). Also, if
        // 'fun is the AVX routine, then it is guaranteed to be supported since
        // we checked the avx2 feature.
        unsafe {
            let fun = FN.load(Ordering::Relaxed);
            mem::transmute::<FnRaw, $fnty>(fun)($($needle),+, $haystack)
        }
    }}
}

/// When std isn't available to provide runtime CPU feature detection, or if
/// runtime CPU feature detection has been explicitly disabled, then just
/// call our optimized SSE2 routine directly. SSE2 is avalbale on all x86_64
/// targets, so no CPU feature detection is necessary.
///
/// # Safety
///
/// There are no safety requirements for this definition of the macro. It is
/// safe for all inputs since it is restricted to either the fallback routine
/// or the SSE routine, which is always safe to call on x86_64.
#[cfg(not(feature = "std"))]
macro_rules! unsafe_ifunc {
    ($fnty:ty, $name:ident, $haystack:ident, $($needle:ident),+) => {{
        if cfg!(memchr_runtime_sse2) {
            unsafe { sse2::$name($($needle),+, $haystack) }
        } else {
            fallback::$name($($needle),+, $haystack)
        }
    }}
}

#[inline(always)]
pub fn memchr(n1: u8, haystack: &[u8]) -> Option<usize> {
    unsafe_ifunc!(fn(u8, &[u8]) -> Option<usize>, memchr, haystack, n1)
}

#[inline(always)]
pub fn memchr2(n1: u8, n2: u8, haystack: &[u8]) -> Option<usize> {
    unsafe_ifunc!(
        fn(u8, u8, &[u8]) -> Option<usize>,
        memchr2,
        haystack,
        n1,
        n2
    )
}

#[inline(always)]
pub fn memchr3(n1: u8, n2: u8, n3: u8, haystack: &[u8]) -> Option<usize> {
    unsafe_ifunc!(
        fn(u8, u8, u8, &[u8]) -> Option<usize>,
        memchr3,
        haystack,
        n1,
        n2,
        n3
    )
}

#[inline(always)]
pub fn memrchr(n1: u8, haystack: &[u8]) -> Option<usize> {
    unsafe_ifunc!(fn(u8, &[u8]) -> Option<usize>, memrchr, haystack, n1)
}

#[inline(always)]
pub fn memrchr2(n1: u8, n2: u8, haystack: &[u8]) -> Option<usize> {
    unsafe_ifunc!(
        fn(u8, u8, &[u8]) -> Option<usize>,
        memrchr2,
        haystack,
        n1,
        n2
    )
}

#[inline(always)]
pub fn memrchr3(n1: u8, n2: u8, n3: u8, haystack: &[u8]) -> Option<usize> {
    unsafe_ifunc!(
        fn(u8, u8, u8, &[u8]) -> Option<usize>,
        memrchr3,
        haystack,
        n1,
        n2,
        n3
    )
}
