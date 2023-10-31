use std::io;

use crate::automaton::Automaton;
use crate::buffer::Buffer;
use crate::dfa::{self, DFA};
use crate::error::Result;
use crate::nfa::{self, NFA};
use crate::packed;
use crate::prefilter::{Prefilter, PrefilterState};
use crate::state_id::StateID;
use crate::Match;

/// An automaton for searching multiple strings in linear time.
///
/// The `AhoCorasick` type supports a few basic ways of constructing an
/// automaton, including
/// [`AhoCorasick::new`](struct.AhoCorasick.html#method.new)
/// and
/// [`AhoCorasick::new_auto_configured`](struct.AhoCorasick.html#method.new_auto_configured).
/// However, there are a fair number of configurable options that can be set
/// by using
/// [`AhoCorasickBuilder`](struct.AhoCorasickBuilder.html)
/// instead. Such options include, but are not limited to, how matches are
/// determined, simple case insensitivity, whether to use a DFA or not and
/// various knobs for controlling the space-vs-time trade offs taken when
/// building the automaton.
///
/// If you aren't sure where to start, try beginning with
/// [`AhoCorasick::new_auto_configured`](struct.AhoCorasick.html#method.new_auto_configured).
///
/// # Resource usage
///
/// Aho-Corasick automatons are always constructed in `O(p)` time, where `p`
/// is the combined length of all patterns being searched. With that said,
/// building an automaton can be fairly costly because of high constant
/// factors, particularly when enabling the
/// [DFA](struct.AhoCorasickBuilder.html#method.dfa)
/// option (which is disabled by default). For this reason, it's generally a
/// good idea to build an automaton once and reuse it as much as possible.
///
/// Aho-Corasick automatons can also use a fair bit of memory. To get a
/// concrete idea of how much memory is being used, try using the
/// [`AhoCorasick::heap_bytes`](struct.AhoCorasick.html#method.heap_bytes)
/// method.
///
/// # Examples
///
/// This example shows how to search for occurrences of multiple patterns
/// simultaneously in a case insensitive fashion. Each match includes the
/// pattern that matched along with the byte offsets of the match.
///
/// ```
/// use aho_corasick::AhoCorasickBuilder;
///
/// let patterns = &["apple", "maple", "snapple"];
/// let haystack = "Nobody likes maple in their apple flavored Snapple.";
///
/// let ac = AhoCorasickBuilder::new()
///     .ascii_case_insensitive(true)
///     .build(patterns);
/// let mut matches = vec![];
/// for mat in ac.find_iter(haystack) {
///     matches.push((mat.pattern(), mat.start(), mat.end()));
/// }
/// assert_eq!(matches, vec![
///     (1, 13, 18),
///     (0, 28, 33),
///     (2, 43, 50),
/// ]);
/// ```
///
/// This example shows how to replace matches with some other string:
///
/// ```
/// use aho_corasick::AhoCorasick;
///
/// let patterns = &["fox", "brown", "quick"];
/// let haystack = "The quick brown fox.";
/// let replace_with = &["sloth", "grey", "slow"];
///
/// let ac = AhoCorasick::new(patterns);
/// let result = ac.replace_all(haystack, replace_with);
/// assert_eq!(result, "The slow grey sloth.");
/// ```
#[derive(Clone, Debug)]
pub struct AhoCorasick<S: StateID = usize> {
    imp: Imp<S>,
    match_kind: MatchKind,
}

impl AhoCorasick {
    /// Create a new Aho-Corasick automaton using the default configuration.
    ///
    /// The default configuration optimizes for less space usage, but at the
    /// expense of longer search times. To change the configuration, use
    /// [`AhoCorasickBuilder`](struct.AhoCorasickBuilder.html)
    /// for fine-grained control, or
    /// [`AhoCorasick::new_auto_configured`](struct.AhoCorasick.html#method.new_auto_configured)
    /// for automatic configuration if you aren't sure which settings to pick.
    ///
    /// This uses the default
    /// [`MatchKind::Standard`](enum.MatchKind.html#variant.Standard)
    /// match semantics, which reports a match as soon as it is found. This
    /// corresponds to the standard match semantics supported by textbook
    /// descriptions of the Aho-Corasick algorithm.
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
    /// assert_eq!(Some(1), ac.find("xxx bar xxx").map(|m| m.pattern()));
    /// ```
    pub fn new<I, P>(patterns: I) -> AhoCorasick
    where
        I: IntoIterator<Item = P>,
        P: AsRef<[u8]>,
    {
        AhoCorasickBuilder::new().build(patterns)
    }

    /// Build an Aho-Corasick automaton with an automatically determined
    /// configuration.
    ///
    /// Specifically, this requires a slice of patterns instead of an iterator
    /// since the configuration is determined by looking at the patterns before
    /// constructing the automaton. The idea here is to balance space and time
    /// automatically. That is, when searching a small number of patterns, this
    /// will attempt to use the fastest possible configuration since the total
    /// space required will be small anyway. As the number of patterns grows,
    /// this will fall back to slower configurations that use less space.
    ///
    /// If you want auto configuration but with match semantics different from
    /// the default `MatchKind::Standard`, then use
    /// [`AhoCorasickBuilder::auto_configure`](struct.AhoCorasickBuilder.html#method.auto_configure).
    ///
    /// # Examples
    ///
    /// Basic usage is just like `new`, except you must provide a slice:
    ///
    /// ```
    /// use aho_corasick::AhoCorasick;
    ///
    /// let ac = AhoCorasick::new_auto_configured(&[
    ///     "foo", "bar", "baz",
    /// ]);
    /// assert_eq!(Some(1), ac.find("xxx bar xxx").map(|m| m.pattern()));
    /// ```
    pub fn new_auto_configured<B>(patterns: &[B]) -> AhoCorasick
    where
        B: AsRef<[u8]>,
    {
        AhoCorasickBuilder::new().auto_configure(patterns).build(patterns)
    }
}

impl<S: StateID> AhoCorasick<S> {
    /// Returns true if and only if this automaton matches the haystack at any
    /// position.
    ///
    /// `haystack` may be any type that is cheaply convertible to a `&[u8]`.
    /// This includes, but is not limited to, `String`, `&str`, `Vec<u8>`, and
    /// `&[u8]` itself.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::AhoCorasick;
    ///
    /// let ac = AhoCorasick::new(&[
    ///     "foo", "bar", "quux", "baz",
    /// ]);
    /// assert!(ac.is_match("xxx bar xxx"));
    /// assert!(!ac.is_match("xxx qux xxx"));
    /// ```
    pub fn is_match<B: AsRef<[u8]>>(&self, haystack: B) -> bool {
        self.earliest_find(haystack).is_some()
    }

    /// Returns the location of the first detected match in `haystack`.
    ///
    /// This method has the same behavior regardless of the
    /// [`MatchKind`](enum.MatchKind.html)
    /// of this automaton.
    ///
    /// `haystack` may be any type that is cheaply convertible to a `&[u8]`.
    /// This includes, but is not limited to, `String`, `&str`, `Vec<u8>`, and
    /// `&[u8]` itself.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::AhoCorasick;
    ///
    /// let ac = AhoCorasick::new(&[
    ///     "abc", "b",
    /// ]);
    /// let mat = ac.earliest_find("abcd").expect("should have match");
    /// assert_eq!(1, mat.pattern());
    /// assert_eq!((1, 2), (mat.start(), mat.end()));
    /// ```
    pub fn earliest_find<B: AsRef<[u8]>>(&self, haystack: B) -> Option<Match> {
        let mut prestate = PrefilterState::new(self.max_pattern_len());
        let mut start = self.imp.start_state();
        self.imp.earliest_find_at(
            &mut prestate,
            haystack.as_ref(),
            0,
            &mut start,
        )
    }

    /// Returns the location of the first match according to the match
    /// semantics that this automaton was constructed with.
    ///
    /// When using `MatchKind::Standard`, this corresponds precisely to the
    /// same behavior as
    /// [`earliest_find`](struct.AhoCorasick.html#method.earliest_find).
    /// Otherwise, match semantics correspond to either
    /// [leftmost-first](enum.MatchKind.html#variant.LeftmostFirst)
    /// or
    /// [leftmost-longest](enum.MatchKind.html#variant.LeftmostLongest).
    ///
    /// `haystack` may be any type that is cheaply convertible to a `&[u8]`.
    /// This includes, but is not limited to, `String`, `&str`, `Vec<u8>`, and
    /// `&[u8]` itself.
    ///
    /// # Examples
    ///
    /// Basic usage, with standard semantics:
    ///
    /// ```
    /// use aho_corasick::{AhoCorasickBuilder, MatchKind};
    ///
    /// let patterns = &["b", "abc", "abcd"];
    /// let haystack = "abcd";
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .match_kind(MatchKind::Standard) // default, not necessary
    ///     .build(patterns);
    /// let mat = ac.find(haystack).expect("should have a match");
    /// assert_eq!("b", &haystack[mat.start()..mat.end()]);
    /// ```
    ///
    /// Now with leftmost-first semantics:
    ///
    /// ```
    /// use aho_corasick::{AhoCorasickBuilder, MatchKind};
    ///
    /// let patterns = &["b", "abc", "abcd"];
    /// let haystack = "abcd";
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .match_kind(MatchKind::LeftmostFirst)
    ///     .build(patterns);
    /// let mat = ac.find(haystack).expect("should have a match");
    /// assert_eq!("abc", &haystack[mat.start()..mat.end()]);
    /// ```
    ///
    /// And finally, leftmost-longest semantics:
    ///
    /// ```
    /// use aho_corasick::{AhoCorasickBuilder, MatchKind};
    ///
    /// let patterns = &["b", "abc", "abcd"];
    /// let haystack = "abcd";
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .match_kind(MatchKind::LeftmostLongest)
    ///     .build(patterns);
    /// let mat = ac.find(haystack).expect("should have a match");
    /// assert_eq!("abcd", &haystack[mat.start()..mat.end()]);
    /// ```
    pub fn find<B: AsRef<[u8]>>(&self, haystack: B) -> Option<Match> {
        let mut prestate = PrefilterState::new(self.max_pattern_len());
        self.imp.find_at_no_state(&mut prestate, haystack.as_ref(), 0)
    }

