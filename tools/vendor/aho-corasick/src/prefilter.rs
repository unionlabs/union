use std::cmp;
use std::fmt;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::u8;

use memchr::{memchr, memchr2, memchr3};

use crate::ahocorasick::MatchKind;
use crate::packed;
use crate::Match;

/// A candidate is the result of running a prefilter on a haystack at a
/// particular position. The result is either no match, a confirmed match or
/// a possible match.
///
/// When no match is returned, the prefilter is guaranteeing that no possible
/// match can be found in the haystack, and the caller may trust this. That is,
/// all correct prefilters must never report false negatives.
///
/// In some cases, a prefilter can confirm a match very quickly, in which case,
/// the caller may use this to stop what it's doing and report the match. In
/// this case, prefilter implementations must never report a false positive.
/// In other cases, the prefilter can only report a potential match, in which
/// case the callers must attempt to confirm the match. In this case, prefilter
/// implementations are permitted to return false positives.
#[derive(Clone, Debug)]
pub enum Candidate {
    None,
    Match(Match),
    PossibleStartOfMatch(usize),
}

impl Candidate {
    /// Convert this candidate into an option. This is useful when callers
    /// do not distinguish between true positives and false positives (i.e.,
    /// the caller must always confirm the match in order to update some other
    /// state).
    pub fn into_option(self) -> Option<usize> {
        match self {
            Candidate::None => None,
            Candidate::Match(ref m) => Some(m.start()),
            Candidate::PossibleStartOfMatch(start) => Some(start),
        }
    }
}

/// A prefilter describes the behavior of fast literal scanners for quickly
/// skipping past bytes in the haystack that we know cannot possibly
/// participate in a match.
pub trait Prefilter:
    Send + Sync + RefUnwindSafe + UnwindSafe + fmt::Debug
{
    /// Returns the next possible match candidate. This may yield false
    /// positives, so callers must confirm a match starting at the position
    /// returned. This, however, must never produce false negatives. That is,
    /// this must, at minimum, return the starting position of the next match
    /// in the given haystack after or at the given position.
    fn next_candidate(
        &self,
        state: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
    ) -> Candidate;

    /// A method for cloning a prefilter, to work-around the fact that Clone
    /// is not object-safe.
    fn clone_prefilter(&self) -> Box<dyn Prefilter>;

    /// Returns the approximate total amount of heap used by this prefilter, in
    /// units of bytes.
    fn heap_bytes(&self) -> usize;

    /// Returns true if and only if this prefilter never returns false
    /// positives. This is useful for completely avoiding the automaton
    /// when the prefilter can quickly confirm its own matches.
    ///
    /// By default, this returns true, which is conservative; it is always
    /// correct to return `true`. Returning `false` here and reporting a false
    /// positive will result in incorrect searches.
    fn reports_false_positives(&self) -> bool {
        true
    }

    /// Returns true if and only if this prefilter may look for a non-starting
    /// position of a match.
    ///
    /// This is useful in a streaming context where prefilters that don't look
    /// for a starting position of a match can be quite difficult to deal with.
    ///
    /// This returns false by default.
    fn looks_for_non_start_of_match(&self) -> bool {
        false
    }
}

impl<'a, P: Prefilter + ?Sized> Prefilter for &'a P {
    #[inline]
    fn next_candidate(
        &self,
        state: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
    ) -> Candidate {
        (**self).next_candidate(state, haystack, at)
    }

    fn clone_prefilter(&self) -> Box<dyn Prefilter> {
        (**self).clone_prefilter()
    }

    fn heap_bytes(&self) -> usize {
        (**self).heap_bytes()
    }

    fn reports_false_positives(&self) -> bool {
        (**self).reports_false_positives()
    }
}

/// A convenience object for representing any type that implements Prefilter
/// and is cloneable.
#[derive(Debug)]
pub struct PrefilterObj(Box<dyn Prefilter>);

impl Clone for PrefilterObj {
    fn clone(&self) -> Self {
        PrefilterObj(self.0.clone_prefilter())
    }
}

