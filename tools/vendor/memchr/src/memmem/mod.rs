/*!
This module provides forward and reverse substring search routines.

Unlike the standard library's substring search routines, these work on
arbitrary bytes. For all non-empty needles, these routines will report exactly
the same values as the corresponding routines in the standard library. For
the empty needle, the standard library reports matches only at valid UTF-8
boundaries, where as these routines will report matches at every position.

Other than being able to work on arbitrary bytes, the primary reason to prefer
these routines over the standard library routines is that these will generally
be faster. In some cases, significantly so.

# Example: iterating over substring matches

This example shows how to use [`find_iter`] to find occurrences of a substring
in a haystack.

```
use memchr::memmem;

let haystack = b"foo bar foo baz foo";

let mut it = memmem::find_iter(haystack, "foo");
assert_eq!(Some(0), it.next());
assert_eq!(Some(8), it.next());
assert_eq!(Some(16), it.next());
assert_eq!(None, it.next());
```

# Example: iterating over substring matches in reverse

This example shows how to use [`rfind_iter`] to find occurrences of a substring
in a haystack starting from the end of the haystack.

**NOTE:** This module does not implement double ended iterators, so reverse
searches aren't done by calling `rev` on a forward iterator.

```
use memchr::memmem;

let haystack = b"foo bar foo baz foo";

let mut it = memmem::rfind_iter(haystack, "foo");
assert_eq!(Some(16), it.next());
assert_eq!(Some(8), it.next());
assert_eq!(Some(0), it.next());
assert_eq!(None, it.next());
```

# Example: repeating a search for the same needle

It may be possible for the overhead of constructing a substring searcher to be
measurable in some workloads. In cases where the same needle is used to search
many haystacks, it is possible to do construction once and thus to avoid it for
subsequent searches. This can be done with a [`Finder`] (or a [`FinderRev`] for
reverse searches).

```
use memchr::memmem;

let finder = memmem::Finder::new("foo");

assert_eq!(Some(4), finder.find(b"baz foo quux"));
assert_eq!(None, finder.find(b"quux baz bar"));
```
*/

pub use self::prefilter::Prefilter;

use crate::{
    cow::CowBytes,
    memmem::{
        prefilter::{Pre, PrefilterFn, PrefilterState},
        rabinkarp::NeedleHash,
        rarebytes::RareNeedleBytes,
    },
};

/// Defines a suite of quickcheck properties for forward and reverse
/// substring searching.
///
/// This is defined in this specific spot so that it can be used freely among
/// the different substring search implementations. I couldn't be bothered to
/// fight with the macro-visibility rules enough to figure out how to stuff it
/// somewhere more convenient.
#[cfg(all(test, feature = "std"))]
macro_rules! define_memmem_quickcheck_tests {
    ($fwd:expr, $rev:expr) => {
        use crate::memmem::proptests;

        quickcheck::quickcheck! {
            fn qc_fwd_prefix_is_substring(bs: Vec<u8>) -> bool {
                proptests::prefix_is_substring(false, &bs, $fwd)
            }

            fn qc_fwd_suffix_is_substring(bs: Vec<u8>) -> bool {
                proptests::suffix_is_substring(false, &bs, $fwd)
            }

            fn qc_fwd_matches_naive(
                haystack: Vec<u8>,
                needle: Vec<u8>
            ) -> bool {
                proptests::matches_naive(false, &haystack, &needle, $fwd)
            }

            fn qc_rev_prefix_is_substring(bs: Vec<u8>) -> bool {
                proptests::prefix_is_substring(true, &bs, $rev)
            }

            fn qc_rev_suffix_is_substring(bs: Vec<u8>) -> bool {
                proptests::suffix_is_substring(true, &bs, $rev)
            }

            fn qc_rev_matches_naive(
                haystack: Vec<u8>,
                needle: Vec<u8>
            ) -> bool {
                proptests::matches_naive(true, &haystack, &needle, $rev)
            }
        }
    };
}

/// Defines a suite of "simple" hand-written tests for a substring
/// implementation.
///
/// This is defined here for the same reason that
/// define_memmem_quickcheck_tests is defined here.
#[cfg(test)]
macro_rules! define_memmem_simple_tests {
    ($fwd:expr, $rev:expr) => {
        use crate::memmem::testsimples;

        #[test]
        fn simple_forward() {
            testsimples::run_search_tests_fwd($fwd);
        }

        #[test]
        fn simple_reverse() {
            testsimples::run_search_tests_rev($rev);
        }
    };
}

mod byte_frequencies;
#[cfg(memchr_runtime_simd)]
mod genericsimd;
mod prefilter;
mod rabinkarp;
mod rarebytes;
mod twoway;
mod util;
#[cfg(memchr_runtime_simd)]
mod vector;
#[cfg(all(memchr_runtime_wasm128))]
mod wasm;
#[cfg(all(not(miri), target_arch = "x86_64", memchr_runtime_simd))]
mod x86;

/// Returns an iterator over all non-overlapping occurrences of a substring in
/// a haystack.
///
/// # Complexity
///
/// This routine is guaranteed to have worst case linear time complexity
/// with respect to both the needle and the haystack. That is, this runs
/// in `O(needle.len() + haystack.len())` time.
///
/// This routine is also guaranteed to have worst case constant space
/// complexity.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use memchr::memmem;
///
/// let haystack = b"foo bar foo baz foo";
/// let mut it = memmem::find_iter(haystack, b"foo");
/// assert_eq!(Some(0), it.next());
/// assert_eq!(Some(8), it.next());
/// assert_eq!(Some(16), it.next());
/// assert_eq!(None, it.next());
/// ```
#[inline]
pub fn find_iter<'h, 'n, N: 'n + ?Sized + AsRef<[u8]>>(
    haystack: &'h [u8],
    needle: &'n N,
) -> FindIter<'h, 'n> {
    FindIter::new(haystack, Finder::new(needle))
}

