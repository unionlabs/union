//! An implementation of the "stringprep" algorithm defined in [RFC 3454][].
//!
//! [RFC 3454]: https://tools.ietf.org/html/rfc3454
#![doc(html_root_url="https://docs.rs/stringprep/0.1.2")]
#![warn(missing_docs)]
extern crate unicode_bidi;
extern crate unicode_normalization;

use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::error;
use std::fmt;
use unicode_normalization::UnicodeNormalization;

mod rfc3454;
pub mod tables;

/// Describes why a string failed stringprep normalization.
#[derive(Debug)]
enum ErrorCause {
    /// Contains stringprep prohibited characters.
    ProhibitedCharacter(char),
    /// Violates stringprep rules for bidirectional text.
    ProhibitedBidirectionalText,
}

/// An error performing the stringprep algorithm.
#[derive(Debug)]
pub struct Error(ErrorCause);

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            ErrorCause::ProhibitedCharacter(c) => write!(fmt, "prohibited character `{}`", c),
            ErrorCause::ProhibitedBidirectionalText => write!(fmt, "prohibited bidirectional text"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "error performing stringprep algorithm"
    }
}

/// Prepares a string with the SASLprep profile of the stringprep algorithm.
///
/// SASLprep is defined in [RFC 4013][].
///
/// [RFC 4013]: https://tools.ietf.org/html/rfc4013
pub fn saslprep<'a>(s: &'a str) -> Result<Cow<'a, str>, Error> {
    // fast path for ascii text
    if s.chars()
           .all(|c| c.is_ascii() && !tables::ascii_control_character(c)) {
        return Ok(Cow::Borrowed(s));
    }

    // 2.1 Mapping
    let mapped = s.chars()
        .map(|c| if tables::non_ascii_space_character(c) {
                 ' '
             } else {
                 c
             })
        .filter(|&c| !tables::commonly_mapped_to_nothing(c));

    // 2.2 Normalization
    let normalized = mapped.nfkc().collect::<String>();

    // 2.3 Prohibited Output
    let prohibited = normalized
        .chars()
        .find(|&c| {
            tables::non_ascii_space_character(c) /* C.1.2 */ ||
            tables::ascii_control_character(c) /* C.2.1 */ ||
            tables::non_ascii_control_character(c) /* C.2.2 */ ||
            tables::private_use(c) /* C.3 */ ||
            tables::non_character_code_point(c) /* C.4 */ ||
            tables::surrogate_code(c) /* C.5 */ ||
            tables::inappropriate_for_plain_text(c) /* C.6 */ ||
            tables::inappropriate_for_canonical_representation(c) /* C.7 */ ||
            tables::change_display_properties_or_deprecated(c) /* C.8 */ ||
            tables::tagging_character(c) /* C.9 */
        });
    if let Some(c) = prohibited {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }

    // 2.4. Bidirectional Characters
    if is_prohibited_bidirectional_text(&normalized) {
        return Err(Error(ErrorCause::ProhibitedBidirectionalText));
    }

    // 2.5 Unassigned Code Points
    let unassigned = normalized
        .chars()
        .find(|&c| tables::unassigned_code_point(c));
    if let Some(c) = unassigned {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }

    Ok(Cow::Owned(normalized))
}

// RFC3454, 6. Bidirectional Characters
fn is_prohibited_bidirectional_text(s: &str) -> bool {
    if s.contains(tables::bidi_r_or_al) {
        // 2) If a string contains any RandALCat character, the string
        // MUST NOT contain any LCat character.
        if s.contains(tables::bidi_l) {
            return true;
        }

        // 3) If a string contains any RandALCat character, a RandALCat
        // character MUST be the first character of the string, and a
        // RandALCat character MUST be the last character of the string.
        if !tables::bidi_r_or_al(s.chars().next().unwrap()) ||
           !tables::bidi_r_or_al(s.chars().next_back().unwrap()) {
            return true;
        }
    }

    false
}

/// Prepares a string with the Nameprep profile of the stringprep algorithm.
///
/// Nameprep is defined in [RFC 3491][].
///
/// [RFC 3491]: https://tools.ietf.org/html/rfc3491
pub fn nameprep<'a>(s: &'a str) -> Result<Cow<'a, str>, Error> {
    // 3. Mapping
    let mapped = s.chars()
        .filter(|&c| !tables::commonly_mapped_to_nothing(c))
        .flat_map(tables::case_fold_for_nfkc);

    // 4. Normalization
    let normalized = mapped.nfkc().collect::<String>();

    // 5. Prohibited Output
    let prohibited = normalized
        .chars()
        .find(|&c| {
            tables::non_ascii_space_character(c) /* C.1.2 */ ||
            tables::non_ascii_control_character(c) /* C.2.2 */ ||
            tables::private_use(c) /* C.3 */ ||
            tables::non_character_code_point(c) /* C.4 */ ||
            tables::surrogate_code(c) /* C.5 */ ||
            tables::inappropriate_for_plain_text(c) /* C.6 */ ||
            tables::inappropriate_for_canonical_representation(c) /* C.7 */ ||
            tables::change_display_properties_or_deprecated(c) /* C.9 */ ||
            tables::tagging_character(c) /* C.9 */
        });
    if let Some(c) = prohibited {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }

    // 6. Bidirectional Characters
    if is_prohibited_bidirectional_text(&normalized) {
        return Err(Error(ErrorCause::ProhibitedBidirectionalText));
    }

    // 7 Unassigned Code Points
    let unassigned = normalized
        .chars()
        .find(|&c| tables::unassigned_code_point(c));
    if let Some(c) = unassigned {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }

    Ok(Cow::Owned(normalized))
}

