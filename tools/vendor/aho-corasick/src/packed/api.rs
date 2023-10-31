use std::u16;

use crate::packed::pattern::Patterns;
use crate::packed::rabinkarp::RabinKarp;
use crate::packed::teddy::{self, Teddy};
use crate::Match;

/// This is a limit placed on the total number of patterns we're willing to try
/// and match at once. As more sophisticated algorithms are added, this number
/// may be increased.
const PATTERN_LIMIT: usize = 128;

/// A knob for controlling the match semantics of a packed multiple string
/// searcher.
///
/// This differs from the
/// [`MatchKind`](../enum.MatchKind.html)
/// type in the top-level crate module in that it doesn't support
/// "standard" match semantics, and instead only supports leftmost-first or
/// leftmost-longest. Namely, "standard" semantics cannot be easily supported
/// by packed searchers.
///
/// For more information on the distinction between leftmost-first and
/// leftmost-longest, see the docs on the top-level `MatchKind` type.
///
/// Unlike the top-level `MatchKind` type, the default match semantics for this
/// type are leftmost-first.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchKind {
    /// Use leftmost-first match semantics, which reports leftmost matches.
    /// When there are multiple possible leftmost matches, the match
    /// corresponding to the pattern that appeared earlier when constructing
    /// the automaton is reported.
    ///
    /// This is the default.
    LeftmostFirst,
    /// Use leftmost-longest match semantics, which reports leftmost matches.
    /// When there are multiple possible leftmost matches, the longest match
    /// is chosen.
    LeftmostLongest,
    /// Hints that destructuring should not be exhaustive.
    ///
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

impl Default for MatchKind {
    fn default() -> MatchKind {
        MatchKind::LeftmostFirst
    }
}

/// The configuration for a packed multiple pattern searcher.
///
/// The configuration is currently limited only to being able to select the
/// match semantics (leftmost-first or leftmost-longest) of a searcher. In the
/// future, more knobs may be made available.
///
/// A configuration produces a [`packed::Builder`](struct.Builder.html), which
/// in turn can be used to construct a
/// [`packed::Searcher`](struct.Searcher.html) for searching.
///
/// # Example
///
/// This example shows how to use leftmost-longest semantics instead of the
/// default (leftmost-first).
///
/// ```
/// use aho_corasick::packed::{Config, MatchKind};
///
/// # fn example() -> Option<()> {
/// let searcher = Config::new()
///     .match_kind(MatchKind::LeftmostLongest)
///     .builder()
///     .add("foo")
///     .add("foobar")
///     .build()?;
/// let matches: Vec<usize> = searcher
///     .find_iter("foobar")
///     .map(|mat| mat.pattern())
///     .collect();
/// assert_eq!(vec![1], matches);
/// # Some(()) }
/// # if cfg!(target_arch = "x86_64") {
/// #     example().unwrap()
/// # } else {
/// #     assert!(example().is_none());
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct Config {
    kind: MatchKind,
    force: Option<ForceAlgorithm>,
    force_teddy_fat: Option<bool>,
    force_avx: Option<bool>,
}

/// An internal option for forcing the use of a particular packed algorithm.
///
/// When an algorithm is forced, if a searcher could not be constructed for it,
/// then no searcher will be returned even if an alternative algorithm would
/// work.
#[derive(Clone, Debug)]
enum ForceAlgorithm {
    Teddy,
    RabinKarp,
}

impl Default for Config {
    fn default() -> Config {
        Config::new()
    }
}

impl Config {
    /// Create a new default configuration. A default configuration uses
    /// leftmost-first match semantics.
    pub fn new() -> Config {
        Config {
            kind: MatchKind::LeftmostFirst,
            force: None,
            force_teddy_fat: None,
            force_avx: None,
        }
    }

    /// Create a packed builder from this configuration. The builder can be
    /// used to accumulate patterns and create a
    /// [`Searcher`](struct.Searcher.html)
    /// from them.
    pub fn builder(&self) -> Builder {
        Builder::from_config(self.clone())
    }