impl PrefilterObj {
    /// Create a new prefilter object.
    pub fn new<T: Prefilter + 'static>(t: T) -> PrefilterObj {
        PrefilterObj(Box::new(t))
    }

    /// Return the underlying prefilter trait object.
    pub fn as_ref(&self) -> &dyn Prefilter {
        &*self.0
    }
}

/// PrefilterState tracks state associated with the effectiveness of a
/// prefilter. It is used to track how many bytes, on average, are skipped by
/// the prefilter. If this average dips below a certain threshold over time,
/// then the state renders the prefilter inert and stops using it.
///
/// A prefilter state should be created for each search. (Where creating an
/// iterator via, e.g., `find_iter`, is treated as a single search.)
#[derive(Clone, Debug)]
pub struct PrefilterState {
    /// The number of skips that has been executed.
    skips: usize,
    /// The total number of bytes that have been skipped.
    skipped: usize,
    /// The maximum length of a match. This is used to help determine how many
    /// bytes on average should be skipped in order for a prefilter to be
    /// effective.
    max_match_len: usize,
    /// Once this heuristic has been deemed permanently ineffective, it will be
    /// inert throughout the rest of its lifetime. This serves as a cheap way
    /// to check inertness.
    inert: bool,
    /// The last (absolute) position at which a prefilter scanned to.
    /// Prefilters can use this position to determine whether to re-scan or
    /// not.
    ///
    /// Unlike other things that impact effectiveness, this is a fleeting
    /// condition. That is, a prefilter can be considered ineffective if it is
    /// at a position before `last_scan_at`, but can become effective again
    /// once the search moves past `last_scan_at`.
    ///
    /// The utility of this is to both avoid additional overhead from calling
    /// the prefilter and to avoid quadratic behavior. This ensures that a
    /// prefilter will scan any particular byte at most once. (Note that some
    /// prefilters, like the start-byte prefilter, do not need to use this
    /// field at all, since it only looks for starting bytes.)
    last_scan_at: usize,
}

impl PrefilterState {
    /// The minimum number of skip attempts to try before considering whether
    /// a prefilter is effective or not.
    const MIN_SKIPS: usize = 40;

    /// The minimum amount of bytes that skipping must average, expressed as a
    /// factor of the multiple of the length of a possible match.
    ///
    /// That is, after MIN_SKIPS have occurred, if the average number of bytes
    /// skipped ever falls below MIN_AVG_FACTOR * max-match-length, then the
    /// prefilter outed to be rendered inert.
    const MIN_AVG_FACTOR: usize = 2;

    /// Create a fresh prefilter state.
    pub fn new(max_match_len: usize) -> PrefilterState {
        PrefilterState {
            skips: 0,
            skipped: 0,
            max_match_len,
            inert: false,
            last_scan_at: 0,
        }
    }

    /// Create a prefilter state that always disables the prefilter.
    pub fn disabled() -> PrefilterState {
        PrefilterState {
            skips: 0,
            skipped: 0,
            max_match_len: 0,
            inert: true,
            last_scan_at: 0,
        }
    }

    /// Update this state with the number of bytes skipped on the last
    /// invocation of the prefilter.
    #[inline]
    fn update_skipped_bytes(&mut self, skipped: usize) {
        self.skips += 1;
        self.skipped += skipped;
    }

    /// Updates the position at which the last scan stopped. This may be
    /// greater than the position of the last candidate reported. For example,
    /// searching for the "rare" byte `z` in `abczdef` for the pattern `abcz`
    /// will report a candidate at position `0`, but the end of its last scan
    /// will be at position `3`.
    ///
    /// This position factors into the effectiveness of this prefilter. If the
    /// current position is less than the last position at which a scan ended,
    /// then the prefilter should not be re-run until the search moves past
    /// that position.
    #[inline]
    fn update_at(&mut self, at: usize) {
        if at > self.last_scan_at {
            self.last_scan_at = at;
        }
    }

    /// Return true if and only if this state indicates that a prefilter is
    /// still effective.
    ///
    /// The given pos should correspond to the current starting position of the
    /// search.
    #[inline]
    pub fn is_effective(&mut self, at: usize) -> bool {
        if self.inert {
            return false;
        }
        if at < self.last_scan_at {
            return false;
        }
        if self.skips < PrefilterState::MIN_SKIPS {
            return true;
        }

        let min_avg = PrefilterState::MIN_AVG_FACTOR * self.max_match_len;
        if self.skipped >= min_avg * self.skips {
            return true;
        }

        // We're inert.
        self.inert = true;
        false
    }
}

