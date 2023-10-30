use crate::memmem::{rarebytes::RareNeedleBytes, NeedleInfo};

mod fallback;
#[cfg(memchr_runtime_simd)]
mod genericsimd;
#[cfg(all(not(miri), target_arch = "wasm32", memchr_runtime_simd))]
mod wasm;
#[cfg(all(not(miri), target_arch = "x86_64", memchr_runtime_simd))]
mod x86;

/// The maximum frequency rank permitted for the fallback prefilter. If the
/// rarest byte in the needle has a frequency rank above this value, then no
/// prefilter is used if the fallback prefilter would otherwise be selected.
const MAX_FALLBACK_RANK: usize = 250;

/// A combination of prefilter effectiveness state, the prefilter function and
/// the needle info required to run a prefilter.
///
/// For the most part, these are grouped into a single type for convenience,
/// instead of needing to pass around all three as distinct function
/// parameters.
pub(crate) struct Pre<'a> {
    /// State that tracks the effectiveness of a prefilter.
    pub(crate) state: &'a mut PrefilterState,
    /// The actual prefilter function.
    pub(crate) prefn: PrefilterFn,
    /// Information about a needle, such as its RK hash and rare byte offsets.
    pub(crate) ninfo: &'a NeedleInfo,
}

impl<'a> Pre<'a> {
    /// Call this prefilter on the given haystack with the given needle.
    #[inline(always)]
    pub(crate) fn call(
        &mut self,
        haystack: &[u8],
        needle: &[u8],
    ) -> Option<usize> {
        self.prefn.call(self.state, self.ninfo, haystack, needle)
    }

    /// Return true if and only if this prefilter should be used.
    #[inline(always)]
    pub(crate) fn should_call(&mut self) -> bool {
        self.state.is_effective()
    }
}

/// A prefilter function.
///
/// A prefilter function describes both forward and reverse searches.
/// (Although, we don't currently implement prefilters for reverse searching.)
/// In the case of a forward search, the position returned corresponds to
/// the starting offset of a match (confirmed or possible). Its minimum
/// value is `0`, and its maximum value is `haystack.len() - 1`. In the case
/// of a reverse search, the position returned corresponds to the position
/// immediately after a match (confirmed or possible). Its minimum value is `1`
/// and its maximum value is `haystack.len()`.
///
/// In both cases, the position returned is the starting (or ending) point of a
/// _possible_ match. That is, returning a false positive is okay. A prefilter,
/// however, must never return any false negatives. That is, if a match exists
/// at a particular position `i`, then a prefilter _must_ return that position.
/// It cannot skip past it.
///
/// # Safety
///
/// A prefilter function is not safe to create, since not all prefilters are
/// safe to call in all contexts. (e.g., A prefilter that uses AVX instructions
/// may only be called on x86_64 CPUs with the relevant AVX feature enabled.)
/// Thus, callers must ensure that when a prefilter function is created that it
/// is safe to call for the current environment.
#[derive(Clone, Copy)]
pub(crate) struct PrefilterFn(PrefilterFnTy);

/// The type of a prefilter function. All prefilters must satisfy this
/// signature.
///
/// Using a function pointer like this does inhibit inlining, but it does
/// eliminate branching and the extra costs associated with copying a larger
/// enum. Note also, that using Box<dyn SomePrefilterTrait> can't really work
/// here, since we want to work in contexts that don't have dynamic memory
/// allocation. Moreover, in the default configuration of this crate on x86_64
/// CPUs released in the past ~decade, we will use an AVX2-optimized prefilter,
/// which generally won't be inlineable into the surrounding code anyway.
/// (Unless AVX2 is enabled at compile time, but this is typically rare, since
/// it produces a non-portable binary.)
pub(crate) type PrefilterFnTy = unsafe fn(
    prestate: &mut PrefilterState,
    ninfo: &NeedleInfo,
    haystack: &[u8],
    needle: &[u8],
) -> Option<usize>;

