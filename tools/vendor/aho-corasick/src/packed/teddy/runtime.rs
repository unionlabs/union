// See the README in this directory for an explanation of the Teddy algorithm.
// It is strongly recommended to peruse the README before trying to grok this
// code, as its use of SIMD is pretty opaque, although I tried to add comments
// where appropriate.
//
// Moreover, while there is a lot of code in this file, most of it is
// repeated variants of the same thing. Specifically, there are three Teddy
// variants: Slim 128-bit Teddy (8 buckets), Slim 256-bit Teddy (8 buckets)
// and Fat 256-bit Teddy (16 buckets). For each variant, there are three
// implementations, corresponding to mask lengths of 1, 2 and 3. Bringing it to
// a total of nine variants. Each one is structured roughly the same:
//
//     while at <= len(haystack) - CHUNK_SIZE:
//         let candidate = find_candidate_in_chunk(haystack, at)
//         if not all zeroes(candidate):
//             if match = verify(haystack, at, candidate):
//                 return match
//
// For the most part, this remains unchanged. The parts that vary are the
// verification routine (for slim vs fat Teddy) and the candidate extraction
// (based on the number of masks).
//
// In the code below, a "candidate" corresponds to a single vector with 8-bit
// lanes. Each lane is itself an 8-bit bitset, where the ith bit is set in the
// jth lane if and only if the byte occurring at position `j` is in the
// bucket `i` (where the `j`th position is the position in the current window
// of the haystack, which is always 16 or 32 bytes). Note to be careful here:
// the ith bit and the jth lane correspond to the least significant bits of the
// vector. So when visualizing how the current window of bytes is stored in a
// vector, you often need to flip it around. For example, the text `abcd` in a
// 4-byte vector would look like this:
//
//     01100100 01100011 01100010 01100001
//         d        c        b        a
//
// When the mask length is 1, then finding the candidate is pretty straight
// forward: you just apply the shuffle indices (from the haystack window) to
// the masks, and then AND them together, as described in the README. But for
// masks of length 2 and 3, you need to keep a little state. Specifically,
// you need to store the final 1 (for mask length 2) or 2 (for mask length 3)
// bytes of the candidate for use when searching the next window. This is for
// handling matches that span two windows.
//
// With respect to the repeated code, it would likely be possible to reduce
// the number of copies of code below using polymorphism, but I find this
// formulation clearer instead of needing to reason through generics. However,
// I admit, there may be a simpler generic construction that I'm missing.
//
// All variants are fairly heavily tested in src/packed/tests.rs.

use std::arch::x86_64::*;
use std::mem;

use crate::packed::pattern::{PatternID, Patterns};
use crate::packed::teddy::compile;
use crate::packed::vector::*;
use crate::Match;

/// The Teddy runtime.
///
/// A Teddy runtime can be used to quickly search for occurrences of one or
/// more patterns. While it does not scale to an arbitrary number of patterns
/// like Aho-Corasick, it does find occurrences for a small set of patterns
/// much more quickly than Aho-Corasick.
///
/// Teddy cannot run on small haystacks below a certain size, which is
/// dependent on the type of matcher used. This size can be queried via the
/// `minimum_len` method. Violating this will result in a panic.
///
/// Finally, when callers use a Teddy runtime, they must provide precisely the
/// patterns used to construct the Teddy matcher. Violating this will result
/// in either a panic or incorrect results, but will never sacrifice memory
/// safety.
#[derive(Clone, Debug)]
pub struct Teddy {
    /// The allocation of patterns in buckets. This only contains the IDs of
    /// patterns. In order to do full verification, callers must provide the
    /// actual patterns when using Teddy.
    pub buckets: Vec<Vec<PatternID>>,
    /// The maximum identifier of a pattern. This is used as a sanity check to
    /// ensure that the patterns provided by the caller are the same as the
    /// patterns that were used to compile the matcher. This sanity check
    /// permits safely eliminating bounds checks regardless of what patterns
    /// are provided by the caller.
    ///
    /// Note that users of the aho-corasick crate cannot get this wrong. Only
    /// code internal to this crate can get it wrong, since neither `Patterns`
    /// type nor the Teddy runtime are public API items.
    pub max_pattern_id: PatternID,
    /// The actual runtime to use.
    pub exec: Exec,
}

