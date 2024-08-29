use core::str::FromStr;

use hex::FromHex;
use serde::{de::Error as _, Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum AccountAddressParseError {
    #[error("AccountAddress data should be exactly 32 bytes long")]
    IncorrectNumberOfBytes,

    #[error("Hex characters are invalid: {0}")]
    InvalidHexChars(String),

    #[error("Hex string is too short, must be 1 to 64 chars long, excluding the leading 0x")]
    TooShort,

    #[error("Hex string is too long, must be 1 to 64 chars long, excluding the leading 0x")]
    TooLong,

    #[error("Hex string must start with a leading 0x")]
    LeadingZeroXRequired,

    #[error(
        "The given hex string is not a special address, it must be represented as 0x + 64 chars"
    )]
    LongFormRequiredUnlessSpecial,

    #[error("The given hex string is a special address not in LONG form, it must be 0x0 to 0xf without padding zeroes")]
    InvalidPaddingZeroes,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccountAddress(pub [u8; Self::LENGTH]);

impl AccountAddress {
    pub const LENGTH: usize = 32;

    #[must_use]
    pub const fn new(address: [u8; Self::LENGTH]) -> Self {
        Self(address)
    }

    /// NOTE: Where possible use `from_str_strict` or `from_str` instead.
    pub fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, AccountAddressParseError> {
        <[u8; Self::LENGTH]>::from_hex(hex)
            .map_err(|e| AccountAddressParseError::InvalidHexChars(format!("{e:#}")))
            .map(Self)
    }

    /// NOTE: Where possible use `from_str_strict` or `from_str` instead.
    pub fn from_hex_literal(literal: &str) -> Result<Self, AccountAddressParseError> {
        if !literal.starts_with("0x") {
            return Err(AccountAddressParseError::LeadingZeroXRequired);
        }

        let hex_len = literal.len() - 2;

        // If the string is too short, pad it
        if hex_len < Self::LENGTH * 2 {
            let mut hex_str = String::with_capacity(Self::LENGTH * 2);
            for _ in 0..Self::LENGTH * 2 - hex_len {
                hex_str.push('0');
            }
            hex_str.push_str(&literal[2..]);
            AccountAddress::from_hex(hex_str)
        } else {
            AccountAddress::from_hex(&literal[2..])
        }
    }
}

impl Serialize for AccountAddress {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        if serializer.is_human_readable() {
            hex::encode(self.0).serialize(serializer)
        } else {
            // See comment in deserialize.
            serializer.serialize_newtype_struct("AccountAddress", &self.0)
        }
    }
}

impl<'de> Deserialize<'de> for AccountAddress {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;
            AccountAddress::from_str(&s).map_err(D::Error::custom)
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            #[derive(::serde::Deserialize)]
            #[serde(rename = "AccountAddress")]
            struct Value([u8; AccountAddress::LENGTH]);

            let value = Value::deserialize(deserializer)?;
            Ok(AccountAddress::new(value.0))
        }
    }
}

impl FromStr for AccountAddress {
    type Err = AccountAddressParseError;

    /// NOTE: This function has relaxed parsing behavior. For strict behavior, please use
    /// the `from_str_strict` function. Where possible use `from_str_strict` rather than
    /// this function.
    ///
    /// Create an instance of `AccountAddress` by parsing a hex string representation.
    ///
    /// This function allows all formats defined by AIP-40. In short this means the
    /// following formats are accepted:
    ///
    /// - LONG, with or without leading 0x
    /// - SHORT, with or without leading 0x
    ///
    /// Where:
    ///
    /// - LONG is 64 hex characters.
    /// - SHORT is 1 to 63 hex characters inclusive.
    ///
    /// Learn more about the different address formats by reading AIP-40:
    /// <https://github.com/aptos-foundation/AIPs/blob/main/aips/aip-40.md>.
    fn from_str(s: &str) -> Result<Self, AccountAddressParseError> {
        if s.starts_with("0x") {
            if s.len() == 2 {
                return Err(AccountAddressParseError::TooShort);
            }
            AccountAddress::from_hex_literal(s)
        } else {
            if s.is_empty() {
                return Err(AccountAddressParseError::TooShort);
            }
            AccountAddress::from_hex_literal(&format!("0x{s}"))
        }
    }
}