/// A builder for constructing the best possible prefilter. When constructed,
/// this builder will heuristically select the best prefilter it can build,
/// if any, and discard the rest.
#[derive(Debug)]
pub struct Builder {
    count: usize,
    ascii_case_insensitive: bool,
    start_bytes: StartBytesBuilder,
    rare_bytes: RareBytesBuilder,
    packed: Option<packed::Builder>,
}

impl Builder {
    /// Create a new builder for constructing the best possible prefilter.
    pub fn new(kind: MatchKind) -> Builder {
        let pbuilder = kind
            .as_packed()
            .map(|kind| packed::Config::new().match_kind(kind).builder());
        Builder {
            count: 0,
            ascii_case_insensitive: false,
            start_bytes: StartBytesBuilder::new(),
            rare_bytes: RareBytesBuilder::new(),
            packed: pbuilder,
        }
    }

    /// Enable ASCII case insensitivity. When set, byte strings added to this
    /// builder will be interpreted without respect to ASCII case.
    pub fn ascii_case_insensitive(mut self, yes: bool) -> Builder {
        self.ascii_case_insensitive = yes;
        self.start_bytes = self.start_bytes.ascii_case_insensitive(yes);
        self.rare_bytes = self.rare_bytes.ascii_case_insensitive(yes);
        self
    }

    /// Return a prefilter suitable for quickly finding potential matches.
    ///
    /// All patterns added to an Aho-Corasick automaton should be added to this
    /// builder before attempting to construct the prefilter.
    pub fn build(&self) -> Option<PrefilterObj> {
        // match (self.start_bytes.build(), self.rare_bytes.build()) {
        match (self.start_bytes.build(), self.rare_bytes.build()) {
            // If we could build both start and rare prefilters, then there are
            // a few cases in which we'd want to use the start-byte prefilter
            // over the rare-byte prefilter, since the former has lower
            // overhead.
            (prestart @ Some(_), prerare @ Some(_)) => {
                // If the start-byte prefilter can scan for a smaller number
                // of bytes than the rare-byte prefilter, then it's probably
                // faster.
                let has_fewer_bytes =
                    self.start_bytes.count < self.rare_bytes.count;
                // Otherwise, if the combined frequency rank of the detected
                // bytes in the start-byte prefilter is "close" to the combined
                // frequency rank of the rare-byte prefilter, then we pick
                // the start-byte prefilter even if the rare-byte prefilter
                // heuristically searches for rare bytes. This is because the
                // rare-byte prefilter has higher constant costs, so we tend to
                // prefer the start-byte prefilter when we can.
                let has_rarer_bytes =
                    self.start_bytes.rank_sum <= self.rare_bytes.rank_sum + 50;
                if has_fewer_bytes || has_rarer_bytes {
                    prestart
                } else {
                    prerare
                }
            }
            (prestart @ Some(_), None) => prestart,
            (None, prerare @ Some(_)) => prerare,
            (None, None) if self.ascii_case_insensitive => None,
            (None, None) => self
                .packed
                .as_ref()
                .and_then(|b| b.build())
                .map(|s| PrefilterObj::new(Packed(s))),
        }
    }

    /// Add a literal string to this prefilter builder.
    pub fn add(&mut self, bytes: &[u8]) {
        self.count += 1;
        self.start_bytes.add(bytes);
        self.rare_bytes.add(bytes);
        if let Some(ref mut pbuilder) = self.packed {
            pbuilder.add(bytes);
        }
    }
}

/// A type that wraps a packed searcher and implements the `Prefilter`
/// interface.
#[derive(Clone, Debug)]
struct Packed(packed::Searcher);

impl Prefilter for Packed {
    fn next_candidate(
        &self,
        _state: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
    ) -> Candidate {
        self.0.find_at(haystack, at).map_or(Candidate::None, Candidate::Match)
    }

    fn clone_prefilter(&self) -> Box<dyn Prefilter> {
        Box::new(self.clone())
    }