    /// Returns an iterator of non-overlapping matches, using the match
    /// semantics that this automaton was constructed with.
    ///
    /// `haystack` may be any type that is cheaply convertible to a `&[u8]`.
    /// This includes, but is not limited to, `String`, `&str`, `Vec<u8>`, and
    /// `&[u8]` itself.
    ///
    /// # Examples
    ///
    /// Basic usage, with standard semantics:
    ///
    /// ```
    /// use aho_corasick::{AhoCorasickBuilder, MatchKind};
    ///
    /// let patterns = &["append", "appendage", "app"];
    /// let haystack = "append the app to the appendage";
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .match_kind(MatchKind::Standard) // default, not necessary
    ///     .build(patterns);
    /// let matches: Vec<usize> = ac
    ///     .find_iter(haystack)
    ///     .map(|mat| mat.pattern())
    ///     .collect();
    /// assert_eq!(vec![2, 2, 2], matches);
    /// ```
    ///
    /// Now with leftmost-first semantics:
    ///
    /// ```
    /// use aho_corasick::{AhoCorasickBuilder, MatchKind};
    ///
    /// let patterns = &["append", "appendage", "app"];
    /// let haystack = "append the app to the appendage";
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .match_kind(MatchKind::LeftmostFirst)
    ///     .build(patterns);
    /// let matches: Vec<usize> = ac
    ///     .find_iter(haystack)
    ///     .map(|mat| mat.pattern())
    ///     .collect();
    /// assert_eq!(vec![0, 2, 0], matches);
    /// ```
    ///
    /// And finally, leftmost-longest semantics:
    ///
    /// ```
    /// use aho_corasick::{AhoCorasickBuilder, MatchKind};
    ///
    /// let patterns = &["append", "appendage", "app"];
    /// let haystack = "append the app to the appendage";
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .match_kind(MatchKind::LeftmostLongest)
    ///     .build(patterns);
    /// let matches: Vec<usize> = ac
    ///     .find_iter(haystack)
    ///     .map(|mat| mat.pattern())
    ///     .collect();
    /// assert_eq!(vec![0, 2, 1], matches);
    /// ```
    pub fn find_iter<'a, 'b, B: ?Sized + AsRef<[u8]>>(
        &'a self,
        haystack: &'b B,
    ) -> FindIter<'a, 'b, S> {
        FindIter::new(self, haystack.as_ref())
    }

    /// Returns an iterator of overlapping matches in the given `haystack`.
    ///
    /// Overlapping matches can _only_ be detected using
    /// `MatchKind::Standard` semantics. If this automaton was constructed with
    /// leftmost semantics, then this method will panic. To determine whether
    /// this will panic at runtime, use the
    /// [`AhoCorasick::supports_overlapping`](struct.AhoCorasick.html#method.supports_overlapping)
    /// method.
    ///
    /// `haystack` may be any type that is cheaply convertible to a `&[u8]`.
    /// This includes, but is not limited to, `String`, `&str`, `Vec<u8>`, and
    /// `&[u8]` itself.
    ///
    /// # Panics
    ///
    /// This panics when `AhoCorasick::supports_overlapping` returns `false`.
    /// That is, this panics when this automaton's match semantics are not
    /// `MatchKind::Standard`.
    ///
    /// # Examples
    ///
    /// Basic usage, with standard semantics:
    ///
    /// ```
    /// use aho_corasick::AhoCorasick;
    ///
    /// let patterns = &["append", "appendage", "app"];
    /// let haystack = "append the app to the appendage";
    ///
    /// let ac = AhoCorasick::new(patterns);
    /// let matches: Vec<usize> = ac
    ///     .find_overlapping_iter(haystack)
    ///     .map(|mat| mat.pattern())
    ///     .collect();
    /// assert_eq!(vec![2, 0, 2, 2, 0, 1], matches);
    /// ```
    pub fn find_overlapping_iter<'a, 'b, B: ?Sized + AsRef<[u8]>>(
        &'a self,
        haystack: &'b B,
    ) -> FindOverlappingIter<'a, 'b, S> {
        FindOverlappingIter::new(self, haystack.as_ref())
    }

    /// Replace all matches with a corresponding value in the `replace_with`
    /// slice given. Matches correspond to the same matches as reported by
    /// [`find_iter`](struct.AhoCorasick.html#method.find_iter).
    ///
    /// Replacements are determined by the index of the matching pattern.
    /// For example, if the pattern with index `2` is found, then it is
    /// replaced by `replace_with[2]`.
    ///
    /// # Panics
    ///
    /// This panics when `replace_with.len()` does not equal the total number
    /// of patterns that are matched by this automaton.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::{AhoCorasickBuilder, MatchKind};
    ///
    /// let patterns = &["append", "appendage", "app"];
    /// let haystack = "append the app to the appendage";
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .match_kind(MatchKind::LeftmostFirst)
    ///     .build(patterns);
    /// let result = ac.replace_all(haystack, &["x", "y", "z"]);
    /// assert_eq!("x the z to the xage", result);
    /// ```
    pub fn replace_all<B>(&self, haystack: &str, replace_with: &[B]) -> String
    where
        B: AsRef<str>,
    {
        assert_eq!(
            replace_with.len(),
            self.pattern_count(),
            "replace_all requires a replacement for every pattern \
             in the automaton"
        );
        let mut dst = String::with_capacity(haystack.len());
        self.replace_all_with(haystack, &mut dst, |mat, _, dst| {
            dst.push_str(replace_with[mat.pattern()].as_ref());
            true
        });
        dst
    }

    /// Replace all matches using raw bytes with a corresponding value in the
    /// `replace_with` slice given. Matches correspond to the same matches as
    /// reported by [`find_iter`](struct.AhoCorasick.html#method.find_iter).
    ///
    /// Replacements are determined by the index of the matching pattern.
    /// For example, if the pattern with index `2` is found, then it is
    /// replaced by `replace_with[2]`.
    ///
    /// # Panics
    ///
    /// This panics when `replace_with.len()` does not equal the total number
    /// of patterns that are matched by this automaton.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::{AhoCorasickBuilder, MatchKind};
    ///
    /// let patterns = &["append", "appendage", "app"];
    /// let haystack = b"append the app to the appendage";
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .match_kind(MatchKind::LeftmostFirst)
    ///     .build(patterns);
    /// let result = ac.replace_all_bytes(haystack, &["x", "y", "z"]);
    /// assert_eq!(b"x the z to the xage".to_vec(), result);
    /// ```
    pub fn replace_all_bytes<B>(
        &self,
        haystack: &[u8],
        replace_with: &[B],
    ) -> Vec<u8>
    where
        B: AsRef<[u8]>,
    {
        assert_eq!(
            replace_with.len(),
            self.pattern_count(),
            "replace_all_bytes requires a replacement for every pattern \
             in the automaton"
        );
        let mut dst = Vec::with_capacity(haystack.len());
        self.replace_all_with_bytes(haystack, &mut dst, |mat, _, dst| {
            dst.extend(replace_with[mat.pattern()].as_ref());
            true
        });
        dst
    }

    /// Replace all matches using a closure called on each match.
    /// Matches correspond to the same matches as reported by
    /// [`find_iter`](struct.AhoCorasick.html#method.find_iter).
    ///
    /// The closure accepts three parameters: the match found, the text of
    /// the match and a string buffer with which to write the replaced text
    /// (if any). If the closure returns `true`, then it continues to the next
    /// match. If the closure returns `false`, then searching is stopped.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::{AhoCorasickBuilder, MatchKind};
    ///
    /// let patterns = &["append", "appendage", "app"];
    /// let haystack = "append the app to the appendage";
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .match_kind(MatchKind::LeftmostFirst)
    ///     .build(patterns);
    /// let mut result = String::new();
    /// ac.replace_all_with(haystack, &mut result, |mat, _, dst| {
    ///     dst.push_str(&mat.pattern().to_string());
    ///     true
    /// });
    /// assert_eq!("0 the 2 to the 0age", result);
    /// ```
    ///
    /// Stopping the replacement by returning `false` (continued from the
    /// example above):
    ///
    /// ```
    /// # use aho_corasick::{AhoCorasickBuilder, MatchKind};
    /// # let patterns = &["append", "appendage", "app"];
    /// # let haystack = "append the app to the appendage";
    /// # let ac = AhoCorasickBuilder::new()
    /// #    .match_kind(MatchKind::LeftmostFirst)
    /// #    .build(patterns);
    /// let mut result = String::new();
    /// ac.replace_all_with(haystack, &mut result, |mat, _, dst| {
    ///     dst.push_str(&mat.pattern().to_string());
    ///     mat.pattern() != 2
    /// });
    /// assert_eq!("0 the 2 to the appendage", result);
    /// ```
    pub fn replace_all_with<F>(
        &self,
        haystack: &str,
        dst: &mut String,
        mut replace_with: F,
    ) where
        F: FnMut(&Match, &str, &mut String) -> bool,
    {
        let mut last_match = 0;
        for mat in self.find_iter(haystack) {
            dst.push_str(&haystack[last_match..mat.start()]);
            last_match = mat.end();
            if !replace_with(&mat, &haystack[mat.start()..mat.end()], dst) {
                break;
            };
        }
        dst.push_str(&haystack[last_match..]);
    }