impl Teddy {
    /// Return the first occurrence of a match in the given haystack after or
    /// starting at `at`.
    ///
    /// The patterns provided must be precisely the same patterns given to the
    /// Teddy builder, otherwise this may panic or produce incorrect results.
    ///
    /// All matches are consistent with the match semantics (leftmost-first or
    /// leftmost-longest) set on `pats`.
    pub fn find_at(
        &self,
        pats: &Patterns,
        haystack: &[u8],
        at: usize,
    ) -> Option<Match> {
        // This assert is a bit subtle, but it's an important guarantee.
        // Namely, if the maximum pattern ID seen by Teddy is the same as the
        // one in the patterns given, then we are guaranteed that every pattern
        // ID in all Teddy buckets are valid indices into `pats`. While this
        // is nominally true, there is no guarantee that callers provide the
        // same `pats` to both the Teddy builder and the searcher, which would
        // otherwise make `find_at` unsafe to call. But this assert lets us
        // keep this routine safe and eliminate an important bounds check in
        // verification.
        assert_eq!(
            self.max_pattern_id,
            pats.max_pattern_id(),
            "teddy must be called with same patterns it was built with",
        );
        // SAFETY: The haystack must have at least a minimum number of bytes
        // for Teddy to be able to work. The minimum number varies depending on
        // which matcher is used below. If this is violated, then it's possible
        // for searching to do out-of-bounds writes.
        assert!(haystack[at..].len() >= self.minimum_len());
        // SAFETY: The various Teddy matchers are always safe to call because
        // the Teddy builder guarantees that a particular Exec variant is
        // built only when it can be run the current CPU. That is, the Teddy
        // builder will not produce a Exec::TeddySlim1Mask256 unless AVX2 is
        // enabled. That is, our dynamic CPU feature detection is performed
        // once in the builder, and we rely on the type system to avoid needing
        // to do it again.
        unsafe {
            match self.exec {
                Exec::TeddySlim1Mask128(ref e) => {
                    e.find_at(pats, self, haystack, at)
                }
                Exec::TeddySlim1Mask256(ref e) => {
                    e.find_at(pats, self, haystack, at)
                }
                Exec::TeddyFat1Mask256(ref e) => {
                    e.find_at(pats, self, haystack, at)
                }
                Exec::TeddySlim2Mask128(ref e) => {
                    e.find_at(pats, self, haystack, at)
                }
                Exec::TeddySlim2Mask256(ref e) => {
                    e.find_at(pats, self, haystack, at)
                }
                Exec::TeddyFat2Mask256(ref e) => {
                    e.find_at(pats, self, haystack, at)
                }
                Exec::TeddySlim3Mask128(ref e) => {
                    e.find_at(pats, self, haystack, at)
                }
                Exec::TeddySlim3Mask256(ref e) => {
                    e.find_at(pats, self, haystack, at)
                }
                Exec::TeddyFat3Mask256(ref e) => {
                    e.find_at(pats, self, haystack, at)
                }
            }
        }
    }

    /// Returns the minimum length of a haystack that must be provided by
    /// callers to this Teddy searcher. Providing a haystack shorter than this
    /// will result in a panic, but will never violate memory safety.
    pub fn minimum_len(&self) -> usize {
        // SAFETY: These values must be correct in order to ensure safety.
        // The Teddy runtime assumes their haystacks have at least these
        // lengths. Violating this will sacrifice memory safety.
        match self.exec {
            Exec::TeddySlim1Mask128(_) => 16,
            Exec::TeddySlim1Mask256(_) => 32,
            Exec::TeddyFat1Mask256(_) => 16,
            Exec::TeddySlim2Mask128(_) => 17,
            Exec::TeddySlim2Mask256(_) => 33,
            Exec::TeddyFat2Mask256(_) => 17,
            Exec::TeddySlim3Mask128(_) => 18,
            Exec::TeddySlim3Mask256(_) => 34,
            Exec::TeddyFat3Mask256(_) => 34,
        }
    }

    /// Returns the approximate total amount of heap used by this searcher, in
    /// units of bytes.
    pub fn heap_bytes(&self) -> usize {
        let num_patterns = self.max_pattern_id as usize + 1;
        self.buckets.len() * mem::size_of::<Vec<PatternID>>()
            + num_patterns * mem::size_of::<PatternID>()
    }

    /// Runs the verification routine for Slim 128-bit Teddy.
    ///
    /// The candidate given should be a collection of 8-bit bitsets (one bitset
    /// per lane), where the ith bit is set in the jth lane if and only if the
    /// byte occurring at `at + j` in `haystack` is in the bucket `i`.
    ///
    /// This is not safe to call unless the SSSE3 target feature is enabled.
    /// The `target_feature` attribute is not applied since this function is
    /// always forcefully inlined.
    #[inline(always)]
    unsafe fn verify128(
        &self,
        pats: &Patterns,
        haystack: &[u8],
        at: usize,
        cand: __m128i,
    ) -> Option<Match> {
        debug_assert!(!is_all_zeroes128(cand));
        debug_assert_eq!(8, self.buckets.len());

        // Convert the candidate into 64-bit chunks, and then verify each of
        // those chunks.
        let parts = unpack64x128(cand);
        for (i, &part) in parts.iter().enumerate() {
            let pos = at + i * 8;
            if let Some(m) = self.verify64(pats, 8, haystack, pos, part) {
                return Some(m);
            }
        }
        None
    }

    /// Runs the verification routine for Slim 256-bit Teddy.
    ///
    /// The candidate given should be a collection of 8-bit bitsets (one bitset
    /// per lane), where the ith bit is set in the jth lane if and only if the
    /// byte occurring at `at + j` in `haystack` is in the bucket `i`.
    ///
    /// This is not safe to call unless the AVX2 target feature is enabled.
    /// The `target_feature` attribute is not applied since this function is
    /// always forcefully inlined.
    #[inline(always)]
    unsafe fn verify256(
        &self,
        pats: &Patterns,
        haystack: &[u8],
        at: usize,
        cand: __m256i,
    ) -> Option<Match> {
        debug_assert!(!is_all_zeroes256(cand));
        debug_assert_eq!(8, self.buckets.len());

        // Convert the candidate into 64-bit chunks, and then verify each of
        // those chunks.
        let parts = unpack64x256(cand);
        for (i, &part) in parts.iter().enumerate() {
            let pos = at + i * 8;
            if let Some(m) = self.verify64(pats, 8, haystack, pos, part) {
                return Some(m);
            }
        }
        None
    }