/// Returns a reverse iterator over all non-overlapping occurrences of a
/// substring in a haystack.
///
/// # Complexity
///
/// This routine is guaranteed to have worst case linear time complexity
/// with respect to both the needle and the haystack. That is, this runs
/// in `O(needle.len() + haystack.len())` time.
///
/// This routine is also guaranteed to have worst case constant space
/// complexity.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use memchr::memmem;
///
/// let haystack = b"foo bar foo baz foo";
/// let mut it = memmem::rfind_iter(haystack, b"foo");
/// assert_eq!(Some(16), it.next());
/// assert_eq!(Some(8), it.next());
/// assert_eq!(Some(0), it.next());
/// assert_eq!(None, it.next());
/// ```
#[inline]
pub fn rfind_iter<'h, 'n, N: 'n + ?Sized + AsRef<[u8]>>(
    haystack: &'h [u8],
    needle: &'n N,
) -> FindRevIter<'h, 'n> {
    FindRevIter::new(haystack, FinderRev::new(needle))
}

/// Returns the index of the first occurrence of the given needle.
///
/// Note that if you're are searching for the same needle in many different
/// small haystacks, it may be faster to initialize a [`Finder`] once,
/// and reuse it for each search.
///
/// # Complexity
///
/// This routine is guaranteed to have worst case linear time complexity
/// with respect to both the needle and the haystack. That is, this runs
/// in `O(needle.len() + haystack.len())` time.
///
/// This routine is also guaranteed to have worst case constant space
/// complexity.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use memchr::memmem;
///
/// let haystack = b"foo bar baz";
/// assert_eq!(Some(0), memmem::find(haystack, b"foo"));
/// assert_eq!(Some(4), memmem::find(haystack, b"bar"));
/// assert_eq!(None, memmem::find(haystack, b"quux"));
/// ```
#[inline]
pub fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if haystack.len() < 64 {
        rabinkarp::find(haystack, needle)
    } else {
        Finder::new(needle).find(haystack)
    }
}

/// Returns the index of the last occurrence of the given needle.
///
/// Note that if you're are searching for the same needle in many different
/// small haystacks, it may be faster to initialize a [`FinderRev`] once,
/// and reuse it for each search.
///
/// # Complexity
///
/// This routine is guaranteed to have worst case linear time complexity
/// with respect to both the needle and the haystack. That is, this runs
/// in `O(needle.len() + haystack.len())` time.
///
/// This routine is also guaranteed to have worst case constant space
/// complexity.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use memchr::memmem;
///
/// let haystack = b"foo bar baz";
/// assert_eq!(Some(0), memmem::rfind(haystack, b"foo"));
/// assert_eq!(Some(4), memmem::rfind(haystack, b"bar"));
/// assert_eq!(Some(8), memmem::rfind(haystack, b"ba"));
/// assert_eq!(None, memmem::rfind(haystack, b"quux"));
/// ```
#[inline]
pub fn rfind(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if haystack.len() < 64 {
        rabinkarp::rfind(haystack, needle)
    } else {
        FinderRev::new(needle).rfind(haystack)
    }
}

/// An iterator over non-overlapping substring matches.
///
/// Matches are reported by the byte offset at which they begin.
///
/// `'h` is the lifetime of the haystack while `'n` is the lifetime of the
/// needle.
#[derive(Debug)]
pub struct FindIter<'h, 'n> {
    haystack: &'h [u8],
    prestate: PrefilterState,
    finder: Finder<'n>,
    pos: usize,
}

impl<'h, 'n> FindIter<'h, 'n> {
    #[inline(always)]
    pub(crate) fn new(
        haystack: &'h [u8],
        finder: Finder<'n>,
    ) -> FindIter<'h, 'n> {
        let prestate = finder.searcher.prefilter_state();
        FindIter { haystack, prestate, finder, pos: 0 }
    }

    /// Convert this iterator into its owned variant, such that it no longer
    /// borrows the finder and needle.
    ///
    /// If this is already an owned iterator, then this is a no-op. Otherwise,
    /// this copies the needle.
    ///
    /// This is only available when the `std` feature is enabled.
    #[cfg(feature = "std")]
    #[inline]
    pub fn into_owned(self) -> FindIter<'h, 'static> {
        FindIter {
            haystack: self.haystack,
            prestate: self.prestate,
            finder: self.finder.into_owned(),
            pos: self.pos,
        }
    }
}

impl<'h, 'n> Iterator for FindIter<'h, 'n> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.pos > self.haystack.len() {
            return None;
        }
        let result = self
            .finder
            .searcher
            .find(&mut self.prestate, &self.haystack[self.pos..]);
        match result {
            None => None,
            Some(i) => {
                let pos = self.pos + i;
                self.pos = pos + core::cmp::max(1, self.finder.needle().len());
                Some(pos)
            }
        }
    }
}