    /// Replace all matches using raw bytes with a closure called on each
    /// match. Matches correspond to the same matches as reported by
    /// [`find_iter`](struct.AhoCorasick.html#method.find_iter).
    ///
    /// The closure accepts three parameters: the match found, the text of
    /// the match and a byte buffer with which to write the replaced text
    /// (if any). If the closure returns `true`, then it continues to the next
    /// match. If the closure returns `false`, then searching is stopped.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::{AhoCorasickBuilder, MatchKind};
    ///
    /// let patterns = &["append", "appendage", "app"];
    /// let haystack = b"append the app to the appendage";
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .match_kind(MatchKind::LeftmostFirst)
    ///     .build(patterns);
    /// let mut result = vec![];
    /// ac.replace_all_with_bytes(haystack, &mut result, |mat, _, dst| {
    ///     dst.extend(mat.pattern().to_string().bytes());
    ///     true
    /// });
    /// assert_eq!(b"0 the 2 to the 0age".to_vec(), result);
    /// ```
    ///
    /// Stopping the replacement by returning `false` (continued from the
    /// example above):
    ///
    /// ```
    /// # use aho_corasick::{AhoCorasickBuilder, MatchKind};
    /// # let patterns = &["append", "appendage", "app"];
    /// # let haystack = b"append the app to the appendage";
    /// # let ac = AhoCorasickBuilder::new()
    /// #    .match_kind(MatchKind::LeftmostFirst)
    /// #    .build(patterns);
    /// let mut result = vec![];
    /// ac.replace_all_with_bytes(haystack, &mut result, |mat, _, dst| {
    ///     dst.extend(mat.pattern().to_string().bytes());
    ///     mat.pattern() != 2
    /// });
    /// assert_eq!(b"0 the 2 to the appendage".to_vec(), result);
    /// ```
    pub fn replace_all_with_bytes<F>(
        &self,
        haystack: &[u8],
        dst: &mut Vec<u8>,
        mut replace_with: F,
    ) where
        F: FnMut(&Match, &[u8], &mut Vec<u8>) -> bool,
    {
        let mut last_match = 0;
        for mat in self.find_iter(haystack) {
            dst.extend(&haystack[last_match..mat.start()]);
            last_match = mat.end();
            if !replace_with(&mat, &haystack[mat.start()..mat.end()], dst) {
                break;
            };
        }
        dst.extend(&haystack[last_match..]);
    }