    /// Runs the verification routine for Fat 256-bit Teddy.
    ///
    /// The candidate given should be a collection of 8-bit bitsets (one bitset
    /// per lane), where the ith bit is set in the jth lane if and only if the
    /// byte occurring at `at + (j < 16 ? j : j - 16)` in `haystack` is in the
    /// bucket `j < 16 ? i : i + 8`.
    ///
    /// This is not safe to call unless the AVX2 target feature is enabled.
    /// The `target_feature` attribute is not applied since this function is
    /// always forcefully inlined.
    #[inline(always)]
    unsafe fn verify_fat256(
        &self,
        pats: &Patterns,
        haystack: &[u8],
        at: usize,
        cand: __m256i,
    ) -> Option<Match> {
        debug_assert!(!is_all_zeroes256(cand));
        debug_assert_eq!(16, self.buckets.len());

        // This is a bit tricky, but we basically want to convert our
        // candidate, which looks like this
        //
        //     a31 a30 ... a17 a16 a15 a14 ... a01 a00
        //
        // where each a(i) is an 8-bit bitset corresponding to the activated
        // buckets, to this
        //
        //     a31 a15 a30 a14 a29 a13 ... a18 a02 a17 a01 a16 a00
        //
        // Namely, for Fat Teddy, the high 128-bits of the candidate correspond
        // to the same bytes in the haystack in the low 128-bits (so we only
        // scan 16 bytes at a time), but are for buckets 8-15 instead of 0-7.
        //
        // The verification routine wants to look at all potentially matching
        // buckets before moving on to the next lane. So for example, both
        // a16 and a00 both correspond to the first byte in our window; a00
        // contains buckets 0-7 and a16 contains buckets 8-15. Specifically,
        // a16 should be checked before a01. So the transformation shown above
        // allows us to use our normal verification procedure with one small
        // change: we treat each bitset as 16 bits instead of 8 bits.

        // Swap the 128-bit lanes in the candidate vector.
        let swap = _mm256_permute4x64_epi64(cand, 0x4E);
        // Interleave the bytes from the low 128-bit lanes, starting with
        // cand first.
        let r1 = _mm256_unpacklo_epi8(cand, swap);
        // Interleave the bytes from the high 128-bit lanes, starting with
        // cand first.
        let r2 = _mm256_unpackhi_epi8(cand, swap);
        // Now just take the 2 low 64-bit integers from both r1 and r2. We
        // can drop the high 64-bit integers because they are a mirror image
        // of the low 64-bit integers. All we care about are the low 128-bit
        // lanes of r1 and r2. Combined, they contain all our 16-bit bitsets
        // laid out in the desired order, as described above.
        let parts = unpacklo64x256(r1, r2);
        for (i, &part) in parts.iter().enumerate() {
            let pos = at + i * 4;
            if let Some(m) = self.verify64(pats, 16, haystack, pos, part) {
                return Some(m);
            }
        }
        None
    }

    /// Verify whether there are any matches starting at or after `at` in the
    /// given `haystack`. The candidate given should correspond to either 8-bit
    /// (for 8 buckets) or 16-bit (16 buckets) bitsets.
    #[inline(always)]
    fn verify64(
        &self,
        pats: &Patterns,
        bucket_count: usize,
        haystack: &[u8],
        at: usize,
        mut cand: u64,
    ) -> Option<Match> {
        // N.B. While the bucket count is known from self.buckets.len(),
        // requiring it as a parameter makes it easier for the optimizer to
        // know its value, and thus produce more efficient codegen.
        debug_assert!(bucket_count == 8 || bucket_count == 16);
        while cand != 0 {
            let bit = cand.trailing_zeros() as usize;
            cand &= !(1 << bit);

            let at = at + (bit / bucket_count);
            let bucket = bit % bucket_count;
            if let Some(m) = self.verify_bucket(pats, haystack, bucket, at) {
                return Some(m);
            }
        }
        None
    }

    /// Verify whether there are any matches starting at `at` in the given
    /// `haystack` corresponding only to patterns in the given bucket.
    #[inline(always)]
    fn verify_bucket(
        &self,
        pats: &Patterns,
        haystack: &[u8],
        bucket: usize,
        at: usize,
    ) -> Option<Match> {
        // Forcing this function to not inline and be "cold" seems to help
        // the codegen for Teddy overall. Interestingly, this is good for a
        // 16% boost in the sherlock/packed/teddy/name/alt1 benchmark (among
        // others). Overall, this seems like a problem with codegen, since
        // creating the Match itself is a very small amount of code.
        #[cold]
        #[inline(never)]
        fn match_from_span(
            pati: PatternID,
            start: usize,
            end: usize,
        ) -> Match {
            Match::from_span(pati as usize, start, end)
        }

        // N.B. The bounds check for this bucket lookup *should* be elided
        // since we assert the number of buckets in each `find_at` routine,
        // and the compiler can prove that the `% 8` (or `% 16`) in callers
        // of this routine will always be in bounds.
        for &pati in &self.buckets[bucket] {
            // SAFETY: This is safe because we are guaranteed that every
            // index in a Teddy bucket is a valid index into `pats`. This
            // guarantee is upheld by the assert checking `max_pattern_id` in
            // the beginning of `find_at` above.
            //
            // This explicit bounds check elision is (amazingly) good for a
            // 25-50% boost in some benchmarks, particularly ones with a lot
            // of short literals.
            let pat = unsafe { pats.get_unchecked(pati) };
            if pat.is_prefix(&haystack[at..]) {
                return Some(match_from_span(pati, at, at + pat.len()));
            }
        }
        None
    }
}