/// An iterator over non-overlapping substring matches in reverse.
///
/// Matches are reported by the byte offset at which they begin.
///
/// `'h` is the lifetime of the haystack while `'n` is the lifetime of the
/// needle.
#[derive(Debug)]
pub struct FindRevIter<'h, 'n> {
    haystack: &'h [u8],
    finder: FinderRev<'n>,
    /// When searching with an empty needle, this gets set to `None` after
    /// we've yielded the last element at `0`.
    pos: Option<usize>,
}

impl<'h, 'n> FindRevIter<'h, 'n> {
    #[inline(always)]
    pub(crate) fn new(
        haystack: &'h [u8],
        finder: FinderRev<'n>,
    ) -> FindRevIter<'h, 'n> {
        let pos = Some(haystack.len());
        FindRevIter { haystack, finder, pos }
    }

    /// Convert this iterator into its owned variant, such that it no longer
    /// borrows the finder and needle.
    ///
    /// If this is already an owned iterator, then this is a no-op. Otherwise,
    /// this copies the needle.
    ///
    /// This is only available when the `std` feature is enabled.
    #[cfg(feature = "std")]
    #[inline]
    pub fn into_owned(self) -> FindRevIter<'h, 'static> {
        FindRevIter {
            haystack: self.haystack,
            finder: self.finder.into_owned(),
            pos: self.pos,
        }
    }
}

impl<'h, 'n> Iterator for FindRevIter<'h, 'n> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let pos = match self.pos {
            None => return None,
            Some(pos) => pos,
        };
        let result = self.finder.rfind(&self.haystack[..pos]);
        match result {
            None => None,
            Some(i) => {
                if pos == i {
                    self.pos = pos.checked_sub(1);
                } else {
                    self.pos = Some(i);
                }
                Some(i)
            }
        }
    }
}

/// A single substring searcher fixed to a particular needle.
///
/// The purpose of this type is to permit callers to construct a substring
/// searcher that can be used to search haystacks without the overhead of
/// constructing the searcher in the first place. This is a somewhat niche
/// concern when it's necessary to re-use the same needle to search multiple
/// different haystacks with as little overhead as possible. In general, using
/// [`find`] is good enough, but `Finder` is useful when you can meaningfully
/// observe searcher construction time in a profile.
///
/// When the `std` feature is enabled, then this type has an `into_owned`
/// version which permits building a `Finder` that is not connected to
/// the lifetime of its needle.
#[derive(Clone, Debug)]
pub struct Finder<'n> {
    searcher: Searcher<'n>,
}

impl<'n> Finder<'n> {
    /// Create a new finder for the given needle.
    #[inline]
    pub fn new<B: ?Sized + AsRef<[u8]>>(needle: &'n B) -> Finder<'n> {
        FinderBuilder::new().build_forward(needle)
    }

    /// Returns the index of the first occurrence of this needle in the given
    /// haystack.
    ///
    /// # Complexity
    ///
    /// This routine is guaranteed to have worst case linear time complexity
    /// with respect to both the needle and the haystack. That is, this runs
    /// in `O(needle.len() + haystack.len())` time.
    ///
    /// This routine is also guaranteed to have worst case constant space
    /// complexity.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use memchr::memmem::Finder;
    ///
    /// let haystack = b"foo bar baz";
    /// assert_eq!(Some(0), Finder::new("foo").find(haystack));
    /// assert_eq!(Some(4), Finder::new("bar").find(haystack));
    /// assert_eq!(None, Finder::new("quux").find(haystack));
    /// ```
    pub fn find(&self, haystack: &[u8]) -> Option<usize> {
        self.searcher.find(&mut self.searcher.prefilter_state(), haystack)
    }

    /// Returns an iterator over all occurrences of a substring in a haystack.
    ///
    /// # Complexity
    ///
    /// This routine is guaranteed to have worst case linear time complexity
    /// with respect to both the needle and the haystack. That is, this runs
    /// in `O(needle.len() + haystack.len())` time.
    ///
    /// This routine is also guaranteed to have worst case constant space
    /// complexity.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use memchr::memmem::Finder;
    ///
    /// let haystack = b"foo bar foo baz foo";
    /// let finder = Finder::new(b"foo");
    /// let mut it = finder.find_iter(haystack);
    /// assert_eq!(Some(0), it.next());
    /// assert_eq!(Some(8), it.next());
    /// assert_eq!(Some(16), it.next());
    /// assert_eq!(None, it.next());
    /// ```
    #[inline]
    pub fn find_iter<'a, 'h>(
        &'a self,
        haystack: &'h [u8],
    ) -> FindIter<'h, 'a> {
        FindIter::new(haystack, self.as_ref())
    }

    /// Convert this finder into its owned variant, such that it no longer
    /// borrows the needle.
    ///
    /// If this is already an owned finder, then this is a no-op. Otherwise,
    /// this copies the needle.
    ///
    /// This is only available when the `std` feature is enabled.
    #[cfg(feature = "std")]
    #[inline]
    pub fn into_owned(self) -> Finder<'static> {
        Finder { searcher: self.searcher.into_owned() }
    }

