use core::mem::size_of;

use crate::memmem::{util::memcmp, vector::Vector, NeedleInfo};

/// The minimum length of a needle required for this algorithm. The minimum
/// is 2 since a length of 1 should just use memchr and a length of 0 isn't
/// a case handled by this searcher.
pub(crate) const MIN_NEEDLE_LEN: usize = 2;

/// The maximum length of a needle required for this algorithm.
///
/// In reality, there is no hard max here. The code below can handle any
/// length needle. (Perhaps that suggests there are missing optimizations.)
/// Instead, this is a heuristic and a bound guaranteeing our linear time
/// complexity.
///
/// It is a heuristic because when a candidate match is found, memcmp is run.
/// For very large needles with lots of false positives, memcmp can make the
/// code run quite slow.
///
/// It is a bound because the worst case behavior with memcmp is multiplicative
/// in the size of the needle and haystack, and we want to keep that additive.
/// This bound ensures we still meet that bound theoretically, since it's just
/// a constant. We aren't acting in bad faith here, memcmp on tiny needles
/// is so fast that even in pathological cases (see pathological vector
/// benchmarks), this is still just as fast or faster in practice.
///
/// This specific number was chosen by tweaking a bit and running benchmarks.
/// The rare-medium-needle, for example, gets about 5% faster by using this
/// algorithm instead of a prefilter-accelerated Two-Way. There's also a
/// theoretical desire to keep this number reasonably low, to mitigate the
/// impact of pathological cases. I did try 64, and some benchmarks got a
/// little better, and others (particularly the pathological ones), got a lot
/// worse. So... 32 it is?
pub(crate) const MAX_NEEDLE_LEN: usize = 32;

/// The implementation of the forward vector accelerated substring search.
///
/// This is extremely similar to the prefilter vector module by the same name.
/// The key difference is that this is not a prefilter. Instead, it handles
/// confirming its own matches. The trade off is that this only works with
/// smaller needles. The speed up here is that an inlined memcmp on a tiny
/// needle is very quick, even on pathological inputs. This is much better than
/// combining a prefilter with Two-Way, where using Two-Way to confirm the
/// match has higher latency.
///
/// So why not use this for all needles? We could, and it would probably work
/// really well on most inputs. But its worst case is multiplicative and we
/// want to guarantee worst case additive time. Some of the benchmarks try to
/// justify this (see the pathological ones).
///
/// The prefilter variant of this has more comments. Also note that we only
/// implement this for forward searches for now. If you have a compelling use
/// case for accelerated reverse search, please file an issue.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Forward {
    rare1i: u8,
    rare2i: u8,
}

impl Forward {
    /// Create a new "generic simd" forward searcher. If one could not be
    /// created from the given inputs, then None is returned.
    pub(crate) fn new(ninfo: &NeedleInfo, needle: &[u8]) -> Option<Forward> {
        let (rare1i, rare2i) = ninfo.rarebytes.as_rare_ordered_u8();
        // If the needle is too short or too long, give up. Also, give up
        // if the rare bytes detected are at the same position. (It likely
        // suggests a degenerate case, although it should technically not be
        // possible.)
        if needle.len() < MIN_NEEDLE_LEN
            || needle.len() > MAX_NEEDLE_LEN
            || rare1i == rare2i
        {
            return None;
        }
        Some(Forward { rare1i, rare2i })
    }

    /// Returns the minimum length of haystack that is needed for this searcher
    /// to work for a particular vector. Passing a haystack with a length
    /// smaller than this will cause `fwd_find` to panic.
    #[inline(always)]
    pub(crate) fn min_haystack_len<V: Vector>(&self) -> usize {
        self.rare2i as usize + size_of::<V>()
    }
}

