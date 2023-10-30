use crate::ahocorasick::MatchKind;
use crate::prefilter::{self, Candidate, Prefilter, PrefilterState};
use crate::state_id::{dead_id, fail_id, StateID};
use crate::Match;

// NOTE: This trait essentially started as a copy of the same trait from from
// regex-automata, with some wording changed since we use this trait for
// NFAs in addition to DFAs in this crate. Additionally, we do not export
// this trait. It's only used internally to reduce code duplication. The
// regex-automata crate needs to expose it because its Regex type is generic
// over implementations of this trait. In this crate, we encapsulate everything
// behind the AhoCorasick type.
//
// This trait is a bit of a mess, but it's not quite clear how to fix it.
// Basically, there are several competing concerns:
//
// * We need performance, so everything effectively needs to get monomorphized.
// * There are several variations on searching Aho-Corasick automatons:
//   overlapping, standard and leftmost. Overlapping and standard are somewhat
//   combined together below, but there is no real way to combine standard with
//   leftmost. Namely, leftmost requires continuing a search even after a match
//   is found, in order to correctly disambiguate a match.
// * On top of that, *sometimes* callers want to know which state the automaton
//   is in after searching. This is principally useful for overlapping and
//   stream searches. However, when callers don't care about this, we really
//   do not want to be forced to compute it, since it sometimes requires extra
//   work. Thus, there are effectively two copies of leftmost searching: one
//   for tracking the state ID and one that doesn't. We should ideally do the
//   same for standard searching, but my sanity stopped me.

// SAFETY RATIONALE: Previously, the code below went to some length to remove
// all bounds checks. This generally produced tighter assembly and lead to
// 20-50% improvements in micro-benchmarks on corpora made up of random
// characters. This somewhat makes sense, since the branch predictor is going
// to be at its worse on random text.
//
// However, using the aho-corasick-debug tool and manually benchmarking
// different inputs, the code *with* bounds checks actually wound up being
// slightly faster:
//
//     $ cat input
//     Sherlock Holmes
//     John Watson
//     Professor Moriarty
//     Irene Adler
//     Mary Watson
//
//     $ aho-corasick-debug-safe \
//         input OpenSubtitles2018.raw.sample.en --kind leftmost-first --dfa
//     pattern read time: 32.824µs
//     automaton build time: 444.687µs
//     automaton heap usage: 72392 bytes
//     match count: 639
//     count time: 1.809961702s
//
//     $ aho-corasick-debug-master \
//         input OpenSubtitles2018.raw.sample.en --kind leftmost-first --dfa
//     pattern read time: 31.425µs
//     automaton build time: 317.434µs
//     automaton heap usage: 72392 bytes
//     match count: 639
//     count time: 2.059157705s
//
// I was able to reproduce this result on two different machines (an i5 and
// an i7). Therefore, we go the route of safe code for now.

/// A trait describing the interface of an Aho-Corasick finite state machine.
///
/// Every automaton has exactly one fail state, one dead state and exactly one
/// start state. Generally, these correspond to the first, second and third
/// states, respectively. The failure state is always treated as a sentinel.
/// That is, no correct Aho-Corasick automaton will ever transition into the
/// fail state. The dead state, however, can be transitioned into, but only
/// when leftmost-first or leftmost-longest match semantics are enabled and
/// only when at least one match has been observed.
///
/// Every automaton also has one or more match states, such that
/// `Automaton::is_match_state(id)` returns `true` if and only if `id`
/// corresponds to a match state.
pub trait Automaton {
    /// The representation used for state identifiers in this automaton.
    ///
    /// Typically, this is one of `u8`, `u16`, `u32`, `u64` or `usize`.
    type ID: StateID;

    /// The type of matching that should be done.
    fn match_kind(&self) -> &MatchKind;

    /// Returns true if and only if this automaton uses anchored searches.
    fn anchored(&self) -> bool;