/// Exec represents the different search strategies supported by the Teddy
/// runtime.
///
/// This enum is an important safety abstraction. Namely, callers should only
/// construct a variant in this enum if it is safe to execute its corresponding
/// target features on the current CPU. The 128-bit searchers require SSSE3,
/// while the 256-bit searchers require AVX2.
#[derive(Clone, Debug)]
pub enum Exec {
    TeddySlim1Mask128(TeddySlim1Mask128),
    TeddySlim1Mask256(TeddySlim1Mask256),
    TeddyFat1Mask256(TeddyFat1Mask256),
    TeddySlim2Mask128(TeddySlim2Mask128),
    TeddySlim2Mask256(TeddySlim2Mask256),
    TeddyFat2Mask256(TeddyFat2Mask256),
    TeddySlim3Mask128(TeddySlim3Mask128),
    TeddySlim3Mask256(TeddySlim3Mask256),
    TeddyFat3Mask256(TeddyFat3Mask256),
}

// Most of the code below remains undocumented because they are effectively
// repeated versions of themselves. The general structure is described in the
// README and in the comments above.

#[derive(Clone, Debug)]
pub struct TeddySlim1Mask128 {
    pub mask1: Mask128,
}

impl TeddySlim1Mask128 {
    #[target_feature(enable = "ssse3")]
    unsafe fn find_at(
        &self,
        pats: &Patterns,
        teddy: &Teddy,
        haystack: &[u8],
        mut at: usize,
    ) -> Option<Match> {
        debug_assert!(haystack[at..].len() >= teddy.minimum_len());
        // This assert helps eliminate bounds checks for bucket lookups in
        // Teddy::verify_bucket, which has a small (3-4%) performance boost.
        assert_eq!(8, teddy.buckets.len());

        let len = haystack.len();
        while at <= len - 16 {
            let c = self.candidate(haystack, at);
            if !is_all_zeroes128(c) {
                if let Some(m) = teddy.verify128(pats, haystack, at, c) {
                    return Some(m);
                }
            }
            at += 16;
        }
        if at < len {
            at = len - 16;
            let c = self.candidate(haystack, at);
            if !is_all_zeroes128(c) {
                if let Some(m) = teddy.verify128(pats, haystack, at, c) {
                    return Some(m);
                }
            }
        }
        None
    }

    #[inline(always)]
    unsafe fn candidate(&self, haystack: &[u8], at: usize) -> __m128i {
        debug_assert!(haystack[at..].len() >= 16);

        let chunk = loadu128(haystack, at);
        members1m128(chunk, self.mask1)
    }
}

#[derive(Clone, Debug)]
pub struct TeddySlim1Mask256 {
    pub mask1: Mask256,
}

impl TeddySlim1Mask256 {
    #[target_feature(enable = "avx2")]
    unsafe fn find_at(
        &self,
        pats: &Patterns,
        teddy: &Teddy,
        haystack: &[u8],
        mut at: usize,
    ) -> Option<Match> {
        debug_assert!(haystack[at..].len() >= teddy.minimum_len());
        // This assert helps eliminate bounds checks for bucket lookups in
        // Teddy::verify_bucket, which has a small (3-4%) performance boost.
        assert_eq!(8, teddy.buckets.len());

        let len = haystack.len();
        while at <= len - 32 {
            let c = self.candidate(haystack, at);
            if !is_all_zeroes256(c) {
                if let Some(m) = teddy.verify256(pats, haystack, at, c) {
                    return Some(m);
                }
            }
            at += 32;
        }
        if at < len {
            at = len - 32;
            let c = self.candidate(haystack, at);
            if !is_all_zeroes256(c) {
                if let Some(m) = teddy.verify256(pats, haystack, at, c) {
                    return Some(m);
                }
            }
        }
        None
    }

    #[inline(always)]
    unsafe fn candidate(&self, haystack: &[u8], at: usize) -> __m256i {
        debug_assert!(haystack[at..].len() >= 32);

        let chunk = loadu256(haystack, at);
        members1m256(chunk, self.mask1)
    }
}

#[derive(Clone, Debug)]
pub struct TeddyFat1Mask256 {
    pub mask1: Mask256,
}

impl TeddyFat1Mask256 {
    #[target_feature(enable = "avx2")]
    unsafe fn find_at(
        &self,
        pats: &Patterns,
        teddy: &Teddy,
        haystack: &[u8],
        mut at: usize,
    ) -> Option<Match> {
        debug_assert!(haystack[at..].len() >= teddy.minimum_len());
        // This assert helps eliminate bounds checks for bucket lookups in
        // Teddy::verify_bucket, which has a small (3-4%) performance boost.
        assert_eq!(16, teddy.buckets.len());

        let len = haystack.len();
        while at <= len - 16 {
            let c = self.candidate(haystack, at);
            if !is_all_zeroes256(c) {
                if let Some(m) = teddy.verify_fat256(pats, haystack, at, c) {
                    return Some(m);
                }
            }
            at += 16;
        }
        if at < len {
            at = len - 16;
            let c = self.candidate(haystack, at);
            if !is_all_zeroes256(c) {
                if let Some(m) = teddy.verify_fat256(pats, haystack, at, c) {
                    return Some(m);
                }
            }
        }
        None
    }

    #[inline(always)]
    unsafe fn candidate(&self, haystack: &[u8], at: usize) -> __m256i {
        debug_assert!(haystack[at..].len() >= 16);

        let chunk = _mm256_broadcastsi128_si256(loadu128(haystack, at));
        members1m256(chunk, self.mask1)
    }
}

#[derive(Clone, Debug)]
pub struct TeddySlim2Mask128 {
    pub mask1: Mask128,
    pub mask2: Mask128,
}

