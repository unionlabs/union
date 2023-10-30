use core::iter::Rev;

pub use self::iter::{Memchr, Memchr2, Memchr3};

// N.B. If you're looking for the cfg knobs for libc, see build.rs.
#[cfg(memchr_libc)]
mod c;
#[allow(dead_code)]
pub mod fallback;
mod iter;
pub mod naive;
#[cfg(all(not(miri), target_arch = "x86_64", memchr_runtime_simd))]
mod x86;

/// An iterator over all occurrences of the needle in a haystack.
#[inline]
pub fn memchr_iter(needle: u8, haystack: &[u8]) -> Memchr<'_> {
    Memchr::new(needle, haystack)
}

/// An iterator over all occurrences of the needles in a haystack.
#[inline]
pub fn memchr2_iter(needle1: u8, needle2: u8, haystack: &[u8]) -> Memchr2<'_> {
    Memchr2::new(needle1, needle2, haystack)
}

/// An iterator over all occurrences of the needles in a haystack.
#[inline]
pub fn memchr3_iter(
    needle1: u8,
    needle2: u8,
    needle3: u8,
    haystack: &[u8],
) -> Memchr3<'_> {
    Memchr3::new(needle1, needle2, needle3, haystack)
}

/// An iterator over all occurrences of the needle in a haystack, in reverse.
#[inline]
pub fn memrchr_iter(needle: u8, haystack: &[u8]) -> Rev<Memchr<'_>> {
    Memchr::new(needle, haystack).rev()
}

/// An iterator over all occurrences of the needles in a haystack, in reverse.
#[inline]
pub fn memrchr2_iter(
    needle1: u8,
    needle2: u8,
    haystack: &[u8],
) -> Rev<Memchr2<'_>> {
    Memchr2::new(needle1, needle2, haystack).rev()
}

/// An iterator over all occurrences of the needles in a haystack, in reverse.
#[inline]
pub fn memrchr3_iter(
    needle1: u8,
    needle2: u8,
    needle3: u8,
    haystack: &[u8],
) -> Rev<Memchr3<'_>> {
    Memchr3::new(needle1, needle2, needle3, haystack).rev()
}

/// Search for the first occurrence of a byte in a slice.
///
/// This returns the index corresponding to the first occurrence of `needle` in
/// `haystack`, or `None` if one is not found. If an index is returned, it is
/// guaranteed to be less than `usize::MAX`.
///
/// While this is operationally the same as something like
/// `haystack.iter().position(|&b| b == needle)`, `memchr` will use a highly
/// optimized routine that can be up to an order of magnitude faster in some
/// cases.
///
/// # Example
///
/// This shows how to find the first position of a byte in a byte string.
///
/// ```
/// use memchr::memchr;
///
/// let haystack = b"the quick brown fox";
/// assert_eq!(memchr(b'k', haystack), Some(8));
/// ```
#[inline]
pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    #[cfg(miri)]
    #[inline(always)]
    fn imp(n1: u8, haystack: &[u8]) -> Option<usize> {
        naive::memchr(n1, haystack)
    }

    #[cfg(all(target_arch = "x86_64", memchr_runtime_simd, not(miri)))]
    #[inline(always)]
    fn imp(n1: u8, haystack: &[u8]) -> Option<usize> {
        x86::memchr(n1, haystack)
    }

    #[cfg(all(
        memchr_libc,
        not(all(target_arch = "x86_64", memchr_runtime_simd)),
        not(miri),
    ))]
    #[inline(always)]
    fn imp(n1: u8, haystack: &[u8]) -> Option<usize> {
        c::memchr(n1, haystack)
    }

    #[cfg(all(
        not(memchr_libc),
        not(all(target_arch = "x86_64", memchr_runtime_simd)),
        not(miri),
    ))]
    #[inline(always)]
    fn imp(n1: u8, haystack: &[u8]) -> Option<usize> {
        fallback::memchr(n1, haystack)
    }

    if haystack.is_empty() {
        None
    } else {
        imp(needle, haystack)
    }
}