    fn heap_bytes(&self) -> usize {
        self.0.heap_bytes()
    }

    fn reports_false_positives(&self) -> bool {
        false
    }
}

/// A builder for constructing a rare byte prefilter.
///
/// A rare byte prefilter attempts to pick out a small set of rare bytes that
/// occurr in the patterns, and then quickly scan to matches of those rare
/// bytes.
#[derive(Clone, Debug)]
struct RareBytesBuilder {
    /// Whether this prefilter should account for ASCII case insensitivity or
    /// not.
    ascii_case_insensitive: bool,
    /// A set of rare bytes, indexed by byte value.
    rare_set: ByteSet,
    /// A set of byte offsets associated with bytes in a pattern. An entry
    /// corresponds to a particular bytes (its index) and is only non-zero if
    /// the byte occurred at an offset greater than 0 in at least one pattern.
    ///
    /// If a byte's offset is not representable in 8 bits, then the rare bytes
    /// prefilter becomes inert.
    byte_offsets: RareByteOffsets,
    /// Whether this is available as a prefilter or not. This can be set to
    /// false during construction if a condition is seen that invalidates the
    /// use of the rare-byte prefilter.
    available: bool,
    /// The number of bytes set to an active value in `byte_offsets`.
    count: usize,
    /// The sum of frequency ranks for the rare bytes detected. This is
    /// intended to give a heuristic notion of how rare the bytes are.
    rank_sum: u16,
}

/// A set of bytes.
#[derive(Clone, Copy)]
struct ByteSet([bool; 256]);

impl ByteSet {
    fn empty() -> ByteSet {
        ByteSet([false; 256])
    }

    fn insert(&mut self, b: u8) -> bool {
        let new = !self.contains(b);
        self.0[b as usize] = true;
        new
    }

    fn contains(&self, b: u8) -> bool {
        self.0[b as usize]
    }
}

impl fmt::Debug for ByteSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut bytes = vec![];
        for b in 0..=255 {
            if self.contains(b) {
                bytes.push(b);
            }
        }
        f.debug_struct("ByteSet").field("set", &bytes).finish()
    }
}

/// A set of byte offsets, keyed by byte.
#[derive(Clone, Copy)]
struct RareByteOffsets {
    /// Each entry corresponds to the maximum offset of the corresponding
    /// byte across all patterns seen.
    set: [RareByteOffset; 256],
}

impl RareByteOffsets {
    /// Create a new empty set of rare byte offsets.
    pub fn empty() -> RareByteOffsets {
        RareByteOffsets { set: [RareByteOffset::default(); 256] }
    }

    /// Add the given offset for the given byte to this set. If the offset is
    /// greater than the existing offset, then it overwrites the previous
    /// value and returns false. If there is no previous value set, then this
    /// sets it and returns true.
    pub fn set(&mut self, byte: u8, off: RareByteOffset) {
        self.set[byte as usize].max =
            cmp::max(self.set[byte as usize].max, off.max);
    }
}

impl fmt::Debug for RareByteOffsets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut offsets = vec![];
        for off in self.set.iter() {
            if off.max > 0 {
                offsets.push(off);
            }
        }
        f.debug_struct("RareByteOffsets").field("set", &offsets).finish()
    }
}

/// Offsets associated with an occurrence of a "rare" byte in any of the
/// patterns used to construct a single Aho-Corasick automaton.
#[derive(Clone, Copy, Debug)]
struct RareByteOffset {
    /// The maximum offset at which a particular byte occurs from the start
    /// of any pattern. This is used as a shift amount. That is, when an
    /// occurrence of this byte is found, the candidate position reported by
    /// the prefilter is `position_of_byte - max`, such that the automaton
    /// will begin its search at a position that is guaranteed to observe a
    /// match.
    ///
    /// To avoid accidentally quadratic behavior, a prefilter is considered
    /// ineffective when it is asked to start scanning from a position that it
    /// has already scanned past.
    ///
    /// Using a `u8` here means that if we ever see a pattern that's longer
    /// than 255 bytes, then the entire rare byte prefilter is disabled.
    max: u8,
}

