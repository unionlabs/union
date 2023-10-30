/// A heuristic frequency based detection of rare bytes for substring search.
///
/// This detector attempts to pick out two bytes in a needle that are predicted
/// to occur least frequently. The purpose is to use these bytes to implement
/// fast candidate search using vectorized code.
///
/// A set of offsets is only computed for needles of length 2 or greater.
/// Smaller needles should be special cased by the substring search algorithm
/// in use. (e.g., Use memchr for single byte needles.)
///
/// Note that we use `u8` to represent the offsets of the rare bytes in a
/// needle to reduce space usage. This means that rare byte occurring after the
/// first 255 bytes in a needle will never be used.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RareNeedleBytes {
    /// The leftmost offset of the rarest byte in the needle, according to
    /// pre-computed frequency analysis. The "leftmost offset" means that
    /// rare1i <= i for all i where needle[i] == needle[rare1i].
    rare1i: u8,
    /// The leftmost offset of the second rarest byte in the needle, according
    /// to pre-computed frequency analysis. The "leftmost offset" means that
    /// rare2i <= i for all i where needle[i] == needle[rare2i].
    ///
    /// The second rarest byte is used as a type of guard for quickly detecting
    /// a mismatch if the first byte matches. This is a hedge against
    /// pathological cases where the pre-computed frequency analysis may be
    /// off. (But of course, does not prevent *all* pathological cases.)
    ///
    /// In general, rare1i != rare2i by construction, although there is no hard
    /// requirement that they be different. However, since the case of a single
    /// byte needle is handled specially by memchr itself, rare2i generally
    /// always should be different from rare1i since it would otherwise be
    /// ineffective as a guard.
    rare2i: u8,
}

impl RareNeedleBytes {
    /// Create a new pair of rare needle bytes with the given offsets. This is
    /// only used in tests for generating input data.
    #[cfg(all(test, feature = "std"))]
    pub(crate) fn new(rare1i: u8, rare2i: u8) -> RareNeedleBytes {
        RareNeedleBytes { rare1i, rare2i }
    }

    /// Detect the leftmost offsets of the two rarest bytes in the given
    /// needle.
    pub(crate) fn forward(needle: &[u8]) -> RareNeedleBytes {
        if needle.len() <= 1 || needle.len() > core::u8::MAX as usize {
            // For needles bigger than u8::MAX, our offsets aren't big enough.
            // (We make our offsets small to reduce stack copying.)
            // If you have a use case for it, please file an issue. In that
            // case, we should probably just adjust the routine below to pick
            // some rare bytes from the first 255 bytes of the needle.
            //
            // Also note that for needles of size 0 or 1, they are special
            // cased in Two-Way.
            //
            // TODO: Benchmar this.
            return RareNeedleBytes { rare1i: 0, rare2i: 0 };
        }

        // Find the rarest two bytes. We make them distinct by construction.
        let (mut rare1, mut rare1i) = (needle[0], 0);
        let (mut rare2, mut rare2i) = (needle[1], 1);
        if rank(rare2) < rank(rare1) {
            core::mem::swap(&mut rare1, &mut rare2);
            core::mem::swap(&mut rare1i, &mut rare2i);
        }
        for (i, &b) in needle.iter().enumerate().skip(2) {
            if rank(b) < rank(rare1) {
                rare2 = rare1;
                rare2i = rare1i;
                rare1 = b;
                rare1i = i as u8;
            } else if b != rare1 && rank(b) < rank(rare2) {
                rare2 = b;
                rare2i = i as u8;
            }
        }
        // While not strictly required, we really don't want these to be
        // equivalent. If they were, it would reduce the effectiveness of
        // candidate searching using these rare bytes by increasing the rate of
        // false positives.
        assert_ne!(rare1i, rare2i);
        RareNeedleBytes { rare1i, rare2i }
    }

    /// Return the rare bytes in the given needle in the forward direction.
    /// The needle given must be the same one given to the RareNeedleBytes
    /// constructor.
    pub(crate) fn as_rare_bytes(&self, needle: &[u8]) -> (u8, u8) {
        (needle[self.rare1i as usize], needle[self.rare2i as usize])
    }

    /// Return the rare offsets such that the first offset is always <= to the
    /// second offset. This is useful when the caller doesn't care whether
    /// rare1 is rarer than rare2, but just wants to ensure that they are
    /// ordered with respect to one another.
    #[cfg(memchr_runtime_simd)]
    pub(crate) fn as_rare_ordered_usize(&self) -> (usize, usize) {
        let (rare1i, rare2i) = self.as_rare_ordered_u8();
        (rare1i as usize, rare2i as usize)
    }

    /// Like as_rare_ordered_usize, but returns the offsets as their native
    /// u8 values.
    #[cfg(memchr_runtime_simd)]
    pub(crate) fn as_rare_ordered_u8(&self) -> (u8, u8) {
        if self.rare1i <= self.rare2i {
            (self.rare1i, self.rare2i)
        } else {
            (self.rare2i, self.rare1i)
        }
    }

    /// Return the rare offsets as usize values in the order in which they were
    /// constructed. rare1, for example, is constructed as the "rarer" byte,
    /// and thus, callers may want to treat it differently from rare2.
    pub(crate) fn as_rare_usize(&self) -> (usize, usize) {
        (self.rare1i as usize, self.rare2i as usize)
    }

    /// Return the byte frequency rank of each byte. The higher the rank, the
    /// more frequency the byte is predicted to be. The needle given must be
    /// the same one given to the RareNeedleBytes constructor.
    pub(crate) fn as_ranks(&self, needle: &[u8]) -> (usize, usize) {
        let (b1, b2) = self.as_rare_bytes(needle);
        (rank(b1), rank(b2))
    }
}

/// Return the heuristical frequency rank of the given byte. A lower rank
/// means the byte is believed to occur less frequently.
fn rank(b: u8) -> usize {
    crate::memmem::byte_frequencies::BYTE_FREQUENCIES[b as usize] as usize
}
