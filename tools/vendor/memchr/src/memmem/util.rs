// These routines are meant to be optimized specifically for low latency as
// compared to the equivalent routines offered by std. (Which may invoke the
// dynamic linker and call out to libc, which introduces a bit more latency
// than we'd like.)

/// Returns true if and only if needle is a prefix of haystack.
#[inline(always)]
pub(crate) fn is_prefix(haystack: &[u8], needle: &[u8]) -> bool {
    needle.len() <= haystack.len() && memcmp(&haystack[..needle.len()], needle)
}

/// Returns true if and only if needle is a suffix of haystack.
#[inline(always)]
pub(crate) fn is_suffix(haystack: &[u8], needle: &[u8]) -> bool {
    needle.len() <= haystack.len()
        && memcmp(&haystack[haystack.len() - needle.len()..], needle)
}

/// Return true if and only if x.len() == y.len() && x[i] == y[i] for all
/// 0 <= i < x.len().
///
/// Why not just use actual memcmp for this? Well, memcmp requires calling out
/// to libc, and this routine is called in fairly hot code paths. Other than
/// just calling out to libc, it also seems to result in worse codegen. By
/// rolling our own memcmp in pure Rust, it seems to appear more friendly to
/// the optimizer.
///
/// We mark this as inline always, although, some callers may not want it
/// inlined for better codegen (like Rabin-Karp). In that case, callers are
/// advised to create a non-inlineable wrapper routine that calls memcmp.
#[inline(always)]
pub(crate) fn memcmp(x: &[u8], y: &[u8]) -> bool {
    if x.len() != y.len() {
        return false;
    }
    // If we don't have enough bytes to do 4-byte at a time loads, then
    // fall back to the naive slow version.
    //
    // TODO: We could do a copy_nonoverlapping combined with a mask instead
    // of a loop. Benchmark it.
    if x.len() < 4 {
        for (&b1, &b2) in x.iter().zip(y) {
            if b1 != b2 {
                return false;
            }
        }
        return true;
    }
    // When we have 4 or more bytes to compare, then proceed in chunks of 4 at
    // a time using unaligned loads.
    //
    // Also, why do 4 byte loads instead of, say, 8 byte loads? The reason is
    // that this particular version of memcmp is likely to be called with tiny
    // needles. That means that if we do 8 byte loads, then a higher proportion
    // of memcmp calls will use the slower variant above. With that said, this
    // is a hypothesis and is only loosely supported by benchmarks. There's
    // likely some improvement that could be made here. The main thing here
    // though is to optimize for latency, not throughput.

    // SAFETY: Via the conditional above, we know that both `px` and `py`
    // have the same length, so `px < pxend` implies that `py < pyend`.
    // Thus, derefencing both `px` and `py` in the loop below is safe.
    //
    // Moreover, we set `pxend` and `pyend` to be 4 bytes before the actual
    // end of of `px` and `py`. Thus, the final dereference outside of the
    // loop is guaranteed to be valid. (The final comparison will overlap with
    // the last comparison done in the loop for lengths that aren't multiples
    // of four.)
    //
    // Finally, we needn't worry about alignment here, since we do unaligned
    // loads.
    unsafe {
        let (mut px, mut py) = (x.as_ptr(), y.as_ptr());
        let (pxend, pyend) = (px.add(x.len() - 4), py.add(y.len() - 4));
        while px < pxend {
            let vx = (px as *const u32).read_unaligned();
            let vy = (py as *const u32).read_unaligned();
            if vx != vy {
                return false;
            }
            px = px.add(4);
            py = py.add(4);
        }
        let vx = (pxend as *const u32).read_unaligned();
        let vy = (pyend as *const u32).read_unaligned();
        vx == vy
    }
}