/// Prepares a string with the Nodeprep profile of the stringprep algorithm.
///
/// Nameprep is defined in [RFC 3920, Appendix A][].
///
/// [RFC 3920, Appendix A]: https://tools.ietf.org/html/rfc3920#appendix-A
pub fn nodeprep<'a>(s: &'a str) -> Result<Cow<'a, str>, Error> {
    // A.3. Mapping
    let mapped = s.chars()
        .filter(|&c| !tables::commonly_mapped_to_nothing(c))
        .flat_map(tables::case_fold_for_nfkc);

    // A.4. Normalization
    let normalized = mapped.nfkc().collect::<String>();

    // A.5. Prohibited Output
    let prohibited = normalized
        .chars()
        .find(|&c| {
            tables::ascii_space_character(c) /* C.1.1 */ ||
            tables::non_ascii_space_character(c) /* C.1.2 */ ||
            tables::ascii_control_character(c) /* C.2.1 */ ||
            tables::non_ascii_control_character(c) /* C.2.2 */ ||
            tables::private_use(c) /* C.3 */ ||
            tables::non_character_code_point(c) /* C.4 */ ||
            tables::surrogate_code(c) /* C.5 */ ||
            tables::inappropriate_for_plain_text(c) /* C.6 */ ||
            tables::inappropriate_for_canonical_representation(c) /* C.7 */ ||
            tables::change_display_properties_or_deprecated(c) /* C.9 */ ||
            tables::tagging_character(c) /* C.9 */ ||
            prohibited_node_character(c)
        });
    if let Some(c) = prohibited {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }

    // A.6. Bidirectional Characters
    if is_prohibited_bidirectional_text(&normalized) {
        return Err(Error(ErrorCause::ProhibitedBidirectionalText));
    }

    let unassigned = normalized
        .chars()
        .find(|&c| tables::unassigned_code_point(c));
    if let Some(c) = unassigned {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }

    Ok(Cow::Owned(normalized))
}

// Additional characters not allowed in JID nodes, by RFC3920.
fn prohibited_node_character(c: char) -> bool {
    match c {
        '"' | '&' | '\'' | '/' | ':' | '<' | '>' | '@' => true,
        _ => false
    }
}

/// Prepares a string with the Resourceprep profile of the stringprep algorithm.
///
/// Nameprep is defined in [RFC 3920, Appendix B][].
///
/// [RFC 3920, Appendix B]: https://tools.ietf.org/html/rfc3920#appendix-B
pub fn resourceprep<'a>(s: &'a str) -> Result<Cow<'a, str>, Error> {
    // B.3. Mapping
    let mapped = s.chars()
        .filter(|&c| !tables::commonly_mapped_to_nothing(c))
        .collect::<String>();

    // B.4. Normalization
    let normalized = mapped.nfkc().collect::<String>();

    // B.5. Prohibited Output
    let prohibited = normalized
        .chars()
        .find(|&c| {
            tables::non_ascii_space_character(c) /* C.1.2 */ ||
            tables::ascii_control_character(c) /* C.2.1 */ ||
            tables::non_ascii_control_character(c) /* C.2.2 */ ||
            tables::private_use(c) /* C.3 */ ||
            tables::non_character_code_point(c) /* C.4 */ ||
            tables::surrogate_code(c) /* C.5 */ ||
            tables::inappropriate_for_plain_text(c) /* C.6 */ ||
            tables::inappropriate_for_canonical_representation(c) /* C.7 */ ||
            tables::change_display_properties_or_deprecated(c) /* C.9 */ ||
            tables::tagging_character(c) /* C.9 */
        });
    if let Some(c) = prohibited {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }

    // B.6. Bidirectional Characters
    if is_prohibited_bidirectional_text(&normalized) {
        return Err(Error(ErrorCause::ProhibitedBidirectionalText));
    }

    let unassigned = normalized
        .chars()
        .find(|&c| tables::unassigned_code_point(c));
    if let Some(c) = unassigned {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }

    Ok(Cow::Owned(normalized))
}

#[cfg(test)]
mod test {
    use super::*;

	fn assert_prohibited_character<T>(result: Result<T, Error>) {
		match result {
			Err(Error(ErrorCause::ProhibitedCharacter(_))) => (),
			_ => assert!(false)
		}
	}

    // RFC4013, 3. Examples
    #[test]
    fn saslprep_examples() {
		assert_prohibited_character(saslprep("\u{0007}"));
    }

	#[test]
	fn nodeprep_examples() {
        assert_prohibited_character(nodeprep(" "));
        assert_prohibited_character(nodeprep("\u{00a0}"));
        assert_prohibited_character(nodeprep("foo@bar"));
	}

    #[test]
    fn resourceprep_examples() {
        assert_eq!("foo@bar", resourceprep("foo@bar").unwrap());
    }
}
