// Copyright 2012-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Functions for computing canonical and compatible decompositions for Unicode characters.
use crate::lookups::{
    canonical_fully_decomposed, cjk_compat_variants_fully_decomposed,
    compatibility_fully_decomposed, composition_table,
};

use core::{char, ops::FnMut};

/// Compute canonical Unicode decomposition for character.
/// See [Unicode Standard Annex #15](http://www.unicode.org/reports/tr15/)
/// for more information.
#[inline]
pub fn decompose_canonical<F>(c: char, emit_char: F)
where
    F: FnMut(char),
{
    decompose(c, canonical_fully_decomposed, emit_char)
}

/// Compute canonical or compatible Unicode decomposition for character.
/// See [Unicode Standard Annex #15](http://www.unicode.org/reports/tr15/)
/// for more information.
#[inline]
pub fn decompose_compatible<F: FnMut(char)>(c: char, emit_char: F) {
    let decompose_char =
        |c| compatibility_fully_decomposed(c).or_else(|| canonical_fully_decomposed(c));
    decompose(c, decompose_char, emit_char)
}

/// Compute standard-variation decomposition for character.
///
/// [Standardized Variation Sequences] are used instead of the standard canonical
/// decompositions, notably for CJK codepoints with singleton canonical decompositions,
/// to avoid losing information. See the
/// [Unicode Variation Sequence FAQ](http://unicode.org/faq/vs.html) and the
/// "Other Enhancements" section of the
/// [Unicode 6.3 Release Summary](https://www.unicode.org/versions/Unicode6.3.0/#Summary)
/// for more information.
#[inline]
pub fn decompose_cjk_compat_variants<F>(c: char, mut emit_char: F)
where
    F: FnMut(char),
{
    // 7-bit ASCII never decomposes
    if c <= '\x7f' {
        emit_char(c);
        return;
    }

    // Don't perform decomposition for Hangul

    if let Some(decomposed) = cjk_compat_variants_fully_decomposed(c) {
        for &d in decomposed {
            emit_char(d);
        }
        return;
    }

    // Finally bottom out.
    emit_char(c);
}

#[inline]
fn decompose<D, F>(c: char, decompose_char: D, mut emit_char: F)
where
    D: Fn(char) -> Option<&'static [char]>,
    F: FnMut(char),
{
    // 7-bit ASCII never decomposes
    if c <= '\x7f' {
        emit_char(c);
        return;
    }

    // Perform decomposition for Hangul
    if is_hangul_syllable(c) {
        decompose_hangul(c, emit_char);
        return;
    }

    if let Some(decomposed) = decompose_char(c) {
        for &d in decomposed {
            emit_char(d);
        }
        return;
    }

    // Finally bottom out.
    emit_char(c);
}

/// Compose two characters into a single character, if possible.
/// See [Unicode Standard Annex #15](http://www.unicode.org/reports/tr15/)
/// for more information.
pub fn compose(a: char, b: char) -> Option<char> {
    compose_hangul(a, b).or_else(|| composition_table(a, b))
}

// Constants from Unicode 9.0.0 Section 3.12 Conjoining Jamo Behavior
// http://www.unicode.org/versions/Unicode9.0.0/ch03.pdf#M9.32468.Heading.310.Combining.Jamo.Behavior
const S_BASE: u32 = 0xAC00;
const L_BASE: u32 = 0x1100;
const V_BASE: u32 = 0x1161;
const T_BASE: u32 = 0x11A7;
const L_COUNT: u32 = 19;
const V_COUNT: u32 = 21;
const T_COUNT: u32 = 28;
const N_COUNT: u32 = V_COUNT * T_COUNT;
const S_COUNT: u32 = L_COUNT * N_COUNT;

const S_LAST: u32 = S_BASE + S_COUNT - 1;
const L_LAST: u32 = L_BASE + L_COUNT - 1;
const V_LAST: u32 = V_BASE + V_COUNT - 1;
const T_LAST: u32 = T_BASE + T_COUNT - 1;

// Composition only occurs for `TPart`s in `U+11A8 ... U+11C2`,
// i.e. `T_BASE + 1 ... T_LAST`.
const T_FIRST: u32 = T_BASE + 1;

pub(crate) fn is_hangul_syllable(c: char) -> bool {
    (c as u32) >= S_BASE && (c as u32) < (S_BASE + S_COUNT)
}

// Decompose a precomposed Hangul syllable
#[allow(unsafe_code)]
#[inline(always)]
fn decompose_hangul<F>(s: char, mut emit_char: F)
where
    F: FnMut(char),
{
    let s_index = s as u32 - S_BASE;
    let l_index = s_index / N_COUNT;
    unsafe {
        emit_char(char::from_u32_unchecked(L_BASE + l_index));

        let v_index = (s_index % N_COUNT) / T_COUNT;
        emit_char(char::from_u32_unchecked(V_BASE + v_index));

        let t_index = s_index % T_COUNT;
        if t_index > 0 {
            emit_char(char::from_u32_unchecked(T_BASE + t_index));
        }
    }
}

#[inline]
pub(crate) fn hangul_decomposition_length(s: char) -> usize {
    let si = s as u32 - S_BASE;
    let ti = si % T_COUNT;
    if ti > 0 {
        3
    } else {
        2
    }
}

// Compose a pair of Hangul Jamo
#[allow(unsafe_code)]
#[inline(always)]
#[allow(ellipsis_inclusive_range_patterns)]
fn compose_hangul(a: char, b: char) -> Option<char> {
    let (a, b) = (a as u32, b as u32);
    match (a, b) {
        // Compose a leading consonant and a vowel together into an LV_Syllable
        (L_BASE...L_LAST, V_BASE...V_LAST) => {
            let l_index = a - L_BASE;
            let v_index = b - V_BASE;
            let lv_index = l_index * N_COUNT + v_index * T_COUNT;
            let s = S_BASE + lv_index;
            Some(unsafe { char::from_u32_unchecked(s) })
        }
        // Compose an LV_Syllable and a trailing consonant into an LVT_Syllable
        (S_BASE...S_LAST, T_FIRST...T_LAST) if (a - S_BASE) % T_COUNT == 0 => {
            Some(unsafe { char::from_u32_unchecked(a + (b - T_BASE)) })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::compose_hangul;

    // Regression test from a bugfix where we were composing an LV_Syllable with
    // T_BASE directly. (We should only compose an LV_Syllable with a character
    // in the range `T_BASE + 1 ... T_LAST`.)
    #[test]
    fn test_hangul_composition() {
        assert_eq!(compose_hangul('\u{c8e0}', '\u{11a7}'), None);
    }
}