impl Default for RareByteOffset {
    fn default() -> RareByteOffset {
        RareByteOffset { max: 0 }
    }
}

impl RareByteOffset {
    /// Create a new rare byte offset. If the given offset is too big, then
    /// None is returned. In that case, callers should render the rare bytes
    /// prefilter inert.
    fn new(max: usize) -> Option<RareByteOffset> {
        if max > u8::MAX as usize {
            None
        } else {
            Some(RareByteOffset { max: max as u8 })
        }
    }
}

impl RareBytesBuilder {
    /// Create a new builder for constructing a rare byte prefilter.
    fn new() -> RareBytesBuilder {
        RareBytesBuilder {
            ascii_case_insensitive: false,
            rare_set: ByteSet::empty(),
            byte_offsets: RareByteOffsets::empty(),
            available: true,
            count: 0,
            rank_sum: 0,
        }
    }

    /// Enable ASCII case insensitivity. When set, byte strings added to this
    /// builder will be interpreted without respect to ASCII case.
    fn ascii_case_insensitive(mut self, yes: bool) -> RareBytesBuilder {
        self.ascii_case_insensitive = yes;
        self
    }

    /// Build the rare bytes prefilter.
    ///
    /// If there are more than 3 distinct starting bytes, or if heuristics
    /// otherwise determine that this prefilter should not be used, then `None`
    /// is returned.
    fn build(&self) -> Option<PrefilterObj> {
        if !self.available || self.count > 3 {
            return None;
        }
        let (mut bytes, mut len) = ([0; 3], 0);
        for b in 0..=255 {
            if self.rare_set.contains(b) {
                bytes[len] = b as u8;
                len += 1;
            }
        }
        match len {
            0 => None,
            1 => Some(PrefilterObj::new(RareBytesOne {
                byte1: bytes[0],
                offset: self.byte_offsets.set[bytes[0] as usize],
            })),
            2 => Some(PrefilterObj::new(RareBytesTwo {
                offsets: self.byte_offsets,
                byte1: bytes[0],
                byte2: bytes[1],
            })),
            3 => Some(PrefilterObj::new(RareBytesThree {
                offsets: self.byte_offsets,
                byte1: bytes[0],
                byte2: bytes[1],
                byte3: bytes[2],
            })),
            _ => unreachable!(),
        }
    }

    /// Add a byte string to this builder.
    ///
    /// All patterns added to an Aho-Corasick automaton should be added to this
    /// builder before attempting to construct the prefilter.
    fn add(&mut self, bytes: &[u8]) {
        // If we've already given up, then do nothing.
        if !self.available {
            return;
        }
        // If we've already blown our budget, then don't waste time looking
        // for more rare bytes.
        if self.count > 3 {
            self.available = false;
            return;
        }
        // If the pattern is too long, then our offset table is bunk, so
        // give up.
        if bytes.len() >= 256 {
            self.available = false;
            return;
        }
        let mut rarest = match bytes.get(0) {
            None => return,
            Some(&b) => (b, freq_rank(b)),
        };
        // The idea here is to look for the rarest byte in each pattern, and
        // add that to our set. As a special exception, if we see a byte that
        // we've already added, then we immediately stop and choose that byte,
        // even if there's another rare byte in the pattern. This helps us
        // apply the rare byte optimization in more cases by attempting to pick
        // bytes that are in common between patterns. So for example, if we
        // were searching for `Sherlock` and `lockjaw`, then this would pick
        // `k` for both patterns, resulting in the use of `memchr` instead of
        // `memchr2` for `k` and `j`.
        let mut found = false;
        for (pos, &b) in bytes.iter().enumerate() {
            self.set_offset(pos, b);
            if found {
                continue;
            }
            if self.rare_set.contains(b) {
                found = true;
                continue;
            }
            let rank = freq_rank(b);
            if rank < rarest.1 {
                rarest = (b, rank);
            }
        }
        if !found {
            self.add_rare_byte(rarest.0);
        }
    }

    fn set_offset(&mut self, pos: usize, byte: u8) {
        // This unwrap is OK because pos is never bigger than our max.
        let offset = RareByteOffset::new(pos).unwrap();
        self.byte_offsets.set(byte, offset);
        if self.ascii_case_insensitive {
            self.byte_offsets.set(opposite_ascii_case(byte), offset);
        }
    }

