use core::arch::x86_64::__m128i;

use crate::memmem::{genericsimd, NeedleInfo};

/// An SSE accelerated vectorized substring search routine that only works on
/// small needles.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Forward(genericsimd::Forward);

impl Forward {
    /// Create a new "generic simd" forward searcher. If one could not be
    /// created from the given inputs, then None is returned.
    pub(crate) fn new(ninfo: &NeedleInfo, needle: &[u8]) -> Option<Forward> {
        if !cfg!(memchr_runtime_sse2) {
            return None;
        }
        genericsimd::Forward::new(ninfo, needle).map(Forward)
    }

    /// Returns the minimum length of haystack that is needed for this searcher
    /// to work. Passing a haystack with a length smaller than this will cause
    /// `find` to panic.
    #[inline(always)]
    pub(crate) fn min_haystack_len(&self) -> usize {
        self.0.min_haystack_len::<__m128i>()
    }

    #[inline(always)]
    pub(crate) fn find(
        &self,
        haystack: &[u8],
        needle: &[u8],
    ) -> Option<usize> {
        // SAFETY: sse2 is enabled on all x86_64 targets, so this is always
        // safe to call.
        unsafe { self.find_impl(haystack, needle) }
    }

    /// The implementation of find marked with the appropriate target feature.
    ///
    /// # Safety
    ///
    /// This is safe to call in all cases since sse2 is guaranteed to be part
    /// of x86_64. It is marked as unsafe because of the target feature
    /// attribute.
    #[target_feature(enable = "sse2")]
    unsafe fn find_impl(
        &self,
        haystack: &[u8],
        needle: &[u8],
    ) -> Option<usize> {
        genericsimd::fwd_find::<__m128i>(&self.0, haystack, needle)
    }
}

#[cfg(all(test, feature = "std", not(miri)))]
mod tests {
    use crate::memmem::{prefilter::PrefilterState, NeedleInfo};

    fn find(
        _: &mut PrefilterState,
        ninfo: &NeedleInfo,
        haystack: &[u8],
        needle: &[u8],
    ) -> Option<usize> {
        super::Forward::new(ninfo, needle).unwrap().find(haystack, needle)
    }

    #[test]
    fn prefilter_permutations() {
        use crate::memmem::prefilter::tests::PrefilterTest;

        // SAFETY: sse2 is enabled on all x86_64 targets, so this is always
        // safe to call.
        unsafe {
            PrefilterTest::run_all_tests_filter(find, |t| {
                // This substring searcher only works on certain configs, so
                // filter our tests such that Forward::new will be guaranteed
                // to succeed. (And also remove tests with a haystack that is
                // too small.)
                let fwd = match super::Forward::new(&t.ninfo, &t.needle) {
                    None => return false,
                    Some(fwd) => fwd,
                };
                t.haystack.len() >= fwd.min_haystack_len()
            })
        }
    }
}
