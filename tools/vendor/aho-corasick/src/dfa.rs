use std::mem::size_of;

use crate::ahocorasick::MatchKind;
use crate::automaton::Automaton;
use crate::classes::ByteClasses;
use crate::error::Result;
use crate::nfa::{PatternID, PatternLength, NFA};
use crate::prefilter::{Prefilter, PrefilterObj, PrefilterState};
use crate::state_id::{dead_id, fail_id, premultiply_overflow_error, StateID};
use crate::Match;

#[derive(Clone, Debug)]
pub enum DFA<S> {
    Standard(Standard<S>),
    ByteClass(ByteClass<S>),
    Premultiplied(Premultiplied<S>),
    PremultipliedByteClass(PremultipliedByteClass<S>),
}

impl<S: StateID> DFA<S> {
    fn repr(&self) -> &Repr<S> {
        match *self {
            DFA::Standard(ref dfa) => dfa.repr(),
            DFA::ByteClass(ref dfa) => dfa.repr(),
            DFA::Premultiplied(ref dfa) => dfa.repr(),
            DFA::PremultipliedByteClass(ref dfa) => dfa.repr(),
        }
    }

    pub fn match_kind(&self) -> &MatchKind {
        &self.repr().match_kind
    }

    pub fn heap_bytes(&self) -> usize {
        self.repr().heap_bytes
    }

    pub fn max_pattern_len(&self) -> usize {
        self.repr().max_pattern_len
    }

    pub fn pattern_count(&self) -> usize {
        self.repr().pattern_count
    }

    pub fn prefilter(&self) -> Option<&dyn Prefilter> {
        self.repr().prefilter.as_ref().map(|p| p.as_ref())
    }

    pub fn start_state(&self) -> S {
        self.repr().start_id
    }

    #[inline(always)]
    pub fn overlapping_find_at(
        &self,
        prestate: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
        state_id: &mut S,
        match_index: &mut usize,
    ) -> Option<Match> {
        match *self {
            DFA::Standard(ref dfa) => dfa.overlapping_find_at(
                prestate,
                haystack,
                at,
                state_id,
                match_index,
            ),
            DFA::ByteClass(ref dfa) => dfa.overlapping_find_at(
                prestate,
                haystack,
                at,
                state_id,
                match_index,
            ),
            DFA::Premultiplied(ref dfa) => dfa.overlapping_find_at(
                prestate,
                haystack,
                at,
                state_id,
                match_index,
            ),
            DFA::PremultipliedByteClass(ref dfa) => dfa.overlapping_find_at(
                prestate,
                haystack,
                at,
                state_id,
                match_index,
            ),
        }
    }

    #[inline(always)]
    pub fn earliest_find_at(
        &self,
        prestate: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
        state_id: &mut S,
    ) -> Option<Match> {
        match *self {
            DFA::Standard(ref dfa) => {
                dfa.earliest_find_at(prestate, haystack, at, state_id)
            }
            DFA::ByteClass(ref dfa) => {
                dfa.earliest_find_at(prestate, haystack, at, state_id)
            }
            DFA::Premultiplied(ref dfa) => {
                dfa.earliest_find_at(prestate, haystack, at, state_id)
            }
            DFA::PremultipliedByteClass(ref dfa) => {
                dfa.earliest_find_at(prestate, haystack, at, state_id)
            }
        }
    }