    /// Convert this finder into its borrowed variant.
    ///
    /// This is primarily useful if your finder is owned and you'd like to
    /// store its borrowed variant in some intermediate data structure.
    ///
    /// Note that the lifetime parameter of the returned finder is tied to the
    /// lifetime of `self`, and may be shorter than the `'n` lifetime of the
    /// needle itself. Namely, a finder's needle can be either borrowed or
    /// owned, so the lifetime of the needle returned must necessarily be the
    /// shorter of the two.
    #[inline]
    pub fn as_ref(&self) -> Finder<'_> {
        Finder { searcher: self.searcher.as_ref() }
    }

    /// Returns the needle that this finder searches for.
    ///
    /// Note that the lifetime of the needle returned is tied to the lifetime
    /// of the finder, and may be shorter than the `'n` lifetime. Namely, a
    /// finder's needle can be either borrowed or owned, so the lifetime of the
    /// needle returned must necessarily be the shorter of the two.
    #[inline]
    pub fn needle(&self) -> &[u8] {
        self.searcher.needle()
    }
}

/// A single substring reverse searcher fixed to a particular needle.
///
/// The purpose of this type is to permit callers to construct a substring
/// searcher that can be used to search haystacks without the overhead of
/// constructing the searcher in the first place. This is a somewhat niche
/// concern when it's necessary to re-use the same needle to search multiple
/// different haystacks with as little overhead as possible. In general,
/// using [`rfind`] is good enough, but `FinderRev` is useful when you can
/// meaningfully observe searcher construction time in a profile.
///
/// When the `std` feature is enabled, then this type has an `into_owned`
/// version which permits building a `FinderRev` that is not connected to
/// the lifetime of its needle.
#[derive(Clone, Debug)]
pub struct FinderRev<'n> {
    searcher: SearcherRev<'n>,
}

impl<'n> FinderRev<'n> {
    /// Create a new reverse finder for the given needle.
    #[inline]
    pub fn new<B: ?Sized + AsRef<[u8]>>(needle: &'n B) -> FinderRev<'n> {
        FinderBuilder::new().build_reverse(needle)
    }

    /// Returns the index of the last occurrence of this needle in the given
    /// haystack.
    ///
    /// The haystack may be any type that can be cheaply converted into a
    /// `&[u8]`. This includes, but is not limited to, `&str` and `&[u8]`.
    ///
    /// # Complexity
    ///
    /// This routine is guaranteed to have worst case linear time complexity
    /// with respect to both the needle and the haystack. That is, this runs
    /// in `O(needle.len() + haystack.len())` time.
    ///
    /// This routine is also guaranteed to have worst case constant space
    /// complexity.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use memchr::memmem::FinderRev;
    ///
    /// let haystack = b"foo bar baz";
    /// assert_eq!(Some(0), FinderRev::new("foo").rfind(haystack));
    /// assert_eq!(Some(4), FinderRev::new("bar").rfind(haystack));
    /// assert_eq!(None, FinderRev::new("quux").rfind(haystack));
    /// ```
    pub fn rfind<B: AsRef<[u8]>>(&self, haystack: B) -> Option<usize> {
        self.searcher.rfind(haystack.as_ref())
    }

    /// Returns a reverse iterator over all occurrences of a substring in a
    /// haystack.
    ///
    /// # Complexity
    ///
    /// This routine is guaranteed to have worst case linear time complexity
    /// with respect to both the needle and the haystack. That is, this runs
    /// in `O(needle.len() + haystack.len())` time.
    ///
    /// This routine is also guaranteed to have worst case constant space
    /// complexity.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use memchr::memmem::FinderRev;
    ///
    /// let haystack = b"foo bar foo baz foo";
    /// let finder = FinderRev::new(b"foo");
    /// let mut it = finder.rfind_iter(haystack);
    /// assert_eq!(Some(16), it.next());
    /// assert_eq!(Some(8), it.next());
    /// assert_eq!(Some(0), it.next());
    /// assert_eq!(None, it.next());
    /// ```
    #[inline]
    pub fn rfind_iter<'a, 'h>(
        &'a self,
        haystack: &'h [u8],
    ) -> FindRevIter<'h, 'a> {
        FindRevIter::new(haystack, self.as_ref())
    }

    /// Convert this finder into its owned variant, such that it no longer
    /// borrows the needle.
    ///
    /// If this is already an owned finder, then this is a no-op. Otherwise,
    /// this copies the needle.
    ///
    /// This is only available when the `std` feature is enabled.
    #[cfg(feature = "std")]
    #[inline]
    pub fn into_owned(self) -> FinderRev<'static> {
        FinderRev { searcher: self.searcher.into_owned() }
    }

    /// Convert this finder into its borrowed variant.
    ///
    /// This is primarily useful if your finder is owned and you'd like to
    /// store its borrowed variant in some intermediate data structure.
    ///
    /// Note that the lifetime parameter of the returned finder is tied to the
    /// lifetime of `self`, and may be shorter than the `'n` lifetime of the
    /// needle itself. Namely, a finder's needle can be either borrowed or
    /// owned, so the lifetime of the needle returned must necessarily be the
    /// shorter of the two.
    #[inline]
    pub fn as_ref(&self) -> FinderRev<'_> {
        FinderRev { searcher: self.searcher.as_ref() }
    }

    /// Returns the needle that this finder searches for.
    ///
    /// Note that the lifetime of the needle returned is tied to the lifetime
    /// of the finder, and may be shorter than the `'n` lifetime. Namely, a
    /// finder's needle can be either borrowed or owned, so the lifetime of the
    /// needle returned must necessarily be the shorter of the two.
    #[inline]
    pub fn needle(&self) -> &[u8] {
        self.searcher.needle()
    }
}

/// A builder for constructing non-default forward or reverse memmem finders.
///
/// A builder is primarily useful for configuring a substring searcher.
/// Currently, the only configuration exposed is the ability to disable
/// heuristic prefilters used to speed up certain searches.
#[derive(Clone, Debug, Default)]
pub struct FinderBuilder {
    config: SearcherConfig,
}

