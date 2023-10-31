use core::mem::size_of;

use crate::memmem::{
    prefilter::{PrefilterFnTy, PrefilterState},
    vector::Vector,
    NeedleInfo,
};

/// The implementation of the forward vector accelerated candidate finder.
///
/// This is inspired by the "generic SIMD" algorithm described here:
/// http://0x80.pl/articles/simd-strfind.html#algorithm-1-generic-simd
///
/// The main difference is that this is just a prefilter. That is, it reports
/// candidates once they are seen and doesn't attempt to confirm them. Also,
/// the bytes this routine uses to check for candidates are selected based on
/// an a priori background frequency distribution. This means that on most
/// haystacks, this will on average spend more time in vectorized code than you
/// would if you just selected the first and last bytes of the needle.
///
/// Note that a non-prefilter variant of this algorithm can be found in the
/// parent module, but it only works on smaller needles.
///
/// `prestate`, `ninfo`, `haystack` and `needle` are the four prefilter
/// function parameters. `fallback` is a prefilter that is used if the haystack
/// is too small to be handled with the given vector size.
///
/// This routine is not safe because it is intended for callers to specialize
/// this with a particular vector (e.g., __m256i) and then call it with the
/// relevant target feature (e.g., avx2) enabled.
///
/// # Panics
///
/// If `needle.len() <= 1`, then this panics.
///
/// # Safety
///
/// Since this is meant to be used with vector functions, callers need to
/// specialize this inside of a function with a `target_feature` attribute.
/// Therefore, callers must ensure that whatever target feature is being used
/// supports the vector functions that this function is specialized for. (For
/// the specific vector functions used, see the Vector trait implementations.)
#[inline(always)]
pub(crate) unsafe fn find<V: Vector>(
    prestate: &mut PrefilterState,
    ninfo: &NeedleInfo,
    haystack: &[u8],
    needle: &[u8],
    fallback: PrefilterFnTy,
) -> Option<usize> {
    assert!(needle.len() >= 2, "needle must be at least 2 bytes");
    let (rare1i, rare2i) = ninfo.rarebytes.as_rare_ordered_usize();
    let min_haystack_len = rare2i + size_of::<V>();
    if haystack.len() < min_haystack_len {
        return fallback(prestate, ninfo, haystack, needle);
    }

    let start_ptr = haystack.as_ptr();
    let end_ptr = start_ptr.add(haystack.len());
    let max_ptr = end_ptr.sub(min_haystack_len);
    let mut ptr = start_ptr;

    let rare1chunk = V::splat(needle[rare1i]);
    let rare2chunk = V::splat(needle[rare2i]);

    // N.B. I did experiment with unrolling the loop to deal with size(V)
    // bytes at a time and 2*size(V) bytes at a time. The double unroll
    // was marginally faster while the quadruple unroll was unambiguously
    // slower. In the end, I decided the complexity from unrolling wasn't
    // worth it. I used the memmem/krate/prebuilt/huge-en/ benchmarks to
    // compare.
    while ptr <= max_ptr {
        let m = find_in_chunk2(ptr, rare1i, rare2i, rare1chunk, rare2chunk);
        if let Some(chunki) = m {
            return Some(matched(prestate, start_ptr, ptr, chunki));
        }
        ptr = ptr.add(size_of::<V>());
    }
    if ptr < end_ptr {
        // This routine immediately quits if a candidate match is found.
        // That means that if we're here, no candidate matches have been
        // found at or before 'ptr'. Thus, we don't need to mask anything
        // out even though we might technically search part of the haystack
        // that we've already searched (because we know it can't match).
        ptr = max_ptr;
        let m = find_in_chunk2(ptr, rare1i, rare2i, rare1chunk, rare2chunk);
        if let Some(chunki) = m {
            return Some(matched(prestate, start_ptr, ptr, chunki));
        }
    }
    prestate.update(haystack.len());
    None
}