    /// Returns an iterator of non-overlapping matches in the given
    /// stream. Matches correspond to the same matches as reported by
    /// [`find_iter`](struct.AhoCorasick.html#method.find_iter).
    ///
    /// The matches yielded by this iterator use absolute position offsets in
    /// the stream given, where the first byte has index `0`. Matches are
    /// yieled until the stream is exhausted.
    ///
    /// Each item yielded by the iterator is an `io::Result<Match>`, where an
    /// error is yielded if there was a problem reading from the reader given.
    ///
    /// When searching a stream, an internal buffer is used. Therefore, callers
    /// should avoiding providing a buffered reader, if possible.
    ///
    /// Searching a stream requires that the automaton was built with
    /// `MatchKind::Standard` semantics. If this automaton was constructed
    /// with leftmost semantics, then this method will panic. To determine
    /// whether this will panic at runtime, use the
    /// [`AhoCorasick::supports_stream`](struct.AhoCorasick.html#method.supports_stream)
    /// method.
    ///
    /// # Memory usage
    ///
    /// In general, searching streams will use a constant amount of memory for
    /// its internal buffer. The one requirement is that the internal buffer
    /// must be at least the size of the longest possible match. In most use
    /// cases, the default buffer size will be much larger than any individual
    /// match.
    ///
    /// # Panics
    ///
    /// This panics when `AhoCorasick::supports_stream` returns `false`.
    /// That is, this panics when this automaton's match semantics are not
    /// `MatchKind::Standard`. This restriction may be lifted in the future.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::AhoCorasick;
    ///
    /// # fn example() -> Result<(), ::std::io::Error> {
    /// let patterns = &["append", "appendage", "app"];
    /// let haystack = "append the app to the appendage";
    ///
    /// let ac = AhoCorasick::new(patterns);
    /// let mut matches = vec![];
    /// for result in ac.stream_find_iter(haystack.as_bytes()) {
    ///     let mat = result?;
    ///     matches.push(mat.pattern());
    /// }
    /// assert_eq!(vec![2, 2, 2], matches);
    /// # Ok(()) }; example().unwrap()
    /// ```
    pub fn stream_find_iter<'a, R: io::Read>(
        &'a self,
        rdr: R,
    ) -> StreamFindIter<'a, R, S> {
        StreamFindIter::new(self, rdr)
    }

    /// Search for and replace all matches of this automaton in
    /// the given reader, and write the replacements to the given
    /// writer. Matches correspond to the same matches as reported by
    /// [`find_iter`](struct.AhoCorasick.html#method.find_iter).
    ///
    /// Replacements are determined by the index of the matching pattern.
    /// For example, if the pattern with index `2` is found, then it is
    /// replaced by `replace_with[2]`.
    ///
    /// After all matches are replaced, the writer is _not_ flushed.
    ///
    /// If there was a problem reading from the given reader or writing to the
    /// given writer, then the corresponding `io::Error` is returned and all
    /// replacement is stopped.
    ///
    /// When searching a stream, an internal buffer is used. Therefore, callers
    /// should avoiding providing a buffered reader, if possible. However,
    /// callers may want to provide a buffered writer.
    ///
    /// Searching a stream requires that the automaton was built with
    /// `MatchKind::Standard` semantics. If this automaton was constructed
    /// with leftmost semantics, then this method will panic. To determine
    /// whether this will panic at runtime, use the
    /// [`AhoCorasick::supports_stream`](struct.AhoCorasick.html#method.supports_stream)
    /// method.
    ///
    /// # Memory usage
    ///
    /// In general, searching streams will use a constant amount of memory for
    /// its internal buffer. The one requirement is that the internal buffer
    /// must be at least the size of the longest possible match. In most use
    /// cases, the default buffer size will be much larger than any individual
    /// match.
    ///
    /// # Panics
    ///
    /// This panics when `AhoCorasick::supports_stream` returns `false`.
    /// That is, this panics when this automaton's match semantics are not
    /// `MatchKind::Standard`. This restriction may be lifted in the future.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::AhoCorasick;
    ///
    /// # fn example() -> Result<(), ::std::io::Error> {
    /// let patterns = &["fox", "brown", "quick"];
    /// let haystack = "The quick brown fox.";
    /// let replace_with = &["sloth", "grey", "slow"];
    ///
    /// let ac = AhoCorasick::new(patterns);
    /// let mut result = vec![];
    /// ac.stream_replace_all(haystack.as_bytes(), &mut result, replace_with)?;
    /// assert_eq!(b"The slow grey sloth.".to_vec(), result);
    /// # Ok(()) }; example().unwrap()
    /// ```
    pub fn stream_replace_all<R, W, B>(
        &self,
        rdr: R,
        wtr: W,
        replace_with: &[B],
    ) -> io::Result<()>
    where
        R: io::Read,
        W: io::Write,
        B: AsRef<[u8]>,
    {
        assert_eq!(
            replace_with.len(),
            self.pattern_count(),
            "stream_replace_all requires a replacement for every pattern \
             in the automaton"
        );
        self.stream_replace_all_with(rdr, wtr, |mat, _, wtr| {
            wtr.write_all(replace_with[mat.pattern()].as_ref())
        })
    }

    /// Search the given reader and replace all matches of this automaton
    /// using the given closure. The result is written to the given
    /// writer. Matches correspond to the same matches as reported by
    /// [`find_iter`](struct.AhoCorasick.html#method.find_iter).
    ///
    /// The closure accepts three parameters: the match found, the text of
    /// the match and the writer with which to write the replaced text (if any).
    ///
    /// After all matches are replaced, the writer is _not_ flushed.
    ///
    /// If there was a problem reading from the given reader or writing to the
    /// given writer, then the corresponding `io::Error` is returned and all
    /// replacement is stopped.
    ///
    /// When searching a stream, an internal buffer is used. Therefore, callers
    /// should avoiding providing a buffered reader, if possible. However,
    /// callers may want to provide a buffered writer.
    ///
    /// Searching a stream requires that the automaton was built with
    /// `MatchKind::Standard` semantics. If this automaton was constructed
    /// with leftmost semantics, then this method will panic. To determine
    /// whether this will panic at runtime, use the
    /// [`AhoCorasick::supports_stream`](struct.AhoCorasick.html#method.supports_stream)
    /// method.
    ///
    /// # Memory usage
    ///
    /// In general, searching streams will use a constant amount of memory for
    /// its internal buffer. The one requirement is that the internal buffer
    /// must be at least the size of the longest possible match. In most use
    /// cases, the default buffer size will be much larger than any individual
    /// match.
    ///
    /// # Panics
    ///
    /// This panics when `AhoCorasick::supports_stream` returns `false`.
    /// That is, this panics when this automaton's match semantics are not
    /// `MatchKind::Standard`. This restriction may be lifted in the future.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::io::Write;
    /// use aho_corasick::AhoCorasick;
    ///
    /// # fn example() -> Result<(), ::std::io::Error> {
    /// let patterns = &["fox", "brown", "quick"];
    /// let haystack = "The quick brown fox.";
    ///
    /// let ac = AhoCorasick::new(patterns);
    /// let mut result = vec![];
    /// ac.stream_replace_all_with(
    ///     haystack.as_bytes(),
    ///     &mut result,
    ///     |mat, _, wtr| {
    ///         wtr.write_all(mat.pattern().to_string().as_bytes())
    ///     },
    /// )?;
    /// assert_eq!(b"The 2 1 0.".to_vec(), result);
    /// # Ok(()) }; example().unwrap()
    /// ```
    pub fn stream_replace_all_with<R, W, F>(
        &self,
        rdr: R,
        mut wtr: W,
        mut replace_with: F,
    ) -> io::Result<()>
    where
        R: io::Read,
        W: io::Write,
        F: FnMut(&Match, &[u8], &mut W) -> io::Result<()>,
    {
        let mut it = StreamChunkIter::new(self, rdr);
        while let Some(result) = it.next() {
            let chunk = result?;
            match chunk {
                StreamChunk::NonMatch { bytes, .. } => {
                    wtr.write_all(bytes)?;
                }
                StreamChunk::Match { bytes, mat } => {
                    replace_with(&mat, bytes, &mut wtr)?;
                }
            }
        }
        Ok(())
    }

    /// Returns the match kind used by this automaton.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::{AhoCorasick, MatchKind};
    ///
    /// let ac = AhoCorasick::new(&[
    ///     "foo", "bar", "quux", "baz",
    /// ]);
    /// assert_eq!(&MatchKind::Standard, ac.match_kind());
    /// ```
    pub fn match_kind(&self) -> &MatchKind {
        self.imp.match_kind()
    }

    /// Returns the length of the longest pattern matched by this automaton.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::AhoCorasick;
    ///
    /// let ac = AhoCorasick::new(&[
    ///     "foo", "bar", "quux", "baz",
    /// ]);
    /// assert_eq!(4, ac.max_pattern_len());
    /// ```
    pub fn max_pattern_len(&self) -> usize {
        self.imp.max_pattern_len()
    }

    /// Return the total number of patterns matched by this automaton.
    ///
    /// This includes patterns that may never participate in a match. For
    /// example, if
    /// [`MatchKind::LeftmostFirst`](enum.MatchKind.html#variant.LeftmostFirst)
    /// match semantics are used, and the patterns `Sam` and `Samwise` were
    /// used to build the automaton, then `Samwise` can never participate in a
    /// match because `Sam` will always take priority.
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
    /// assert_eq!(3, ac.pattern_count());
    /// ```
    pub fn pattern_count(&self) -> usize {
        self.imp.pattern_count()
    }

    /// Returns true if and only if this automaton supports reporting
    /// overlapping matches.
    ///
    /// If this returns false and overlapping matches are requested, then it
    /// will result in a panic.
    ///
    /// Since leftmost matching is inherently incompatible with overlapping
    /// matches, only
    /// [`MatchKind::Standard`](enum.MatchKind.html#variant.Standard)
    /// supports overlapping matches. This is unlikely to change in the future.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::{AhoCorasickBuilder, MatchKind};
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .match_kind(MatchKind::Standard)
    ///     .build(&["foo", "bar", "baz"]);
    /// assert!(ac.supports_overlapping());
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .match_kind(MatchKind::LeftmostFirst)
    ///     .build(&["foo", "bar", "baz"]);
    /// assert!(!ac.supports_overlapping());
    /// ```
    pub fn supports_overlapping(&self) -> bool {
        self.match_kind.supports_overlapping()
    }

    /// Returns true if and only if this automaton supports stream searching.
    ///
    /// If this returns false and stream searching (or replacing) is attempted,
    /// then it will result in a panic.
    ///
    /// Currently, only
    /// [`MatchKind::Standard`](enum.MatchKind.html#variant.Standard)
    /// supports streaming. This may be expanded in the future.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::{AhoCorasickBuilder, MatchKind};
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .match_kind(MatchKind::Standard)
    ///     .build(&["foo", "bar", "baz"]);
    /// assert!(ac.supports_stream());
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .match_kind(MatchKind::LeftmostFirst)
    ///     .build(&["foo", "bar", "baz"]);
    /// assert!(!ac.supports_stream());
    /// ```
    pub fn supports_stream(&self) -> bool {
        self.match_kind.supports_stream()
    }

    /// Returns the approximate total amount of heap used by this automaton, in
    /// units of bytes.
    ///
    /// # Examples
    ///
    /// This example shows the difference in heap usage between a few
    /// configurations:
    ///
    /// ```ignore
    /// use aho_corasick::{AhoCorasickBuilder, MatchKind};
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .dfa(false) // default
    ///     .build(&["foo", "bar", "baz"]);
    /// assert_eq!(10_336, ac.heap_bytes());
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .dfa(false) // default
    ///     .ascii_case_insensitive(true)
    ///     .build(&["foo", "bar", "baz"]);
    /// assert_eq!(10_384, ac.heap_bytes());
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .dfa(true)
    ///     .ascii_case_insensitive(true)
    ///     .build(&["foo", "bar", "baz"]);
    /// assert_eq!(1_248, ac.heap_bytes());
    /// ```
    pub fn heap_bytes(&self) -> usize {
        match self.imp {
            Imp::NFA(ref nfa) => nfa.heap_bytes(),
            Imp::DFA(ref dfa) => dfa.heap_bytes(),
        }
    }
}

/// The internal implementation of Aho-Corasick, which is either an NFA or
/// a DFA. The NFA is slower but uses less memory. The DFA is faster but uses
/// more memory.
#[derive(Clone, Debug)]
enum Imp<S: StateID> {
    NFA(NFA<S>),
    DFA(DFA<S>),
}

impl<S: StateID> Imp<S> {
    /// Returns the type of match semantics implemented by this automaton.
    fn match_kind(&self) -> &MatchKind {
        match *self {
            Imp::NFA(ref nfa) => nfa.match_kind(),
            Imp::DFA(ref dfa) => dfa.match_kind(),
        }
    }

    /// Returns the identifier of the start state.
    fn start_state(&self) -> S {
        match *self {
            Imp::NFA(ref nfa) => nfa.start_state(),
            Imp::DFA(ref dfa) => dfa.start_state(),
        }
    }

    /// The length, in bytes, of the longest pattern in this automaton. This
    /// information is useful for maintaining correct buffer sizes when
    /// searching on streams.
    fn max_pattern_len(&self) -> usize {
        match *self {
            Imp::NFA(ref nfa) => nfa.max_pattern_len(),
            Imp::DFA(ref dfa) => dfa.max_pattern_len(),
        }
    }

    /// The total number of patterns added to this automaton. This includes
    /// patterns that may never match. The maximum matching pattern that can be
    /// reported is exactly one less than this number.
    fn pattern_count(&self) -> usize {
        match *self {
            Imp::NFA(ref nfa) => nfa.pattern_count(),
            Imp::DFA(ref dfa) => dfa.pattern_count(),
        }
    }

    /// Returns the prefilter object, if one exists, for the underlying
    /// automaton.
    fn prefilter(&self) -> Option<&dyn Prefilter> {
        match *self {
            Imp::NFA(ref nfa) => nfa.prefilter(),
            Imp::DFA(ref dfa) => dfa.prefilter(),
        }
    }

    /// Returns true if and only if we should attempt to use a prefilter.
    fn use_prefilter(&self) -> bool {
        let p = match self.prefilter() {
            None => return false,
            Some(p) => p,
        };
        !p.looks_for_non_start_of_match()
    }

    #[inline(always)]
    fn overlapping_find_at(
        &self,
        prestate: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
        state_id: &mut S,
        match_index: &mut usize,
    ) -> Option<Match> {
        match *self {
            Imp::NFA(ref nfa) => nfa.overlapping_find_at(
                prestate,
                haystack,
                at,
                state_id,
                match_index,
            ),
            Imp::DFA(ref dfa) => dfa.overlapping_find_at(
                prestate,
                haystack,
                at,
                state_id,
                match_index,
            ),
        }
    }