    /// Set the match semantics for this configuration.
    pub fn match_kind(&mut self, kind: MatchKind) -> &mut Config {
        self.kind = kind;
        self
    }

    /// An undocumented method for forcing the use of the Teddy algorithm.
    ///
    /// This is only exposed for more precise testing and benchmarks. Callers
    /// should not use it as it is not part of the API stability guarantees of
    /// this crate.
    #[doc(hidden)]
    pub fn force_teddy(&mut self, yes: bool) -> &mut Config {
        if yes {
            self.force = Some(ForceAlgorithm::Teddy);
        } else {
            self.force = None;
        }
        self
    }

    /// An undocumented method for forcing the use of the Fat Teddy algorithm.
    ///
    /// This is only exposed for more precise testing and benchmarks. Callers
    /// should not use it as it is not part of the API stability guarantees of
    /// this crate.
    #[doc(hidden)]
    pub fn force_teddy_fat(&mut self, yes: Option<bool>) -> &mut Config {
        self.force_teddy_fat = yes;
        self
    }

    /// An undocumented method for forcing the use of SSE (`Some(false)`) or
    /// AVX (`Some(true)`) algorithms.
    ///
    /// This is only exposed for more precise testing and benchmarks. Callers
    /// should not use it as it is not part of the API stability guarantees of
    /// this crate.
    #[doc(hidden)]
    pub fn force_avx(&mut self, yes: Option<bool>) -> &mut Config {
        self.force_avx = yes;
        self
    }

    /// An undocumented method for forcing the use of the Rabin-Karp algorithm.
    ///
    /// This is only exposed for more precise testing and benchmarks. Callers
    /// should not use it as it is not part of the API stability guarantees of
    /// this crate.
    #[doc(hidden)]
    pub fn force_rabin_karp(&mut self, yes: bool) -> &mut Config {
        if yes {
            self.force = Some(ForceAlgorithm::RabinKarp);
        } else {
            self.force = None;
        }
        self
    }
}

/// A builder for constructing a packed searcher from a collection of patterns.
///
/// # Example
///
/// This example shows how to use a builder to construct a searcher. By
/// default, leftmost-first match semantics are used.
///
/// ```
/// use aho_corasick::packed::{Builder, MatchKind};
///
/// # fn example() -> Option<()> {
/// let searcher = Builder::new()
///     .add("foobar")
///     .add("foo")
///     .build()?;
/// let matches: Vec<usize> = searcher
///     .find_iter("foobar")
///     .map(|mat| mat.pattern())
///     .collect();
/// assert_eq!(vec![0], matches);
/// # Some(()) }
/// # if cfg!(target_arch = "x86_64") {
/// #     example().unwrap()
/// # } else {
/// #     assert!(example().is_none());
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct Builder {
    /// The configuration of this builder and subsequent matcher.
    config: Config,
    /// Set to true if the builder detects that a matcher cannot be built.
    inert: bool,
    /// The patterns provided by the caller.
    patterns: Patterns,
}

impl Builder {
    /// Create a new builder for constructing a multi-pattern searcher. This
    /// constructor uses the default configuration.
    pub fn new() -> Builder {
        Builder::from_config(Config::new())
    }

    fn from_config(config: Config) -> Builder {
        Builder { config, inert: false, patterns: Patterns::new() }
    }

