use std::{fmt, str::FromStr};

use unionlabs::bounded::{BoundedI64, BoundedIntError, BoundedIntParseError};

pub mod abci;
pub mod crypto;
pub mod p2p;
pub mod types;
pub mod version;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct CometbftHeight(BoundedI64<0, { i64::MAX }>);

impl CometbftHeight {
    pub const fn inner(self) -> i64 {
        self.0.inner()
    }
}

impl ::serde::Serialize for CometbftHeight {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            serializer.serialize_i64(self.0.inner())
        }
    }
}

impl<'de> ::serde::Deserialize<'de> for CometbftHeight {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            String::deserialize(deserializer)
                .and_then(|s| s.parse().map(Self).map_err(::serde::de::Error::custom))
        } else {
            BoundedI64::deserialize(deserializer).map(Self)
        }
    }
}

impl fmt::Display for CometbftHeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl FromStr for CometbftHeight {
    type Err = BoundedIntParseError<i64>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl TryFrom<i64> for CometbftHeight {
    type Error = BoundedIntError<i64, i64>;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

impl From<CometbftHeight> for i64 {
    fn from(value: CometbftHeight) -> i64 {
        value.0.inner()
    }
}

impl TryFrom<u64> for CometbftHeight {
    type Error = BoundedIntError<i64, u64>;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        BoundedI64::new(value).map(Self)
    }
}

pub mod utils {
    use unionlabs::{
        errors::{ExpectedLength, InvalidLength},
        primitives::{FixedBytesError, H256, encoding::HexUnprefixed},
    };

    pub fn maybe_empty_h256(value: &[u8]) -> Result<Option<H256<HexUnprefixed>>, InvalidLength> {
        Ok(if value.is_empty() {
            None
        } else {
            Some(
                value
                    .try_into()
                    .map_err(|err: FixedBytesError| InvalidLength {
                        expected: ExpectedLength::Either(0, 32),
                        found: err.found_len,
                    })?,
            )
        })
    }
}

pub mod serde {
    pub mod maybe_empty_h256 {
        use serde::{Deserialize, Deserializer, Serializer, de};
        use unionlabs::primitives::{H256, encoding::HexUnprefixed};

        pub fn serialize<S>(
            data: &Option<H256<HexUnprefixed>>,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match data {
                Some(data) => serializer.collect_str(&data),
                None => serializer.collect_str(""),
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<H256<HexUnprefixed>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;

            if s.is_empty() {
                Ok(None)
            } else {
                s.parse().map_err(de::Error::custom).map(Some)
            }
        }
    }
}

pub mod code {
    use core::{fmt, num::NonZeroU32};

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    #[serde(from = "u32", into = "u32")]
    pub enum Code {
        Ok,
        Err(NonZeroU32),
    }

    impl Code {
        #[must_use]
        pub const fn raw(&self) -> u32 {
            match self {
                Code::Ok => 0,
                Code::Err(err) => err.get(),
            }
        }

        #[must_use]
        pub const fn new(value: u32) -> Self {
            match NonZeroU32::new(value) {
                Some(err) => Self::Err(err),
                None => Self::Ok,
            }
        }

        /// Returns `true` if the code is [`Ok`].
        ///
        /// [`Ok`]: Code::Ok
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            matches!(self, Self::Ok)
        }

        /// Returns `true` if the code is [`Err`].
        ///
        /// [`Err`]: Code::Err
        #[must_use]
        pub const fn is_err(&self) -> bool {
            matches!(self, Self::Err(..))
        }

        /// Returns `true` if the code is the specified [`Err`].
        ///
        /// [`Err`]: Code::Err
        #[must_use]
        pub const fn is_err_code(&self, code: NonZeroU32) -> bool {
            matches!(self, Self::Err(e) if e.get() == code.get())
        }

        pub fn as_err(&self) -> Option<&NonZeroU32> {
            if let Self::Err(v) = self {
                Some(v)
            } else {
                None
            }
        }
    }

    impl fmt::Display for Code {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Display::fmt(&self.raw(), f)
        }
    }

    impl From<u32> for Code {
        fn from(value: u32) -> Self {
            Self::new(value)
        }
    }

    impl From<Code> for u32 {
        fn from(value: Code) -> Self {
            value.raw()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn code_serde() {
            assert_eq!(serde_json::from_str::<Code>("0").unwrap(), Code::Ok);
            assert_eq!(
                serde_json::from_str::<Code>("1").unwrap(),
                Code::Err(NonZeroU32::new(1).unwrap())
            );
            assert_eq!(
                serde_json::from_str::<Code>("2").unwrap(),
                Code::Err(NonZeroU32::new(2).unwrap())
            );
        }
    }
}
