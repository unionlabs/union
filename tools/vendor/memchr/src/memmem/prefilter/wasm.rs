use core::arch::wasm32::v128;

use crate::memmem::{
    prefilter::{PrefilterFnTy, PrefilterState},
    NeedleInfo,
};

// Check that the functions below satisfy the Prefilter function type.
const _: PrefilterFnTy = find;

/// A `v128`-accelerated candidate finder for single-substring search.
#[target_feature(enable = "simd128")]
pub(crate) fn find(
    prestate: &mut PrefilterState,
    ninfo: &NeedleInfo,
    haystack: &[u8],
    needle: &[u8],
) -> Option<usize> {
    unsafe {
        super::genericsimd::find::<v128>(
            prestate,
            ninfo,
            haystack,
            needle,
            super::simple_memchr_fallback,
        )
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    #[test]
    #[cfg(not(miri))]
    fn prefilter_permutations() {
        use crate::memmem::prefilter::tests::PrefilterTest;
        // SAFETY: super::find is safe to call for all inputs on x86.
        unsafe { PrefilterTest::run_all_tests(super::find) };
    }
}