    #[inline(always)]
    fn earliest_find_at(
        &self,
        prestate: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
        state_id: &mut S,
    ) -> Option<Match> {
        match *self {
            Imp::NFA(ref nfa) => {
                nfa.earliest_find_at(prestate, haystack, at, state_id)
            }
            Imp::DFA(ref dfa) => {
                dfa.earliest_find_at(prestate, haystack, at, state_id)
            }
        }
    }

    #[inline(always)]
    fn find_at_no_state(
        &self,
        prestate: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
    ) -> Option<Match> {
        match *self {
            Imp::NFA(ref nfa) => nfa.find_at_no_state(prestate, haystack, at),
            Imp::DFA(ref dfa) => dfa.find_at_no_state(prestate, haystack, at),
        }
    }
}

/// An iterator of non-overlapping matches in a particular haystack.
///
/// This iterator yields matches according to the
/// [`MatchKind`](enum.MatchKind.html)
/// used by this automaton.
///
/// This iterator is constructed via the
/// [`AhoCorasick::find_iter`](struct.AhoCorasick.html#method.find_iter)
/// method.
///
/// The type variable `S` refers to the representation used for state
/// identifiers. (By default, this is `usize`.)
///
/// The lifetime `'a` refers to the lifetime of the `AhoCorasick` automaton.
///
/// The lifetime `'b` refers to the lifetime of the haystack being searched.
#[derive(Debug)]
pub struct FindIter<'a, 'b, S: StateID> {
    fsm: &'a Imp<S>,
    prestate: PrefilterState,
    haystack: &'b [u8],
    pos: usize,
}

impl<'a, 'b, S: StateID> FindIter<'a, 'b, S> {
    fn new(ac: &'a AhoCorasick<S>, haystack: &'b [u8]) -> FindIter<'a, 'b, S> {
        let prestate = PrefilterState::new(ac.max_pattern_len());
        FindIter { fsm: &ac.imp, prestate, haystack, pos: 0 }
    }
}

impl<'a, 'b, S: StateID> Iterator for FindIter<'a, 'b, S> {
    type Item = Match;

    fn next(&mut self) -> Option<Match> {
        if self.pos > self.haystack.len() {
            return None;
        }
        let result = self.fsm.find_at_no_state(
            &mut self.prestate,
            self.haystack,
            self.pos,
        );
        let mat = match result {
            None => return None,
            Some(mat) => mat,
        };
        if mat.end() == self.pos {
            // If the automaton can match the empty string and if we found an
            // empty match, then we need to forcefully move the position.
            self.pos += 1;
        } else {
            self.pos = mat.end();
        }
        Some(mat)
    }
}

/// An iterator of overlapping matches in a particular haystack.
///
/// This iterator will report all possible matches in a particular haystack,
/// even when the matches overlap.
///
/// This iterator is constructed via the
/// [`AhoCorasick::find_overlapping_iter`](struct.AhoCorasick.html#method.find_overlapping_iter)
/// method.
///
/// The type variable `S` refers to the representation used for state
/// identifiers. (By default, this is `usize`.)
///
/// The lifetime `'a` refers to the lifetime of the `AhoCorasick` automaton.
///
/// The lifetime `'b` refers to the lifetime of the haystack being searched.
#[derive(Debug)]
pub struct FindOverlappingIter<'a, 'b, S: StateID> {
    fsm: &'a Imp<S>,
    prestate: PrefilterState,
    haystack: &'b [u8],
    pos: usize,
    last_match_end: usize,
    state_id: S,
    match_index: usize,
}

impl<'a, 'b, S: StateID> FindOverlappingIter<'a, 'b, S> {
    fn new(
        ac: &'a AhoCorasick<S>,
        haystack: &'b [u8],
    ) -> FindOverlappingIter<'a, 'b, S> {
        assert!(
            ac.supports_overlapping(),
            "automaton does not support overlapping searches"
        );
        let prestate = PrefilterState::new(ac.max_pattern_len());
        FindOverlappingIter {
            fsm: &ac.imp,
            prestate,
            haystack,
            pos: 0,
            last_match_end: 0,
            state_id: ac.imp.start_state(),
            match_index: 0,
        }
    }
}

impl<'a, 'b, S: StateID> Iterator for FindOverlappingIter<'a, 'b, S> {
    type Item = Match;

    fn next(&mut self) -> Option<Match> {
        let result = self.fsm.overlapping_find_at(
            &mut self.prestate,
            self.haystack,
            self.pos,
            &mut self.state_id,
            &mut self.match_index,
        );
        match result {
            None => return None,
            Some(m) => {
                self.pos = m.end();
                Some(m)
            }
        }
    }
}

/// An iterator that reports Aho-Corasick matches in a stream.
///
/// This iterator yields elements of type `io::Result<Match>`, where an error
/// is reported if there was a problem reading from the underlying stream.
/// The iterator terminates only when the underlying stream reaches `EOF`.
///
/// This iterator is constructed via the
/// [`AhoCorasick::stream_find_iter`](struct.AhoCorasick.html#method.stream_find_iter)
/// method.
///
/// The type variable `R` refers to the `io::Read` stream that is being read
/// from.
///
/// The type variable `S` refers to the representation used for state
/// identifiers. (By default, this is `usize`.)
///
/// The lifetime `'a` refers to the lifetime of the `AhoCorasick` automaton.
#[derive(Debug)]
pub struct StreamFindIter<'a, R, S: StateID> {
    it: StreamChunkIter<'a, R, S>,
}

impl<'a, R: io::Read, S: StateID> StreamFindIter<'a, R, S> {
    fn new(ac: &'a AhoCorasick<S>, rdr: R) -> StreamFindIter<'a, R, S> {
        StreamFindIter { it: StreamChunkIter::new(ac, rdr) }
    }
}

impl<'a, R: io::Read, S: StateID> Iterator for StreamFindIter<'a, R, S> {
    type Item = io::Result<Match>;

    fn next(&mut self) -> Option<io::Result<Match>> {
        loop {
            match self.it.next() {
                None => return None,
                Some(Err(err)) => return Some(Err(err)),
                Some(Ok(StreamChunk::NonMatch { .. })) => {}
                Some(Ok(StreamChunk::Match { mat, .. })) => {
                    return Some(Ok(mat));
                }
            }
        }
    }
}

/// An iterator over chunks in an underlying reader. Each chunk either
/// corresponds to non-matching bytes or matching bytes, but all bytes from
/// the underlying reader are reported in sequence. There may be an arbitrary
/// number of non-matching chunks before seeing a matching chunk.
///
/// N.B. This does not actually implement Iterator because we need to borrow
/// from the underlying reader. But conceptually, it's still an iterator.
#[derive(Debug)]
struct StreamChunkIter<'a, R, S: StateID> {
    /// The AC automaton.
    fsm: &'a Imp<S>,
    /// State associated with this automaton's prefilter. It is a heuristic
    /// for stopping the prefilter if it's deemed ineffective.
    prestate: PrefilterState,
    /// The source of bytes we read from.
    rdr: R,
    /// A fixed size buffer. This is what we actually search. There are some
    /// invariants around the buffer's size, namely, it must be big enough to
    /// contain the longest possible match.
    buf: Buffer,
    /// The ID of the FSM state we're currently in.
    state_id: S,
    /// The current position at which to start the next search in `buf`.
    search_pos: usize,
    /// The absolute position of `search_pos`, where `0` corresponds to the
    /// position of the first byte read from `rdr`.
    absolute_pos: usize,
    /// The ending position of the last StreamChunk that was returned to the
    /// caller. This position is used to determine whether we need to emit
    /// non-matching bytes before emitting a match.
    report_pos: usize,
    /// A match that should be reported on the next call.
    pending_match: Option<Match>,
    /// Enabled only when the automaton can match the empty string. When
    /// enabled, we need to execute one final search after consuming the
    /// reader to find the trailing empty match.
    has_empty_match_at_end: bool,
}

