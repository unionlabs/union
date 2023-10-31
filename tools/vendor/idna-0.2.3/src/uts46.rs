// Copyright 2013-2014 The rust-url developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! [*Unicode IDNA Compatibility Processing*
//! (Unicode Technical Standard #46)](http://www.unicode.org/reports/tr46/)

use self::Mapping::*;
use crate::punycode;
use std::{error::Error as StdError, fmt};
use unicode_bidi::{bidi_class, BidiClass};
use unicode_normalization::char::is_combining_mark;
use unicode_normalization::{is_nfc, UnicodeNormalization};

include!("uts46_mapping_table.rs");

const PUNYCODE_PREFIX: &str = "xn--";

#[derive(Debug)]
struct StringTableSlice {
    // Store these as separate fields so the structure will have an
    // alignment of 1 and thus pack better into the Mapping enum, below.
    byte_start_lo: u8,
    byte_start_hi: u8,
    byte_len: u8,
}

fn decode_slice(slice: &StringTableSlice) -> &'static str {
    let lo = slice.byte_start_lo as usize;
    let hi = slice.byte_start_hi as usize;
    let start = (hi << 8) | lo;
    let len = slice.byte_len as usize;
    &STRING_TABLE[start..(start + len)]
}

#[repr(u8)]
#[derive(Debug)]
enum Mapping {
    Valid,
    Ignored,
    Mapped(StringTableSlice),
    Deviation(StringTableSlice),
    Disallowed,
    DisallowedStd3Valid,
    DisallowedStd3Mapped(StringTableSlice),
    DisallowedIdna2008,
}

fn find_char(codepoint: char) -> &'static Mapping {
    let idx = match TABLE.binary_search_by_key(&codepoint, |&val| val.0) {
        Ok(idx) => idx,
        Err(idx) => idx - 1,
    };

    const SINGLE_MARKER: u16 = 1 << 15;

    let (base, x) = TABLE[idx];
    let single = (x & SINGLE_MARKER) != 0;
    let offset = !SINGLE_MARKER & x;

    if single {
        &MAPPING_TABLE[offset as usize]
    } else {
        &MAPPING_TABLE[(offset + (codepoint as u16 - base as u16)) as usize]
    }
}

struct Mapper<'a> {
    chars: std::str::Chars<'a>,
    config: Config,
    errors: &'a mut Errors,
    slice: Option<std::str::Chars<'static>>,
}

impl<'a> Iterator for Mapper<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(s) = &mut self.slice {
                match s.next() {
                    Some(c) => return Some(c),
                    None => {
                        self.slice = None;
                    }
                }
            }

            let codepoint = self.chars.next()?;
            if let '.' | '-' | 'a'..='z' | '0'..='9' = codepoint {
                return Some(codepoint);
            }

            return Some(match *find_char(codepoint) {
                Mapping::Valid => codepoint,
                Mapping::Ignored => continue,
                Mapping::Mapped(ref slice) => {
                    self.slice = Some(decode_slice(slice).chars());
                    continue;
                }
                Mapping::Deviation(ref slice) => {
                    if self.config.transitional_processing {
                        self.slice = Some(decode_slice(slice).chars());
                        continue;
                    } else {
                        codepoint
                    }
                }
                Mapping::Disallowed => {
                    self.errors.disallowed_character = true;
                    codepoint
                }
                Mapping::DisallowedStd3Valid => {
                    if self.config.use_std3_ascii_rules {
                        self.errors.disallowed_by_std3_ascii_rules = true;
                    };
                    codepoint
                }
                Mapping::DisallowedStd3Mapped(ref slice) => {
                    if self.config.use_std3_ascii_rules {
                        self.errors.disallowed_mapped_in_std3 = true;
                    };
                    self.slice = Some(decode_slice(slice).chars());
                    continue;
                }
                Mapping::DisallowedIdna2008 => {
                    if self.config.use_idna_2008_rules {
                        self.errors.disallowed_in_idna_2008 = true;
                    }
                    codepoint
                }
            });
        }
    }
}

