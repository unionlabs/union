use core::fmt::Debug;

use crate::{
    errors::{ExpectedLength, InvalidLength},
    validated::{Validate, Validated},
};

pub type PortIdValidator = (Bounded<2, 128>, Ics024IdentifierCharacters);
pub type PortId = Validated<String, PortIdValidator>;

pub type ClientIdValidator = (Bounded<9, 64>, Ics024IdentifierCharacters);
pub type ClientId = Validated<String, ClientIdValidator>;

pub type ConnectionIdValidator = (Bounded<10, 64>, Ics024IdentifierCharacters);
pub type ConnectionId = Validated<String, ConnectionIdValidator>;

pub type ChannelIdValidator = (Bounded<8, 64>, Ics024IdentifierCharacters);
pub type ChannelId = Validated<String, ChannelIdValidator>;

// https://github.com/cosmos/ibc/tree/main/spec/core/ics-024-host-requirements#paths-identifiers-separators
pub struct Ics024IdentifierCharacters;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("invalid ics-024 identifier character: `{0}`")]
pub struct InvalidIcs024IdentifierCharacter(char);

impl<T: Into<String> + From<String>> Validate<T> for Ics024IdentifierCharacters {
    type Error = InvalidIcs024IdentifierCharacter;

    fn validate(t: T) -> Result<T, Self::Error> {
        let s = t.into();

        for c in s.chars() {
            match c {
                'a'..='z'
                | 'A'..='Z'
                | '0'..='9'
                | '.'
                | '_'
                | '+'
                | '-'
                | '#'
                | '['
                | ']'
                | '<'
                | '>' => {}
                _ => return Err(InvalidIcs024IdentifierCharacter(c)),
            }
        }

        Ok(T::from(s))
    }
}

#[cfg(feature = "arbitrary")]
impl<T: Into<String> + From<String>> crate::validated::ValidateExt<T>
    for Ics024IdentifierCharacters
{
    fn restrict(t: T, u: &mut arbitrary::Unstructured) -> arbitrary::Result<T> {
        const VALID_CHARS: [u8; 71] =
            *b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890._+-#[]<>";

        let s: String = t.into();

        Ok(T::from(
            s.bytes()
                .map(|c| match c {
                    b'a'..=b'z'
                    | b'A'..=b'Z'
                    | b'0'..=b'9'
                    | b'.'
                    | b'_'
                    | b'+'
                    | b'-'
                    | b'#'
                    | b'['
                    | b']'
                    | b'<'
                    | b'>' => Ok(c as char),
                    _ => Ok(*u.choose(&VALID_CHARS)? as char),
                })
                .collect::<Result<String, _>>()?,
        ))
    }
}

pub struct Bounded<const MIN: usize, const MAX: usize>;

impl<T: Into<String> + From<String>, const MIN: usize, const MAX: usize> Validate<T>
    for Bounded<MIN, MAX>
{
    type Error = InvalidLength;

    fn validate(t: T) -> Result<T, Self::Error> {
        const { assert!(MIN <= MAX) };

        let s: String = t.into();

        let len = s.len();

        if (MIN..=MAX).contains(&len) {
            Ok(T::from(s))
        } else {
            Err(InvalidLength {
                expected: ExpectedLength::Between(MIN, MAX),
                found: len,
            })
        }
    }
}

#[cfg(feature = "arbitrary")]
impl<T: Into<String> + From<String>, const MIN: usize, const MAX: usize>
    crate::validated::ValidateExt<T> for Bounded<MIN, MAX>
{
    fn restrict(t: T, u: &mut arbitrary::Unstructured) -> arbitrary::Result<T> {
        const { assert!(MIN <= MAX) };

        let s: String = t.into();

        let len = s.len();

        if len < MIN {
            // can't add more data, since that might invalidate other validations run before this
            Err(arbitrary::Error::IncorrectFormat)
        } else if len > MAX {
            fn floor_char_boundary(string: &str, index: usize) -> &str {
                if string.is_char_boundary(index) {
                    &string[..index]
                } else {
                    floor_char_boundary(string, index - 1)
                }
            }

            Ok(T::from(
                floor_char_boundary(&s, u.int_in_range(MIN..=MAX)?).to_owned(),
            ))
        } else {
            Ok(T::from(s))
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;

    use super::*;
    use crate::validated::ValidateT;

    fn ics024(
        s: Cow<'_, str>,
    ) -> Result<Cow<'_, str>, <Ics024IdentifierCharacters as Validate<Cow<'_, str>>>::Error> {
        s.validate::<Ics024IdentifierCharacters>()
            .map(Validated::value)
    }

    #[test]
    fn ics024_identifier_characters() {
        assert_eq!(ics024("".into()), Ok("".into()));
        assert_eq!(ics024("valid".into()), Ok("valid".into()));
        assert_eq!(
            ics024(
                "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890._+-#[]<>".into()
            ),
            Ok("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890._+-#[]<>".into())
        );
        assert_eq!(
            ics024("/".into()),
            Err(InvalidIcs024IdentifierCharacter('/'))
        );
    }

    fn bound<const MIN: usize, const MAX: usize>(
        s: Cow<'_, str>,
    ) -> Result<Cow<'_, str>, <Bounded<MIN, MAX> as Validate<Cow<'_, str>>>::Error> {
        s.validate::<Bounded<MIN, MAX>>().map(Validated::value)
    }

    #[test]
    fn bounded() {
        assert_eq!(bound::<0, 1>("".into()), Ok("".into()));
        assert_eq!(bound::<0, 1>("a".into()), Ok("a".into()));
        assert_eq!(
            bound::<0, 1>("aa".into()),
            Err(InvalidLength {
                expected: ExpectedLength::Between(0, 1),
                found: 2
            })
        );
    }
}