impl FinderBuilder {
    /// Create a new finder builder with default settings.
    pub fn new() -> FinderBuilder {
        FinderBuilder::default()
    }

    /// Build a forward finder using the given needle from the current
    /// settings.
    pub fn build_forward<'n, B: ?Sized + AsRef<[u8]>>(
        &self,
        needle: &'n B,
    ) -> Finder<'n> {
        Finder { searcher: Searcher::new(self.config, needle.as_ref()) }
    }

    /// Build a reverse finder using the given needle from the current
    /// settings.
    pub fn build_reverse<'n, B: ?Sized + AsRef<[u8]>>(
        &self,
        needle: &'n B,
    ) -> FinderRev<'n> {
        FinderRev { searcher: SearcherRev::new(needle.as_ref()) }
    }

    /// Configure the prefilter setting for the finder.
    ///
    /// See the documentation for [`Prefilter`] for more discussion on why
    /// you might want to configure this.
    pub fn prefilter(&mut self, prefilter: Prefilter) -> &mut FinderBuilder {
        self.config.prefilter = prefilter;
        self
    }
}

/// The internal implementation of a forward substring searcher.
///
/// The reality is that this is a "meta" searcher. Namely, depending on a
/// variety of parameters (CPU support, target, needle size, haystack size and
/// even dynamic properties such as prefilter effectiveness), the actual
/// algorithm employed to do substring search may change.
#[derive(Clone, Debug)]
struct Searcher<'n> {
    /// The actual needle we're searching for.
    ///
    /// A CowBytes is like a Cow<[u8]>, except in no_std environments, it is
    /// specialized to a single variant (the borrowed form).
    needle: CowBytes<'n>,
    /// A collection of facts computed on the needle that are useful for more
    /// than one substring search algorithm.
    ninfo: NeedleInfo,
    /// A prefilter function, if it was deemed appropriate.
    ///
    /// Some substring search implementations (like Two-Way) benefit greatly
    /// if we can quickly find candidate starting positions for a match.
    prefn: Option<PrefilterFn>,
    /// The actual substring implementation in use.
    kind: SearcherKind,
}

/// A collection of facts computed about a search needle.
///
/// We group these things together because it's useful to be able to hand them
/// to prefilters or substring algorithms that want them.
#[derive(Clone, Copy, Debug)]
pub(crate) struct NeedleInfo {
    /// The offsets of "rare" bytes detected in the needle.
    ///
    /// This is meant to be a heuristic in order to maximize the effectiveness
    /// of vectorized code. Namely, vectorized code tends to focus on only
    /// one or two bytes. If we pick bytes from the needle that occur
    /// infrequently, then more time will be spent in the vectorized code and
    /// will likely make the overall search (much) faster.
    ///
    /// Of course, this is only a heuristic based on a background frequency
    /// distribution of bytes. But it tends to work very well in practice.
    pub(crate) rarebytes: RareNeedleBytes,
    /// A Rabin-Karp hash of the needle.
    ///
    /// This is store here instead of in a more specific Rabin-Karp search
    /// since Rabin-Karp may be used even if another SearchKind corresponds
    /// to some other search implementation. e.g., If measurements suggest RK
    /// is faster in some cases or if a search implementation can't handle
    /// particularly small haystack. (Moreover, we cannot use RK *generally*,
    /// since its worst case time is multiplicative. Instead, we only use it
    /// some small haystacks, where "small" is a constant.)
    pub(crate) nhash: NeedleHash,
}

/// Configuration for substring search.
#[derive(Clone, Copy, Debug, Default)]
struct SearcherConfig {
    /// This permits changing the behavior of the prefilter, since it can have
    /// a variable impact on performance.
    prefilter: Prefilter,
}

#[derive(Clone, Debug)]
enum SearcherKind {
    /// A special case for empty needles. An empty needle always matches, even
    /// in an empty haystack.
    Empty,
    /// This is used whenever the needle is a single byte. In this case, we
    /// always use memchr.
    OneByte(u8),
    /// Two-Way is the generic work horse and is what provides our additive
    /// linear time guarantee. In general, it's used when the needle is bigger
    /// than 8 bytes or so.
    TwoWay(twoway::Forward),
    #[cfg(all(not(miri), target_arch = "x86_64", memchr_runtime_simd))]
    GenericSIMD128(x86::sse::Forward),
    #[cfg(memchr_runtime_wasm128)]
    GenericSIMD128(wasm::Forward),
    #[cfg(all(not(miri), target_arch = "x86_64", memchr_runtime_simd))]
    GenericSIMD256(x86::avx::Forward),
}

