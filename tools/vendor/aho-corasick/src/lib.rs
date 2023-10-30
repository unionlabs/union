/*!
A library for finding occurrences of many patterns at once. This library
provides multiple pattern search principally through an implementation of the
[Aho-Corasick algorithm](https://en.wikipedia.org/wiki/Aho%E2%80%93Corasick_algorithm),
which builds a fast finite state machine for executing searches in linear time.

Additionally, this library provides a number of configuration options for
building the automaton that permit controlling the space versus time trade
off. Other features include simple ASCII case insensitive matching, finding
overlapping matches, replacements, searching streams and even searching and
replacing text in streams.

Finally, unlike all other (known) Aho-Corasick implementations, this one
supports enabling
[leftmost-first](enum.MatchKind.html#variant.LeftmostFirst)
or
[leftmost-longest](enum.MatchKind.html#variant.LeftmostFirst)
match semantics, using a (seemingly) novel alternative construction algorithm.
For more details on what match semantics means, see the
[`MatchKind`](enum.MatchKind.html)
type.

# Overview

This section gives a brief overview of the primary types in this crate:

* [`AhoCorasick`](struct.AhoCorasick.html) is the primary type and represents
  an Aho-Corasick automaton. This is the type you use to execute searches.
* [`AhoCorasickBuilder`](struct.AhoCorasickBuilder.html) can be used to build
  an Aho-Corasick automaton, and supports configuring a number of options.
* [`Match`](struct.Match.html) represents a single match reported by an
  Aho-Corasick automaton. Each match has two pieces of information: the pattern
  that matched and the start and end byte offsets corresponding to the position
  in the haystack at which it matched.

Additionally, the [`packed`](packed/index.html) sub-module contains a lower
level API for using fast vectorized routines for finding a small number of
patterns in a haystack.

# Example: basic searching

This example shows how to search for occurrences of multiple patterns
simultaneously. Each match includes the pattern that matched along with the
byte offsets of the match.

```
use aho_corasick::AhoCorasick;

let patterns = &["apple", "maple", "Snapple"];
let haystack = "Nobody likes maple in their apple flavored Snapple.";

let ac = AhoCorasick::new(patterns);
let mut matches = vec![];
for mat in ac.find_iter(haystack) {
    matches.push((mat.pattern(), mat.start(), mat.end()));
}
assert_eq!(matches, vec![
    (1, 13, 18),
    (0, 28, 33),
    (2, 43, 50),
]);
```

# Example: case insensitivity

This is like the previous example, but matches `Snapple` case insensitively
using `AhoCorasickBuilder`:

```
use aho_corasick::AhoCorasickBuilder;

let patterns = &["apple", "maple", "snapple"];
let haystack = "Nobody likes maple in their apple flavored Snapple.";

let ac = AhoCorasickBuilder::new()
    .ascii_case_insensitive(true)
    .build(patterns);
let mut matches = vec![];
for mat in ac.find_iter(haystack) {
    matches.push((mat.pattern(), mat.start(), mat.end()));
}
assert_eq!(matches, vec![
    (1, 13, 18),
    (0, 28, 33),
    (2, 43, 50),
]);
```

# Example: replacing matches in a stream

This example shows how to execute a search and replace on a stream without
loading the entire stream into memory first.

```
use aho_corasick::AhoCorasick;

# fn example() -> Result<(), ::std::io::Error> {
let patterns = &["fox", "brown", "quick"];
let replace_with = &["sloth", "grey", "slow"];

// In a real example, these might be `std::fs::File`s instead. All you need to
// do is supply a pair of `std::io::Read` and `std::io::Write` implementations.
let rdr = "The quick brown fox.";
let mut wtr = vec![];

let ac = AhoCorasick::new(patterns);
ac.stream_replace_all(rdr.as_bytes(), &mut wtr, replace_with)?;
assert_eq!(b"The slow grey sloth.".to_vec(), wtr);
# Ok(()) }; example().unwrap()
```

# Example: finding the leftmost first match

In the textbook description of Aho-Corasick, its formulation is typically
structured such that it reports all possible matches, even when they overlap
with another. In many cases, overlapping matches may not be desired, such as
the case of finding all successive non-overlapping matches like you might with
a standard regular expression.

Unfortunately the "obvious" way to modify the Aho-Corasick algorithm to do
this doesn't always work in the expected way, since it will report matches as
soon as they are seen. For example, consider matching the regex `Samwise|Sam`
against the text `Samwise`. Most regex engines (that are Perl-like, or
non-POSIX) will report `Samwise` as a match, but the standard Aho-Corasick
algorithm modified for reporting non-overlapping matches will report `Sam`.

A novel contribution of this library is the ability to change the match
semantics of Aho-Corasick (without additional search time overhead) such that
`Samwise` is reported instead. For example, here's the standard approach:

```
use aho_corasick::AhoCorasick;

let patterns = &["Samwise", "Sam"];
let haystack = "Samwise";

let ac = AhoCorasick::new(patterns);
let mat = ac.find(haystack).expect("should have a match");
assert_eq!("Sam", &haystack[mat.start()..mat.end()]);
```

And now here's the leftmost-first version, which matches how a Perl-like
regex will work:

```
use aho_corasick::{AhoCorasickBuilder, MatchKind};

let patterns = &["Samwise", "Sam"];
let haystack = "Samwise";

let ac = AhoCorasickBuilder::new()
    .match_kind(MatchKind::LeftmostFirst)
    .build(patterns);
let mat = ac.find(haystack).expect("should have a match");
assert_eq!("Samwise", &haystack[mat.start()..mat.end()]);
```

In addition to leftmost-first semantics, this library also supports
leftmost-longest semantics, which match the POSIX behavior of a regular
expression alternation. See
[`MatchKind`](enum.MatchKind.html)
for more details.

# Prefilters

While an Aho-Corasick automaton can perform admirably when compared to more
naive solutions, it is generally slower than more specialized algorithms that
are accelerated using vector instructions such as SIMD.

For that reason, this library will internally use a "prefilter" to attempt
to accelerate searches when possible. Currently, this library has several
different algorithms it might use depending on the patterns provided. Once the
number of patterns gets too big, prefilters are no longer used.

While a prefilter is generally good to have on by default since it works
well in the common case, it can lead to less predictable or even sub-optimal
performance in some cases. For that reason, prefilters can be explicitly
disabled via
[`AhoCorasickBuilder::prefilter`](struct.AhoCorasickBuilder.html#method.prefilter).
*/