// If the haystack is too small for SSE2, then just run memchr on the
// rarest byte and be done with it. (It is likely that this code path is
// rarely exercised, since a higher level routine will probably dispatch to
// Rabin-Karp for such a small haystack.)
#[cfg(memchr_runtime_simd)]
fn simple_memchr_fallback(
    _prestate: &mut PrefilterState,
    ninfo: &NeedleInfo,
    haystack: &[u8],
    needle: &[u8],
) -> Option<usize> {
    let (rare, _) = ninfo.rarebytes.as_rare_ordered_usize();
    crate::memchr(needle[rare], haystack).map(|i| i.saturating_sub(rare))
}

impl PrefilterFn {
    /// Create a new prefilter function from the function pointer given.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the given prefilter function is safe to call
    /// for all inputs in the current environment. For example, if the given
    /// prefilter function uses AVX instructions, then the caller must ensure
    /// that the appropriate AVX CPU features are enabled.
    pub(crate) unsafe fn new(prefn: PrefilterFnTy) -> PrefilterFn {
        PrefilterFn(prefn)
    }

    /// Call the underlying prefilter function with the given arguments.
    pub fn call(
        self,
        prestate: &mut PrefilterState,
        ninfo: &NeedleInfo,
        haystack: &[u8],
        needle: &[u8],
    ) -> Option<usize> {
        // SAFETY: Callers have the burden of ensuring that a prefilter
        // function is safe to call for all inputs in the current environment.
        unsafe { (self.0)(prestate, ninfo, haystack, needle) }
    }
}

impl core::fmt::Debug for PrefilterFn {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        "<prefilter-fn(...)>".fmt(f)
    }
}

/// Prefilter controls whether heuristics are used to accelerate searching.
///
/// A prefilter refers to the idea of detecting candidate matches very quickly,
/// and then confirming whether those candidates are full matches. This
/// idea can be quite effective since it's often the case that looking for
/// candidates can be a lot faster than running a complete substring search
/// over the entire input. Namely, looking for candidates can be done with
/// extremely fast vectorized code.
///
/// The downside of a prefilter is that it assumes false positives (which are
/// candidates generated by a prefilter that aren't matches) are somewhat rare
/// relative to the frequency of full matches. That is, if a lot of false
/// positives are generated, then it's possible for search time to be worse
/// than if the prefilter wasn't enabled in the first place.
///
/// Another downside of a prefilter is that it can result in highly variable
/// performance, where some cases are extraordinarily fast and others aren't.
/// Typically, variable performance isn't a problem, but it may be for your use
/// case.
///
/// The use of prefilters in this implementation does use a heuristic to detect
/// when a prefilter might not be carrying its weight, and will dynamically
/// disable its use. Nevertheless, this configuration option gives callers
/// the ability to disable prefilters if you have knowledge that they won't be
/// useful.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum Prefilter {
    /// Never used a prefilter in substring search.
    None,
    /// Automatically detect whether a heuristic prefilter should be used. If
    /// it is used, then heuristics will be used to dynamically disable the
    /// prefilter if it is believed to not be carrying its weight.
    Auto,
}

impl Default for Prefilter {
    fn default() -> Prefilter {
        Prefilter::Auto
    }
}

impl Prefilter {
    pub(crate) fn is_none(&self) -> bool {
        match *self {
            Prefilter::None => true,
            _ => false,
        }
    }
}

/// PrefilterState tracks state associated with the effectiveness of a
/// prefilter. It is used to track how many bytes, on average, are skipped by
/// the prefilter. If this average dips below a certain threshold over time,
/// then the state renders the prefilter inert and stops using it.
///
/// A prefilter state should be created for each search. (Where creating an
/// iterator is treated as a single search.) A prefilter state should only be
/// created from a `Freqy`. e.g., An inert `Freqy` will produce an inert
/// `PrefilterState`.
#[derive(Clone, Debug)]
pub(crate) struct PrefilterState {
    /// The number of skips that has been executed. This is always 1 greater
    /// than the actual number of skips. The special sentinel value of 0
    /// indicates that the prefilter is inert. This is useful to avoid
    /// additional checks to determine whether the prefilter is still
    /// "effective." Once a prefilter becomes inert, it should no longer be
    /// used (according to our heuristics).
    skips: u32,
    /// The total number of bytes that have been skipped.
    skipped: u32,
}

impl PrefilterState {
    /// The minimum number of skip attempts to try before considering whether
    /// a prefilter is effective or not.
    const MIN_SKIPS: u32 = 50;