impl<'n> Searcher<'n> {
    fn new(config: SearcherConfig, needle: &'n [u8]) -> Searcher<'n> {
        use self::SearcherKind::*;

        let ninfo = NeedleInfo::new(needle);
        let mk = |kind: SearcherKind| {
            let prefn = prefilter::forward(
                &config.prefilter,
                &ninfo.rarebytes,
                needle,
            );
            Searcher { needle: CowBytes::new(needle), ninfo, prefn, kind }
        };
        if needle.len() == 0 {
            return mk(Empty);
        }
        if needle.len() == 1 {
            return mk(OneByte(needle[0]));
        }
        #[cfg(all(not(miri), target_arch = "x86_64", memchr_runtime_simd))]
        {
            if let Some(fwd) = x86::avx::Forward::new(&ninfo, needle) {
                return mk(GenericSIMD256(fwd));
            } else if let Some(fwd) = x86::sse::Forward::new(&ninfo, needle) {
                return mk(GenericSIMD128(fwd));
            }
        }
        #[cfg(all(target_arch = "wasm32", memchr_runtime_simd))]
        {
            if let Some(fwd) = wasm::Forward::new(&ninfo, needle) {
                return mk(GenericSIMD128(fwd));
            }
        }

        mk(TwoWay(twoway::Forward::new(needle)))
    }

    /// Return a fresh prefilter state that can be used with this searcher.
    /// A prefilter state is used to track the effectiveness of a searcher's
    /// prefilter for speeding up searches. Therefore, the prefilter state
    /// should generally be reused on subsequent searches (such as in an
    /// iterator). For searches on a different haystack, then a new prefilter
    /// state should be used.
    ///
    /// This always initializes a valid (but possibly inert) prefilter state
    /// even if this searcher does not have a prefilter enabled.
    fn prefilter_state(&self) -> PrefilterState {
        if self.prefn.is_none() {
            PrefilterState::inert()
        } else {
            PrefilterState::new()
        }
    }

    fn needle(&self) -> &[u8] {
        self.needle.as_slice()
    }

    fn as_ref(&self) -> Searcher<'_> {
        use self::SearcherKind::*;

        let kind = match self.kind {
            Empty => Empty,
            OneByte(b) => OneByte(b),
            TwoWay(tw) => TwoWay(tw),
            #[cfg(all(not(miri), memchr_runtime_simd))]
            GenericSIMD128(gs) => GenericSIMD128(gs),
            #[cfg(all(
                not(miri),
                target_arch = "x86_64",
                memchr_runtime_simd
            ))]
            GenericSIMD256(gs) => GenericSIMD256(gs),
        };
        Searcher {
            needle: CowBytes::new(self.needle()),
            ninfo: self.ninfo,
            prefn: self.prefn,
            kind,
        }
    }

    #[cfg(feature = "std")]
    fn into_owned(self) -> Searcher<'static> {
        use self::SearcherKind::*;

        let kind = match self.kind {
            Empty => Empty,
            OneByte(b) => OneByte(b),
            TwoWay(tw) => TwoWay(tw),
            #[cfg(all(not(miri), memchr_runtime_simd))]
            GenericSIMD128(gs) => GenericSIMD128(gs),
            #[cfg(all(
                not(miri),
                target_arch = "x86_64",
                memchr_runtime_simd
            ))]
            GenericSIMD256(gs) => GenericSIMD256(gs),
        };
        Searcher {
            needle: self.needle.into_owned(),
            ninfo: self.ninfo,
            prefn: self.prefn,
            kind,
        }
    }

    /// Implements forward substring search by selecting the implementation
    /// chosen at construction and executing it on the given haystack with the
    /// prefilter's current state of effectiveness.
    #[inline(always)]
    fn find(
        &self,
        state: &mut PrefilterState,
        haystack: &[u8],
    ) -> Option<usize> {
        use self::SearcherKind::*;

        let needle = self.needle();
        if haystack.len() < needle.len() {
            return None;
        }
        match self.kind {
            Empty => Some(0),
            OneByte(b) => crate::memchr(b, haystack),
            TwoWay(ref tw) => {
                // For very short haystacks (e.g., where the prefilter probably
                // can't run), it's faster to just run RK.
                if rabinkarp::is_fast(haystack, needle) {
                    rabinkarp::find_with(&self.ninfo.nhash, haystack, needle)
                } else {
                    self.find_tw(tw, state, haystack, needle)
                }
            }
            #[cfg(all(not(miri), memchr_runtime_simd))]
            GenericSIMD128(ref gs) => {
                // The SIMD matcher can't handle particularly short haystacks,
                // so we fall back to RK in these cases.
                if haystack.len() < gs.min_haystack_len() {
                    rabinkarp::find_with(&self.ninfo.nhash, haystack, needle)
                } else {
                    gs.find(haystack, needle)
                }
            }
            #[cfg(all(
                not(miri),
                target_arch = "x86_64",
                memchr_runtime_simd
            ))]
            GenericSIMD256(ref gs) => {
                // The SIMD matcher can't handle particularly short haystacks,
                // so we fall back to RK in these cases.
                if haystack.len() < gs.min_haystack_len() {
                    rabinkarp::find_with(&self.ninfo.nhash, haystack, needle)
                } else {
                    gs.find(haystack, needle)
                }
            }
        }
    }

    /// Calls Two-Way on the given haystack/needle.
    ///
    /// This is marked as unlineable since it seems to have a better overall
    /// effect on benchmarks. However, this is one of those cases where
    /// inlining it results an improvement in other benchmarks too, so I
    /// suspect we just don't have enough data yet to make the right call here.
    ///
    /// I suspect the main problem is that this function contains two different
    /// inlined copies of Two-Way: one with and one without prefilters enabled.
    #[inline(never)]
    fn find_tw(
        &self,
        tw: &twoway::Forward,
        state: &mut PrefilterState,
        haystack: &[u8],
        needle: &[u8],
    ) -> Option<usize> {
        if let Some(prefn) = self.prefn {
            // We used to look at the length of a haystack here. That is, if
            // it was too small, then don't bother with the prefilter. But two
            // things changed: the prefilter falls back to memchr for small
            // haystacks, and, above, Rabin-Karp is employed for tiny haystacks
            // anyway.
            if state.is_effective() {
                let mut pre = Pre { state, prefn, ninfo: &self.ninfo };
                return tw.find(Some(&mut pre), haystack, needle);
            }
        }
        tw.find(None, haystack, needle)
    }
}