#![deny(missing_docs)]

// We can never be truly no_std, but we could be alloc-only some day, so
// require the std feature for now.
#[cfg(not(feature = "std"))]
compile_error!("`std` feature is currently required to build this crate");

// #[cfg(doctest)]
// #[macro_use]
// extern crate doc_comment;

// #[cfg(doctest)]
// doctest!("../README.md");

pub use crate::ahocorasick::{
    AhoCorasick, AhoCorasickBuilder, FindIter, FindOverlappingIter, MatchKind,
    StreamFindIter,
};
pub use crate::error::{Error, ErrorKind};
pub use crate::state_id::StateID;

mod ahocorasick;
mod automaton;
mod buffer;
mod byte_frequencies;
mod classes;
mod dfa;
mod error;
mod nfa;
pub mod packed;
mod prefilter;
mod state_id;
#[cfg(test)]
mod tests;

/// A representation of a match reported by an Aho-Corasick automaton.
///
/// A match has two essential pieces of information: the identifier of the
/// pattern that matched, along with the start and end offsets of the match
/// in the haystack.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use aho_corasick::AhoCorasick;
///
/// let ac = AhoCorasick::new(&[
///     "foo", "bar", "baz",
/// ]);
/// let mat = ac.find("xxx bar xxx").expect("should have a match");
/// assert_eq!(1, mat.pattern());
/// assert_eq!(4, mat.start());
/// assert_eq!(7, mat.end());
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Match {
    /// The pattern id.
    pattern: usize,
    /// The length of this match, such that the starting position of the match
    /// is `end - len`.
    ///
    /// We use length here because, other than the pattern id, the only
    /// information about each pattern that the automaton stores is its length.
    /// So using the length here is just a bit more natural. But it isn't
    /// technically required.
    len: usize,
    /// The end offset of the match, exclusive.
    end: usize,
}

impl Match {
    /// Returns the identifier of the pattern that matched.
    ///
    /// The identifier of a pattern is derived from the position in which it
    /// was originally inserted into the corresponding automaton. The first
    /// pattern has identifier `0`, and each subsequent pattern is `1`, `2`
    /// and so on.
    #[inline]
    pub fn pattern(&self) -> usize {
        self.pattern
    }

    /// The starting position of the match.
    #[inline]
    pub fn start(&self) -> usize {
        self.end - self.len
    }

    /// The ending position of the match.
    #[inline]
    pub fn end(&self) -> usize {
        self.end
    }

    /// Returns true if and only if this match is empty. That is, when
    /// `start() == end()`.
    ///
    /// An empty match can only be returned when the empty string was among
    /// the patterns used to build the Aho-Corasick automaton.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    fn increment(&self, by: usize) -> Match {
        Match { pattern: self.pattern, len: self.len, end: self.end + by }
    }

    #[inline]
    fn from_span(id: usize, start: usize, end: usize) -> Match {
        Match { pattern: id, len: end - start, end }
    }
}
