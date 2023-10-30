use core::arch::x86_64::__m256i;

use crate::memmem::{
    prefilter::{PrefilterFnTy, PrefilterState},
    NeedleInfo,
};

// Check that the functions below satisfy the Prefilter function type.
const _: PrefilterFnTy = find;

/// An AVX2 accelerated candidate finder for single-substring search.
///
/// # Safety
///
/// Callers must ensure that the avx2 CPU feature is enabled in the current
/// environment.
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn find(
    prestate: &mut PrefilterState,
    ninfo: &NeedleInfo,
    haystack: &[u8],
    needle: &[u8],
) -> Option<usize> {
    super::super::genericsimd::find::<__m256i>(
        prestate,
        ninfo,
        haystack,
        needle,
        super::sse::find,
    )
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(not(miri))]
    fn prefilter_permutations() {
        use crate::memmem::prefilter::tests::PrefilterTest;
        if !is_x86_feature_detected!("avx2") {
            return;
        }
        // SAFETY: The safety of super::find only requires that the current
        // CPU support AVX2, which we checked above.
        unsafe { PrefilterTest::run_all_tests(super::find) };
    }
}