// http://tools.ietf.org/html/rfc5893#section-2
fn passes_bidi(label: &str, is_bidi_domain: bool) -> bool {
    // Rule 0: Bidi Rules apply to Bidi Domain Names: a name with at least one RTL label.  A label
    // is RTL if it contains at least one character of bidi class R, AL or AN.
    if !is_bidi_domain {
        return true;
    }

    let mut chars = label.chars();
    let first_char_class = match chars.next() {
        Some(c) => bidi_class(c),
        None => return true, // empty string
    };

    match first_char_class {
        // LTR label
        BidiClass::L => {
            // Rule 5
            while let Some(c) = chars.next() {
                if !matches!(
                    bidi_class(c),
                    BidiClass::L
                        | BidiClass::EN
                        | BidiClass::ES
                        | BidiClass::CS
                        | BidiClass::ET
                        | BidiClass::ON
                        | BidiClass::BN
                        | BidiClass::NSM
                ) {
                    return false;
                }
            }

            // Rule 6
            // must end in L or EN followed by 0 or more NSM
            let mut rev_chars = label.chars().rev();
            let mut last_non_nsm = rev_chars.next();
            loop {
                match last_non_nsm {
                    Some(c) if bidi_class(c) == BidiClass::NSM => {
                        last_non_nsm = rev_chars.next();
                        continue;
                    }
                    _ => {
                        break;
                    }
                }
            }
            match last_non_nsm {
                Some(c) if bidi_class(c) == BidiClass::L || bidi_class(c) == BidiClass::EN => {}
                Some(_) => {
                    return false;
                }
                _ => {}
            }
        }

        // RTL label
        BidiClass::R | BidiClass::AL => {
            let mut found_en = false;
            let mut found_an = false;

            // Rule 2
            for c in chars {
                let char_class = bidi_class(c);
                if char_class == BidiClass::EN {
                    found_en = true;
                } else if char_class == BidiClass::AN {
                    found_an = true;
                }

                if !matches!(
                    char_class,
                    BidiClass::R
                        | BidiClass::AL
                        | BidiClass::AN
                        | BidiClass::EN
                        | BidiClass::ES
                        | BidiClass::CS
                        | BidiClass::ET
                        | BidiClass::ON
                        | BidiClass::BN
                        | BidiClass::NSM
                ) {
                    return false;
                }
            }
            // Rule 3
            let mut rev_chars = label.chars().rev();
            let mut last = rev_chars.next();
            loop {
                // must end in L or EN followed by 0 or more NSM
                match last {
                    Some(c) if bidi_class(c) == BidiClass::NSM => {
                        last = rev_chars.next();
                        continue;
                    }
                    _ => {
                        break;
                    }
                }
            }
            match last {
                Some(c)
                    if matches!(
                        bidi_class(c),
                        BidiClass::R | BidiClass::AL | BidiClass::EN | BidiClass::AN
                    ) => {}
                _ => {
                    return false;
                }
            }

            // Rule 4
            if found_an && found_en {
                return false;
            }
        }

        // Rule 1: Should start with L or R/AL
        _ => {
            return false;
        }
    }

    true
}

/// Check the validity criteria for the given label
///
/// V1 (NFC) and V8 (Bidi) are checked inside `processing()` to prevent doing duplicate work.
///
/// http://www.unicode.org/reports/tr46/#Validity_Criteria
fn check_validity(label: &str, config: Config, errors: &mut Errors) {
    let first_char = label.chars().next();
    if first_char == None {
        // Empty string, pass
        return;
    }

    // V2: No U+002D HYPHEN-MINUS in both third and fourth positions.
    //
    // NOTE: Spec says that the label must not contain a HYPHEN-MINUS character in both the
    // third and fourth positions. But nobody follows this criteria. See the spec issue below:
    // https://github.com/whatwg/url/issues/53

    // V3: neither begin nor end with a U+002D HYPHEN-MINUS
    if config.check_hyphens && (label.starts_with('-') || label.ends_with('-')) {
        errors.check_hyphens = true;
        return;
    }

    // V4: not contain a U+002E FULL STOP
    //
    // Here, label can't contain '.' since the input is from .split('.')

    // V5: not begin with a GC=Mark
    if is_combining_mark(first_char.unwrap()) {
        errors.start_combining_mark = true;
        return;
    }

    // V6: Check against Mapping Table
    if label.chars().any(|c| match *find_char(c) {
        Mapping::Valid | Mapping::DisallowedIdna2008 => false,
        Mapping::Deviation(_) => config.transitional_processing,
        Mapping::DisallowedStd3Valid => config.use_std3_ascii_rules,
        _ => true,
    }) {
        errors.invalid_mapping = true;
    }

    // V7: ContextJ rules
    //
    // TODO: Implement rules and add *CheckJoiners* flag.

    // V8: Bidi rules are checked inside `processing()`
}