/// Like `memchr`, but searches for either of two bytes instead of just one.
///
/// This returns the index corresponding to the first occurrence of `needle1`
/// or the first occurrence of `needle2` in `haystack` (whichever occurs
/// earlier), or `None` if neither one is found. If an index is returned, it is
/// guaranteed to be less than `usize::MAX`.
///
/// While this is operationally the same as something like
/// `haystack.iter().position(|&b| b == needle1 || b == needle2)`, `memchr2`
/// will use a highly optimized routine that can be up to an order of magnitude
/// faster in some cases.
///
/// # Example
///
/// This shows how to find the first position of either of two bytes in a byte
/// string.
///
/// ```
/// use memchr::memchr2;
///
/// let haystack = b"the quick brown fox";
/// assert_eq!(memchr2(b'k', b'q', haystack), Some(4));
/// ```
#[inline]
pub fn memchr2(needle1: u8, needle2: u8, haystack: &[u8]) -> Option<usize> {
    #[cfg(miri)]
    #[inline(always)]
    fn imp(n1: u8, n2: u8, haystack: &[u8]) -> Option<usize> {
        naive::memchr2(n1, n2, haystack)
    }

    #[cfg(all(target_arch = "x86_64", memchr_runtime_simd, not(miri)))]
    #[inline(always)]
    fn imp(n1: u8, n2: u8, haystack: &[u8]) -> Option<usize> {
        x86::memchr2(n1, n2, haystack)
    }

    #[cfg(all(
        not(all(target_arch = "x86_64", memchr_runtime_simd)),
        not(miri),
    ))]
    #[inline(always)]
    fn imp(n1: u8, n2: u8, haystack: &[u8]) -> Option<usize> {
        fallback::memchr2(n1, n2, haystack)
    }

    if haystack.is_empty() {
        None
    } else {
        imp(needle1, needle2, haystack)
    }
}

/// Like `memchr`, but searches for any of three bytes instead of just one.
///
/// This returns the index corresponding to the first occurrence of `needle1`,
/// the first occurrence of `needle2`, or the first occurrence of `needle3` in
/// `haystack` (whichever occurs earliest), or `None` if none are found. If an
/// index is returned, it is guaranteed to be less than `usize::MAX`.
///
/// While this is operationally the same as something like
/// `haystack.iter().position(|&b| b == needle1 || b == needle2 ||
/// b == needle3)`, `memchr3` will use a highly optimized routine that can be
/// up to an order of magnitude faster in some cases.
///
/// # Example
///
/// This shows how to find the first position of any of three bytes in a byte
/// string.
///
/// ```
/// use memchr::memchr3;
///
/// let haystack = b"the quick brown fox";
/// assert_eq!(memchr3(b'k', b'q', b'e', haystack), Some(2));
/// ```
#[inline]
pub fn memchr3(
    needle1: u8,
    needle2: u8,
    needle3: u8,
    haystack: &[u8],
) -> Option<usize> {
    #[cfg(miri)]
    #[inline(always)]
    fn imp(n1: u8, n2: u8, n3: u8, haystack: &[u8]) -> Option<usize> {
        naive::memchr3(n1, n2, n3, haystack)
    }

    #[cfg(all(target_arch = "x86_64", memchr_runtime_simd, not(miri)))]
    #[inline(always)]
    fn imp(n1: u8, n2: u8, n3: u8, haystack: &[u8]) -> Option<usize> {
        x86::memchr3(n1, n2, n3, haystack)
    }

    #[cfg(all(
        not(all(target_arch = "x86_64", memchr_runtime_simd)),
        not(miri),
    ))]
    #[inline(always)]
    fn imp(n1: u8, n2: u8, n3: u8, haystack: &[u8]) -> Option<usize> {
        fallback::memchr3(n1, n2, n3, haystack)
    }

    if haystack.is_empty() {
        None
    } else {
        imp(needle1, needle2, needle3, haystack)
    }
}

/// Search for the last occurrence of a byte in a slice.
///
/// This returns the index corresponding to the last occurrence of `needle` in
/// `haystack`, or `None` if one is not found. If an index is returned, it is
/// guaranteed to be less than `usize::MAX`.
///
/// While this is operationally the same as something like
/// `haystack.iter().rposition(|&b| b == needle)`, `memrchr` will use a highly
/// optimized routine that can be up to an order of magnitude faster in some
/// cases.
///
/// # Example
///
/// This shows how to find the last position of a byte in a byte string.
///
/// ```
/// use memchr::memrchr;
///
/// let haystack = b"the quick brown fox";
/// assert_eq!(memrchr(b'o', haystack), Some(17));
/// ```
#[inline]
pub fn memrchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    #[cfg(miri)]
    #[inline(always)]
    fn imp(n1: u8, haystack: &[u8]) -> Option<usize> {
        naive::memrchr(n1, haystack)
    }

    #[cfg(all(target_arch = "x86_64", memchr_runtime_simd, not(miri)))]
    #[inline(always)]
    fn imp(n1: u8, haystack: &[u8]) -> Option<usize> {
        x86::memrchr(n1, haystack)
    }

    #[cfg(all(
        memchr_libc,
        target_os = "linux",
        not(all(target_arch = "x86_64", memchr_runtime_simd)),
        not(miri)
    ))]
    #[inline(always)]
    fn imp(n1: u8, haystack: &[u8]) -> Option<usize> {
        c::memrchr(n1, haystack)
    }

    #[cfg(all(
        not(all(memchr_libc, target_os = "linux")),
        not(all(target_arch = "x86_64", memchr_runtime_simd)),
        not(miri),
    ))]
    #[inline(always)]
    fn imp(n1: u8, haystack: &[u8]) -> Option<usize> {
        fallback::memrchr(n1, haystack)
    }

    if haystack.is_empty() {
        None
    } else {
        imp(needle, haystack)
    }
}