    /// The minimum amount of bytes that skipping must average.
    ///
    /// This value was chosen based on varying it and checking
    /// the microbenchmarks. In particular, this can impact the
    /// pathological/repeated-{huge,small} benchmarks quite a bit if it's set
    /// too low.
    const MIN_SKIP_BYTES: u32 = 8;

    /// Create a fresh prefilter state.
    pub(crate) fn new() -> PrefilterState {
        PrefilterState { skips: 1, skipped: 0 }
    }

    /// Create a fresh prefilter state that is always inert.
    pub(crate) fn inert() -> PrefilterState {
        PrefilterState { skips: 0, skipped: 0 }
    }

    /// Update this state with the number of bytes skipped on the last
    /// invocation of the prefilter.
    #[inline]
    pub(crate) fn update(&mut self, skipped: usize) {
        self.skips = self.skips.saturating_add(1);
        // We need to do this dance since it's technically possible for
        // `skipped` to overflow a `u32`. (And we use a `u32` to reduce the
        // size of a prefilter state.)
        if skipped > core::u32::MAX as usize {
            self.skipped = core::u32::MAX;
        } else {
            self.skipped = self.skipped.saturating_add(skipped as u32);
        }
    }

    /// Return true if and only if this state indicates that a prefilter is
    /// still effective.
    #[inline]
    pub(crate) fn is_effective(&mut self) -> bool {
        if self.is_inert() {
            return false;
        }
        if self.skips() < PrefilterState::MIN_SKIPS {
            return true;
        }
        if self.skipped >= PrefilterState::MIN_SKIP_BYTES * self.skips() {
            return true;
        }

        // We're inert.
        self.skips = 0;
        false
    }

    #[inline]
    fn is_inert(&self) -> bool {
        self.skips == 0
    }

    #[inline]
    fn skips(&self) -> u32 {
        self.skips.saturating_sub(1)
    }
}

/// Determine which prefilter function, if any, to use.
///
/// This only applies to x86_64 when runtime SIMD detection is enabled (which
/// is the default). In general, we try to use an AVX prefilter, followed by
/// SSE and then followed by a generic one based on memchr.
#[inline(always)]
pub(crate) fn forward(
    config: &Prefilter,
    rare: &RareNeedleBytes,
    needle: &[u8],
) -> Option<PrefilterFn> {
    if config.is_none() || needle.len() <= 1 {
        return None;
    }

    #[cfg(all(not(miri), target_arch = "x86_64", memchr_runtime_simd))]
    {
        #[cfg(feature = "std")]
        {
            if cfg!(memchr_runtime_avx) {
                if is_x86_feature_detected!("avx2") {
                    // SAFETY: x86::avx::find only requires the avx2 feature,
                    // which we've just checked above.
                    return unsafe { Some(PrefilterFn::new(x86::avx::find)) };
                }
            }
        }
        if cfg!(memchr_runtime_sse2) {
            // SAFETY: x86::sse::find only requires the sse2 feature, which is
            // guaranteed to be available on x86_64.
            return unsafe { Some(PrefilterFn::new(x86::sse::find)) };
        }
    }
    #[cfg(all(not(miri), target_arch = "wasm32", memchr_runtime_simd))]
    {
        // SAFETY: `wasm::find` is actually a safe function
        //
        // Also note that the `if true` is here to prevent, on wasm with simd,
        // rustc warning about the code below being dead code.
        if true {
            return unsafe { Some(PrefilterFn::new(wasm::find)) };
        }
    }
    // Check that our rarest byte has a reasonably low rank. The main issue
    // here is that the fallback prefilter can perform pretty poorly if it's
    // given common bytes. So we try to avoid the worst cases here.
    let (rare1_rank, _) = rare.as_ranks(needle);
    if rare1_rank <= MAX_FALLBACK_RANK {
        // SAFETY: fallback::find is safe to call in all environments.
        return unsafe { Some(PrefilterFn::new(fallback::find)) };
    }
    None
}