// Below are two different techniques for checking whether a candidate
// match exists in a given chunk or not. find_in_chunk2 checks two bytes
// where as find_in_chunk3 checks three bytes. The idea behind checking
// three bytes is that while we do a bit more work per iteration, we
// decrease the chances of a false positive match being reported and thus
// make the search faster overall. This actually works out for the
// memmem/krate/prebuilt/huge-en/never-all-common-bytes benchmark, where
// using find_in_chunk3 is about 25% faster than find_in_chunk2. However,
// it turns out that find_in_chunk2 is faster for all other benchmarks, so
// perhaps the extra check isn't worth it in practice.
//
// For now, we go with find_in_chunk2, but we leave find_in_chunk3 around
// to make it easy to switch to and benchmark when possible.

/// Search for an occurrence of two rare bytes from the needle in the current
/// chunk pointed to by ptr.
///
/// rare1chunk and rare2chunk correspond to vectors with the rare1 and rare2
/// bytes repeated in each 8-bit lane, respectively.
///
/// # Safety
///
/// It must be safe to do an unaligned read of size(V) bytes starting at both
/// (ptr + rare1i) and (ptr + rare2i).
#[inline(always)]
unsafe fn find_in_chunk2<V: Vector>(
    ptr: *const u8,
    rare1i: usize,
    rare2i: usize,
    rare1chunk: V,
    rare2chunk: V,
) -> Option<usize> {
    let chunk0 = V::load_unaligned(ptr.add(rare1i));
    let chunk1 = V::load_unaligned(ptr.add(rare2i));

    let eq0 = chunk0.cmpeq(rare1chunk);
    let eq1 = chunk1.cmpeq(rare2chunk);

    let match_offsets = eq0.and(eq1).movemask();
    if match_offsets == 0 {
        return None;
    }
    Some(match_offsets.trailing_zeros() as usize)
}

/// Search for an occurrence of two rare bytes and the first byte (even if one
/// of the rare bytes is equivalent to the first byte) from the needle in the
/// current chunk pointed to by ptr.
///
/// firstchunk, rare1chunk and rare2chunk correspond to vectors with the first,
/// rare1 and rare2 bytes repeated in each 8-bit lane, respectively.
///
/// # Safety
///
/// It must be safe to do an unaligned read of size(V) bytes starting at ptr,
/// (ptr + rare1i) and (ptr + rare2i).
#[allow(dead_code)]
#[inline(always)]
unsafe fn find_in_chunk3<V: Vector>(
    ptr: *const u8,
    rare1i: usize,
    rare2i: usize,
    firstchunk: V,
    rare1chunk: V,
    rare2chunk: V,
) -> Option<usize> {
    let chunk0 = V::load_unaligned(ptr);
    let chunk1 = V::load_unaligned(ptr.add(rare1i));
    let chunk2 = V::load_unaligned(ptr.add(rare2i));

    let eq0 = chunk0.cmpeq(firstchunk);
    let eq1 = chunk1.cmpeq(rare1chunk);
    let eq2 = chunk2.cmpeq(rare2chunk);

    let match_offsets = eq0.and(eq1).and(eq2).movemask();
    if match_offsets == 0 {
        return None;
    }
    Some(match_offsets.trailing_zeros() as usize)
}

/// Accepts a chunk-relative offset and returns a haystack relative offset
/// after updating the prefilter state.
///
/// Why do we use this unlineable function when a search completes? Well,
/// I don't know. Really. Obviously this function was not here initially.
/// When doing profiling, the codegen for the inner loop here looked bad and
/// I didn't know why. There were a couple extra 'add' instructions and an
/// extra 'lea' instruction that I couldn't explain. I hypothesized that the
/// optimizer was having trouble untangling the hot code in the loop from the
/// code that deals with a candidate match. By putting the latter into an
/// unlineable function, it kind of forces the issue and it had the intended
/// effect: codegen improved measurably. It's good for a ~10% improvement
/// across the board on the memmem/krate/prebuilt/huge-en/ benchmarks.
#[cold]
#[inline(never)]
fn matched(
    prestate: &mut PrefilterState,
    start_ptr: *const u8,
    ptr: *const u8,
    chunki: usize,
) -> usize {
    let found = diff(ptr, start_ptr) + chunki;
    prestate.update(found);
    found
}

/// Subtract `b` from `a` and return the difference. `a` must be greater than
/// or equal to `b`.
fn diff(a: *const u8, b: *const u8) -> usize {
    debug_assert!(a >= b);
    (a as usize) - (b as usize)
}