impl TeddySlim2Mask128 {
    #[target_feature(enable = "ssse3")]
    unsafe fn find_at(
        &self,
        pats: &Patterns,
        teddy: &Teddy,
        haystack: &[u8],
        mut at: usize,
    ) -> Option<Match> {
        debug_assert!(haystack[at..].len() >= teddy.minimum_len());
        // This assert helps eliminate bounds checks for bucket lookups in
        // Teddy::verify_bucket, which has a small (3-4%) performance boost.
        assert_eq!(8, teddy.buckets.len());

        at += 1;
        let len = haystack.len();
        let mut prev0 = ones128();
        while at <= len - 16 {
            let c = self.candidate(haystack, at, &mut prev0);
            if !is_all_zeroes128(c) {
                if let Some(m) = teddy.verify128(pats, haystack, at - 1, c) {
                    return Some(m);
                }
            }
            at += 16;
        }
        if at < len {
            at = len - 16;
            prev0 = ones128();

            let c = self.candidate(haystack, at, &mut prev0);
            if !is_all_zeroes128(c) {
                if let Some(m) = teddy.verify128(pats, haystack, at - 1, c) {
                    return Some(m);
                }
            }
        }
        None
    }

    #[inline(always)]
    unsafe fn candidate(
        &self,
        haystack: &[u8],
        at: usize,
        prev0: &mut __m128i,
    ) -> __m128i {
        debug_assert!(haystack[at..].len() >= 16);

        let chunk = loadu128(haystack, at);
        let (res0, res1) = members2m128(chunk, self.mask1, self.mask2);
        let res0prev0 = _mm_alignr_epi8(res0, *prev0, 15);
        _mm_and_si128(res0prev0, res1)
    }
}

#[derive(Clone, Debug)]
pub struct TeddySlim2Mask256 {
    pub mask1: Mask256,
    pub mask2: Mask256,
}

impl TeddySlim2Mask256 {
    #[target_feature(enable = "avx2")]
    unsafe fn find_at(
        &self,
        pats: &Patterns,
        teddy: &Teddy,
        haystack: &[u8],
        mut at: usize,
    ) -> Option<Match> {
        debug_assert!(haystack[at..].len() >= teddy.minimum_len());
        // This assert helps eliminate bounds checks for bucket lookups in
        // Teddy::verify_bucket, which has a small (3-4%) performance boost.
        assert_eq!(8, teddy.buckets.len());

        at += 1;
        let len = haystack.len();
        let mut prev0 = ones256();
        while at <= len - 32 {
            let c = self.candidate(haystack, at, &mut prev0);
            if !is_all_zeroes256(c) {
                if let Some(m) = teddy.verify256(pats, haystack, at - 1, c) {
                    return Some(m);
                }
            }
            at += 32;
        }
        if at < len {
            at = len - 32;
            prev0 = ones256();

            let c = self.candidate(haystack, at, &mut prev0);
            if !is_all_zeroes256(c) {
                if let Some(m) = teddy.verify256(pats, haystack, at - 1, c) {
                    return Some(m);
                }
            }
        }
        None
    }

    #[inline(always)]
    unsafe fn candidate(
        &self,
        haystack: &[u8],
        at: usize,
        prev0: &mut __m256i,
    ) -> __m256i {
        debug_assert!(haystack[at..].len() >= 32);

        let chunk = loadu256(haystack, at);
        let (res0, res1) = members2m256(chunk, self.mask1, self.mask2);
        let res0prev0 = alignr256_15(res0, *prev0);
        let res = _mm256_and_si256(res0prev0, res1);
        *prev0 = res0;
        res
    }
}

#[derive(Clone, Debug)]
pub struct TeddyFat2Mask256 {
    pub mask1: Mask256,
    pub mask2: Mask256,
}

impl TeddyFat2Mask256 {
    #[target_feature(enable = "avx2")]
    unsafe fn find_at(
        &self,
        pats: &Patterns,
        teddy: &Teddy,
        haystack: &[u8],
        mut at: usize,
    ) -> Option<Match> {
        debug_assert!(haystack[at..].len() >= teddy.minimum_len());
        // This assert helps eliminate bounds checks for bucket lookups in
        // Teddy::verify_bucket, which has a small (3-4%) performance boost.
        assert_eq!(16, teddy.buckets.len());

        at += 1;
        let len = haystack.len();
        let mut prev0 = ones256();
        while at <= len - 16 {
            let c = self.candidate(haystack, at, &mut prev0);
            if !is_all_zeroes256(c) {
                if let Some(m) = teddy.verify_fat256(pats, haystack, at - 1, c)
                {
                    return Some(m);
                }
            }
            at += 16;
        }
        if at < len {
            at = len - 16;
            prev0 = ones256();

            let c = self.candidate(haystack, at, &mut prev0);
            if !is_all_zeroes256(c) {
                if let Some(m) = teddy.verify_fat256(pats, haystack, at - 1, c)
                {
                    return Some(m);
                }
            }
        }
        None
    }

    #[inline(always)]
    unsafe fn candidate(
        &self,
        haystack: &[u8],
        at: usize,
        prev0: &mut __m256i,
    ) -> __m256i {
        debug_assert!(haystack[at..].len() >= 16);

        let chunk = _mm256_broadcastsi128_si256(loadu128(haystack, at));
        let (res0, res1) = members2m256(chunk, self.mask1, self.mask2);
        let res0prev0 = _mm256_alignr_epi8(res0, *prev0, 15);
        let res = _mm256_and_si256(res0prev0, res1);
        *prev0 = res0;
        res
    }
}

#[derive(Clone, Debug)]
pub struct TeddySlim3Mask128 {
    pub mask1: Mask128,
    pub mask2: Mask128,
    pub mask3: Mask128,
}

