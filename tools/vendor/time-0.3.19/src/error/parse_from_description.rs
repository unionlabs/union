//! Error parsing an input into a [`Parsed`](crate::parsing::Parsed) struct

use core::fmt;

use crate::error;

/// An error that occurred while parsing the input into a [`Parsed`](crate::parsing::Parsed) struct.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseFromDescription {
    /// A string literal was not what was expected.
    #[non_exhaustive]
    InvalidLiteral,
    /// A dynamic component was not valid.
    InvalidComponent(&'static str),
}

impl fmt::Display for ParseFromDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLiteral => f.write_str("a character literal was not valid"),
            Self::InvalidComponent(name) => {
                write!(f, "the '{name}' component could not be parsed")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseFromDescription {}

impl From<ParseFromDescription> for crate::Error {
    fn from(original: ParseFromDescription) -> Self {
        Self::ParseFromDescription(original)
    }
}

impl TryFrom<crate::Error> for ParseFromDescription {
    type Error = error::DifferentVariant;

    fn try_from(err: crate::Error) -> Result<Self, Self::Error> {
        match err {
            crate::Error::ParseFromDescription(err) => Ok(err),
            _ => Err(error::DifferentVariant),
        }
    }
}