    /// Build a searcher from the patterns added to this builder so far.
    pub fn build(&self) -> Option<Searcher> {
        if self.inert || self.patterns.is_empty() {
            return None;
        }
        let mut patterns = self.patterns.clone();
        patterns.set_match_kind(self.config.kind);
        let rabinkarp = RabinKarp::new(&patterns);
        // Effectively, we only want to return a searcher if we can use Teddy,
        // since Teddy is our only fast packed searcher at the moment.
        // Rabin-Karp is only used when searching haystacks smaller than what
        // Teddy can support. Thus, the only way to get a Rabin-Karp searcher
        // is to force it using undocumented APIs (for tests/benchmarks).
        let (search_kind, minimum_len) = match self.config.force {
            None | Some(ForceAlgorithm::Teddy) => {
                let teddy = match self.build_teddy(&patterns) {
                    None => return None,
                    Some(teddy) => teddy,
                };
                let minimum_len = teddy.minimum_len();
                (SearchKind::Teddy(teddy), minimum_len)
            }
            Some(ForceAlgorithm::RabinKarp) => (SearchKind::RabinKarp, 0),
        };
        Some(Searcher {
            config: self.config.clone(),
            patterns,
            rabinkarp,
            search_kind,
            minimum_len,
        })
    }

    fn build_teddy(&self, patterns: &Patterns) -> Option<Teddy> {
        teddy::Builder::new()
            .avx(self.config.force_avx)
            .fat(self.config.force_teddy_fat)
            .build(&patterns)
    }

    /// Add the given pattern to this set to match.
    ///
    /// The order in which patterns are added is significant. Namely, when
    /// using leftmost-first match semantics, then when multiple patterns can
    /// match at a particular location, the pattern that was added first is
    /// used as the match.
    ///
    /// If the number of patterns added exceeds the amount supported by packed
    /// searchers, then the builder will stop accumulating patterns and render
    /// itself inert. At this point, constructing a searcher will always return
    /// `None`.
    pub fn add<P: AsRef<[u8]>>(&mut self, pattern: P) -> &mut Builder {
        if self.inert {
            return self;
        } else if self.patterns.len() >= PATTERN_LIMIT {
            self.inert = true;
            self.patterns.reset();
            return self;
        }
        // Just in case PATTERN_LIMIT increases beyond u16::MAX.
        assert!(self.patterns.len() <= u16::MAX as usize);

        let pattern = pattern.as_ref();
        if pattern.is_empty() {
            self.inert = true;
            self.patterns.reset();
            return self;
        }
        self.patterns.add(pattern);
        self
    }

    /// Add the given iterator of patterns to this set to match.
    ///
    /// The iterator must yield elements that can be converted into a `&[u8]`.
    ///
    /// The order in which patterns are added is significant. Namely, when
    /// using leftmost-first match semantics, then when multiple patterns can
    /// match at a particular location, the pattern that was added first is
    /// used as the match.
    ///
    /// If the number of patterns added exceeds the amount supported by packed
    /// searchers, then the builder will stop accumulating patterns and render
    /// itself inert. At this point, constructing a searcher will always return
    /// `None`.
    pub fn extend<I, P>(&mut self, patterns: I) -> &mut Builder
    where
        I: IntoIterator<Item = P>,
        P: AsRef<[u8]>,
    {
        for p in patterns {
            self.add(p);
        }
        self
    }
}

impl Default for Builder {
    fn default() -> Builder {
        Builder::new()
    }
}

/// A packed searcher for quickly finding occurrences of multiple patterns.
///
/// If callers need more flexible construction, or if one wants to change the
/// match semantics (either leftmost-first or leftmost-longest), then one can
/// use the [`Config`](struct.Config.html) and/or
/// [`Builder`](struct.Builder.html) types for more fine grained control.
///
/// # Example
///
/// This example shows how to create a searcher from an iterator of patterns.
/// By default, leftmost-first match semantics are used.
///
/// ```
/// use aho_corasick::packed::{MatchKind, Searcher};
///
/// # fn example() -> Option<()> {
/// let searcher = Searcher::new(["foobar", "foo"].iter().cloned())?;
/// let matches: Vec<usize> = searcher
///     .find_iter("foobar")
///     .map(|mat| mat.pattern())
///     .collect();
/// assert_eq!(vec![0], matches);
/// # Some(()) }
/// # if cfg!(target_arch = "x86_64") {
/// #     example().unwrap()
/// # } else {
/// #     assert!(example().is_none());
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct Searcher {
    config: Config,
    patterns: Patterns,
    rabinkarp: RabinKarp,
    search_kind: SearchKind,
    minimum_len: usize,
}