    #[inline(always)]
    pub fn find_at_no_state(
        &self,
        prestate: &mut PrefilterState,
        haystack: &[u8],
        at: usize,
    ) -> Option<Match> {
        match *self {
            DFA::Standard(ref dfa) => {
                dfa.find_at_no_state(prestate, haystack, at)
            }
            DFA::ByteClass(ref dfa) => {
                dfa.find_at_no_state(prestate, haystack, at)
            }
            DFA::Premultiplied(ref dfa) => {
                dfa.find_at_no_state(prestate, haystack, at)
            }
            DFA::PremultipliedByteClass(ref dfa) => {
                dfa.find_at_no_state(prestate, haystack, at)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Standard<S>(Repr<S>);

impl<S: StateID> Standard<S> {
    fn repr(&self) -> &Repr<S> {
        &self.0
    }
}

impl<S: StateID> Automaton for Standard<S> {
    type ID = S;

    fn match_kind(&self) -> &MatchKind {
        &self.repr().match_kind
    }

    fn anchored(&self) -> bool {
        self.repr().anchored
    }

    fn prefilter(&self) -> Option<&dyn Prefilter> {
        self.repr().prefilter.as_ref().map(|p| p.as_ref())
    }

    fn start_state(&self) -> S {
        self.repr().start_id
    }

    fn is_valid(&self, id: S) -> bool {
        id.to_usize() < self.repr().state_count
    }

    fn is_match_state(&self, id: S) -> bool {
        self.repr().is_match_state(id)
    }

    fn is_match_or_dead_state(&self, id: S) -> bool {
        self.repr().is_match_or_dead_state(id)
    }

    fn get_match(
        &self,
        id: S,
        match_index: usize,
        end: usize,
    ) -> Option<Match> {
        self.repr().get_match(id, match_index, end)
    }

    fn match_count(&self, id: S) -> usize {
        self.repr().match_count(id)
    }

    fn next_state(&self, current: S, input: u8) -> S {
        let o = current.to_usize() * 256 + input as usize;
        self.repr().trans[o]
    }
}

#[derive(Clone, Debug)]
pub struct ByteClass<S>(Repr<S>);

impl<S: StateID> ByteClass<S> {
    fn repr(&self) -> &Repr<S> {
        &self.0
    }
}

impl<S: StateID> Automaton for ByteClass<S> {
    type ID = S;

    fn match_kind(&self) -> &MatchKind {
        &self.repr().match_kind
    }

    fn anchored(&self) -> bool {
        self.repr().anchored
    }

    fn prefilter(&self) -> Option<&dyn Prefilter> {
        self.repr().prefilter.as_ref().map(|p| p.as_ref())
    }

    fn start_state(&self) -> S {
        self.repr().start_id
    }

    fn is_valid(&self, id: S) -> bool {
        id.to_usize() < self.repr().state_count
    }

    fn is_match_state(&self, id: S) -> bool {
        self.repr().is_match_state(id)
    }

    fn is_match_or_dead_state(&self, id: S) -> bool {
        self.repr().is_match_or_dead_state(id)
    }

    fn get_match(
        &self,
        id: S,
        match_index: usize,
        end: usize,
    ) -> Option<Match> {
        self.repr().get_match(id, match_index, end)
    }

    fn match_count(&self, id: S) -> usize {
        self.repr().match_count(id)
    }

    fn next_state(&self, current: S, input: u8) -> S {
        let alphabet_len = self.repr().byte_classes.alphabet_len();
        let input = self.repr().byte_classes.get(input);
        let o = current.to_usize() * alphabet_len + input as usize;
        self.repr().trans[o]
    }
}

#[derive(Clone, Debug)]
pub struct Premultiplied<S>(Repr<S>);

impl<S: StateID> Premultiplied<S> {
    fn repr(&self) -> &Repr<S> {
        &self.0
    }
}

impl<S: StateID> Automaton for Premultiplied<S> {
    type ID = S;

    fn match_kind(&self) -> &MatchKind {
        &self.repr().match_kind
    }

    fn anchored(&self) -> bool {
        self.repr().anchored
    }

    fn prefilter(&self) -> Option<&dyn Prefilter> {
        self.repr().prefilter.as_ref().map(|p| p.as_ref())
    }

    fn start_state(&self) -> S {
        self.repr().start_id
    }

    fn is_valid(&self, id: S) -> bool {
        (id.to_usize() / 256) < self.repr().state_count
    }

    fn is_match_state(&self, id: S) -> bool {
        self.repr().is_match_state(id)
    }

    fn is_match_or_dead_state(&self, id: S) -> bool {
        self.repr().is_match_or_dead_state(id)
    }

    fn get_match(
        &self,
        id: S,
        match_index: usize,
        end: usize,
    ) -> Option<Match> {
        if id > self.repr().max_match {
            return None;
        }
        self.repr()
            .matches
            .get(id.to_usize() / 256)
            .and_then(|m| m.get(match_index))
            .map(|&(id, len)| Match { pattern: id, len, end })
    }

    fn match_count(&self, id: S) -> usize {
        let o = id.to_usize() / 256;
        self.repr().matches[o].len()
    }

    fn next_state(&self, current: S, input: u8) -> S {
        let o = current.to_usize() + input as usize;
        self.repr().trans[o]
    }
}

#[derive(Clone, Debug)]
pub struct PremultipliedByteClass<S>(Repr<S>);

impl<S: StateID> PremultipliedByteClass<S> {
    fn repr(&self) -> &Repr<S> {
        &self.0
    }
}

impl<S: StateID> Automaton for PremultipliedByteClass<S> {
    type ID = S;

    fn match_kind(&self) -> &MatchKind {
        &self.repr().match_kind
    }

    fn anchored(&self) -> bool {
        self.repr().anchored
    }

    fn prefilter(&self) -> Option<&dyn Prefilter> {
        self.repr().prefilter.as_ref().map(|p| p.as_ref())
    }

    fn start_state(&self) -> S {
        self.repr().start_id
    }

    fn is_valid(&self, id: S) -> bool {
        (id.to_usize() / self.repr().alphabet_len()) < self.repr().state_count
    }

    fn is_match_state(&self, id: S) -> bool {
        self.repr().is_match_state(id)
    }

    fn is_match_or_dead_state(&self, id: S) -> bool {
        self.repr().is_match_or_dead_state(id)
    }

    fn get_match(
        &self,
        id: S,
        match_index: usize,
        end: usize,
    ) -> Option<Match> {
        if id > self.repr().max_match {
            return None;
        }
        self.repr()
            .matches
            .get(id.to_usize() / self.repr().alphabet_len())
            .and_then(|m| m.get(match_index))
            .map(|&(id, len)| Match { pattern: id, len, end })
    }

    fn match_count(&self, id: S) -> usize {
        let o = id.to_usize() / self.repr().alphabet_len();
        self.repr().matches[o].len()
    }

    fn next_state(&self, current: S, input: u8) -> S {
        let input = self.repr().byte_classes.get(input);
        let o = current.to_usize() + input as usize;
        self.repr().trans[o]
    }
}

#[derive(Clone, Debug)]
pub struct Repr<S> {
    match_kind: MatchKind,
    anchored: bool,
    premultiplied: bool,
    start_id: S,
    /// The length, in bytes, of the longest pattern in this automaton. This
    /// information is useful for keeping correct buffer sizes when searching
    /// on streams.
    max_pattern_len: usize,
    /// The total number of patterns added to this automaton. This includes
    /// patterns that may never match.
    pattern_count: usize,
    state_count: usize,
    max_match: S,
    /// The number of bytes of heap used by this NFA's transition table.
    heap_bytes: usize,
    /// A prefilter for quickly detecting candidate matchs, if pertinent.
    prefilter: Option<PrefilterObj>,
    byte_classes: ByteClasses,
    trans: Vec<S>,
    matches: Vec<Vec<(PatternID, PatternLength)>>,
}

impl<S: StateID> Repr<S> {
    /// Returns the total alphabet size for this DFA.
    ///
    /// If byte classes are enabled, then this corresponds to the number of
    /// equivalence classes. If they are disabled, then this is always 256.
    fn alphabet_len(&self) -> usize {
        self.byte_classes.alphabet_len()
    }

    /// Returns true only if the given state is a match state.
    fn is_match_state(&self, id: S) -> bool {
        id <= self.max_match && id > dead_id()
    }

    /// Returns true only if the given state is either a dead state or a match
    /// state.
    fn is_match_or_dead_state(&self, id: S) -> bool {
        id <= self.max_match
    }

    /// Get the ith match for the given state, where the end position of a
    /// match was found at `end`.
    ///
    /// # Panics
    ///
    /// The caller must ensure that the given state identifier is valid,
    /// otherwise this may panic. The `match_index` need not be valid. That is,
    /// if the given state has no matches then this returns `None`.
    fn get_match(
        &self,
        id: S,
        match_index: usize,
        end: usize,
    ) -> Option<Match> {
        if id > self.max_match {
            return None;
        }
        self.matches
            .get(id.to_usize())
            .and_then(|m| m.get(match_index))
            .map(|&(id, len)| Match { pattern: id, len, end })
    }

    /// Return the total number of matches for the given state.
    ///
    /// # Panics
    ///
    /// The caller must ensure that the given identifier is valid, or else
    /// this panics.
    fn match_count(&self, id: S) -> usize {
        self.matches[id.to_usize()].len()
    }

    /// Get the next state given `from` as the current state and `byte` as the
    /// current input byte.
    fn next_state(&self, from: S, byte: u8) -> S {
        let alphabet_len = self.alphabet_len();
        let byte = self.byte_classes.get(byte);
        self.trans[from.to_usize() * alphabet_len + byte as usize]
    }

    /// Set the `byte` transition for the `from` state to point to `to`.
    fn set_next_state(&mut self, from: S, byte: u8, to: S) {
        let alphabet_len = self.alphabet_len();
        let byte = self.byte_classes.get(byte);
        self.trans[from.to_usize() * alphabet_len + byte as usize] = to;
    }

    /// Swap the given states in place.
    fn swap_states(&mut self, id1: S, id2: S) {
        assert!(!self.premultiplied, "can't swap states in premultiplied DFA");

        let o1 = id1.to_usize() * self.alphabet_len();
        let o2 = id2.to_usize() * self.alphabet_len();
        for b in 0..self.alphabet_len() {
            self.trans.swap(o1 + b, o2 + b);
        }
        self.matches.swap(id1.to_usize(), id2.to_usize());
    }

    /// This routine shuffles all match states in this DFA to the beginning
    /// of the DFA such that every non-match state appears after every match
    /// state. (With one exception: the special fail and dead states remain as
    /// the first two states.)
    ///
    /// The purpose of doing this shuffling is to avoid an extra conditional
    /// in the search loop, and in particular, detecting whether a state is a
    /// match or not does not need to access any memory.
    ///
    /// This updates `self.max_match` to point to the last matching state as
    /// well as `self.start` if the starting state was moved.
    fn shuffle_match_states(&mut self) {
        assert!(
            !self.premultiplied,
            "cannot shuffle match states of premultiplied DFA"
        );

        if self.state_count <= 1 {
            return;
        }

        let mut first_non_match = self.start_id.to_usize();
        while first_non_match < self.state_count
            && self.matches[first_non_match].len() > 0
        {
            first_non_match += 1;
        }

        let mut swaps: Vec<S> = vec![fail_id(); self.state_count];
        let mut cur = self.state_count - 1;
        while cur > first_non_match {
            if self.matches[cur].len() > 0 {
                self.swap_states(
                    S::from_usize(cur),
                    S::from_usize(first_non_match),
                );
                swaps[cur] = S::from_usize(first_non_match);
                swaps[first_non_match] = S::from_usize(cur);

                first_non_match += 1;
                while first_non_match < cur
                    && self.matches[first_non_match].len() > 0
                {
                    first_non_match += 1;
                }
            }
            cur -= 1;
        }
        for id in (0..self.state_count).map(S::from_usize) {
            let alphabet_len = self.alphabet_len();
            let offset = id.to_usize() * alphabet_len;
            for next in &mut self.trans[offset..offset + alphabet_len] {
                if swaps[next.to_usize()] != fail_id() {
                    *next = swaps[next.to_usize()];
                }
            }
        }
        if swaps[self.start_id.to_usize()] != fail_id() {
            self.start_id = swaps[self.start_id.to_usize()];
        }
        self.max_match = S::from_usize(first_non_match - 1);
    }

    fn premultiply(&mut self) -> Result<()> {
        if self.premultiplied || self.state_count <= 1 {
            return Ok(());
        }

        let alpha_len = self.alphabet_len();
        premultiply_overflow_error(
            S::from_usize(self.state_count - 1),
            alpha_len,
        )?;

        for id in (2..self.state_count).map(S::from_usize) {
            let offset = id.to_usize() * alpha_len;
            for next in &mut self.trans[offset..offset + alpha_len] {
                if *next == dead_id() {
                    continue;
                }
                *next = S::from_usize(next.to_usize() * alpha_len);
            }
        }
        self.premultiplied = true;
        self.start_id = S::from_usize(self.start_id.to_usize() * alpha_len);
        self.max_match = S::from_usize(self.max_match.to_usize() * alpha_len);
        Ok(())
    }

    /// Computes the total amount of heap used by this NFA in bytes.
    fn calculate_size(&mut self) {
        let mut size = (self.trans.len() * size_of::<S>())
            + (self.matches.len()
                * size_of::<Vec<(PatternID, PatternLength)>>());
        for state_matches in &self.matches {
            size +=
                state_matches.len() * size_of::<(PatternID, PatternLength)>();
        }
        size += self.prefilter.as_ref().map_or(0, |p| p.as_ref().heap_bytes());
        self.heap_bytes = size;
    }
}

/// A builder for configuring the determinization of an NFA into a DFA.
#[derive(Clone, Debug)]
pub struct Builder {
    premultiply: bool,
    byte_classes: bool,
}

impl Builder {
    /// Create a new builder for a DFA.
    pub fn new() -> Builder {
        Builder { premultiply: true, byte_classes: true }
    }

    /// Build a DFA from the given NFA.
    ///
    /// This returns an error if the state identifiers exceed their
    /// representation size. This can only happen when state ids are
    /// premultiplied (which is enabled by default).
    pub fn build<S: StateID>(&self, nfa: &NFA<S>) -> Result<DFA<S>> {
        let byte_classes = if self.byte_classes {
            nfa.byte_classes().clone()
        } else {
            ByteClasses::singletons()
        };
        let alphabet_len = byte_classes.alphabet_len();
        let trans = vec![fail_id(); alphabet_len * nfa.state_len()];
        let matches = vec![vec![]; nfa.state_len()];
        let mut repr = Repr {
            match_kind: nfa.match_kind().clone(),
            anchored: nfa.anchored(),
            premultiplied: false,
            start_id: nfa.start_state(),
            max_pattern_len: nfa.max_pattern_len(),
            pattern_count: nfa.pattern_count(),
            state_count: nfa.state_len(),
            max_match: fail_id(),
            heap_bytes: 0,
            prefilter: nfa.prefilter_obj().map(|p| p.clone()),
            byte_classes: byte_classes.clone(),
            trans,
            matches,
        };
        for id in (0..nfa.state_len()).map(S::from_usize) {
            repr.matches[id.to_usize()].extend_from_slice(nfa.matches(id));

            let fail = nfa.failure_transition(id);
            nfa.iter_all_transitions(&byte_classes, id, |b, mut next| {
                if next == fail_id() {
                    next = nfa_next_state_memoized(nfa, &repr, id, fail, b);
                }
                repr.set_next_state(id, b, next);
            });
        }
        repr.shuffle_match_states();
        repr.calculate_size();
        if self.premultiply {
            repr.premultiply()?;
            if byte_classes.is_singleton() {
                Ok(DFA::Premultiplied(Premultiplied(repr)))
            } else {
                Ok(DFA::PremultipliedByteClass(PremultipliedByteClass(repr)))
            }
        } else {
            if byte_classes.is_singleton() {
                Ok(DFA::Standard(Standard(repr)))
            } else {
                Ok(DFA::ByteClass(ByteClass(repr)))
            }
        }
    }

    /// Whether to use byte classes or in the DFA.
    pub fn byte_classes(&mut self, yes: bool) -> &mut Builder {
        self.byte_classes = yes;
        self
    }

    /// Whether to premultiply state identifier in the DFA.
    pub fn premultiply(&mut self, yes: bool) -> &mut Builder {
        self.premultiply = yes;
        self
    }
}

/// This returns the next NFA transition (including resolving failure
/// transitions), except once it sees a state id less than the id of the DFA
/// state that is currently being populated, then we no longer need to follow
/// failure transitions and can instead query the pre-computed state id from
/// the DFA itself.
///
/// In general, this should only be called when a failure transition is seen.
fn nfa_next_state_memoized<S: StateID>(
    nfa: &NFA<S>,
    dfa: &Repr<S>,
    populating: S,
    mut current: S,
    input: u8,
) -> S {
    loop {
        if current < populating {
            return dfa.next_state(current, input);
        }
        let next = nfa.next_state(current, input);
        if next != fail_id() {
            return next;
        }
        current = nfa.failure_transition(current);
    }
}