/// Like `memrchr`, but searches for either of two bytes instead of just one.
///
/// This returns the index corresponding to the last occurrence of `needle1` or
/// the last occurrence of `needle2` in `haystack` (whichever occurs later), or
/// `None` if neither one is found. If an index is returned, it is guaranteed
/// to be less than `usize::MAX`.
///
/// While this is operationally the same as something like
/// `haystack.iter().rposition(|&b| b == needle1 || b == needle2)`, `memrchr2`
/// will use a highly optimized routine that can be up to an order of magnitude
/// faster in some cases.
///
/// # Example
///
/// This shows how to find the last position of either of two bytes in a byte
/// string.
///
/// ```
/// use memchr::memrchr2;
///
/// let haystack = b"the quick brown fox";
/// assert_eq!(memrchr2(b'k', b'q', haystack), Some(8));
/// ```
#[inline]
pub fn memrchr2(needle1: u8, needle2: u8, haystack: &[u8]) -> Option<usize> {
    #[cfg(miri)]
    #[inline(always)]
    fn imp(n1: u8, n2: u8, haystack: &[u8]) -> Option<usize> {
        naive::memrchr2(n1, n2, haystack)
    }

    #[cfg(all(target_arch = "x86_64", memchr_runtime_simd, not(miri)))]
    #[inline(always)]
    fn imp(n1: u8, n2: u8, haystack: &[u8]) -> Option<usize> {
        x86::memrchr2(n1, n2, haystack)
    }

    #[cfg(all(
        not(all(target_arch = "x86_64", memchr_runtime_simd)),
        not(miri),
    ))]
    #[inline(always)]
    fn imp(n1: u8, n2: u8, haystack: &[u8]) -> Option<usize> {
        fallback::memrchr2(n1, n2, haystack)
    }

    if haystack.is_empty() {
        None
    } else {
        imp(needle1, needle2, haystack)
    }
}

/// Like `memrchr`, but searches for any of three bytes instead of just one.
///
/// This returns the index corresponding to the last occurrence of `needle1`,
/// the last occurrence of `needle2`, or the last occurrence of `needle3` in
/// `haystack` (whichever occurs later), or `None` if none are found. If an
/// index is returned, it is guaranteed to be less than `usize::MAX`.
///
/// While this is operationally the same as something like
/// `haystack.iter().rposition(|&b| b == needle1 || b == needle2 ||
/// b == needle3)`, `memrchr3` will use a highly optimized routine that can be
/// up to an order of magnitude faster in some cases.
///
/// # Example
///
/// This shows how to find the last position of any of three bytes in a byte
/// string.
///
/// ```
/// use memchr::memrchr3;
///
/// let haystack = b"the quick brown fox";
/// assert_eq!(memrchr3(b'k', b'q', b'e', haystack), Some(8));
/// ```
#[inline]
pub fn memrchr3(
    needle1: u8,
    needle2: u8,
    needle3: u8,
    haystack: &[u8],
) -> Option<usize> {
    #[cfg(miri)]
    #[inline(always)]
    fn imp(n1: u8, n2: u8, n3: u8, haystack: &[u8]) -> Option<usize> {
        naive::memrchr3(n1, n2, n3, haystack)
    }

    #[cfg(all(target_arch = "x86_64", memchr_runtime_simd, not(miri)))]
    #[inline(always)]
    fn imp(n1: u8, n2: u8, n3: u8, haystack: &[u8]) -> Option<usize> {
        x86::memrchr3(n1, n2, n3, haystack)
    }

    #[cfg(all(
        not(all(target_arch = "x86_64", memchr_runtime_simd)),
        not(miri),
    ))]
    #[inline(always)]
    fn imp(n1: u8, n2: u8, n3: u8, haystack: &[u8]) -> Option<usize> {
        fallback::memrchr3(n1, n2, n3, haystack)
    }

    if haystack.is_empty() {
        None
    } else {
        imp(needle1, needle2, needle3, haystack)
    }
}