impl NeedleInfo {
    pub(crate) fn new(needle: &[u8]) -> NeedleInfo {
        NeedleInfo {
            rarebytes: RareNeedleBytes::forward(needle),
            nhash: NeedleHash::forward(needle),
        }
    }
}

/// The internal implementation of a reverse substring searcher.
///
/// See the forward searcher docs for more details. Currently, the reverse
/// searcher is considerably simpler since it lacks prefilter support. This
/// was done because it adds a lot of code, and more surface area to test. And
/// in particular, it's not clear whether a prefilter on reverse searching is
/// worth it. (If you have a compelling use case, please file an issue!)
#[derive(Clone, Debug)]
struct SearcherRev<'n> {
    /// The actual needle we're searching for.
    needle: CowBytes<'n>,
    /// A Rabin-Karp hash of the needle.
    nhash: NeedleHash,
    /// The actual substring implementation in use.
    kind: SearcherRevKind,
}

#[derive(Clone, Debug)]
enum SearcherRevKind {
    /// A special case for empty needles. An empty needle always matches, even
    /// in an empty haystack.
    Empty,
    /// This is used whenever the needle is a single byte. In this case, we
    /// always use memchr.
    OneByte(u8),
    /// Two-Way is the generic work horse and is what provides our additive
    /// linear time guarantee. In general, it's used when the needle is bigger
    /// than 8 bytes or so.
    TwoWay(twoway::Reverse),
}

impl<'n> SearcherRev<'n> {
    fn new(needle: &'n [u8]) -> SearcherRev<'n> {
        use self::SearcherRevKind::*;

        let kind = if needle.len() == 0 {
            Empty
        } else if needle.len() == 1 {
            OneByte(needle[0])
        } else {
            TwoWay(twoway::Reverse::new(needle))
        };
        SearcherRev {
            needle: CowBytes::new(needle),
            nhash: NeedleHash::reverse(needle),
            kind,
        }
    }

    fn needle(&self) -> &[u8] {
        self.needle.as_slice()
    }

    fn as_ref(&self) -> SearcherRev<'_> {
        use self::SearcherRevKind::*;

        let kind = match self.kind {
            Empty => Empty,
            OneByte(b) => OneByte(b),
            TwoWay(tw) => TwoWay(tw),
        };
        SearcherRev {
            needle: CowBytes::new(self.needle()),
            nhash: self.nhash,
            kind,
        }
    }

    #[cfg(feature = "std")]
    fn into_owned(self) -> SearcherRev<'static> {
        use self::SearcherRevKind::*;

        let kind = match self.kind {
            Empty => Empty,
            OneByte(b) => OneByte(b),
            TwoWay(tw) => TwoWay(tw),
        };
        SearcherRev {
            needle: self.needle.into_owned(),
            nhash: self.nhash,
            kind,
        }
    }

    /// Implements reverse substring search by selecting the implementation
    /// chosen at construction and executing it on the given haystack with the
    /// prefilter's current state of effectiveness.
    #[inline(always)]
    fn rfind(&self, haystack: &[u8]) -> Option<usize> {
        use self::SearcherRevKind::*;

        let needle = self.needle();
        if haystack.len() < needle.len() {
            return None;
        }
        match self.kind {
            Empty => Some(haystack.len()),
            OneByte(b) => crate::memrchr(b, haystack),
            TwoWay(ref tw) => {
                // For very short haystacks (e.g., where the prefilter probably
                // can't run), it's faster to just run RK.
                if rabinkarp::is_fast(haystack, needle) {
                    rabinkarp::rfind_with(&self.nhash, haystack, needle)
                } else {
                    tw.rfind(haystack, needle)
                }
            }
        }
    }
}

/// This module defines some generic quickcheck properties useful for testing
/// any substring search algorithm. It also runs those properties for the
/// top-level public API memmem routines. (The properties are also used to
/// test various substring search implementations more granularly elsewhere as
/// well.)
#[cfg(all(test, feature = "std", not(miri)))]
mod proptests {
    // N.B. This defines the quickcheck tests using the properties defined
    // below. Because of macro-visibility weirdness, the actual macro is
    // defined at the top of this file.
    define_memmem_quickcheck_tests!(super::find, super::rfind);

    /// Check that every prefix of the given byte string is a substring.
    pub(crate) fn prefix_is_substring(
        reverse: bool,
        bs: &[u8],
        mut search: impl FnMut(&[u8], &[u8]) -> Option<usize>,
    ) -> bool {
        if bs.is_empty() {
            return true;
        }
        for i in 0..(bs.len() - 1) {
            let prefix = &bs[..i];
            if reverse {
                assert_eq!(naive_rfind(bs, prefix), search(bs, prefix));
            } else {
                assert_eq!(naive_find(bs, prefix), search(bs, prefix));
            }
        }
        true
    }

    /// Check that every suffix of the given byte string is a substring.
    pub(crate) fn suffix_is_substring(
        reverse: bool,
        bs: &[u8],
        mut search: impl FnMut(&[u8], &[u8]) -> Option<usize>,
    ) -> bool {
        if bs.is_empty() {
            return true;
        }
        for i in 0..(bs.len() - 1) {
            let suffix = &bs[i..];
            if reverse {
                assert_eq!(naive_rfind(bs, suffix), search(bs, suffix));
            } else {
                assert_eq!(naive_find(bs, suffix), search(bs, suffix));
            }
        }
        true
    }