/// A single chunk yielded by the stream chunk iterator.
///
/// The `'r` lifetime refers to the lifetime of the stream chunk iterator.
#[derive(Debug)]
enum StreamChunk<'r> {
    /// A chunk that does not contain any matches.
    NonMatch { bytes: &'r [u8], start: usize },
    /// A chunk that precisely contains a match.
    Match { bytes: &'r [u8], mat: Match },
}

impl<'a, R: io::Read, S: StateID> StreamChunkIter<'a, R, S> {
    fn new(ac: &'a AhoCorasick<S>, rdr: R) -> StreamChunkIter<'a, R, S> {
        assert!(
            ac.supports_stream(),
            "stream searching is only supported for Standard match semantics"
        );

        let prestate = if ac.imp.use_prefilter() {
            PrefilterState::new(ac.max_pattern_len())
        } else {
            PrefilterState::disabled()
        };
        let buf = Buffer::new(ac.imp.max_pattern_len());
        let state_id = ac.imp.start_state();
        StreamChunkIter {
            fsm: &ac.imp,
            prestate,
            rdr,
            buf,
            state_id,
            absolute_pos: 0,
            report_pos: 0,
            search_pos: 0,
            pending_match: None,
            has_empty_match_at_end: ac.is_match(""),
        }
    }

    fn next<'r>(&'r mut self) -> Option<io::Result<StreamChunk<'r>>> {
        loop {
            if let Some(mut mat) = self.pending_match.take() {
                let bytes = &self.buf.buffer()[mat.start()..mat.end()];
                self.report_pos = mat.end();
                mat = mat.increment(self.absolute_pos);
                return Some(Ok(StreamChunk::Match { bytes, mat }));
            }
            if self.search_pos >= self.buf.len() {
                if let Some(end) = self.unreported() {
                    let bytes = &self.buf.buffer()[self.report_pos..end];
                    let start = self.absolute_pos + self.report_pos;
                    self.report_pos = end;
                    return Some(Ok(StreamChunk::NonMatch { bytes, start }));
                }
                if self.buf.len() >= self.buf.min_buffer_len() {
                    // This is the point at which we roll our buffer, which we
                    // only do if our buffer has at least the minimum amount of
                    // bytes in it. Before rolling, we update our various
                    // positions to be consistent with the buffer after it has
                    // been rolled.

                    self.report_pos -=
                        self.buf.len() - self.buf.min_buffer_len();
                    self.absolute_pos +=
                        self.search_pos - self.buf.min_buffer_len();
                    self.search_pos = self.buf.min_buffer_len();
                    self.buf.roll();
                }
                match self.buf.fill(&mut self.rdr) {
                    Err(err) => return Some(Err(err)),
                    Ok(false) => {
                        // We've hit EOF, but if there are still some
                        // unreported bytes remaining, return them now.
                        if self.report_pos < self.buf.len() {
                            let bytes = &self.buf.buffer()[self.report_pos..];
                            let start = self.absolute_pos + self.report_pos;
                            self.report_pos = self.buf.len();

                            let chunk = StreamChunk::NonMatch { bytes, start };
                            return Some(Ok(chunk));
                        } else {
                            // We've reported everything, but there might still
                            // be a match at the very last position.
                            if !self.has_empty_match_at_end {
                                return None;
                            }
                            // fallthrough for another search to get trailing
                            // empty matches
                            self.has_empty_match_at_end = false;
                        }
                    }
                    Ok(true) => {}
                }
            }
            let result = self.fsm.earliest_find_at(
                &mut self.prestate,
                self.buf.buffer(),
                self.search_pos,
                &mut self.state_id,
            );
            match result {
                None => {
                    self.search_pos = self.buf.len();
                }
                Some(mat) => {
                    self.state_id = self.fsm.start_state();
                    if mat.end() == self.search_pos {
                        // If the automaton can match the empty string and if
                        // we found an empty match, then we need to forcefully
                        // move the position.
                        self.search_pos += 1;
                    } else {
                        self.search_pos = mat.end();
                    }
                    self.pending_match = Some(mat.clone());
                    if self.report_pos < mat.start() {
                        let bytes =
                            &self.buf.buffer()[self.report_pos..mat.start()];
                        let start = self.absolute_pos + self.report_pos;
                        self.report_pos = mat.start();

                        let chunk = StreamChunk::NonMatch { bytes, start };
                        return Some(Ok(chunk));
                    }
                }
            }
        }
    }

    fn unreported(&self) -> Option<usize> {
        let end = self.search_pos.saturating_sub(self.buf.min_buffer_len());
        if self.report_pos < end {
            Some(end)
        } else {
            None
        }
    }
}

/// A builder for configuring an Aho-Corasick automaton.
#[derive(Clone, Debug)]
pub struct AhoCorasickBuilder {
    nfa_builder: nfa::Builder,
    dfa_builder: dfa::Builder,
    dfa: bool,
}

impl Default for AhoCorasickBuilder {
    fn default() -> AhoCorasickBuilder {
        AhoCorasickBuilder::new()
    }
}

impl AhoCorasickBuilder {
    /// Create a new builder for configuring an Aho-Corasick automaton.
    ///
    /// If you don't need fine grained configuration or aren't sure which knobs
    /// to set, try using
    /// [`AhoCorasick::new_auto_configured`](struct.AhoCorasick.html#method.new_auto_configured)
    /// instead.
    pub fn new() -> AhoCorasickBuilder {
        AhoCorasickBuilder {
            nfa_builder: nfa::Builder::new(),
            dfa_builder: dfa::Builder::new(),
            dfa: false,
        }
    }

    /// Build an Aho-Corasick automaton using the configuration set on this
    /// builder.
    ///
    /// A builder may be reused to create more automatons.
    ///
    /// This method will use the default for representing internal state
    /// identifiers, which is `usize`. This guarantees that building the
    /// automaton will succeed and is generally a good default, but can make
    /// the size of the automaton 2-8 times bigger than it needs to be,
    /// depending on your target platform.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::AhoCorasickBuilder;
    ///
    /// let patterns = &["foo", "bar", "baz"];
    /// let ac = AhoCorasickBuilder::new()
    ///     .build(patterns);
    /// assert_eq!(Some(1), ac.find("xxx bar xxx").map(|m| m.pattern()));
    /// ```
    pub fn build<I, P>(&self, patterns: I) -> AhoCorasick
    where
        I: IntoIterator<Item = P>,
        P: AsRef<[u8]>,
    {
        // The builder only returns an error if the chosen state ID
        // representation is too small to fit all of the given patterns. In
        // this case, since we fix the representation to usize, it will always
        // work because it's impossible to overflow usize since the underlying
        // storage would OOM long before that happens.
        self.build_with_size::<usize, I, P>(patterns)
            .expect("usize state ID type should always work")
    }

    /// Build an Aho-Corasick automaton using the configuration set on this
    /// builder with a specific state identifier representation. This only has
    /// an effect when the `dfa` option is enabled.
    ///
    /// Generally, the choices for a state identifier representation are
    /// `u8`, `u16`, `u32`, `u64` or `usize`, with `usize` being the default.
    /// The advantage of choosing a smaller state identifier representation
    /// is that the automaton produced will be smaller. This might be
    /// beneficial for just generally using less space, or might even allow it
    /// to fit more of the automaton in your CPU's cache, leading to overall
    /// better search performance.
    ///
    /// Unlike the standard `build` method, this can report an error if the
    /// state identifier representation cannot support the size of the
    /// automaton.
    ///
    /// Note that the state identifier representation is determined by the
    /// `S` type variable. This requires a type hint of some sort, either
    /// by specifying the return type or using the turbofish, e.g.,
    /// `build_with_size::<u16, _, _>(...)`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
    ///
    /// # fn example() -> Result<(), ::aho_corasick::Error> {
    /// let patterns = &["foo", "bar", "baz"];
    /// let ac: AhoCorasick<u8> = AhoCorasickBuilder::new()
    ///     .build_with_size(patterns)?;
    /// assert_eq!(Some(1), ac.find("xxx bar xxx").map(|m| m.pattern()));
    /// # Ok(()) }; example().unwrap()
    /// ```
    ///
    /// Or alternatively, with turbofish:
    ///
    /// ```
    /// use aho_corasick::AhoCorasickBuilder;
    ///
    /// # fn example() -> Result<(), ::aho_corasick::Error> {
    /// let patterns = &["foo", "bar", "baz"];
    /// let ac = AhoCorasickBuilder::new()
    ///     .build_with_size::<u8, _, _>(patterns)?;
    /// assert_eq!(Some(1), ac.find("xxx bar xxx").map(|m| m.pattern()));
    /// # Ok(()) }; example().unwrap()
    /// ```
    pub fn build_with_size<S, I, P>(
        &self,
        patterns: I,
    ) -> Result<AhoCorasick<S>>
    where
        S: StateID,
        I: IntoIterator<Item = P>,
        P: AsRef<[u8]>,
    {
        let nfa = self.nfa_builder.build(patterns)?;
        let match_kind = nfa.match_kind().clone();
        let imp = if self.dfa {
            let dfa = self.dfa_builder.build(&nfa)?;
            Imp::DFA(dfa)
        } else {
            Imp::NFA(nfa)
        };
        Ok(AhoCorasick { imp, match_kind })
    }

    /// Automatically configure the settings on this builder according to the
    /// patterns that will be used to construct the automaton.
    ///
    /// The idea here is to balance space and time automatically. That is, when
    /// searching a small number of patterns, this will attempt to use the
    /// fastest possible configuration since the total space required will be
    /// small anyway. As the number of patterns grows, this will fall back to
    /// slower configurations that use less space.
    ///
    /// This is guaranteed to never set `match_kind`, but any other option may
    /// be overridden.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::AhoCorasickBuilder;
    ///
    /// let patterns = &["foo", "bar", "baz"];
    /// let ac = AhoCorasickBuilder::new()
    ///     .auto_configure(patterns)
    ///     .build(patterns);
    /// assert_eq!(Some(1), ac.find("xxx bar xxx").map(|m| m.pattern()));
    /// ```
    pub fn auto_configure<B: AsRef<[u8]>>(
        &mut self,
        patterns: &[B],
    ) -> &mut AhoCorasickBuilder {
        // N.B. Currently we only use the length of `patterns` to make a
        // decision here, and could therefore ask for an `ExactSizeIterator`
        // instead. But it's conceivable that we might adapt this to look at
        // the total number of bytes, which would requires a second pass.
        //
        // The logic here is fairly rudimentary at the moment, but probably
        // OK. The idea here is to use the fastest thing possible for a small
        // number of patterns. That is, a DFA with no byte classes, since byte
        // classes require an extra indirection for every byte searched. With a
        // moderate number of patterns, we still want a DFA, but save on both
        // space and compilation time by enabling byte classes. Finally, fall
        // back to the slower but smaller NFA.
        if patterns.len() <= 100 {
            // N.B. Using byte classes can actually be faster by improving
            // locality, but this only really applies for multi-megabyte
            // automata (i.e., automata that don't fit in your CPU's cache).
            self.dfa(true);
        } else if patterns.len() <= 5000 {
            self.dfa(true);
        }
        self
    }