#[derive(Clone, Debug)]
enum SearchKind {
    Teddy(Teddy),
    RabinKarp,
}

impl Searcher {
    /// A convenience function for constructing a searcher from an iterator
    /// of things that can be converted to a `&[u8]`.
    ///
    /// If a searcher could not be constructed (either because of an
    /// unsupported CPU or because there are too many patterns), then `None`
    /// is returned.
    ///
    /// # Example
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::packed::{MatchKind, Searcher};
    ///
    /// # fn example() -> Option<()> {
    /// let searcher = Searcher::new(["foobar", "foo"].iter().cloned())?;
    /// let matches: Vec<usize> = searcher
    ///     .find_iter("foobar")
    ///     .map(|mat| mat.pattern())
    ///     .collect();
    /// assert_eq!(vec![0], matches);
    /// # Some(()) }
    /// # if cfg!(target_arch = "x86_64") {
    /// #     example().unwrap()
    /// # } else {
    /// #     assert!(example().is_none());
    /// # }
    /// ```
    pub fn new<I, P>(patterns: I) -> Option<Searcher>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<[u8]>,
    {
        Builder::new().extend(patterns).build()
    }

    /// Return the first occurrence of any of the patterns in this searcher,
    /// according to its match semantics, in the given haystack. The `Match`
    /// returned will include the identifier of the pattern that matched, which
    /// corresponds to the index of the pattern (starting from `0`) in which it
    /// was added.
    ///
    /// # Example
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::packed::{MatchKind, Searcher};
    ///
    /// # fn example() -> Option<()> {
    /// let searcher = Searcher::new(["foobar", "foo"].iter().cloned())?;
    /// let mat = searcher.find("foobar")?;
    /// assert_eq!(0, mat.pattern());
    /// assert_eq!(0, mat.start());
    /// assert_eq!(6, mat.end());
    /// # Some(()) }
    /// # if cfg!(target_arch = "x86_64") {
    /// #     example().unwrap()
    /// # } else {
    /// #     assert!(example().is_none());
    /// # }
    /// ```
    pub fn find<B: AsRef<[u8]>>(&self, haystack: B) -> Option<Match> {
        self.find_at(haystack, 0)
    }

    /// Return the first occurrence of any of the patterns in this searcher,
    /// according to its match semantics, in the given haystack starting from
    /// the given position.
    ///
    /// The `Match` returned will include the identifier of the pattern that
    /// matched, which corresponds to the index of the pattern (starting from
    /// `0`) in which it was added. The offsets in the `Match` will be relative
    /// to the start of `haystack` (and not `at`).
    ///
    /// # Example
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::packed::{MatchKind, Searcher};
    ///
    /// # fn example() -> Option<()> {
    /// let searcher = Searcher::new(["foobar", "foo"].iter().cloned())?;
    /// let mat = searcher.find_at("foofoobar", 3)?;
    /// assert_eq!(0, mat.pattern());
    /// assert_eq!(3, mat.start());
    /// assert_eq!(9, mat.end());
    /// # Some(()) }
    /// # if cfg!(target_arch = "x86_64") {
    /// #     example().unwrap()
    /// # } else {
    /// #     assert!(example().is_none());
    /// # }
    /// ```
    pub fn find_at<B: AsRef<[u8]>>(
        &self,
        haystack: B,
        at: usize,
    ) -> Option<Match> {
        let haystack = haystack.as_ref();
        match self.search_kind {
            SearchKind::Teddy(ref teddy) => {
                if haystack[at..].len() < teddy.minimum_len() {
                    return self.slow_at(haystack, at);
                }
                teddy.find_at(&self.patterns, haystack, at)
            }
            SearchKind::RabinKarp => {
                self.rabinkarp.find_at(&self.patterns, haystack, at)
            }
        }
    }

