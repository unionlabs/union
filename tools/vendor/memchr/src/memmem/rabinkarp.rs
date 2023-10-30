/*
This module implements the classical Rabin-Karp substring search algorithm,
with no extra frills. While its use would seem to break our time complexity
guarantee of O(m+n) (RK's time complexity is O(mn)), we are careful to only
ever use RK on a constant subset of haystacks. The main point here is that
RK has good latency properties for small needles/haystacks. It's very quick
to compute a needle hash and zip through the haystack when compared to
initializing Two-Way, for example. And this is especially useful for cases
where the haystack is just too short for vector instructions to do much good.

The hashing function used here is the same one recommended by ESMAJ.

Another choice instead of Rabin-Karp would be Shift-Or. But its latency
isn't quite as good since its preprocessing time is a bit more expensive
(both in practice and in theory). However, perhaps Shift-Or has a place
somewhere else for short patterns. I think the main problem is that it
requires space proportional to the alphabet and the needle. If we, for
example, supported needles up to length 16, then the total table size would be
len(alphabet)*size_of::<u16>()==512 bytes. Which isn't exactly small, and it's
probably bad to put that on the stack. So ideally, we'd throw it on the heap,
but we'd really like to write as much code without using alloc/std as possible.
But maybe it's worth the special casing. It's a TODO to benchmark.

Wikipedia has a decent explanation, if a bit heavy on the theory:
https://en.wikipedia.org/wiki/Rabin%E2%80%93Karp_algorithm

But ESMAJ provides something a bit more concrete:
http://www-igm.univ-mlv.fr/~lecroq/string/node5.html

Finally, aho-corasick uses Rabin-Karp for multiple pattern match in some cases:
https://github.com/BurntSushi/aho-corasick/blob/3852632f10587db0ff72ef29e88d58bf305a0946/src/packed/rabinkarp.rs
*/

/// Whether RK is believed to be very fast for the given needle/haystack.
pub(crate) fn is_fast(haystack: &[u8], _needle: &[u8]) -> bool {
    haystack.len() < 16
}

/// Search for the first occurrence of needle in haystack using Rabin-Karp.
pub(crate) fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    find_with(&NeedleHash::forward(needle), haystack, needle)
}

/// Search for the first occurrence of needle in haystack using Rabin-Karp with
/// a pre-computed needle hash.
pub(crate) fn find_with(
    nhash: &NeedleHash,
    mut haystack: &[u8],
    needle: &[u8],
) -> Option<usize> {
    if haystack.len() < needle.len() {
        return None;
    }
    let start = haystack.as_ptr() as usize;
    let mut hash = Hash::from_bytes_fwd(&haystack[..needle.len()]);
    // N.B. I've experimented with unrolling this loop, but couldn't realize
    // any obvious gains.
    loop {
        if nhash.eq(hash) && is_prefix(haystack, needle) {
            return Some(haystack.as_ptr() as usize - start);
        }
        if needle.len() >= haystack.len() {
            return None;
        }
        hash.roll(&nhash, haystack[0], haystack[needle.len()]);
        haystack = &haystack[1..];
    }
}

/// Search for the last occurrence of needle in haystack using Rabin-Karp.
pub(crate) fn rfind(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    rfind_with(&NeedleHash::reverse(needle), haystack, needle)
}

/// Search for the last occurrence of needle in haystack using Rabin-Karp with
/// a pre-computed needle hash.
pub(crate) fn rfind_with(
    nhash: &NeedleHash,
    mut haystack: &[u8],
    needle: &[u8],
) -> Option<usize> {
    if haystack.len() < needle.len() {
        return None;
    }
    let mut hash =
        Hash::from_bytes_rev(&haystack[haystack.len() - needle.len()..]);
    loop {
        if nhash.eq(hash) && is_suffix(haystack, needle) {
            return Some(haystack.len() - needle.len());
        }
        if needle.len() >= haystack.len() {
            return None;
        }
        hash.roll(
            &nhash,
            haystack[haystack.len() - 1],
            haystack[haystack.len() - needle.len() - 1],
        );
        haystack = &haystack[..haystack.len() - 1];
    }
}

