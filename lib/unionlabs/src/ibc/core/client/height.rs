use std::{
    fmt::{Debug, Display},
    num::ParseIntError,
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};
use tree_hash::TreeHash;

#[derive(Clone, Copy, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
pub struct Height {
    // REVIEW: Why default?
    #[serde(default)]
    pub revision_number: u64,
    pub revision_height: u64,
}

impl Debug for Height {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Height({self})")
    }
}

impl Height {
    #[must_use]
    pub fn increment(mut self) -> Self {
        self.revision_height += 1;
        self
    }

    #[must_use]
    pub fn new(revision_number: u64, revision_height: u64) -> Self {
        Height {
            revision_number,
            revision_height,
        }
    }
}

impl FromStr for Height {
    type Err = HeightFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_once('-')
            .ok_or(HeightFromStrError::Invalid)
            .and_then(|(n, h)| {
                Ok(Self {
                    revision_number: n.parse().map_err(HeightFromStrError::ParseIntError)?,
                    revision_height: h.parse().map_err(HeightFromStrError::ParseIntError)?,
                })
            })
    }
}

#[derive(Debug, PartialEq)]
pub enum HeightFromStrError {
    ParseIntError(ParseIntError),
    Invalid,
}

impl Display for HeightFromStrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeightFromStrError::ParseIntError(e) => write!(f, "invalid height string: {e}"),
            HeightFromStrError::Invalid => write!(f, "invalid height string"),
        }
    }
}

impl std::error::Error for HeightFromStrError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HeightFromStrError::ParseIntError(e) => Some(e),
            HeightFromStrError::Invalid => None,
        }
    }
}

impl From<protos::ibc::core::client::v1::Height> for Height {
    fn from(proto: protos::ibc::core::client::v1::Height) -> Self {
        Self {
            revision_number: proto.revision_number,
            revision_height: proto.revision_height,
        }
    }
}

impl From<Height> for protos::ibc::core::client::v1::Height {
    fn from(value: Height) -> Self {
        Self {
            revision_number: value.revision_number,
            revision_height: value.revision_height,
        }
    }
}

// REVIEW(benluelo): Ordering for heights with different revision numbers?
impl PartialOrd for Height {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(match self.revision_number.cmp(&other.revision_number) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            std::cmp::Ordering::Equal => self.revision_height.cmp(&other.revision_height),
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
        })
    }
}

impl std::fmt::Display for Height {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.revision_number, self.revision_height)
    }
}

#[cfg(feature = "ethabi")]
impl From<Height> for contracts::shared_types::IbcCoreClientV1HeightData {
    fn from(value: Height) -> Self {
        Self {
            revision_number: value.revision_number,
            revision_height: value.revision_height,
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<contracts::shared_types::IbcCoreClientV1HeightData> for Height {
    fn from(value: contracts::shared_types::IbcCoreClientV1HeightData) -> Self {
        Self {
            revision_number: value.revision_number,
            revision_height: value.revision_height,
        }
    }
}

pub trait IsHeight:
    FromStr<Err = HeightFromStrError>
    + Display
    + Debug
    + Copy
    + PartialEq
    + Serialize
    + for<'de> Deserialize<'de>
    + From<Height>
    + Into<Height>
    + Send
    + Sync
    + 'static
{
    fn into_height(self) -> Height {
        Into::<Height>::into(self)
    }

    #[must_use]
    fn increment(self) -> Self {
        Height {
            revision_number: self.revision_number(),
            revision_height: self.revision_height() + 1,
        }
        .into()
    }

    #[must_use]
    fn decrement(self) -> Self {
        Height {
            revision_number: self.revision_number(),
            revision_height: self.revision_height() - 1,
        }
        .into()
    }

    fn revision_number(&self) -> u64 {
        self.into_height().revision_number
    }
    fn revision_height(&self) -> u64 {
        self.into_height().revision_height
    }
}

impl<T> IsHeight for T where
    T: FromStr<Err = HeightFromStrError>
        + Display
        + Debug
        + Copy
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>
        + From<Height>
        + Into<Height>
        + Send
        + Sync
        + 'static
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_impls_is_height() {
        fn f(_: impl IsHeight) {}

        f(Height {
            revision_number: 0,
            revision_height: 0,
        });
    }

    #[test]
    fn from_str() {
        assert_eq!(
            Height::from_str("1-1"),
            Ok(Height {
                revision_number: 1,
                revision_height: 1,
            })
        );

        assert_eq!(
            Height::from_str("0-0"),
            Ok(Height {
                revision_number: 0,
                revision_height: 0,
            })
        );

        assert_eq!(
            Height::from_str(&format!("{0}-{0}", u64::MAX)),
            Ok(Height {
                revision_number: u64::MAX,
                revision_height: u64::MAX,
            })
        );

        // will try to parse "2-0" as a u64
        assert!(matches!(
            Height::from_str("4-2-0"),
            Err(HeightFromStrError::ParseIntError(_))
        ));

        assert_eq!(
            Height::from_str("gibberish"),
            Err(HeightFromStrError::Invalid)
        );
    }
}