impl TeddySlim3Mask128 {
    #[target_feature(enable = "ssse3")]
    unsafe fn find_at(
        &self,
        pats: &Patterns,
        teddy: &Teddy,
        haystack: &[u8],
        mut at: usize,
    ) -> Option<Match> {
        debug_assert!(haystack[at..].len() >= teddy.minimum_len());
        // This assert helps eliminate bounds checks for bucket lookups in
        // Teddy::verify_bucket, which has a small (3-4%) performance boost.
        assert_eq!(8, teddy.buckets.len());

        at += 2;
        let len = haystack.len();
        let (mut prev0, mut prev1) = (ones128(), ones128());
        while at <= len - 16 {
            let c = self.candidate(haystack, at, &mut prev0, &mut prev1);
            if !is_all_zeroes128(c) {
                if let Some(m) = teddy.verify128(pats, haystack, at - 2, c) {
                    return Some(m);
                }
            }
            at += 16;
        }
        if at < len {
            at = len - 16;
            prev0 = ones128();
            prev1 = ones128();

            let c = self.candidate(haystack, at, &mut prev0, &mut prev1);
            if !is_all_zeroes128(c) {
                if let Some(m) = teddy.verify128(pats, haystack, at - 2, c) {
                    return Some(m);
                }
            }
        }
        None
    }

    #[inline(always)]
    unsafe fn candidate(
        &self,
        haystack: &[u8],
        at: usize,
        prev0: &mut __m128i,
        prev1: &mut __m128i,
    ) -> __m128i {
        debug_assert!(haystack[at..].len() >= 16);

        let chunk = loadu128(haystack, at);
        let (res0, res1, res2) =
            members3m128(chunk, self.mask1, self.mask2, self.mask3);
        let res0prev0 = _mm_alignr_epi8(res0, *prev0, 14);
        let res1prev1 = _mm_alignr_epi8(res1, *prev1, 15);
        let res = _mm_and_si128(_mm_and_si128(res0prev0, res1prev1), res2);
        *prev0 = res0;
        *prev1 = res1;
        res
    }
}

#[derive(Clone, Debug)]
pub struct TeddySlim3Mask256 {
    pub mask1: Mask256,
    pub mask2: Mask256,
    pub mask3: Mask256,
}

impl TeddySlim3Mask256 {
    #[target_feature(enable = "avx2")]
    unsafe fn find_at(
        &self,
        pats: &Patterns,
        teddy: &Teddy,
        haystack: &[u8],
        mut at: usize,
    ) -> Option<Match> {
        debug_assert!(haystack[at..].len() >= teddy.minimum_len());
        // This assert helps eliminate bounds checks for bucket lookups in
        // Teddy::verify_bucket, which has a small (3-4%) performance boost.
        assert_eq!(8, teddy.buckets.len());

        at += 2;
        let len = haystack.len();
        let (mut prev0, mut prev1) = (ones256(), ones256());
        while at <= len - 32 {
            let c = self.candidate(haystack, at, &mut prev0, &mut prev1);
            if !is_all_zeroes256(c) {
                if let Some(m) = teddy.verify256(pats, haystack, at - 2, c) {
                    return Some(m);
                }
            }
            at += 32;
        }
        if at < len {
            at = len - 32;
            prev0 = ones256();
            prev1 = ones256();

            let c = self.candidate(haystack, at, &mut prev0, &mut prev1);
            if !is_all_zeroes256(c) {
                if let Some(m) = teddy.verify256(pats, haystack, at - 2, c) {
                    return Some(m);
                }
            }
        }
        None
    }

    #[inline(always)]
    unsafe fn candidate(
        &self,
        haystack: &[u8],
        at: usize,
        prev0: &mut __m256i,
        prev1: &mut __m256i,
    ) -> __m256i {
        debug_assert!(haystack[at..].len() >= 32);

        let chunk = loadu256(haystack, at);
        let (res0, res1, res2) =
            members3m256(chunk, self.mask1, self.mask2, self.mask3);
        let res0prev0 = alignr256_14(res0, *prev0);
        let res1prev1 = alignr256_15(res1, *prev1);
        let res =
            _mm256_and_si256(_mm256_and_si256(res0prev0, res1prev1), res2);
        *prev0 = res0;
        *prev1 = res1;
        res
    }
}

#[derive(Clone, Debug)]
pub struct TeddyFat3Mask256 {
    pub mask1: Mask256,
    pub mask2: Mask256,
    pub mask3: Mask256,
}

impl TeddyFat3Mask256 {
    #[target_feature(enable = "avx2")]
    unsafe fn find_at(
        &self,
        pats: &Patterns,
        teddy: &Teddy,
        haystack: &[u8],
        mut at: usize,
    ) -> Option<Match> {
        debug_assert!(haystack[at..].len() >= teddy.minimum_len());
        // This assert helps eliminate bounds checks for bucket lookups in
        // Teddy::verify_bucket, which has a small (3-4%) performance boost.
        assert_eq!(16, teddy.buckets.len());

        at += 2;
        let len = haystack.len();
        let (mut prev0, mut prev1) = (ones256(), ones256());
        while at <= len - 16 {
            let c = self.candidate(haystack, at, &mut prev0, &mut prev1);
            if !is_all_zeroes256(c) {
                if let Some(m) = teddy.verify_fat256(pats, haystack, at - 2, c)
                {
                    return Some(m);
                }
            }
            at += 16;
        }
        if at < len {
            at = len - 16;
            prev0 = ones256();
            prev1 = ones256();

            let c = self.candidate(haystack, at, &mut prev0, &mut prev1);
            if !is_all_zeroes256(c) {
                if let Some(m) = teddy.verify_fat256(pats, haystack, at - 2, c)
                {
                    return Some(m);
                }
            }
        }
        None
    }