    fn add_rare_byte(&mut self, byte: u8) {
        self.add_one_rare_byte(byte);
        if self.ascii_case_insensitive {
            self.add_one_rare_byte(opposite_ascii_case(byte));
        }
    }

    fn add_one_rare_byte(&mut self, byte: u8) {
        if self.rare_set.insert(byte) {
            self.count += 1;
            self.rank_sum += freq_rank(byte) as u16;
        }
    }
}

/// A prefilter for scanning for a single "rare" byte.
#[derive(Clone, Debug)]
struct RareBytesOne {
    byte1: u8,
    offset: RareByteOffset,
}

impl Prefilter for RareBytesOne {
    fn next_candidate(
        &self,
        state: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
    ) -> Candidate {
        memchr(self.byte1, &haystack[at..])
            .map(|i| {
                let pos = at + i;
                state.last_scan_at = pos;
                cmp::max(at, pos.saturating_sub(self.offset.max as usize))
            })
            .map_or(Candidate::None, Candidate::PossibleStartOfMatch)
    }

    fn clone_prefilter(&self) -> Box<dyn Prefilter> {
        Box::new(self.clone())
    }

    fn heap_bytes(&self) -> usize {
        0
    }

    fn looks_for_non_start_of_match(&self) -> bool {
        // TODO: It should be possible to use a rare byte prefilter in a
        // streaming context. The main problem is that we usually assume that
        // if a prefilter has scanned some text and not found anything, then no
        // match *starts* in that text. This doesn't matter in non-streaming
        // contexts, but in a streaming context, if we're looking for a byte
        // that doesn't start at the beginning of a match and don't find it,
        // then it's still possible for a match to start at the end of the
        // current buffer content. In order to fix this, the streaming searcher
        // would need to become aware of prefilters that do this and use the
        // appropriate offset in various places. It is quite a delicate change
        // and probably shouldn't be attempted until streaming search has a
        // better testing strategy. In particular, we'd really like to be able
        // to vary the buffer size to force strange cases that occur at the
        // edge of the buffer. If we make the buffer size minimal, then these
        // cases occur more frequently and easier.
        //
        // This is also a bummer because this means that if the prefilter
        // builder chose a rare byte prefilter, then a streaming search won't
        // use any prefilter at all because the builder doesn't know how it's
        // going to be used. Assuming we don't make streaming search aware of
        // these special types of prefilters as described above, we could fix
        // this by building a "backup" prefilter that could be used when the
        // rare byte prefilter could not. But that's a bandaide. Sigh.
        true
    }
}

/// A prefilter for scanning for two "rare" bytes.
#[derive(Clone, Debug)]
struct RareBytesTwo {
    offsets: RareByteOffsets,
    byte1: u8,
    byte2: u8,
}

impl Prefilter for RareBytesTwo {
    fn next_candidate(
        &self,
        state: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
    ) -> Candidate {
        memchr2(self.byte1, self.byte2, &haystack[at..])
            .map(|i| {
                let pos = at + i;
                state.update_at(pos);
                let offset = self.offsets.set[haystack[pos] as usize].max;
                cmp::max(at, pos.saturating_sub(offset as usize))
            })
            .map_or(Candidate::None, Candidate::PossibleStartOfMatch)
    }

    fn clone_prefilter(&self) -> Box<dyn Prefilter> {
        Box::new(self.clone())
    }

    fn heap_bytes(&self) -> usize {
        0
    }

    fn looks_for_non_start_of_match(&self) -> bool {
        // TODO: See Prefilter impl for RareBytesOne.
        true
    }
}

/// A prefilter for scanning for three "rare" bytes.
#[derive(Clone, Debug)]
struct RareBytesThree {
    offsets: RareByteOffsets,
    byte1: u8,
    byte2: u8,
    byte3: u8,
}

impl Prefilter for RareBytesThree {
    fn next_candidate(
        &self,
        state: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
    ) -> Candidate {
        memchr3(self.byte1, self.byte2, self.byte3, &haystack[at..])
            .map(|i| {
                let pos = at + i;
                state.update_at(pos);
                let offset = self.offsets.set[haystack[pos] as usize].max;
                cmp::max(at, pos.saturating_sub(offset as usize))
            })
            .map_or(Candidate::None, Candidate::PossibleStartOfMatch)
    }

