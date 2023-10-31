use ::regex::{Error, Regex, RegexSet};

use core::{convert::TryFrom, ops::Deref, str::FromStr};
use std::borrow::Cow;

/// Contains either a regular expression or a set of them or a reference to one.
///
/// See [Arg::validator_regex(][crate::Arg::validator_regex] to set this on an argument.
#[derive(Debug, Clone)]
pub enum RegexRef<'a> {
    /// Used if the underlying is a regex set
    RegexSet(Cow<'a, RegexSet>),
    /// Used if the underlying is a regex
    Regex(Cow<'a, Regex>),
}

impl<'a> RegexRef<'a> {
    pub(crate) fn is_match(&self, text: &str) -> bool {
        match self {
            Self::Regex(r) => r.deref().is_match(text),
            Self::RegexSet(r) => r.deref().is_match(text),
        }
    }
}

impl<'a> From<&'a Regex> for RegexRef<'a> {
    fn from(r: &'a Regex) -> Self {
        Self::Regex(Cow::Borrowed(r))
    }
}

impl<'a> From<Regex> for RegexRef<'a> {
    fn from(r: Regex) -> Self {
        Self::Regex(Cow::Owned(r))
    }
}

impl<'a> From<&'a RegexSet> for RegexRef<'a> {
    fn from(r: &'a RegexSet) -> Self {
        Self::RegexSet(Cow::Borrowed(r))
    }
}

impl<'a> From<RegexSet> for RegexRef<'a> {
    fn from(r: RegexSet) -> Self {
        Self::RegexSet(Cow::Owned(r))
    }
}

impl<'a> TryFrom<&'a str> for RegexRef<'a> {
    type Error = <Self as FromStr>::Err;

    fn try_from(r: &'a str) -> Result<Self, Self::Error> {
        Self::from_str(r)
    }
}

impl<'a> FromStr for RegexRef<'a> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Regex::from_str(s).map(|v| Self::Regex(Cow::Owned(v)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::TryInto;

    #[test]
    fn test_try_from_with_valid_string() {
        let t: Result<RegexRef, _> = "^Hello, World$".try_into();
        assert!(t.is_ok())
    }

    #[test]
    fn test_try_from_with_invalid_string() {
        let t: Result<RegexRef, _> = "^Hello, World)$".try_into();
        assert!(t.is_err());
    }

    #[test]
    fn from_str() {
        let t: Result<RegexRef, _> = RegexRef::from_str("^Hello, World");
        assert!(t.is_ok());
    }
}