    /// An optional prefilter for quickly skipping to the next candidate match.
    /// A prefilter must report at least every match, although it may report
    /// positions that do not correspond to a match. That is, it must not allow
    /// false negatives, but can allow false positives.
    ///
    /// Currently, a prefilter only runs when the automaton is in the start
    /// state. That is, the position reported by a prefilter should always
    /// correspond to the start of a potential match.
    fn prefilter(&self) -> Option<&dyn Prefilter>;

    /// Return the identifier of this automaton's start state.
    fn start_state(&self) -> Self::ID;

    /// Returns true if and only if the given state identifier refers to a
    /// valid state.
    fn is_valid(&self, id: Self::ID) -> bool;

    /// Returns true if and only if the given identifier corresponds to a match
    /// state.
    ///
    /// The state ID given must be valid, or else implementors may panic.
    fn is_match_state(&self, id: Self::ID) -> bool;

    /// Returns true if and only if the given identifier corresponds to a state
    /// that is either the dead state or a match state.
    ///
    /// Depending on the implementation of the automaton, this routine can
    /// be used to save a branch in the core matching loop. Nevertheless,
    /// `is_match_state(id) || id == dead_id()` is always a valid
    /// implementation. Indeed, this is the default implementation.
    ///
    /// The state ID given must be valid, or else implementors may panic.
    fn is_match_or_dead_state(&self, id: Self::ID) -> bool {
        id == dead_id() || self.is_match_state(id)
    }

    /// If the given state is a match state, return the match corresponding
    /// to the given match index. `end` must be the ending position of the
    /// detected match. If no match exists or if `match_index` exceeds the
    /// number of matches in this state, then `None` is returned.
    ///
    /// The state ID given must be valid, or else implementors may panic.
    ///
    /// If the given state ID is correct and if the `match_index` is less than
    /// the number of matches for that state, then this is guaranteed to return
    /// a match.
    fn get_match(
        &self,
        id: Self::ID,
        match_index: usize,
        end: usize,
    ) -> Option<Match>;

    /// Returns the number of matches for the given state. If the given state
    /// is not a match state, then this returns 0.
    ///
    /// The state ID given must be valid, or else implementors must panic.
    fn match_count(&self, id: Self::ID) -> usize;

    /// Given the current state that this automaton is in and the next input
    /// byte, this method returns the identifier of the next state. The
    /// identifier returned must always be valid and may never correspond to
    /// the fail state. The returned identifier may, however, point to the
    /// dead state.
    ///
    /// This is not safe so that implementors may look up the next state
    /// without memory safety checks such as bounds checks. As such, callers
    /// must ensure that the given identifier corresponds to a valid automaton
    /// state. Implementors must, in turn, ensure that this routine is safe for
    /// all valid state identifiers and for all possible `u8` values.
    fn next_state(&self, current: Self::ID, input: u8) -> Self::ID;

    /// Like next_state, but debug_asserts that the underlying
    /// implementation never returns a `fail_id()` for the next state.
    fn next_state_no_fail(&self, current: Self::ID, input: u8) -> Self::ID {
        let next = self.next_state(current, input);
        // We should never see a transition to the failure state.
        debug_assert!(
            next != fail_id(),
            "automaton should never return fail_id for next state"
        );
        next
    }

    /// Execute a search using standard match semantics.
    ///
    /// This can be used even when the automaton was constructed with leftmost
    /// match semantics when you want to find the earliest possible match. This
    /// can also be used as part of an overlapping search implementation.
    ///
    /// N.B. This does not report a match if `state_id` is given as a matching
    /// state. As such, this should not be used directly.
    #[inline(always)]
    fn standard_find_at(
        &self,
        prestate: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
        state_id: &mut Self::ID,
    ) -> Option<Match> {
        if let Some(pre) = self.prefilter() {
            self.standard_find_at_imp(
                prestate,
                Some(pre),
                haystack,
                at,
                state_id,
            )
        } else {
            self.standard_find_at_imp(prestate, None, haystack, at, state_id)
        }
    }