    /// Set the desired match semantics.
    ///
    /// The default is `MatchKind::Standard`, which corresponds to the match
    /// semantics supported by the standard textbook description of the
    /// Aho-Corasick algorithm. Namely, matches are reported as soon as they
    /// are found. Moreover, this is the only way to get overlapping matches
    /// or do stream searching.
    ///
    /// The other kinds of match semantics that are supported are
    /// `MatchKind::LeftmostFirst` and `MatchKind::LeftmostLongest`. The former
    /// corresponds to the match you would get if you were to try to match
    /// each pattern at each position in the haystack in the same order that
    /// you give to the automaton. That is, it returns the leftmost match
    /// corresponding the earliest pattern given to the automaton. The latter
    /// corresponds to finding the longest possible match among all leftmost
    /// matches.
    ///
    /// For more details on match semantics, see the
    /// [documentation for `MatchKind`](enum.MatchKind.html).
    ///
    /// # Examples
    ///
    /// In these examples, we demonstrate the differences between match
    /// semantics for a particular set of patterns in a specific order:
    /// `b`, `abc`, `abcd`.
    ///
    /// Standard semantics:
    ///
    /// ```
    /// use aho_corasick::{AhoCorasickBuilder, MatchKind};
    ///
    /// let patterns = &["b", "abc", "abcd"];
    /// let haystack = "abcd";
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .match_kind(MatchKind::Standard) // default, not necessary
    ///     .build(patterns);
    /// let mat = ac.find(haystack).expect("should have a match");
    /// assert_eq!("b", &haystack[mat.start()..mat.end()]);
    /// ```
    ///
    /// Leftmost-first semantics:
    ///
    /// ```
    /// use aho_corasick::{AhoCorasickBuilder, MatchKind};
    ///
    /// let patterns = &["b", "abc", "abcd"];
    /// let haystack = "abcd";
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .match_kind(MatchKind::LeftmostFirst)
    ///     .build(patterns);
    /// let mat = ac.find(haystack).expect("should have a match");
    /// assert_eq!("abc", &haystack[mat.start()..mat.end()]);
    /// ```
    ///
    /// Leftmost-longest semantics:
    ///
    /// ```
    /// use aho_corasick::{AhoCorasickBuilder, MatchKind};
    ///
    /// let patterns = &["b", "abc", "abcd"];
    /// let haystack = "abcd";
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .match_kind(MatchKind::LeftmostLongest)
    ///     .build(patterns);
    /// let mat = ac.find(haystack).expect("should have a match");
    /// assert_eq!("abcd", &haystack[mat.start()..mat.end()]);
    /// ```
    pub fn match_kind(&mut self, kind: MatchKind) -> &mut AhoCorasickBuilder {
        self.nfa_builder.match_kind(kind);
        self
    }

    /// Enable anchored mode, which requires all matches to start at the
    /// first position in a haystack.
    ///
    /// This option is disabled by default.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::AhoCorasickBuilder;
    ///
    /// let patterns = &["foo", "bar"];
    /// let haystack = "foobar";
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .anchored(true)
    ///     .build(patterns);
    /// assert_eq!(1, ac.find_iter(haystack).count());
    /// ```
    ///
    /// When searching for overlapping matches, all matches that start at
    /// the beginning of a haystack will be reported:
    ///
    /// ```
    /// use aho_corasick::AhoCorasickBuilder;
    ///
    /// let patterns = &["foo", "foofoo"];
    /// let haystack = "foofoo";
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .anchored(true)
    ///     .build(patterns);
    /// assert_eq!(2, ac.find_overlapping_iter(haystack).count());
    /// // A non-anchored search would return 3 matches.
    /// ```
    pub fn anchored(&mut self, yes: bool) -> &mut AhoCorasickBuilder {
        self.nfa_builder.anchored(yes);
        self
    }

    /// Enable ASCII-aware case insensitive matching.
    ///
    /// When this option is enabled, searching will be performed without
    /// respect to case for ASCII letters (`a-z` and `A-Z`) only.
    ///
    /// Enabling this option does not change the search algorithm, but it may
    /// increase the size of the automaton.
    ///
    /// **NOTE:** In the future, support for full Unicode case insensitivity
    /// may be added, but ASCII case insensitivity is comparatively much
    /// simpler to add.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::AhoCorasickBuilder;
    ///
    /// let patterns = &["FOO", "bAr", "BaZ"];
    /// let haystack = "foo bar baz";
    ///
    /// let ac = AhoCorasickBuilder::new()
    ///     .ascii_case_insensitive(true)
    ///     .build(patterns);
    /// assert_eq!(3, ac.find_iter(haystack).count());
    /// ```
    pub fn ascii_case_insensitive(
        &mut self,
        yes: bool,
    ) -> &mut AhoCorasickBuilder {
        self.nfa_builder.ascii_case_insensitive(yes);
        self
    }

    /// Set the limit on how many NFA states use a dense representation for
    /// their transitions.
    ///
    /// A dense representation uses more space, but supports faster access to
    /// transitions at search time. Thus, this setting permits the control of a
    /// space vs time trade off when using the NFA variant of Aho-Corasick.
    ///
    /// This limit is expressed in terms of the depth of a state, i.e., the
    /// number of transitions from the starting state of the NFA. The idea is
    /// that most of the time searching will be spent near the starting state
    /// of the automaton, so states near the start state should use a dense
    /// representation. States further away from the start state would then use
    /// a sparse representation, which uses less space but is slower to access
    /// transitions at search time.
    ///
    /// By default, this is set to a low but non-zero number.
    ///
    /// This setting has no effect if the `dfa` option is enabled.
    pub fn dense_depth(&mut self, depth: usize) -> &mut AhoCorasickBuilder {
        self.nfa_builder.dense_depth(depth);
        self
    }

    /// Compile the standard Aho-Corasick automaton into a deterministic finite
    /// automaton (DFA).
    ///
    /// When this is disabled (which is the default), then a non-deterministic
    /// finite automaton (NFA) is used instead.
    ///
    /// The main benefit to a DFA is that it can execute searches more quickly
    /// than a NFA (perhaps 2-4 times as fast). The main drawback is that the
    /// DFA uses more space and can take much longer to build.
    ///
    /// Enabling this option does not change the time complexity for
    /// constructing the Aho-Corasick automaton (which is `O(p)` where
    /// `p` is the total number of patterns being compiled). Enabling this
    /// option does however reduce the time complexity of non-overlapping
    /// searches from `O(n + p)` to `O(n)`, where `n` is the length of the
    /// haystack.
    ///
    /// In general, it's a good idea to enable this if you're searching a
    /// small number of fairly short patterns (~1000), or if you want the
    /// fastest possible search without regard to compilation time or space
    /// usage.
    pub fn dfa(&mut self, yes: bool) -> &mut AhoCorasickBuilder {
        self.dfa = yes;
        self
    }

    /// Enable heuristic prefilter optimizations.
    ///
    /// When enabled, searching will attempt to quickly skip to match
    /// candidates using specialized literal search routines. A prefilter
    /// cannot always be used, and is generally treated as a heuristic. It
    /// can be useful to disable this if the prefilter is observed to be
    /// sub-optimal for a particular workload.
    ///
    /// This is enabled by default.
    pub fn prefilter(&mut self, yes: bool) -> &mut AhoCorasickBuilder {
        self.nfa_builder.prefilter(yes);
        self
    }