    /// Return an iterator of non-overlapping occurrences of the patterns in
    /// this searcher, according to its match semantics, in the given haystack.
    ///
    /// # Example
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::packed::{MatchKind, Searcher};
    ///
    /// # fn example() -> Option<()> {
    /// let searcher = Searcher::new(["foobar", "foo"].iter().cloned())?;
    /// let matches: Vec<usize> = searcher
    ///     .find_iter("foobar fooba foofoo")
    ///     .map(|mat| mat.pattern())
    ///     .collect();
    /// assert_eq!(vec![0, 1, 1, 1], matches);
    /// # Some(()) }
    /// # if cfg!(target_arch = "x86_64") {
    /// #     example().unwrap()
    /// # } else {
    /// #     assert!(example().is_none());
    /// # }
    /// ```
    pub fn find_iter<'a, 'b, B: ?Sized + AsRef<[u8]>>(
        &'a self,
        haystack: &'b B,
    ) -> FindIter<'a, 'b> {
        FindIter { searcher: self, haystack: haystack.as_ref(), at: 0 }
    }

    /// Returns the match kind used by this packed searcher.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::packed::{MatchKind, Searcher};
    ///
    /// # fn example() -> Option<()> {
    /// let searcher = Searcher::new(["foobar", "foo"].iter().cloned())?;
    /// // leftmost-first is the default.
    /// assert_eq!(&MatchKind::LeftmostFirst, searcher.match_kind());
    /// # Some(()) }
    /// # if cfg!(target_arch = "x86_64") {
    /// #     example().unwrap()
    /// # } else {
    /// #     assert!(example().is_none());
    /// # }
    /// ```
    pub fn match_kind(&self) -> &MatchKind {
        self.patterns.match_kind()
    }

    /// Returns the minimum length of a haystack that is required in order for
    /// packed searching to be effective.
    ///
    /// In some cases, the underlying packed searcher may not be able to search
    /// very short haystacks. When that occurs, the implementation will defer
    /// to a slower non-packed searcher (which is still generally faster than
    /// Aho-Corasick for a small number of patterns). However, callers may
    /// want to avoid ever using the slower variant, which one can do by
    /// never passing a haystack shorter than the minimum length returned by
    /// this method.
    pub fn minimum_len(&self) -> usize {
        self.minimum_len
    }

    /// Returns the approximate total amount of heap used by this searcher, in
    /// units of bytes.
    pub fn heap_bytes(&self) -> usize {
        self.patterns.heap_bytes()
            + self.rabinkarp.heap_bytes()
            + self.search_kind.heap_bytes()
    }

    /// Use a slow (non-packed) searcher.
    ///
    /// This is useful when a packed searcher could be constructed, but could
    /// not be used to search a specific haystack. For example, if Teddy was
    /// built but the haystack is smaller than ~34 bytes, then Teddy might not
    /// be able to run.
    fn slow_at(&self, haystack: &[u8], at: usize) -> Option<Match> {
        self.rabinkarp.find_at(&self.patterns, haystack, at)
    }
}

impl SearchKind {
    fn heap_bytes(&self) -> usize {
        match *self {
            SearchKind::Teddy(ref ted) => ted.heap_bytes(),
            SearchKind::RabinKarp => 0,
        }
    }
}

/// An iterator over non-overlapping matches from a packed searcher.
///
/// The lifetime `'s` refers to the lifetime of the underlying
/// [`Searcher`](struct.Searcher.html), while the lifetime `'h` refers to the
/// lifetime of the haystack being searched.
#[derive(Debug)]
pub struct FindIter<'s, 'h> {
    searcher: &'s Searcher,
    haystack: &'h [u8],
    at: usize,
}

impl<'s, 'h> Iterator for FindIter<'s, 'h> {
    type Item = Match;

    fn next(&mut self) -> Option<Match> {
        if self.at > self.haystack.len() {
            return None;
        }
        match self.searcher.find_at(&self.haystack, self.at) {
            None => None,
            Some(c) => {
                self.at = c.end;
                Some(c)
            }
        }
    }
}