    // It's important for this to always be inlined. Namely, its only caller
    // is standard_find_at, and the inlining should remove the case analysis
    // for prefilter scanning when there is no prefilter available.
    #[inline(always)]
    fn standard_find_at_imp(
        &self,
        prestate: &mut PrefilterState,
        prefilter: Option<&dyn Prefilter>,
        haystack: &[u8],
        mut at: usize,
        state_id: &mut Self::ID,
    ) -> Option<Match> {
        while at < haystack.len() {
            if let Some(pre) = prefilter {
                if prestate.is_effective(at) && *state_id == self.start_state()
                {
                    let c = prefilter::next(prestate, pre, haystack, at)
                        .into_option();
                    match c {
                        None => return None,
                        Some(i) => {
                            at = i;
                        }
                    }
                }
            }
            // CORRECTNESS: next_state is correct for all possible u8 values,
            // so the only thing we're concerned about is the validity of
            // `state_id`. `state_id` either comes from the caller (in which
            // case, we assume it is correct), or it comes from the return
            // value of next_state, which is guaranteed to be correct.
            *state_id = self.next_state_no_fail(*state_id, haystack[at]);
            at += 1;
            // This routine always quits immediately after seeing a
            // match, and since dead states can only come after seeing
            // a match, seeing a dead state here is impossible. (Unless
            // we have an anchored automaton, in which case, dead states
            // are used to stop a search.)
            debug_assert!(
                *state_id != dead_id() || self.anchored(),
                "standard find should never see a dead state"
            );

            if self.is_match_or_dead_state(*state_id) {
                return if *state_id == dead_id() {
                    None
                } else {
                    self.get_match(*state_id, 0, at)
                };
            }
        }
        None
    }

    /// Execute a search using leftmost (either first or longest) match
    /// semantics.
    ///
    /// The principle difference between searching with standard semantics and
    /// searching with leftmost semantics is that leftmost searching will
    /// continue searching even after a match has been found. Once a match
    /// is found, the search does not stop until either the haystack has been
    /// exhausted or a dead state is observed in the automaton. (Dead states
    /// only exist in automatons constructed with leftmost semantics.) That is,
    /// we rely on the construction of the automaton to tell us when to quit.
    #[inline(never)]
    fn leftmost_find_at(
        &self,
        prestate: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
        state_id: &mut Self::ID,
    ) -> Option<Match> {
        if let Some(pre) = self.prefilter() {
            self.leftmost_find_at_imp(
                prestate,
                Some(pre),
                haystack,
                at,
                state_id,
            )
        } else {
            self.leftmost_find_at_imp(prestate, None, haystack, at, state_id)
        }
    }

    // It's important for this to always be inlined. Namely, its only caller
    // is leftmost_find_at, and the inlining should remove the case analysis
    // for prefilter scanning when there is no prefilter available.
    #[inline(always)]
    fn leftmost_find_at_imp(
        &self,
        prestate: &mut PrefilterState,
        prefilter: Option<&dyn Prefilter>,
        haystack: &[u8],
        mut at: usize,
        state_id: &mut Self::ID,
    ) -> Option<Match> {
        debug_assert!(self.match_kind().is_leftmost());
        if self.anchored() && at > 0 && *state_id == self.start_state() {
            return None;
        }
        let mut last_match = self.get_match(*state_id, 0, at);
        while at < haystack.len() {
            if let Some(pre) = prefilter {
                if prestate.is_effective(at) && *state_id == self.start_state()
                {
                    let c = prefilter::next(prestate, pre, haystack, at)
                        .into_option();
                    match c {
                        None => return None,
                        Some(i) => {
                            at = i;
                        }
                    }
                }
            }
            // CORRECTNESS: next_state is correct for all possible u8 values,
            // so the only thing we're concerned about is the validity of
            // `state_id`. `state_id` either comes from the caller (in which
            // case, we assume it is correct), or it comes from the return
            // value of next_state, which is guaranteed to be correct.
            *state_id = self.next_state_no_fail(*state_id, haystack[at]);
            at += 1;
            if self.is_match_or_dead_state(*state_id) {
                if *state_id == dead_id() {
                    // The only way to enter into a dead state is if a match
                    // has been found, so we assert as much. This is different
                    // from normal automata, where you might enter a dead state
                    // if you know a subsequent match will never be found
                    // (regardless of whether a match has already been found).
                    // For Aho-Corasick, it is built so that we can match at
                    // any position, so the possibility of a match always
                    // exists.
                    //
                    // (Unless we have an anchored automaton, in which case,
                    // dead states are used to stop a search.)
                    debug_assert!(
                        last_match.is_some() || self.anchored(),
                        "failure state should only be seen after match"
                    );
                    return last_match;
                }
                last_match = self.get_match(*state_id, 0, at);
            }
        }
        last_match
    }