    fn clone_prefilter(&self) -> Box<dyn Prefilter> {
        Box::new(self.clone())
    }

    fn heap_bytes(&self) -> usize {
        0
    }

    fn looks_for_non_start_of_match(&self) -> bool {
        // TODO: See Prefilter impl for RareBytesOne.
        true
    }
}

/// A builder for constructing a starting byte prefilter.
///
/// A starting byte prefilter is a simplistic prefilter that looks for possible
/// matches by reporting all positions corresponding to a particular byte. This
/// generally only takes affect when there are at most 3 distinct possible
/// starting bytes. e.g., the patterns `foo`, `bar`, and `baz` have two
/// distinct starting bytes (`f` and `b`), and this prefilter returns all
/// occurrences of either `f` or `b`.
///
/// In some cases, a heuristic frequency analysis may determine that it would
/// be better not to use this prefilter even when there are 3 or fewer distinct
/// starting bytes.
#[derive(Clone, Debug)]
struct StartBytesBuilder {
    /// Whether this prefilter should account for ASCII case insensitivity or
    /// not.
    ascii_case_insensitive: bool,
    /// The set of starting bytes observed.
    byteset: Vec<bool>,
    /// The number of bytes set to true in `byteset`.
    count: usize,
    /// The sum of frequency ranks for the rare bytes detected. This is
    /// intended to give a heuristic notion of how rare the bytes are.
    rank_sum: u16,
}

impl StartBytesBuilder {
    /// Create a new builder for constructing a start byte prefilter.
    fn new() -> StartBytesBuilder {
        StartBytesBuilder {
            ascii_case_insensitive: false,
            byteset: vec![false; 256],
            count: 0,
            rank_sum: 0,
        }
    }

    /// Enable ASCII case insensitivity. When set, byte strings added to this
    /// builder will be interpreted without respect to ASCII case.
    fn ascii_case_insensitive(mut self, yes: bool) -> StartBytesBuilder {
        self.ascii_case_insensitive = yes;
        self
    }

    /// Build the starting bytes prefilter.
    ///
    /// If there are more than 3 distinct starting bytes, or if heuristics
    /// otherwise determine that this prefilter should not be used, then `None`
    /// is returned.
    fn build(&self) -> Option<PrefilterObj> {
        if self.count > 3 {
            return None;
        }
        let (mut bytes, mut len) = ([0; 3], 0);
        for b in 0..256 {
            if !self.byteset[b] {
                continue;
            }
            // We don't handle non-ASCII bytes for now. Getting non-ASCII
            // bytes right is trickier, since we generally don't want to put
            // a leading UTF-8 code unit into a prefilter that isn't ASCII,
            // since they can frequently. Instead, it would be better to use a
            // continuation byte, but this requires more sophisticated analysis
            // of the automaton and a richer prefilter API.
            if b > 0x7F {
                return None;
            }
            bytes[len] = b as u8;
            len += 1;
        }
        match len {
            0 => None,
            1 => Some(PrefilterObj::new(StartBytesOne { byte1: bytes[0] })),
            2 => Some(PrefilterObj::new(StartBytesTwo {
                byte1: bytes[0],
                byte2: bytes[1],
            })),
            3 => Some(PrefilterObj::new(StartBytesThree {
                byte1: bytes[0],
                byte2: bytes[1],
                byte3: bytes[2],
            })),
            _ => unreachable!(),
        }
    }

    /// Add a byte string to this builder.
    ///
    /// All patterns added to an Aho-Corasick automaton should be added to this
    /// builder before attempting to construct the prefilter.
    fn add(&mut self, bytes: &[u8]) {
        if self.count > 3 {
            return;
        }
        if let Some(&byte) = bytes.get(0) {
            self.add_one_byte(byte);
            if self.ascii_case_insensitive {
                self.add_one_byte(opposite_ascii_case(byte));
            }
        }
    }

