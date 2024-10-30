use alloc::borrow::Cow;
use core::{
    fmt::{self, Debug, Display},
    num::ParseIntError,
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::errors::{ExpectedLength, InvalidLength};

const DELIMITER: char = '-';

pub const CONNECTION_ID_PREFIX: &str = "connection";
pub const CHANNEL_ID_PREFIX: &str = "channel";

#[derive(macros::Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
// #[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
#[debug("ClientId({}-{})", self.prefix, self.id)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
// #[cfg_attr(
//     feature = "valuable",
//     derive(valuable::Valuable),
//     valuable(transparent)
// )]
pub struct ClientId {
    prefix: Cow<'static, str>,
    id: u32,
}

impl ClientId {
    #[must_use]
    pub fn new(prefix: impl Into<Cow<'static, str>>, id: u32) -> Self {
        Self {
            prefix: prefix.into(),
            id,
        }
    }

    #[must_use]
    pub fn new_static(prefix: &'static str, id: u32) -> Self {
        Self {
            prefix: Cow::Borrowed(prefix),
            id,
        }
    }

    #[must_use]
    pub const fn id(&self) -> u32 {
        self.id
    }
}

impl From<ClientId> for String {
    fn from(value: ClientId) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for ClientId {
    type Error = ParsePrefixedIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl FromStr for ClientId {
    type Err = ParsePrefixedIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // NOTE: rsplit bc prefixes can contain `-`
        let Some((prefix, id)) = s.rsplit_once(DELIMITER) else {
            return Err(ParsePrefixedIdError::MissingPrefix);
        };

        id.parse()
            .map(|id| Self {
                prefix: prefix.to_owned().into(),
                id,
            })
            .map_err(ParsePrefixedIdError::ParseIntError)
    }
}

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.prefix, self.id)
    }
}

#[derive(macros::Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
// #[cfg_attr(feature = "serde", serde(transparent))]
#[debug("ConnectionId({})", self.0)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
// #[cfg_attr(
//     feature = "valuable",
//     derive(valuable::Valuable),
//     valuable(transparent)
// )]
pub struct ConnectionId(#[doc(hidden)] u32);

impl ConnectionId {
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn from_str_prefixed(s: &str) -> Result<Self, ParsePrefixedIdError> {
        let Some((prefix, id)) = s.split_once(DELIMITER) else {
            return Err(ParsePrefixedIdError::MissingPrefix);
        };

        if prefix != CONNECTION_ID_PREFIX {
            return Err(ParsePrefixedIdError::UnexpectedPrefix {
                expected: CONNECTION_ID_PREFIX,
                found: s.to_owned(),
            });
        }

        id.parse()
            .map(Self)
            .map_err(ParsePrefixedIdError::ParseIntError)
    }

    /// Formats this [`ConnectionId`] with it's alternate [`Display`] formatting, prefixing the ID with [`CONNECTION_ID_PREFIX`].
    ///
    /// ```rust
    /// # use unionlabs::id::ConnectionId;
    /// assert_eq!(
    ///     ConnectionId::new(123).to_string_prefixed(),
    ///     "connection-123"
    /// );
    /// ```
    #[must_use]
    pub fn to_string_prefixed(&self) -> String {
        format!("{self:#}")
    }

    #[must_use]
    pub const fn id(&self) -> u32 {
        self.0
    }
}

impl Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{CONNECTION_ID_PREFIX}{DELIMITER}{}", self.id())
        } else {
            write!(f, "{}", self.id())
        }
    }
}

#[derive(macros::Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
// #[cfg_attr(feature = "serde", serde(transparent))]
#[debug("ChannelId({})", self.0)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
// #[cfg_attr(
//     feature = "valuable",
//     derive(valuable::Valuable),
//     valuable(transparent)
// )]
pub struct ChannelId(#[doc(hidden)] u32);

impl ChannelId {
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[must_use]
    pub const fn id(&self) -> u32 {
        self.0
    }

    pub fn from_str_prefixed(s: &str) -> Result<Self, ParsePrefixedIdError> {
        let Some((prefix, id)) = s.split_once(DELIMITER) else {
            return Err(ParsePrefixedIdError::MissingPrefix);
        };

        if prefix != CHANNEL_ID_PREFIX {
            return Err(ParsePrefixedIdError::UnexpectedPrefix {
                expected: CHANNEL_ID_PREFIX,
                found: s.to_owned(),
            });
        }

        id.parse()
            .map(Self)
            .map_err(ParsePrefixedIdError::ParseIntError)
    }

    /// Formats this [`ChannelId`] with it's alternate [`Display`] formatting, prefixing the ID with [`CHANNEL_ID_PREFIX`].
    ///
    /// ```rust
    /// # use unionlabs::id::ChannelId;
    /// assert_eq!(
    ///     ChannelId::new(123).to_string_prefixed(),
    ///     "channel-123"
    /// );
    /// ```
    #[must_use]
    pub fn to_string_prefixed(&self) -> String {
        format!("{self:#}")
    }
}

impl Display for ChannelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{CHANNEL_ID_PREFIX}{DELIMITER}{}", self.id())
        } else {
            write!(f, "{}", self.id())
        }
    }
}