    /// Check that naive substring search matches the result of the given search
    /// algorithm.
    pub(crate) fn matches_naive(
        reverse: bool,
        haystack: &[u8],
        needle: &[u8],
        mut search: impl FnMut(&[u8], &[u8]) -> Option<usize>,
    ) -> bool {
        if reverse {
            naive_rfind(haystack, needle) == search(haystack, needle)
        } else {
            naive_find(haystack, needle) == search(haystack, needle)
        }
    }

    /// Naively search forwards for the given needle in the given haystack.
    fn naive_find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        if needle.is_empty() {
            return Some(0);
        } else if haystack.len() < needle.len() {
            return None;
        }
        for i in 0..(haystack.len() - needle.len() + 1) {
            if needle == &haystack[i..i + needle.len()] {
                return Some(i);
            }
        }
        None
    }

    /// Naively search in reverse for the given needle in the given haystack.
    fn naive_rfind(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        if needle.is_empty() {
            return Some(haystack.len());
        } else if haystack.len() < needle.len() {
            return None;
        }
        for i in (0..(haystack.len() - needle.len() + 1)).rev() {
            if needle == &haystack[i..i + needle.len()] {
                return Some(i);
            }
        }
        None
    }
}

/// This module defines some hand-written "simple" substring tests. It
/// also provides routines for easily running them on any substring search
/// implementation.
#[cfg(test)]
mod testsimples {
    define_memmem_simple_tests!(super::find, super::rfind);

    /// Each test is a (needle, haystack, expected_fwd, expected_rev) tuple.
    type SearchTest =
        (&'static str, &'static str, Option<usize>, Option<usize>);

    const SEARCH_TESTS: &'static [SearchTest] = &[
        ("", "", Some(0), Some(0)),
        ("", "a", Some(0), Some(1)),
        ("", "ab", Some(0), Some(2)),
        ("", "abc", Some(0), Some(3)),
        ("a", "", None, None),
        ("a", "a", Some(0), Some(0)),
        ("a", "aa", Some(0), Some(1)),
        ("a", "ba", Some(1), Some(1)),
        ("a", "bba", Some(2), Some(2)),
        ("a", "bbba", Some(3), Some(3)),
        ("a", "bbbab", Some(3), Some(3)),
        ("a", "bbbabb", Some(3), Some(3)),
        ("a", "bbbabbb", Some(3), Some(3)),
        ("a", "bbbbbb", None, None),
        ("ab", "", None, None),
        ("ab", "a", None, None),
        ("ab", "b", None, None),
        ("ab", "ab", Some(0), Some(0)),
        ("ab", "aab", Some(1), Some(1)),
        ("ab", "aaab", Some(2), Some(2)),
        ("ab", "abaab", Some(0), Some(3)),
        ("ab", "baaab", Some(3), Some(3)),
        ("ab", "acb", None, None),
        ("ab", "abba", Some(0), Some(0)),
        ("abc", "ab", None, None),
        ("abc", "abc", Some(0), Some(0)),
        ("abc", "abcz", Some(0), Some(0)),
        ("abc", "abczz", Some(0), Some(0)),
        ("abc", "zabc", Some(1), Some(1)),
        ("abc", "zzabc", Some(2), Some(2)),
        ("abc", "azbc", None, None),
        ("abc", "abzc", None, None),
        ("abczdef", "abczdefzzzzzzzzzzzzzzzzzzzz", Some(0), Some(0)),
        ("abczdef", "zzzzzzzzzzzzzzzzzzzzabczdef", Some(20), Some(20)),
        ("xyz", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaxyz", Some(32), Some(32)),
        // Failures caught by quickcheck.
        ("\u{0}\u{15}", "\u{0}\u{15}\u{15}\u{0}", Some(0), Some(0)),
        ("\u{0}\u{1e}", "\u{1e}\u{0}", None, None),
    ];

    /// Run the substring search tests. `search` should be a closure that
    /// accepts a haystack and a needle and returns the starting position
    /// of the first occurrence of needle in the haystack, or `None` if one
    /// doesn't exist.
    pub(crate) fn run_search_tests_fwd(
        mut search: impl FnMut(&[u8], &[u8]) -> Option<usize>,
    ) {
        for &(needle, haystack, expected_fwd, _) in SEARCH_TESTS {
            let (n, h) = (needle.as_bytes(), haystack.as_bytes());
            assert_eq!(
                expected_fwd,
                search(h, n),
                "needle: {:?}, haystack: {:?}, expected: {:?}",
                n,
                h,
                expected_fwd
            );
        }
    }

    /// Run the substring search tests. `search` should be a closure that
    /// accepts a haystack and a needle and returns the starting position of
    /// the last occurrence of needle in the haystack, or `None` if one doesn't
    /// exist.
    pub(crate) fn run_search_tests_rev(
        mut search: impl FnMut(&[u8], &[u8]) -> Option<usize>,
    ) {
        for &(needle, haystack, _, expected_rev) in SEARCH_TESTS {
            let (n, h) = (needle.as_bytes(), haystack.as_bytes());
            assert_eq!(
                expected_rev,
                search(h, n),
                "needle: {:?}, haystack: {:?}, expected: {:?}",
                n,
                h,
                expected_rev
            );
        }
    }
}