    /// Shrink the size of the transition alphabet by mapping bytes to their
    /// equivalence classes. This only has an effect when the `dfa` option is
    /// enabled.
    ///
    /// When enabled, each a DFA will use a map from all possible bytes
    /// to their corresponding equivalence class. Each equivalence class
    /// represents a set of bytes that does not discriminate between a match
    /// and a non-match in the DFA. For example, the patterns `bar` and `baz`
    /// have at least five equivalence classes: singleton sets of `b`, `a`, `r`
    /// and `z`, and a final set that contains every other byte.
    ///
    /// The advantage of this map is that the size of the transition table can
    /// be reduced drastically from `#states * 256 * sizeof(id)` to
    /// `#states * k * sizeof(id)` where `k` is the number of equivalence
    /// classes. As a result, total space usage can decrease substantially.
    /// Moreover, since a smaller alphabet is used, compilation becomes faster
    /// as well.
    ///
    /// The disadvantage of this map is that every byte searched must be
    /// passed through this map before it can be used to determine the next
    /// transition. This has a small match time performance cost. However, if
    /// the DFA is otherwise very large without byte classes, then using byte
    /// classes can greatly improve memory locality and thus lead to better
    /// overall performance.
    ///
    /// This option is enabled by default.
    #[deprecated(
        since = "0.7.16",
        note = "not carrying its weight, will be always enabled, see: https://github.com/BurntSushi/aho-corasick/issues/57"
    )]
    pub fn byte_classes(&mut self, yes: bool) -> &mut AhoCorasickBuilder {
        self.dfa_builder.byte_classes(yes);
        self
    }

    /// Premultiply state identifiers in the transition table. This only has
    /// an effect when the `dfa` option is enabled.
    ///
    /// When enabled, state identifiers are premultiplied to point to their
    /// corresponding row in the transition table. That is, given the `i`th
    /// state, its corresponding premultiplied identifier is `i * k` where `k`
    /// is the alphabet size of the automaton. (The alphabet size is at most
    /// 256, but is in practice smaller if byte classes is enabled.)
    ///
    /// When state identifiers are not premultiplied, then the identifier of
    /// the `i`th state is `i`.
    ///
    /// The advantage of premultiplying state identifiers is that is saves a
    /// multiplication instruction per byte when searching with a DFA. This has
    /// been observed to lead to a 20% performance benefit in micro-benchmarks.
    ///
    /// The primary disadvantage of premultiplying state identifiers is
    /// that they require a larger integer size to represent. For example,
    /// if the DFA has 200 states, then its premultiplied form requires 16
    /// bits to represent every possible state identifier, where as its
    /// non-premultiplied form only requires 8 bits.
    ///
    /// This option is enabled by default.
    #[deprecated(
        since = "0.7.16",
        note = "not carrying its weight, will be always enabled, see: https://github.com/BurntSushi/aho-corasick/issues/57"
    )]
    pub fn premultiply(&mut self, yes: bool) -> &mut AhoCorasickBuilder {
        self.dfa_builder.premultiply(yes);
        self
    }
}

/// A knob for controlling the match semantics of an Aho-Corasick automaton.
///
/// There are two generally different ways that Aho-Corasick automatons can
/// report matches. The first way is the "standard" approach that results from
/// implementing most textbook explanations of Aho-Corasick. The second way is
/// to report only the leftmost non-overlapping matches. The leftmost approach
/// is in turn split into two different ways of resolving ambiguous matches:
/// leftmost-first and leftmost-longest.
///
/// The `Standard` match kind is the default and is the only one that supports
/// overlapping matches and stream searching. (Trying to find overlapping
/// or streaming matches using leftmost match semantics will result in a
/// panic.) The `Standard` match kind will report matches as they are seen.
/// When searching for overlapping matches, then all possible matches are
/// reported. When searching for non-overlapping matches, the first match seen
/// is reported. For example, for non-overlapping matches, given the patterns
/// `abcd` and `b` and the subject string `abcdef`, only a match for `b` is
/// reported since it is detected first. The `abcd` match is never reported
/// since it overlaps with the `b` match.
///
/// In contrast, the leftmost match kind always prefers the leftmost match
/// among all possible matches. Given the same example as above with `abcd` and
/// `b` as patterns and `abcdef` as the subject string, the leftmost match is
/// `abcd` since it begins before the `b` match, even though the `b` match is
/// detected before the `abcd` match. In this case, the `b` match is not
/// reported at all since it overlaps with the `abcd` match.
///
/// The difference between leftmost-first and leftmost-longest is in how they
/// resolve ambiguous matches when there are multiple leftmost matches to
/// choose from. Leftmost-first always chooses the pattern that was provided
/// earliest, where as leftmost-longest always chooses the longest matching
/// pattern. For example, given the patterns `a` and `ab` and the subject
/// string `ab`, the leftmost-first match is `a` but the leftmost-longest match
/// is `ab`. Conversely, if the patterns were given in reverse order, i.e.,
/// `ab` and `a`, then both the leftmost-first and leftmost-longest matches
/// would be `ab`. Stated differently, the leftmost-first match depends on the
/// order in which the patterns were given to the Aho-Corasick automaton.
/// Because of that, when leftmost-first matching is used, if a pattern `A`
/// that appears before a pattern `B` is a prefix of `B`, then it is impossible
/// to ever observe a match of `B`.
///
/// If you're not sure which match kind to pick, then stick with the standard
/// kind, which is the default. In particular, if you need overlapping or
/// streaming matches, then you _must_ use the standard kind. The leftmost
/// kinds are useful in specific circumstances. For example, leftmost-first can
/// be very useful as a way to implement match priority based on the order of
/// patterns given and leftmost-longest can be useful for dictionary searching
/// such that only the longest matching words are reported.
///
/// # Relationship with regular expression alternations
///
/// Understanding match semantics can be a little tricky, and one easy way
/// to conceptualize non-overlapping matches from an Aho-Corasick automaton
/// is to think about them as a simple alternation of literals in a regular
/// expression. For example, let's say we wanted to match the strings
/// `Sam` and `Samwise`, which would turn into the regex `Sam|Samwise`. It
/// turns out that regular expression engines have two different ways of
/// matching this alternation. The first way, leftmost-longest, is commonly
/// found in POSIX compatible implementations of regular expressions (such as
/// `grep`). The second way, leftmost-first, is commonly found in backtracking
/// implementations such as Perl. (Some regex engines, such as RE2 and Rust's
/// regex engine do not use backtracking, but still implement leftmost-first
/// semantics in an effort to match the behavior of dominant backtracking
/// regex engines such as those found in Perl, Ruby, Python, Javascript and
/// PHP.)
///
/// That is, when matching `Sam|Samwise` against `Samwise`, a POSIX regex
/// will match `Samwise` because it is the longest possible match, but a
/// Perl-like regex will match `Sam` since it appears earlier in the
/// alternation. Indeed, the regex `Sam|Samwise` in a Perl-like regex engine
/// will never match `Samwise` since `Sam` will always have higher priority.
/// Conversely, matching the regex `Samwise|Sam` against `Samwise` will lead to
/// a match of `Samwise` in both POSIX and Perl-like regexes since `Samwise` is
/// still longest match, but it also appears earlier than `Sam`.
///
/// The "standard" match semantics of Aho-Corasick generally don't correspond
/// to the match semantics of any large group of regex implementations, so
/// there's no direct analogy that can be made here. Standard match semantics
/// are generally useful for overlapping matches, or if you just want to see
/// matches as they are detected.
///
/// The main conclusion to draw from this section is that the match semantics
/// can be tweaked to precisely match either Perl-like regex alternations or
/// POSIX regex alternations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchKind {
    /// Use standard match semantics, which support overlapping matches. When
    /// used with non-overlapping matches, matches are reported as they are
    /// seen.
    Standard,
    /// Use leftmost-first match semantics, which reports leftmost matches.
    /// When there are multiple possible leftmost matches, the match
    /// corresponding to the pattern that appeared earlier when constructing
    /// the automaton is reported.
    ///
    /// This does **not** support overlapping matches or stream searching. If
    /// this match kind is used, attempting to find overlapping matches or
    /// stream matches will panic.
    LeftmostFirst,
    /// Use leftmost-longest match semantics, which reports leftmost matches.
    /// When there are multiple possible leftmost matches, the longest match
    /// is chosen.
    ///
    /// This does **not** support overlapping matches or stream searching. If
    /// this match kind is used, attempting to find overlapping matches or
    /// stream matches will panic.
    LeftmostLongest,
    /// Hints that destructuring should not be exhaustive.
    ///
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

/// The default match kind is `MatchKind::Standard`.
impl Default for MatchKind {
    fn default() -> MatchKind {
        MatchKind::Standard
    }
}

impl MatchKind {
    fn supports_overlapping(&self) -> bool {
        self.is_standard()
    }

    fn supports_stream(&self) -> bool {
        // TODO: It may be possible to support this. It's hard.
        //
        // See: https://github.com/rust-lang/regex/issues/425#issuecomment-471367838
        self.is_standard()
    }

    pub(crate) fn is_standard(&self) -> bool {
        *self == MatchKind::Standard
    }

    pub(crate) fn is_leftmost(&self) -> bool {
        *self == MatchKind::LeftmostFirst
            || *self == MatchKind::LeftmostLongest
    }

    pub(crate) fn is_leftmost_first(&self) -> bool {
        *self == MatchKind::LeftmostFirst
    }

    /// Convert this match kind into a packed match kind. If this match kind
    /// corresponds to standard semantics, then this returns None, since
    /// packed searching does not support standard semantics.
    pub(crate) fn as_packed(&self) -> Option<packed::MatchKind> {
        match *self {
            MatchKind::Standard => None,
            MatchKind::LeftmostFirst => Some(packed::MatchKind::LeftmostFirst),
            MatchKind::LeftmostLongest => {
                Some(packed::MatchKind::LeftmostLongest)
            }
            MatchKind::__Nonexhaustive => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oibits() {
        use std::panic::{RefUnwindSafe, UnwindSafe};

        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        fn assert_unwind_safe<T: RefUnwindSafe + UnwindSafe>() {}

        assert_send::<AhoCorasick>();
        assert_sync::<AhoCorasick>();
        assert_unwind_safe::<AhoCorasick>();
        assert_send::<AhoCorasickBuilder>();
        assert_sync::<AhoCorasickBuilder>();
        assert_unwind_safe::<AhoCorasickBuilder>();
    }
}