#[derive(macros::Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
// #[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
#[serde(try_from = "String", into = "String")]
#[debug("PortId({})", self.0)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
pub struct PortId(#[doc(hidden)] Cow<'static, str>);

// #[cfg(feature = "valuable")]
// impl valuable::Valuable for PortId {
//     fn as_value(&self) -> valuable::Value<'_> {
//         valuable::Value::String(&self.0)
//     }

//     fn visit(&self, visit: &mut dyn valuable::Visit) {
//         visit.visit_value(self.as_value());
//     }
// }

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ParsePrefixedIdError {
    #[error("missing prefix")]
    MissingPrefix,
    #[error("expected prefix `{expected}` but found `{found}`")]
    UnexpectedPrefix {
        expected: &'static str,
        found: String,
    },
    #[error("error parsing integer portion of id")]
    ParseIntError(#[from] ParseIntError),
}

impl PortId {
    pub const MIN_LEN: usize = 2;
    pub const MAX_LEN: usize = 128;

    pub fn new(s: impl Into<Cow<'static, str>>) -> Result<Self, Ics24IdParseError> {
        let s: Cow<'_, str> = s.into();

        validate_id(&s, Self::MIN_LEN, Self::MAX_LEN)?;

        Ok(Self(s))
    }

    pub const fn new_static(s: &'static str) -> Result<Self, Ics24IdParseError> {
        if let Err(e) = validate_id(s, Self::MIN_LEN, Self::MAX_LEN) {
            return Err(e);
        }

        Ok(Self(Cow::Borrowed(s)))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

const fn validate_id(s: &str, min_len: usize, max_len: usize) -> Result<(), Ics24IdParseError> {
    let len = s.len();

    if len < min_len || len > max_len {
        return Err(Ics24IdParseError::InvalidLength(InvalidLength {
            expected: ExpectedLength::Between(min_len, max_len),
            found: len,
        }));
    }

    let mut i = len - 1;
    let bz = s.as_bytes();

    // https://github.com/cosmos/ibc/tree/main/spec/core/ics-024-host-requirements#paths-identifiers-separators
    loop {
        if i == 0 {
            break;
        }

        let c = bz[i];
        match c {
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
            | b'>' => {}
            _ => {
                return Err(Ics24IdParseError::InvalidCharacter(
                    InvalidIcs024IdentifierCharacter(c),
                ))
            }
        }

        i -= 1;
    }

    Ok(())
}

impl From<PortId> for String {
    fn from(value: PortId) -> Self {
        value.0.into()
    }
}

impl TryFrom<String> for PortId {
    type Error = Ics24IdParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl FromStr for PortId {
    type Err = Ics24IdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_owned())
    }
}

impl fmt::Display for PortId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Ics24IdParseError {
    #[error(transparent)]
    InvalidCharacter(InvalidIcs024IdentifierCharacter),
    #[error(transparent)]
    InvalidLength(InvalidLength),
}

// #[cfg(feature = "schemars")]
// static_assertions::assert_impl_all!(ClientId: schemars::JsonSchema);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("invalid ics-024 identifier character: `{0:x}`")]
pub struct InvalidIcs024IdentifierCharacter(u8);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_id() {
        assert_eq!(
            PortId::new(""),
            Err(Ics24IdParseError::InvalidLength(InvalidLength {
                expected: ExpectedLength::Between(PortId::MIN_LEN, PortId::MAX_LEN),
                found: 0
            }))
        );
        assert_eq!(
            PortId::new("a"),
            Err(Ics24IdParseError::InvalidLength(InvalidLength {
                expected: ExpectedLength::Between(PortId::MIN_LEN, PortId::MAX_LEN),
                found: 1
            }))
        );
        assert_eq!(PortId::new("aa").as_ref().map(PortId::as_str), Ok("aa"));
        assert_eq!(
            PortId::new("a".repeat(PortId::MAX_LEN))
                .as_ref()
                .map(PortId::as_str),
            Ok(&*"a".repeat(PortId::MAX_LEN))
        );

        // assert_eq!(ics024("".into()), Ok("".into()));
        // assert_eq!(ics024("valid".into()), Ok("valid".into()));
        // assert_eq!(
        //     ics024(
        //         "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890._+-#[]<>".into()
        //     ),
        //     Ok("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890._+-#[]<>".into())
        // );
        // assert_eq!(
        //     ics024("/".into()),
        //     Err(InvalidIcs024IdentifierCharacter(b'/'))
        // );
    }
}