    #[inline(always)]
    unsafe fn candidate(
        &self,
        haystack: &[u8],
        at: usize,
        prev0: &mut __m256i,
        prev1: &mut __m256i,
    ) -> __m256i {
        debug_assert!(haystack[at..].len() >= 16);

        let chunk = _mm256_broadcastsi128_si256(loadu128(haystack, at));
        let (res0, res1, res2) =
            members3m256(chunk, self.mask1, self.mask2, self.mask3);
        let res0prev0 = _mm256_alignr_epi8(res0, *prev0, 14);
        let res1prev1 = _mm256_alignr_epi8(res1, *prev1, 15);
        let res =
            _mm256_and_si256(_mm256_and_si256(res0prev0, res1prev1), res2);
        *prev0 = res0;
        *prev1 = res1;
        res
    }
}

/// A 128-bit mask for the low and high nybbles in a set of patterns. Each
/// lane `j` corresponds to a bitset where the `i`th bit is set if and only if
/// the nybble `j` is in the bucket `i` at a particular position.
#[derive(Clone, Copy, Debug)]
pub struct Mask128 {
    lo: __m128i,
    hi: __m128i,
}

impl Mask128 {
    /// Create a new SIMD mask from the mask produced by the Teddy builder.
    pub fn new(mask: compile::Mask) -> Mask128 {
        // SAFETY: This is safe since [u8; 16] has the same representation
        // as __m128i.
        unsafe {
            Mask128 {
                lo: mem::transmute(mask.lo128()),
                hi: mem::transmute(mask.hi128()),
            }
        }
    }
}

/// A 256-bit mask for the low and high nybbles in a set of patterns. Each
/// lane `j` corresponds to a bitset where the `i`th bit is set if and only if
/// the nybble `j` is in the bucket `i` at a particular position.
///
/// This is slightly tweaked dependending on whether Slim or Fat Teddy is being
/// used. For Slim Teddy, the bitsets in the lower 128-bits are the same as
/// the bitsets in the higher 128-bits, so that we can search 32 bytes at a
/// time. (Remember, the nybbles in the haystack are used as indices into these
/// masks, and 256-bit shuffles only operate on 128-bit lanes.)
///
/// For Fat Teddy, the bitsets are not repeated, but instead, the high 128
/// bits correspond to buckets 8-15. So that a bitset `00100010` has buckets
/// 1 and 5 set if it's in the lower 128 bits, but has buckets 9 and 13 set
/// if it's in the higher 128 bits.
#[derive(Clone, Copy, Debug)]
pub struct Mask256 {
    lo: __m256i,
    hi: __m256i,
}

impl Mask256 {
    /// Create a new SIMD mask from the mask produced by the Teddy builder.
    pub fn new(mask: compile::Mask) -> Mask256 {
        // SAFETY: This is safe since [u8; 32] has the same representation
        // as __m256i.
        unsafe {
            Mask256 {
                lo: mem::transmute(mask.lo256()),
                hi: mem::transmute(mask.hi256()),
            }
        }
    }
}

// The "members" routines below are responsible for taking a chunk of bytes,
// a number of nybble masks and returning the result of using the masks to
// lookup bytes in the chunk. The results of the high and low nybble masks are
// AND'ed together, such that each candidate returned is a vector, with byte
// sized lanes, and where each lane is an 8-bit bitset corresponding to the
// buckets that contain the corresponding byte.
//
// In the case of masks of length greater than 1, callers will need to keep
// the results from the previous haystack's window, and then shift the vectors
// so that they all line up. Then they can be AND'ed together.

/// Return a candidate for Slim 128-bit Teddy, where `chunk` corresponds to a
/// 16-byte window of the haystack (where the least significant byte
/// corresponds to the start of the window), and `mask1` corresponds to a
/// low/high mask for the first byte of all patterns that are being searched.
#[target_feature(enable = "ssse3")]
unsafe fn members1m128(chunk: __m128i, mask1: Mask128) -> __m128i {
    let lomask = _mm_set1_epi8(0xF);
    let hlo = _mm_and_si128(chunk, lomask);
    let hhi = _mm_and_si128(_mm_srli_epi16(chunk, 4), lomask);
    _mm_and_si128(
        _mm_shuffle_epi8(mask1.lo, hlo),
        _mm_shuffle_epi8(mask1.hi, hhi),
    )
}

/// Return a candidate for Slim 256-bit Teddy, where `chunk` corresponds to a
/// 32-byte window of the haystack (where the least significant byte
/// corresponds to the start of the window), and `mask1` corresponds to a
/// low/high mask for the first byte of all patterns that are being searched.
///
/// Note that this can also be used for Fat Teddy, where the high 128 bits in
/// `chunk` is the same as the low 128 bits, which corresponds to a 16 byte
/// window in the haystack.
#[target_feature(enable = "avx2")]
unsafe fn members1m256(chunk: __m256i, mask1: Mask256) -> __m256i {
    let lomask = _mm256_set1_epi8(0xF);
    let hlo = _mm256_and_si256(chunk, lomask);
    let hhi = _mm256_and_si256(_mm256_srli_epi16(chunk, 4), lomask);
    _mm256_and_si256(
        _mm256_shuffle_epi8(mask1.lo, hlo),
        _mm256_shuffle_epi8(mask1.hi, hhi),
    )
}