/// Searches the given haystack for the given needle. The needle given should
/// be the same as the needle that this searcher was initialized with.
///
/// # Panics
///
/// When the given haystack has a length smaller than `min_haystack_len`.
///
/// # Safety
///
/// Since this is meant to be used with vector functions, callers need to
/// specialize this inside of a function with a `target_feature` attribute.
/// Therefore, callers must ensure that whatever target feature is being used
/// supports the vector functions that this function is specialized for. (For
/// the specific vector functions used, see the Vector trait implementations.)
#[inline(always)]
pub(crate) unsafe fn fwd_find<V: Vector>(
    fwd: &Forward,
    haystack: &[u8],
    needle: &[u8],
) -> Option<usize> {
    // It would be nice if we didn't have this check here, since the meta
    // searcher should handle it for us. But without this, I don't think we
    // guarantee that end_ptr.sub(needle.len()) won't result in UB. We could
    // put it as part of the safety contract, but it makes it more complicated
    // than necessary.
    if haystack.len() < needle.len() {
        return None;
    }
    let min_haystack_len = fwd.min_haystack_len::<V>();
    assert!(haystack.len() >= min_haystack_len, "haystack too small");
    debug_assert!(needle.len() <= haystack.len());
    debug_assert!(
        needle.len() >= MIN_NEEDLE_LEN,
        "needle must be at least {} bytes",
        MIN_NEEDLE_LEN,
    );
    debug_assert!(
        needle.len() <= MAX_NEEDLE_LEN,
        "needle must be at most {} bytes",
        MAX_NEEDLE_LEN,
    );

    let (rare1i, rare2i) = (fwd.rare1i as usize, fwd.rare2i as usize);
    let rare1chunk = V::splat(needle[rare1i]);
    let rare2chunk = V::splat(needle[rare2i]);

    let start_ptr = haystack.as_ptr();
    let end_ptr = start_ptr.add(haystack.len());
    let max_ptr = end_ptr.sub(min_haystack_len);
    let mut ptr = start_ptr;

    // N.B. I did experiment with unrolling the loop to deal with size(V)
    // bytes at a time and 2*size(V) bytes at a time. The double unroll was
    // marginally faster while the quadruple unroll was unambiguously slower.
    // In the end, I decided the complexity from unrolling wasn't worth it. I
    // used the memmem/krate/prebuilt/huge-en/ benchmarks to compare.
    while ptr <= max_ptr {
        let m = fwd_find_in_chunk(
            fwd, needle, ptr, end_ptr, rare1chunk, rare2chunk, !0,
        );
        if let Some(chunki) = m {
            return Some(matched(start_ptr, ptr, chunki));
        }
        ptr = ptr.add(size_of::<V>());
    }
    if ptr < end_ptr {
        let remaining = diff(end_ptr, ptr);
        debug_assert!(
            remaining < min_haystack_len,
            "remaining bytes should be smaller than the minimum haystack \
             length of {}, but there are {} bytes remaining",
            min_haystack_len,
            remaining,
        );
        if remaining < needle.len() {
            return None;
        }
        debug_assert!(
            max_ptr < ptr,
            "after main loop, ptr should have exceeded max_ptr",
        );
        let overlap = diff(ptr, max_ptr);
        debug_assert!(
            overlap > 0,
            "overlap ({}) must always be non-zero",
            overlap,
        );
        debug_assert!(
            overlap < size_of::<V>(),
            "overlap ({}) cannot possibly be >= than a vector ({})",
            overlap,
            size_of::<V>(),
        );
        // The mask has all of its bits set except for the first N least
        // significant bits, where N=overlap. This way, any matches that
        // occur in find_in_chunk within the overlap are automatically
        // ignored.
        let mask = !((1 << overlap) - 1);
        ptr = max_ptr;
        let m = fwd_find_in_chunk(
            fwd, needle, ptr, end_ptr, rare1chunk, rare2chunk, mask,
        );
        if let Some(chunki) = m {
            return Some(matched(start_ptr, ptr, chunki));
        }
    }
    None
}

/// Search for an occurrence of two rare bytes from the needle in the chunk
/// pointed to by ptr, with the end of the haystack pointed to by end_ptr. When
/// an occurrence is found, memcmp is run to check if a match occurs at the
/// corresponding position.
///
/// rare1chunk and rare2chunk correspond to vectors with the rare1 and rare2
/// bytes repeated in each 8-bit lane, respectively.
///
/// mask should have bits set corresponding the positions in the chunk in which
/// matches are considered. This is only used for the last vector load where
/// the beginning of the vector might have overlapped with the last load in
/// the main loop. The mask lets us avoid visiting positions that have already
/// been discarded as matches.
///
/// # Safety
///
/// It must be safe to do an unaligned read of size(V) bytes starting at both
/// (ptr + rare1i) and (ptr + rare2i). It must also be safe to do unaligned
/// loads on ptr up to (end_ptr - needle.len()).
#[inline(always)]
unsafe fn fwd_find_in_chunk<V: Vector>(
    fwd: &Forward,
    needle: &[u8],
    ptr: *const u8,
    end_ptr: *const u8,
    rare1chunk: V,
    rare2chunk: V,
    mask: u32,
) -> Option<usize> {
    let chunk0 = V::load_unaligned(ptr.add(fwd.rare1i as usize));
    let chunk1 = V::load_unaligned(ptr.add(fwd.rare2i as usize));

    let eq0 = chunk0.cmpeq(rare1chunk);
    let eq1 = chunk1.cmpeq(rare2chunk);

    let mut match_offsets = eq0.and(eq1).movemask() & mask;
    while match_offsets != 0 {
        let offset = match_offsets.trailing_zeros() as usize;
        let ptr = ptr.add(offset);
        if end_ptr.sub(needle.len()) < ptr {
            return None;
        }
        let chunk = core::slice::from_raw_parts(ptr, needle.len());
        if memcmp(needle, chunk) {
            return Some(offset);
        }
        match_offsets &= match_offsets - 1;
    }
    None
}

/// Accepts a chunk-relative offset and returns a haystack relative offset
/// after updating the prefilter state.
///
/// See the same function with the same name in the prefilter variant of this
/// algorithm to learned why it's tagged with inline(never). Even here, where
/// the function is simpler, inlining it leads to poorer codegen. (Although
/// it does improve some benchmarks, like prebuiltiter/huge-en/common-you.)
#[cold]
#[inline(never)]
fn matched(start_ptr: *const u8, ptr: *const u8, chunki: usize) -> usize {
    diff(ptr, start_ptr) + chunki
}

/// Subtract `b` from `a` and return the difference. `a` must be greater than
/// or equal to `b`.
fn diff(a: *const u8, b: *const u8) -> usize {
    debug_assert!(a >= b);
    (a as usize) - (b as usize)
}