    /// This is like leftmost_find_at, but does not need to track a caller
    /// provided state id. In other words, the only output of this routine is a
    /// match, if one exists.
    ///
    /// It is regrettable that we need to effectively copy a chunk of
    /// implementation twice, but when we don't need to track the state ID, we
    /// can allow the prefilter to report matches immediately without having
    /// to re-confirm them with the automaton. The re-confirmation step is
    /// necessary in leftmost_find_at because tracing through the automaton is
    /// the only way to correctly set the state ID. (Perhaps an alternative
    /// would be to keep a map from pattern ID to matching state ID, but that
    /// complicates the code and still doesn't permit us to defer to the
    /// prefilter entirely when possible.)
    ///
    /// I did try a few things to avoid the code duplication here, but nothing
    /// optimized as well as this approach. (In microbenchmarks, there was
    /// about a 25% difference.)
    #[inline(never)]
    fn leftmost_find_at_no_state(
        &self,
        prestate: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
    ) -> Option<Match> {
        if let Some(pre) = self.prefilter() {
            self.leftmost_find_at_no_state_imp(
                prestate,
                Some(pre),
                haystack,
                at,
            )
        } else {
            self.leftmost_find_at_no_state_imp(prestate, None, haystack, at)
        }
    }

    // It's important for this to always be inlined. Namely, its only caller
    // is leftmost_find_at_no_state, and the inlining should remove the case
    // analysis for prefilter scanning when there is no prefilter available.
    #[inline(always)]
    fn leftmost_find_at_no_state_imp(
        &self,
        prestate: &mut PrefilterState,
        prefilter: Option<&dyn Prefilter>,
        haystack: &[u8],
        mut at: usize,
    ) -> Option<Match> {
        debug_assert!(self.match_kind().is_leftmost());
        if self.anchored() && at > 0 {
            return None;
        }
        // If our prefilter handles confirmation of matches 100% of the
        // time, and since we don't need to track state IDs, we can avoid
        // Aho-Corasick completely.
        if let Some(pre) = prefilter {
            // We should never have a prefilter during an anchored search.
            debug_assert!(!self.anchored());
            if !pre.reports_false_positives() {
                return match pre.next_candidate(prestate, haystack, at) {
                    Candidate::None => None,
                    Candidate::Match(m) => Some(m),
                    Candidate::PossibleStartOfMatch(_) => unreachable!(),
                };
            }
        }

        let mut state_id = self.start_state();
        let mut last_match = self.get_match(state_id, 0, at);
        while at < haystack.len() {
            if let Some(pre) = prefilter {
                if prestate.is_effective(at) && state_id == self.start_state()
                {
                    match prefilter::next(prestate, pre, haystack, at) {
                        Candidate::None => return None,
                        // Since we aren't tracking a state ID, we can
                        // quit early once we know we have a match.
                        Candidate::Match(m) => return Some(m),
                        Candidate::PossibleStartOfMatch(i) => {
                            at = i;
                        }
                    }
                }
            }
            // CORRECTNESS: next_state is correct for all possible u8 values,
            // so the only thing we're concerned about is the validity of
            // `state_id`. `state_id` either comes from the caller (in which
            // case, we assume it is correct), or it comes from the return
            // value of next_state, which is guaranteed to be correct.
            state_id = self.next_state_no_fail(state_id, haystack[at]);
            at += 1;
            if self.is_match_or_dead_state(state_id) {
                if state_id == dead_id() {
                    // The only way to enter into a dead state is if a
                    // match has been found, so we assert as much. This
                    // is different from normal automata, where you might
                    // enter a dead state if you know a subsequent match
                    // will never be found (regardless of whether a match
                    // has already been found). For Aho-Corasick, it is
                    // built so that we can match at any position, so the
                    // possibility of a match always exists.
                    //
                    // (Unless we have an anchored automaton, in which
                    // case, dead states are used to stop a search.)
                    debug_assert!(
                        last_match.is_some() || self.anchored(),
                        "failure state should only be seen after match"
                    );
                    return last_match;
                }
                last_match = self.get_match(state_id, 0, at);
            }
        }
        last_match
    }