/// A hash derived from a needle.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct NeedleHash {
    /// The actual hash.
    hash: Hash,
    /// The factor needed to multiply a byte by in order to subtract it from
    /// the hash. It is defined to be 2^(n-1) (using wrapping exponentiation),
    /// where n is the length of the needle. This is how we "remove" a byte
    /// from the hash once the hash window rolls past it.
    hash_2pow: u32,
}

impl NeedleHash {
    /// Create a new Rabin-Karp hash for the given needle for use in forward
    /// searching.
    pub(crate) fn forward(needle: &[u8]) -> NeedleHash {
        let mut nh = NeedleHash { hash: Hash::new(), hash_2pow: 1 };
        if needle.is_empty() {
            return nh;
        }
        nh.hash.add(needle[0]);
        for &b in needle.iter().skip(1) {
            nh.hash.add(b);
            nh.hash_2pow = nh.hash_2pow.wrapping_shl(1);
        }
        nh
    }

    /// Create a new Rabin-Karp hash for the given needle for use in reverse
    /// searching.
    pub(crate) fn reverse(needle: &[u8]) -> NeedleHash {
        let mut nh = NeedleHash { hash: Hash::new(), hash_2pow: 1 };
        if needle.is_empty() {
            return nh;
        }
        nh.hash.add(needle[needle.len() - 1]);
        for &b in needle.iter().rev().skip(1) {
            nh.hash.add(b);
            nh.hash_2pow = nh.hash_2pow.wrapping_shl(1);
        }
        nh
    }

    /// Return true if the hashes are equivalent.
    fn eq(&self, hash: Hash) -> bool {
        self.hash == hash
    }
}

/// A Rabin-Karp hash. This might represent the hash of a needle, or the hash
/// of a rolling window in the haystack.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Hash(u32);

impl Hash {
    /// Create a new hash that represents the empty string.
    pub(crate) fn new() -> Hash {
        Hash(0)
    }

    /// Create a new hash from the bytes given for use in forward searches.
    pub(crate) fn from_bytes_fwd(bytes: &[u8]) -> Hash {
        let mut hash = Hash::new();
        for &b in bytes {
            hash.add(b);
        }
        hash
    }

    /// Create a new hash from the bytes given for use in reverse searches.
    fn from_bytes_rev(bytes: &[u8]) -> Hash {
        let mut hash = Hash::new();
        for &b in bytes.iter().rev() {
            hash.add(b);
        }
        hash
    }

    /// Add 'new' and remove 'old' from this hash. The given needle hash should
    /// correspond to the hash computed for the needle being searched for.
    ///
    /// This is meant to be used when the rolling window of the haystack is
    /// advanced.
    fn roll(&mut self, nhash: &NeedleHash, old: u8, new: u8) {
        self.del(nhash, old);
        self.add(new);
    }

    /// Add a byte to this hash.
    fn add(&mut self, byte: u8) {
        self.0 = self.0.wrapping_shl(1).wrapping_add(byte as u32);
    }

    /// Remove a byte from this hash. The given needle hash should correspond
    /// to the hash computed for the needle being searched for.
    fn del(&mut self, nhash: &NeedleHash, byte: u8) {
        let factor = nhash.hash_2pow;
        self.0 = self.0.wrapping_sub((byte as u32).wrapping_mul(factor));
    }
}

/// Returns true if the given needle is a prefix of the given haystack.
///
/// We forcefully don't inline the is_prefix call and hint at the compiler that
/// it is unlikely to be called. This causes the inner rabinkarp loop above
/// to be a bit tighter and leads to some performance improvement. See the
/// memmem/krate/prebuilt/sliceslice-words/words benchmark.
#[cold]
#[inline(never)]
fn is_prefix(haystack: &[u8], needle: &[u8]) -> bool {
    crate::memmem::util::is_prefix(haystack, needle)
}

/// Returns true if the given needle is a suffix of the given haystack.
///
/// See is_prefix for why this is forcefully not inlined.
#[cold]
#[inline(never)]
fn is_suffix(haystack: &[u8], needle: &[u8]) -> bool {
    crate::memmem::util::is_suffix(haystack, needle)
}

#[cfg(test)]
mod simpletests {
    define_memmem_simple_tests!(super::find, super::rfind);
}

#[cfg(all(test, feature = "std", not(miri)))]
mod proptests {
    define_memmem_quickcheck_tests!(super::find, super::rfind);
}
