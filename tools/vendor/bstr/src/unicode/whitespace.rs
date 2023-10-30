use regex_automata::DFA;

use crate::unicode::fsm::{
    whitespace_anchored_fwd::WHITESPACE_ANCHORED_FWD,
    whitespace_anchored_rev::WHITESPACE_ANCHORED_REV,
};

/// Return the first position of a non-whitespace character.
pub fn whitespace_len_fwd(slice: &[u8]) -> usize {
    WHITESPACE_ANCHORED_FWD.find(slice).unwrap_or(0)
}

/// Return the last position of a non-whitespace character.
pub fn whitespace_len_rev(slice: &[u8]) -> usize {
    WHITESPACE_ANCHORED_REV.rfind(slice).unwrap_or(slice.len())
}