    /// Execute an overlapping search.
    ///
    /// When executing an overlapping match, the previous state ID in addition
    /// to the previous match index should be given. If there are more matches
    /// at the given state, then the match is reported and the given index is
    /// incremented.
    #[inline(always)]
    fn overlapping_find_at(
        &self,
        prestate: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
        state_id: &mut Self::ID,
        match_index: &mut usize,
    ) -> Option<Match> {
        if self.anchored() && at > 0 && *state_id == self.start_state() {
            return None;
        }

        let match_count = self.match_count(*state_id);
        if *match_index < match_count {
            // This is guaranteed to return a match since
            // match_index < match_count.
            let result = self.get_match(*state_id, *match_index, at);
            debug_assert!(result.is_some(), "must be a match");
            *match_index += 1;
            return result;
        }

        *match_index = 0;
        match self.standard_find_at(prestate, haystack, at, state_id) {
            None => None,
            Some(m) => {
                *match_index = 1;
                Some(m)
            }
        }
    }

    /// Return the earliest match found. This returns as soon as we know that
    /// we have a match. As such, this does not necessarily correspond to the
    /// leftmost starting match, but rather, the leftmost position at which a
    /// match ends.
    #[inline(always)]
    fn earliest_find_at(
        &self,
        prestate: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
        state_id: &mut Self::ID,
    ) -> Option<Match> {
        if *state_id == self.start_state() {
            if self.anchored() && at > 0 {
                return None;
            }
            if let Some(m) = self.get_match(*state_id, 0, at) {
                return Some(m);
            }
        }
        self.standard_find_at(prestate, haystack, at, state_id)
    }

    /// A convenience function for finding the next match according to the
    /// match semantics of this automaton. For standard match semantics, this
    /// finds the earliest match. Otherwise, the leftmost match is found.
    #[inline(always)]
    fn find_at(
        &self,
        prestate: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
        state_id: &mut Self::ID,
    ) -> Option<Match> {
        match *self.match_kind() {
            MatchKind::Standard => {
                self.earliest_find_at(prestate, haystack, at, state_id)
            }
            MatchKind::LeftmostFirst | MatchKind::LeftmostLongest => {
                self.leftmost_find_at(prestate, haystack, at, state_id)
            }
            MatchKind::__Nonexhaustive => unreachable!(),
        }
    }

    /// Like find_at, but does not track state identifiers. This permits some
    /// optimizations when a prefilter that confirms its own matches is
    /// present.
    #[inline(always)]
    fn find_at_no_state(
        &self,
        prestate: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
    ) -> Option<Match> {
        match *self.match_kind() {
            MatchKind::Standard => {
                let mut state = self.start_state();
                self.earliest_find_at(prestate, haystack, at, &mut state)
            }
            MatchKind::LeftmostFirst | MatchKind::LeftmostLongest => {
                self.leftmost_find_at_no_state(prestate, haystack, at)
            }
            MatchKind::__Nonexhaustive => unreachable!(),
        }
    }
}