/// http://www.unicode.org/reports/tr46/#Processing
#[allow(clippy::manual_strip)] // introduced in 1.45, MSRV is 1.36
fn processing(
    domain: &str,
    config: Config,
    normalized: &mut String,
    output: &mut String,
) -> Errors {
    // Weed out the simple cases: only allow all lowercase ASCII characters and digits where none
    // of the labels start with PUNYCODE_PREFIX and labels don't start or end with hyphen.
    let (mut prev, mut simple, mut puny_prefix) = ('?', !domain.is_empty(), 0);
    for c in domain.chars() {
        if c == '.' {
            if prev == '-' {
                simple = false;
                break;
            }
            puny_prefix = 0;
            continue;
        } else if puny_prefix == 0 && c == '-' {
            simple = false;
            break;
        } else if puny_prefix < 5 {
            if c == ['x', 'n', '-', '-'][puny_prefix] {
                puny_prefix += 1;
                if puny_prefix == 4 {
                    simple = false;
                    break;
                }
            } else {
                puny_prefix = 5;
            }
        }
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() {
            simple = false;
            break;
        }
        prev = c;
    }

    if simple {
        output.push_str(domain);
        return Errors::default();
    }

    normalized.clear();
    let mut errors = Errors::default();
    let offset = output.len();

    let iter = Mapper {
        chars: domain.chars(),
        config,
        errors: &mut errors,
        slice: None,
    };

    normalized.extend(iter.nfc());

    let mut decoder = punycode::Decoder::default();
    let non_transitional = config.transitional_processing(false);
    let (mut first, mut has_bidi_labels) = (true, false);
    for label in normalized.split('.') {
        if !first {
            output.push('.');
        }
        first = false;
        if label.starts_with(PUNYCODE_PREFIX) {
            match decoder.decode(&label[PUNYCODE_PREFIX.len()..]) {
                Ok(decode) => {
                    let start = output.len();
                    output.extend(decode);
                    let decoded_label = &output[start..];

                    if !has_bidi_labels {
                        has_bidi_labels |= is_bidi_domain(decoded_label);
                    }

                    if !errors.is_err() {
                        if !is_nfc(&decoded_label) {
                            errors.nfc = true;
                        } else {
                            check_validity(decoded_label, non_transitional, &mut errors);
                        }
                    }
                }
                Err(()) => {
                    has_bidi_labels = true;
                    errors.punycode = true;
                }
            }
        } else {
            if !has_bidi_labels {
                has_bidi_labels |= is_bidi_domain(label);
            }

            // `normalized` is already `NFC` so we can skip that check
            check_validity(label, config, &mut errors);
            output.push_str(label)
        }
    }

    for label in output[offset..].split('.') {
        // V8: Bidi rules
        //
        // TODO: Add *CheckBidi* flag
        if !passes_bidi(label, has_bidi_labels) {
            errors.check_bidi = true;
            break;
        }
    }

    errors
}

#[derive(Default)]
pub struct Idna {
    config: Config,
    normalized: String,
    output: String,
}

impl Idna {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            normalized: String::new(),
            output: String::new(),
        }
    }

    /// http://www.unicode.org/reports/tr46/#ToASCII
    #[allow(clippy::wrong_self_convention)]
    pub fn to_ascii<'a>(&'a mut self, domain: &str, out: &mut String) -> Result<(), Errors> {
        let mut errors = processing(domain, self.config, &mut self.normalized, &mut self.output);

        let mut first = true;
        for label in self.output.split('.') {
            if !first {
                out.push('.');
            }
            first = false;

            if label.is_ascii() {
                out.push_str(label);
            } else {
                let offset = out.len();
                out.push_str(PUNYCODE_PREFIX);
                if let Err(()) = punycode::encode_into(label.chars(), out) {
                    errors.punycode = true;
                    out.truncate(offset);
                }
            }
        }

        if self.config.verify_dns_length {
            let domain = if out.ends_with('.') {
                &out[..out.len() - 1]
            } else {
                &*out
            };
            if domain.is_empty() || domain.split('.').any(|label| label.is_empty()) {
                errors.too_short_for_dns = true;
            }
            if domain.len() > 253 || domain.split('.').any(|label| label.len() > 63) {
                errors.too_long_for_dns = true;
            }
        }

        errors.into()
    }

    /// http://www.unicode.org/reports/tr46/#ToUnicode
    #[allow(clippy::wrong_self_convention)]
    pub fn to_unicode<'a>(&'a mut self, domain: &str, out: &mut String) -> Result<(), Errors> {
        processing(domain, self.config, &mut self.normalized, out).into()
    }
}

#[derive(Clone, Copy)]
pub struct Config {
    use_std3_ascii_rules: bool,
    transitional_processing: bool,
    verify_dns_length: bool,
    check_hyphens: bool,
    use_idna_2008_rules: bool,
}

/// The defaults are that of https://url.spec.whatwg.org/#idna
impl Default for Config {
    fn default() -> Self {
        Config {
            use_std3_ascii_rules: false,
            transitional_processing: false,
            check_hyphens: false,
            // check_bidi: true,
            // check_joiners: true,

            // Only use for to_ascii, not to_unicode
            verify_dns_length: false,
            use_idna_2008_rules: false,
        }
    }
}

impl Config {
    #[inline]
    pub fn use_std3_ascii_rules(mut self, value: bool) -> Self {
        self.use_std3_ascii_rules = value;
        self
    }

    #[inline]
    pub fn transitional_processing(mut self, value: bool) -> Self {
        self.transitional_processing = value;
        self
    }

