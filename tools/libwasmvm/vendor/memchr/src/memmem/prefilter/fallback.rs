/*
This module implements a "fallback" prefilter that only relies on memchr to
function. While memchr works best when it's explicitly vectorized, its
fallback implementations are fast enough to make a prefilter like this
worthwhile.

The essence of this implementation is to identify two rare bytes in a needle
based on a background frequency distribution of bytes. We then run memchr on the
rarer byte. For each match, we use the second rare byte as a guard to quickly
check if a match is possible. If the position passes the guard test, then we do
a naive memcmp to confirm the match.

In practice, this formulation works amazingly well, primarily because of the
heuristic use of a background frequency distribution. However, it does have a
number of weaknesses where it can get quite slow when its background frequency
distribution doesn't line up with the haystack being searched. This is why we
have specialized vector routines that essentially take this idea and move the
guard check into vectorized code. (Those specialized vector routines do still
make use of the background frequency distribution of bytes though.)

This fallback implementation was originally formulated in regex many moons ago:
https://github.com/rust-lang/regex/blob/3db8722d0b204a85380fe2a65e13d7065d7dd968/src/literal/imp.rs#L370-L501
Prior to that, I'm not aware of anyone using this technique in any prominent
substring search implementation. Although, I'm sure folks have had this same
insight long before me.

Another version of this also appeared in bstr:
https://github.com/BurntSushi/bstr/blob/a444256ca7407fe180ee32534688549655b7a38e/src/search/prefilter.rs#L83-L340
*/

use crate::memmem::{
    prefilter::{PrefilterFnTy, PrefilterState},
    NeedleInfo,
};

// Check that the functions below satisfy the Prefilter function type.
const _: PrefilterFnTy = find;

/// Look for a possible occurrence of needle. The position returned
/// corresponds to the beginning of the occurrence, if one exists.
///
/// Callers may assume that this never returns false negatives (i.e., it
/// never misses an actual occurrence), but must check that the returned
/// position corresponds to a match. That is, it can return false
/// positives.
///
/// This should only be used when Freqy is constructed for forward
/// searching.
pub(crate) fn find(
    prestate: &mut PrefilterState,
    ninfo: &NeedleInfo,
    haystack: &[u8],
    needle: &[u8],
) -> Option<usize> {
    let mut i = 0;
    let (rare1i, rare2i) = ninfo.rarebytes.as_rare_usize();
    let (rare1, rare2) = ninfo.rarebytes.as_rare_bytes(needle);
    while prestate.is_effective() {
        // Use a fast vectorized implementation to skip to the next
        // occurrence of the rarest byte (heuristically chosen) in the
        // needle.
        let found = crate::memchr(rare1, &haystack[i..])?;
        prestate.update(found);
        i += found;

        // If we can't align our first match with the haystack, then a
        // match is impossible.
        if i < rare1i {
            i += 1;
            continue;
        }

        // Align our rare2 byte with the haystack. A mismatch means that
        // a match is impossible.
        let aligned_rare2i = i - rare1i + rare2i;
        if haystack.get(aligned_rare2i) != Some(&rare2) {
            i += 1;
            continue;
        }

        // We've done what we can. There might be a match here.
        return Some(i - rare1i);
    }
    // The only way we get here is if we believe our skipping heuristic
    // has become ineffective. We're allowed to return false positives,
    // so return the position at which we advanced to, aligned to the
    // haystack.
    Some(i.saturating_sub(rare1i))
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    fn freqy_find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        let ninfo = NeedleInfo::new(needle);
        let mut prestate = PrefilterState::new();
        find(&mut prestate, &ninfo, haystack, needle)
    }

    #[test]
    fn freqy_forward() {
        assert_eq!(Some(0), freqy_find(b"BARFOO", b"BAR"));
        assert_eq!(Some(3), freqy_find(b"FOOBAR", b"BAR"));
        assert_eq!(Some(0), freqy_find(b"zyzz", b"zyzy"));
        assert_eq!(Some(2), freqy_find(b"zzzy", b"zyzy"));
        assert_eq!(None, freqy_find(b"zazb", b"zyzy"));
        assert_eq!(Some(0), freqy_find(b"yzyy", b"yzyz"));
        assert_eq!(Some(2), freqy_find(b"yyyz", b"yzyz"));
        assert_eq!(None, freqy_find(b"yayb", b"yzyz"));
    }

    #[test]
    #[cfg(not(miri))]
    fn prefilter_permutations() {
        use crate::memmem::prefilter::tests::PrefilterTest;

        // SAFETY: super::find is safe to call for all inputs and on all
        // platforms.
        unsafe { PrefilterTest::run_all_tests(super::find) };
    }
}