/// Return candidates for Slim 128-bit Teddy, where `chunk` corresponds
/// to a 16-byte window of the haystack (where the least significant byte
/// corresponds to the start of the window), and the masks correspond to a
/// low/high mask for the first and second bytes of all patterns that are being
/// searched. The vectors returned correspond to candidates for the first and
/// second bytes in the patterns represented by the masks.
#[target_feature(enable = "ssse3")]
unsafe fn members2m128(
    chunk: __m128i,
    mask1: Mask128,
    mask2: Mask128,
) -> (__m128i, __m128i) {
    let lomask = _mm_set1_epi8(0xF);
    let hlo = _mm_and_si128(chunk, lomask);
    let hhi = _mm_and_si128(_mm_srli_epi16(chunk, 4), lomask);
    let res0 = _mm_and_si128(
        _mm_shuffle_epi8(mask1.lo, hlo),
        _mm_shuffle_epi8(mask1.hi, hhi),
    );
    let res1 = _mm_and_si128(
        _mm_shuffle_epi8(mask2.lo, hlo),
        _mm_shuffle_epi8(mask2.hi, hhi),
    );
    (res0, res1)
}

/// Return candidates for Slim 256-bit Teddy, where `chunk` corresponds
/// to a 32-byte window of the haystack (where the least significant byte
/// corresponds to the start of the window), and the masks correspond to a
/// low/high mask for the first and second bytes of all patterns that are being
/// searched. The vectors returned correspond to candidates for the first and
/// second bytes in the patterns represented by the masks.
///
/// Note that this can also be used for Fat Teddy, where the high 128 bits in
/// `chunk` is the same as the low 128 bits, which corresponds to a 16 byte
/// window in the haystack.
#[target_feature(enable = "avx2")]
unsafe fn members2m256(
    chunk: __m256i,
    mask1: Mask256,
    mask2: Mask256,
) -> (__m256i, __m256i) {
    let lomask = _mm256_set1_epi8(0xF);
    let hlo = _mm256_and_si256(chunk, lomask);
    let hhi = _mm256_and_si256(_mm256_srli_epi16(chunk, 4), lomask);
    let res0 = _mm256_and_si256(
        _mm256_shuffle_epi8(mask1.lo, hlo),
        _mm256_shuffle_epi8(mask1.hi, hhi),
    );
    let res1 = _mm256_and_si256(
        _mm256_shuffle_epi8(mask2.lo, hlo),
        _mm256_shuffle_epi8(mask2.hi, hhi),
    );
    (res0, res1)
}

/// Return candidates for Slim 128-bit Teddy, where `chunk` corresponds
/// to a 16-byte window of the haystack (where the least significant byte
/// corresponds to the start of the window), and the masks correspond to a
/// low/high mask for the first, second and third bytes of all patterns that
/// are being searched. The vectors returned correspond to candidates for the
/// first, second and third bytes in the patterns represented by the masks.
#[target_feature(enable = "ssse3")]
unsafe fn members3m128(
    chunk: __m128i,
    mask1: Mask128,
    mask2: Mask128,
    mask3: Mask128,
) -> (__m128i, __m128i, __m128i) {
    let lomask = _mm_set1_epi8(0xF);
    let hlo = _mm_and_si128(chunk, lomask);
    let hhi = _mm_and_si128(_mm_srli_epi16(chunk, 4), lomask);
    let res0 = _mm_and_si128(
        _mm_shuffle_epi8(mask1.lo, hlo),
        _mm_shuffle_epi8(mask1.hi, hhi),
    );
    let res1 = _mm_and_si128(
        _mm_shuffle_epi8(mask2.lo, hlo),
        _mm_shuffle_epi8(mask2.hi, hhi),
    );
    let res2 = _mm_and_si128(
        _mm_shuffle_epi8(mask3.lo, hlo),
        _mm_shuffle_epi8(mask3.hi, hhi),
    );
    (res0, res1, res2)
}

/// Return candidates for Slim 256-bit Teddy, where `chunk` corresponds
/// to a 32-byte window of the haystack (where the least significant byte
/// corresponds to the start of the window), and the masks correspond to a
/// low/high mask for the first, second and third bytes of all patterns that
/// are being searched. The vectors returned correspond to candidates for the
/// first, second and third bytes in the patterns represented by the masks.
///
/// Note that this can also be used for Fat Teddy, where the high 128 bits in
/// `chunk` is the same as the low 128 bits, which corresponds to a 16 byte
/// window in the haystack.
#[target_feature(enable = "avx2")]
unsafe fn members3m256(
    chunk: __m256i,
    mask1: Mask256,
    mask2: Mask256,
    mask3: Mask256,
) -> (__m256i, __m256i, __m256i) {
    let lomask = _mm256_set1_epi8(0xF);
    let hlo = _mm256_and_si256(chunk, lomask);
    let hhi = _mm256_and_si256(_mm256_srli_epi16(chunk, 4), lomask);
    let res0 = _mm256_and_si256(
        _mm256_shuffle_epi8(mask1.lo, hlo),
        _mm256_shuffle_epi8(mask1.hi, hhi),
    );
    let res1 = _mm256_and_si256(
        _mm256_shuffle_epi8(mask2.lo, hlo),
        _mm256_shuffle_epi8(mask2.hi, hhi),
    );
    let res2 = _mm256_and_si256(
        _mm256_shuffle_epi8(mask3.lo, hlo),
        _mm256_shuffle_epi8(mask3.hi, hhi),
    );
    (res0, res1, res2)
}
