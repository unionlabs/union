#[cfg(not(feature = "std"))]
pub(crate) use self::nostd::Forward;
#[cfg(feature = "std")]
pub(crate) use self::std::Forward;

#[cfg(feature = "std")]
mod std {
    use core::arch::x86_64::{__m128i, __m256i};

    use crate::memmem::{genericsimd, NeedleInfo};

    /// An AVX accelerated vectorized substring search routine that only works
    /// on small needles.
    #[derive(Clone, Copy, Debug)]
    pub(crate) struct Forward(genericsimd::Forward);

    impl Forward {
        /// Create a new "generic simd" forward searcher. If one could not be
        /// created from the given inputs, then None is returned.
        pub(crate) fn new(
            ninfo: &NeedleInfo,
            needle: &[u8],
        ) -> Option<Forward> {
            if !cfg!(memchr_runtime_avx) || !is_x86_feature_detected!("avx2") {
                return None;
            }
            genericsimd::Forward::new(ninfo, needle).map(Forward)
        }

        /// Returns the minimum length of haystack that is needed for this
        /// searcher to work. Passing a haystack with a length smaller than
        /// this will cause `find` to panic.
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
            // SAFETY: The only way a Forward value can exist is if the avx2
            // target feature is enabled. This is the only safety requirement
            // for calling the genericsimd searcher.
            unsafe { self.find_impl(haystack, needle) }
        }

        /// The implementation of find marked with the appropriate target
        /// feature.
        ///
        /// # Safety
        ///
        /// Callers must ensure that the avx2 CPU feature is enabled in the
        /// current environment.
        #[target_feature(enable = "avx2")]
        unsafe fn find_impl(
            &self,
            haystack: &[u8],
            needle: &[u8],
        ) -> Option<usize> {
            if haystack.len() < self.0.min_haystack_len::<__m256i>() {
                genericsimd::fwd_find::<__m128i>(&self.0, haystack, needle)
            } else {
                genericsimd::fwd_find::<__m256i>(&self.0, haystack, needle)
            }
        }
    }
}

// We still define the avx "forward" type on nostd to make caller code a bit
// simpler. This avoids needing a lot more conditional compilation.
#[cfg(not(feature = "std"))]
mod nostd {
    use crate::memmem::NeedleInfo;

    #[derive(Clone, Copy, Debug)]
    pub(crate) struct Forward(());

    impl Forward {
        pub(crate) fn new(
            ninfo: &NeedleInfo,
            needle: &[u8],
        ) -> Option<Forward> {
            None
        }

        pub(crate) fn min_haystack_len(&self) -> usize {
            unreachable!()
        }

        pub(crate) fn find(
            &self,
            haystack: &[u8],
            needle: &[u8],
        ) -> Option<usize> {
            unreachable!()
        }
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

        if !is_x86_feature_detected!("avx2") {
            return;
        }
        // SAFETY: The safety of find only requires that the current CPU
        // support AVX2, which we checked above.
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
