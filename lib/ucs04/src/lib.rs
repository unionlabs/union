//! This crate provides an implementation of the [UCS04 - Universal Chain ID][ucs04] standard.
//!
//! Well-known chain ids are available in [`well_known`].
//!
//! [ucs04]: https://docs.union.build/ucs/04/

#![warn(clippy::pedantic, missing_docs)]
#![no_std]

use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    string::{String, ToString},
};
use core::{fmt::Display, ptr, str::FromStr};

extern crate alloc;

include!(concat!(env!("OUT_DIR"), "/out.rs"));

/// A representation of a universal chain id.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UniversalChainId<'a> {
    family: Family,
    id: Cow<'a, Id>,
}

impl<'a> UniversalChainId<'a> {
    /// Construct a new [`UniversalChainId`].
    ///
    /// See [`UniversalChainId::new_owned`] if you have an owned [`Id`].
    #[must_use]
    pub const fn new(family: Family, id: &'a Id) -> Self {
        Self {
            family,
            id: Cow::Borrowed(id),
        }
    }

    /// Convert this [`UniversalChainId`] into an owned version.
    #[must_use]
    pub fn into_owned(self) -> UniversalChainId<'static> {
        UniversalChainId {
            family: self.family,
            id: Cow::Owned(self.id.into_owned()),
        }
    }

    /// Parse a universal chain id from the provided string slice.
    ///
    /// ```rust
    /// # use ucs04::{UniversalChainId, well_known};
    /// assert_eq!(
    ///     "ethereum.1".parse::<UniversalChainId>().unwrap(),
    ///     well_known::ETHEREUM_1,
    /// );
    /// ```
    ///
    /// # Errors
    ///
    /// See [`UniversalChainIdParseError`] for possible failure modes of this function.
    pub fn parse(s: &'a str) -> Result<UniversalChainId<'a>, UniversalChainIdParseError> {
        match s.split_once('.') {
            Some((family, chain_id)) => Ok(Self {
                family: family.parse()?,
                id: Cow::Borrowed(
                    Id::new(chain_id)
                        .ok_or_else(|| UniversalChainIdParseError::Id(chain_id.to_owned()))?,
                ),
            }),
            None => Err(UniversalChainIdParseError::Invalid(s.to_owned())),
        }
    }

    /// Get the [`Family`] portion of this universal chain id.
    #[must_use]
    pub fn family(&self) -> Family {
        self.family
    }

    /// Get the [`Id`] portion of this universal chain id.
    #[must_use]
    pub fn id(&self) -> &Id {
        &self.id
    }
}

impl UniversalChainId<'static> {
    /// Construct a new (owned) [`UniversalChainId`].
    ///
    /// See [`UniversalChainId::new`] if you have a ref [`Id`].
    #[must_use]
    pub const fn new_owned(family: Family, id: Box<Id>) -> Self {
        Self {
            family,
            id: Cow::Owned(id),
        }
    }
}

impl Display for UniversalChainId<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}", self.family, self.id)
    }
}

impl FromStr for UniversalChainId<'static> {
    type Err = UniversalChainIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // FromStr doesn't allow borrowing from the input
        UniversalChainId::parse(s).map(UniversalChainId::into_owned)
    }
}

impl<'a> TryFrom<&'a str> for UniversalChainId<'a> {
    type Error = UniversalChainIdParseError;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        UniversalChainId::parse(s)
    }
}

/// Error returned by [`UniversalChainId::from_str`].
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum UniversalChainIdParseError {
    /// Missing separator `.`.
    #[error("missing separator")]
    Invalid(String),
    /// Invalid `<chain_family_name>` portion.
    #[error(transparent)]
    Family(#[from] UnknownFamily),
    /// Invalid `<chain_id>` portion.
    #[error("invalid chain id {0:?}")]
    Id(String),
}

/// The `<chain_id>` portion of a universal chain id.
///
/// This is a thin wrapper around [`str`], and as such will most often be behind some form of indirection (usually either a ref (`&Id`) for borrowed or `Box<Id>` for owned).
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Id(str);

impl Display for Id {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Id {
    /// Construct a new borrowed [`Id`].
    ///
    /// This function will return None if the passed str is empty or contains the universal chain id separator character `.`.
    #[must_use]
    pub const fn new(s: &str) -> Option<&Self> {
        if !is_valid_id(s) {
            return None;
        }

        Some(unsafe { &*(ptr::from_ref::<str>(s) as *const Self) })
    }

    /// Construct a new owned [`Id`].
    ///
    /// This function will return None if the passed str is empty or contains the universal chain id separator character `.`.
    #[must_use]
    pub fn new_owned(s: String) -> Option<Box<Self>> {
        if !is_valid_id(&s) {
            return None;
        }

        Some(unsafe { Box::from_raw(Box::into_raw(s.into_boxed_str()) as *mut Self) })
    }
}

impl ToOwned for Id {
    type Owned = Box<Self>;

    fn to_owned(&self) -> Self::Owned {
        unsafe { Box::from_raw(Box::into_raw(self.0.to_string().into_boxed_str()) as *mut Self) }
    }
}

const fn is_valid_id(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut i = 0;
    let s = s.as_bytes();

    while i < s.len() {
        if s[i] == b'.' {
            return false;
        }

        i += 1;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        assert_eq!(
            "union.a".parse::<UniversalChainId>().unwrap(),
            UniversalChainId::new(Family::Union, Id::new("a").unwrap())
        );

        // same as above, but with TryFrom<&str> instead of FromStr

        assert_eq!(
            UniversalChainId::try_from("union.a").unwrap(),
            UniversalChainId::new(Family::Union, Id::new("a").unwrap())
        );
    }

    #[test]
    fn parse_invalid() {
        assert_eq!(
            "union.".parse::<UniversalChainId>().unwrap_err(),
            UniversalChainIdParseError::Id(String::new())
        );

        assert_eq!(
            "union..".parse::<UniversalChainId>().unwrap_err(),
            UniversalChainIdParseError::Id(".".to_owned())
        );

        assert_eq!(
            "union.a.".parse::<UniversalChainId>().unwrap_err(),
            UniversalChainIdParseError::Id("a.".to_owned())
        );

        assert_eq!(
            "union2.a".parse::<UniversalChainId>().unwrap_err(),
            UniversalChainIdParseError::Family(UnknownFamily("union2".to_owned()))
        );

        // same as above, but with TryFrom<&str> instead of FromStr

        assert_eq!(
            UniversalChainId::try_from("union.").unwrap_err(),
            UniversalChainIdParseError::Id(String::new())
        );

        assert_eq!(
            UniversalChainId::try_from("union..").unwrap_err(),
            UniversalChainIdParseError::Id(".".to_owned())
        );

        assert_eq!(
            UniversalChainId::try_from("union.a.").unwrap_err(),
            UniversalChainIdParseError::Id("a.".to_owned())
        );

        assert_eq!(
            UniversalChainId::try_from("union2.a").unwrap_err(),
            UniversalChainIdParseError::Family(UnknownFamily("union2".to_owned()))
        );
    }
}
