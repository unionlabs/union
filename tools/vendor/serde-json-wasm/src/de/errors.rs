use serde::de;
use std::{error, fmt};

/// Deserialization result
pub type Result<T> = core::result::Result<T, Error>;

/// This type represents all possible errors that can occur when deserializing JSON data
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// Control character (U+0000 to U+001F) found in string. Those must always be escaped.
    ControlCharacterInString,

    /// EOF while parsing a list.
    EofWhileParsingList,

    /// EOF while parsing an object.
    EofWhileParsingObject,

    /// EOF while parsing a string.
    EofWhileParsingString,

    /// EOF while parsing a JSON value.
    EofWhileParsingValue,

    /// Expected this character to be a `':'`.
    ExpectedColon,

    /// Expected a high surrogate (D800–DBFF) but found something else
    ExpectedHighSurrogate,

    /// Expected this character to be either a `','` or a `']'`.
    ExpectedListCommaOrEnd,

    /// Expected a low surrogate (DC00–DFFF) but found something else
    ExpectedLowSurrogate,

    /// Expected this character to be either a `','` or a `'}'`.
    ExpectedObjectCommaOrEnd,

    /// Expected to parse either a `true`, `false`, or a `null`.
    ExpectedSomeIdent,

    /// Expected this character to start a JSON value.
    ExpectedSomeValue,

    /// Invalid escape sequence
    InvalidEscape,

    /// Invalid number.
    InvalidNumber,

    /// Invalid type
    InvalidType,

    /// Invalid unicode code point.
    InvalidUnicodeCodePoint,

    /// Object key is not a string.
    KeyMustBeAString,

    /// Found a lone surrogate, which can exist in JSON but cannot be encoded to UTF-8
    LoneSurrogateFound,

    /// JSON has non-whitespace trailing characters after the value.
    TrailingCharacters,

    /// JSON has a comma after the last value in an array or map.
    TrailingComma,

    /// Custom error message from serde
    Custom(String),
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "(use display)"
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::Custom(msg.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::ControlCharacterInString => "Control character found in string.",
                Error::EofWhileParsingList => "EOF while parsing a list.",
                Error::EofWhileParsingObject => "EOF while parsing an object.",
                Error::EofWhileParsingString => "EOF while parsing a string.",
                Error::EofWhileParsingValue => "EOF while parsing a JSON value.",
                Error::ExpectedColon => "Expected this character to be a `':'`.",
                Error::ExpectedHighSurrogate => "Expected a high surrogate (D800–DBFF).",
                Error::ExpectedListCommaOrEnd => {
                    "Expected this character to be either a `','` or\
                     a \
                     `']'`."
                }
                Error::ExpectedLowSurrogate => "Expected a low surrogate (DC00–DFFF).",
                Error::ExpectedObjectCommaOrEnd => {
                    "Expected this character to be either a `','` \
                     or a \
                     `'}'`."
                }
                Error::ExpectedSomeIdent => {
                    "Expected to parse either a `true`, `false`, or a \
                     `null`."
                }
                Error::ExpectedSomeValue => "Expected this character to start a JSON value.",
                Error::InvalidEscape => "Invalid escape sequence.",
                Error::InvalidNumber => "Invalid number.",
                Error::InvalidType => "Invalid type",
                Error::InvalidUnicodeCodePoint => "Invalid unicode code point.",
                Error::KeyMustBeAString => "Object key is not a string.",
                Error::LoneSurrogateFound => "Found a lone surrogate, which can exist in JSON but cannot be encoded to UTF-8.",
                Error::TrailingCharacters => {
                    "JSON has non-whitespace trailing characters after \
                     the \
                     value."
                }
                Error::TrailingComma => "JSON has a comma after the last value in an array or map.",
                Error::Custom(msg) => msg,
            }
        )
    }
}