    #[inline]
    pub fn verify_dns_length(mut self, value: bool) -> Self {
        self.verify_dns_length = value;
        self
    }

    #[inline]
    pub fn check_hyphens(mut self, value: bool) -> Self {
        self.check_hyphens = value;
        self
    }

    #[inline]
    pub fn use_idna_2008_rules(mut self, value: bool) -> Self {
        self.use_idna_2008_rules = value;
        self
    }

    /// http://www.unicode.org/reports/tr46/#ToASCII
    pub fn to_ascii(self, domain: &str) -> Result<String, Errors> {
        let mut result = String::new();
        let mut codec = Idna::new(self);
        codec.to_ascii(domain, &mut result).map(|()| result)
    }

    /// http://www.unicode.org/reports/tr46/#ToUnicode
    pub fn to_unicode(self, domain: &str) -> (String, Result<(), Errors>) {
        let mut codec = Idna::new(self);
        let mut out = String::with_capacity(domain.len());
        let result = codec.to_unicode(domain, &mut out);
        (out, result)
    }
}

fn is_bidi_domain(s: &str) -> bool {
    for c in s.chars() {
        if c.is_ascii_graphic() {
            continue;
        }
        match bidi_class(c) {
            BidiClass::R | BidiClass::AL | BidiClass::AN => return true,
            _ => {}
        }
    }
    false
}

/// Errors recorded during UTS #46 processing.
///
/// This is opaque for now, indicating what types of errors have been encountered at least once.
/// More details may be exposed in the future.
#[derive(Default)]
pub struct Errors {
    punycode: bool,
    check_hyphens: bool,
    check_bidi: bool,
    start_combining_mark: bool,
    invalid_mapping: bool,
    nfc: bool,
    disallowed_by_std3_ascii_rules: bool,
    disallowed_mapped_in_std3: bool,
    disallowed_character: bool,
    too_long_for_dns: bool,
    too_short_for_dns: bool,
    disallowed_in_idna_2008: bool,
}

impl Errors {
    fn is_err(&self) -> bool {
        let Errors {
            punycode,
            check_hyphens,
            check_bidi,
            start_combining_mark,
            invalid_mapping,
            nfc,
            disallowed_by_std3_ascii_rules,
            disallowed_mapped_in_std3,
            disallowed_character,
            too_long_for_dns,
            too_short_for_dns,
            disallowed_in_idna_2008,
        } = *self;
        punycode
            || check_hyphens
            || check_bidi
            || start_combining_mark
            || invalid_mapping
            || nfc
            || disallowed_by_std3_ascii_rules
            || disallowed_mapped_in_std3
            || disallowed_character
            || too_long_for_dns
            || too_short_for_dns
            || disallowed_in_idna_2008
    }
}

impl fmt::Debug for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Errors {
            punycode,
            check_hyphens,
            check_bidi,
            start_combining_mark,
            invalid_mapping,
            nfc,
            disallowed_by_std3_ascii_rules,
            disallowed_mapped_in_std3,
            disallowed_character,
            too_long_for_dns,
            too_short_for_dns,
            disallowed_in_idna_2008,
        } = *self;

        let fields = [
            ("punycode", punycode),
            ("check_hyphens", check_hyphens),
            ("check_bidi", check_bidi),
            ("start_combining_mark", start_combining_mark),
            ("invalid_mapping", invalid_mapping),
            ("nfc", nfc),
            (
                "disallowed_by_std3_ascii_rules",
                disallowed_by_std3_ascii_rules,
            ),
            ("disallowed_mapped_in_std3", disallowed_mapped_in_std3),
            ("disallowed_character", disallowed_character),
            ("too_long_for_dns", too_long_for_dns),
            ("too_short_for_dns", too_short_for_dns),
            ("disallowed_in_idna_2008", disallowed_in_idna_2008),
        ];

        let mut empty = true;
        f.write_str("Errors { ")?;
        for (name, val) in &fields {
            if *val {
                if !empty {
                    f.write_str(", ")?;
                }
                f.write_str(*name)?;
                empty = false;
            }
        }

        if !empty {
            f.write_str(" }")
        } else {
            f.write_str("}")
        }
    }
}

impl From<Errors> for Result<(), Errors> {
    fn from(e: Errors) -> Result<(), Errors> {
        if !e.is_err() {
            Ok(())
        } else {
            Err(e)
        }
    }
}

impl StdError for Errors {}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::{find_char, Mapping};

    #[test]
    fn mapping_fast_path() {
        assert_matches!(find_char('-'), &Mapping::Valid);
        assert_matches!(find_char('.'), &Mapping::Valid);
        for c in &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'] {
            assert_matches!(find_char(*c), &Mapping::Valid);
        }
        for c in &[
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
            'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        ] {
            assert_matches!(find_char(*c), &Mapping::Valid);
        }
    }
}