/// Return the minimum length of the haystack in which a prefilter should be
/// used. If the haystack is below this length, then it's probably not worth
/// the overhead of running the prefilter.
///
/// We used to look at the length of a haystack here. That is, if it was too
/// small, then don't bother with the prefilter. But two things changed:
/// the prefilter falls back to memchr for small haystacks, and, at the
/// meta-searcher level, Rabin-Karp is employed for tiny haystacks anyway.
///
/// We keep it around for now in case we want to bring it back.
#[allow(dead_code)]
pub(crate) fn minimum_len(_haystack: &[u8], needle: &[u8]) -> usize {
    // If the haystack length isn't greater than needle.len() * FACTOR, then
    // no prefilter will be used. The presumption here is that since there
    // are so few bytes to check, it's not worth running the prefilter since
    // there will need to be a validation step anyway. Thus, the prefilter is
    // largely redundant work.
    //
    // Increase the factor noticeably hurts the
    // memmem/krate/prebuilt/teeny-*/never-john-watson benchmarks.
    const PREFILTER_LENGTH_FACTOR: usize = 2;
    const VECTOR_MIN_LENGTH: usize = 16;
    let min = core::cmp::max(
        VECTOR_MIN_LENGTH,
        PREFILTER_LENGTH_FACTOR * needle.len(),
    );
    // For haystacks with length==min, we still want to avoid the prefilter,
    // so add 1.
    min + 1
}

#[cfg(all(test, feature = "std", not(miri)))]
pub(crate) mod tests {
    use std::convert::{TryFrom, TryInto};

    use super::*;
    use crate::memmem::{
        prefilter::PrefilterFnTy, rabinkarp, rarebytes::RareNeedleBytes,
    };

    // Below is a small jig that generates prefilter tests. The main purpose
    // of this jig is to generate tests of varying needle/haystack lengths
    // in order to try and exercise all code paths in our prefilters. And in
    // particular, this is especially important for vectorized prefilters where
    // certain code paths might only be exercised at certain lengths.

    /// A test that represents the input and expected output to a prefilter
    /// function. The test should be able to run with any prefilter function
    /// and get the expected output.
    pub(crate) struct PrefilterTest {
        // These fields represent the inputs and expected output of a forwards
        // prefilter function.
        pub(crate) ninfo: NeedleInfo,
        pub(crate) haystack: Vec<u8>,
        pub(crate) needle: Vec<u8>,
        pub(crate) output: Option<usize>,
    }

    impl PrefilterTest {
        /// Run all generated forward prefilter tests on the given prefn.
        ///
        /// # Safety
        ///
        /// Callers must ensure that the given prefilter function pointer is
        /// safe to call for all inputs in the current environment.
        pub(crate) unsafe fn run_all_tests(prefn: PrefilterFnTy) {
            PrefilterTest::run_all_tests_filter(prefn, |_| true)
        }

        /// Run all generated forward prefilter tests that pass the given
        /// predicate on the given prefn.
        ///
        /// # Safety
        ///
        /// Callers must ensure that the given prefilter function pointer is
        /// safe to call for all inputs in the current environment.
        pub(crate) unsafe fn run_all_tests_filter(
            prefn: PrefilterFnTy,
            mut predicate: impl FnMut(&PrefilterTest) -> bool,
        ) {
            for seed in PREFILTER_TEST_SEEDS {
                for test in seed.generate() {
                    if predicate(&test) {
                        test.run(prefn);
                    }
                }
            }
        }

        /// Create a new prefilter test from a seed and some chose offsets to
        /// rare bytes in the seed's needle.
        ///
        /// If a valid test could not be constructed, then None is returned.
        /// (Currently, we take the approach of massaging tests to be valid
        /// instead of rejecting them outright.)
        fn new(
            seed: PrefilterTestSeed,
            rare1i: usize,
            rare2i: usize,
            haystack_len: usize,
            needle_len: usize,
            output: Option<usize>,
        ) -> Option<PrefilterTest> {
            let mut rare1i: u8 = rare1i.try_into().unwrap();
            let mut rare2i: u8 = rare2i.try_into().unwrap();
            // The '#' byte is never used in a haystack (unless we're expecting
            // a match), while the '@' byte is never used in a needle.
            let mut haystack = vec![b'@'; haystack_len];
            let mut needle = vec![b'#'; needle_len];
            needle[0] = seed.first;
            needle[rare1i as usize] = seed.rare1;
            needle[rare2i as usize] = seed.rare2;
            // If we're expecting a match, then make sure the needle occurs
            // in the haystack at the expected position.
            if let Some(i) = output {
                haystack[i..i + needle.len()].copy_from_slice(&needle);
            }
            // If the operations above lead to rare offsets pointing to the
            // non-first occurrence of a byte, then adjust it. This might lead
            // to redundant tests, but it's simpler than trying to change the
            // generation process I think.
            if let Some(i) = crate::memchr(seed.rare1, &needle) {
                rare1i = u8::try_from(i).unwrap();
            }
            if let Some(i) = crate::memchr(seed.rare2, &needle) {
                rare2i = u8::try_from(i).unwrap();
            }
            let ninfo = NeedleInfo {
                rarebytes: RareNeedleBytes::new(rare1i, rare2i),
                nhash: rabinkarp::NeedleHash::forward(&needle),
            };
            Some(PrefilterTest { ninfo, haystack, needle, output })
        }