    fn add_one_byte(&mut self, byte: u8) {
        if !self.byteset[byte as usize] {
            self.byteset[byte as usize] = true;
            self.count += 1;
            self.rank_sum += freq_rank(byte) as u16;
        }
    }
}

/// A prefilter for scanning for a single starting byte.
#[derive(Clone, Debug)]
struct StartBytesOne {
    byte1: u8,
}

impl Prefilter for StartBytesOne {
    fn next_candidate(
        &self,
        _state: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
    ) -> Candidate {
        memchr(self.byte1, &haystack[at..])
            .map(|i| at + i)
            .map_or(Candidate::None, Candidate::PossibleStartOfMatch)
    }

    fn clone_prefilter(&self) -> Box<dyn Prefilter> {
        Box::new(self.clone())
    }

    fn heap_bytes(&self) -> usize {
        0
    }
}

/// A prefilter for scanning for two starting bytes.
#[derive(Clone, Debug)]
struct StartBytesTwo {
    byte1: u8,
    byte2: u8,
}

impl Prefilter for StartBytesTwo {
    fn next_candidate(
        &self,
        _state: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
    ) -> Candidate {
        memchr2(self.byte1, self.byte2, &haystack[at..])
            .map(|i| at + i)
            .map_or(Candidate::None, Candidate::PossibleStartOfMatch)
    }

    fn clone_prefilter(&self) -> Box<dyn Prefilter> {
        Box::new(self.clone())
    }

    fn heap_bytes(&self) -> usize {
        0
    }
}

/// A prefilter for scanning for three starting bytes.
#[derive(Clone, Debug)]
struct StartBytesThree {
    byte1: u8,
    byte2: u8,
    byte3: u8,
}

impl Prefilter for StartBytesThree {
    fn next_candidate(
        &self,
        _state: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
    ) -> Candidate {
        memchr3(self.byte1, self.byte2, self.byte3, &haystack[at..])
            .map(|i| at + i)
            .map_or(Candidate::None, Candidate::PossibleStartOfMatch)
    }

    fn clone_prefilter(&self) -> Box<dyn Prefilter> {
        Box::new(self.clone())
    }

    fn heap_bytes(&self) -> usize {
        0
    }
}

/// Return the next candidate reported by the given prefilter while
/// simultaneously updating the given prestate.
///
/// The caller is responsible for checking the prestate before deciding whether
/// to initiate a search.
#[inline]
pub fn next<P: Prefilter>(
    prestate: &mut PrefilterState,
    prefilter: P,
    haystack: &[u8],
    at: usize,
) -> Candidate {
    let cand = prefilter.next_candidate(prestate, haystack, at);
    match cand {
        Candidate::None => {
            prestate.update_skipped_bytes(haystack.len() - at);
        }
        Candidate::Match(ref m) => {
            prestate.update_skipped_bytes(m.start() - at);
        }
        Candidate::PossibleStartOfMatch(i) => {
            prestate.update_skipped_bytes(i - at);
        }
    }
    cand
}

/// If the given byte is an ASCII letter, then return it in the opposite case.
/// e.g., Given `b'A'`, this returns `b'a'`, and given `b'a'`, this returns
/// `b'A'`. If a non-ASCII letter is given, then the given byte is returned.
pub fn opposite_ascii_case(b: u8) -> u8 {
    if b'A' <= b && b <= b'Z' {
        b.to_ascii_lowercase()
    } else if b'a' <= b && b <= b'z' {
        b.to_ascii_uppercase()
    } else {
        b
    }
}

/// Return the frequency rank of the given byte. The higher the rank, the more
/// common the byte (heuristically speaking).
fn freq_rank(b: u8) -> u8 {
    use crate::byte_frequencies::BYTE_FREQUENCIES;
    BYTE_FREQUENCIES[b as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scratch() {
        let mut b = Builder::new(MatchKind::LeftmostFirst);
        b.add(b"Sherlock");
        b.add(b"locjaw");
        // b.add(b"Sherlock");
        // b.add(b"Holmes");
        // b.add(b"Watson");
        // b.add("Шерлок Холмс".as_bytes());
        // b.add("Джон Уотсон".as_bytes());

        let s = b.build().unwrap();
        println!("{:?}", s);
    }
}