        /// Run this specific test on the given prefilter function. If the
        /// outputs do no match, then this routine panics with a failure
        /// message.
        ///
        /// # Safety
        ///
        /// Callers must ensure that the given prefilter function pointer is
        /// safe to call for all inputs in the current environment.
        unsafe fn run(&self, prefn: PrefilterFnTy) {
            let mut prestate = PrefilterState::new();
            assert_eq!(
                self.output,
                prefn(
                    &mut prestate,
                    &self.ninfo,
                    &self.haystack,
                    &self.needle
                ),
                "ninfo: {:?}, haystack(len={}): {:?}, needle(len={}): {:?}",
                self.ninfo,
                self.haystack.len(),
                std::str::from_utf8(&self.haystack).unwrap(),
                self.needle.len(),
                std::str::from_utf8(&self.needle).unwrap(),
            );
        }
    }

    /// A set of prefilter test seeds. Each seed serves as the base for the
    /// generation of many other tests. In essence, the seed captures the
    /// "rare" and first bytes among our needle. The tests generated from each
    /// seed essentially vary the length of the needle and haystack, while
    /// using the rare/first byte configuration from the seed.
    ///
    /// The purpose of this is to test many different needle/haystack lengths.
    /// In particular, some of the vector optimizations might only have bugs
    /// in haystacks of a certain size.
    const PREFILTER_TEST_SEEDS: &[PrefilterTestSeed] = &[
        PrefilterTestSeed { first: b'x', rare1: b'y', rare2: b'z' },
        PrefilterTestSeed { first: b'x', rare1: b'x', rare2: b'z' },
        PrefilterTestSeed { first: b'x', rare1: b'y', rare2: b'x' },
        PrefilterTestSeed { first: b'x', rare1: b'x', rare2: b'x' },
        PrefilterTestSeed { first: b'x', rare1: b'y', rare2: b'y' },
    ];

    /// Data that describes a single prefilter test seed.
    #[derive(Clone, Copy)]
    struct PrefilterTestSeed {
        first: u8,
        rare1: u8,
        rare2: u8,
    }

    impl PrefilterTestSeed {
        /// Generate a series of prefilter tests from this seed.
        fn generate(self) -> impl Iterator<Item = PrefilterTest> {
            let len_start = 2;
            // The iterator below generates *a lot* of tests. The number of
            // tests was chosen somewhat empirically to be "bearable" when
            // running the test suite.
            //
            // We use an iterator here because the collective haystacks of all
            // these test cases add up to enough memory to OOM a conservative
            // sandbox or a small laptop.
            (len_start..=40).flat_map(move |needle_len| {
                let rare_start = len_start - 1;
                (rare_start..needle_len).flat_map(move |rare1i| {
                    (rare1i..needle_len).flat_map(move |rare2i| {
                        (needle_len..=66).flat_map(move |haystack_len| {
                            PrefilterTest::new(
                                self,
                                rare1i,
                                rare2i,
                                haystack_len,
                                needle_len,
                                None,
                            )
                            .into_iter()
                            .chain(
                                (0..=(haystack_len - needle_len)).flat_map(
                                    move |output| {
                                        PrefilterTest::new(
                                            self,
                                            rare1i,
                                            rare2i,
                                            haystack_len,
                                            needle_len,
                                            Some(output),
                                        )
                                    },
                                ),
                            )
                        })
                    })
                })
            })
        }
    }
}
