//! Yoinked from <https://github.com/aptos-labs/aptos-core/blob/3b433805f5bd89d5c5b9942158efac1cc0077bf5/third_party/move/move-core/types/src/account_address.rs>
///
/// We only use the strict parsing and display functionality, and wrap our `H256` type instead of `[u8; 32]`.
use core::{fmt, str::FromStr};

use serde::{de::Error as _, Deserialize, Serialize};

use crate::{errors::InvalidLength, hash::H256};

#[derive(macros::Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[debug("AccountAddress({})", self)]
pub struct AccountAddress(pub H256);

impl AccountAddress {
    /// Returns whether the address is a "special" address. Addresses are considered
    /// special if the first 63 characters of the hex string are zero. In other words,
    /// an address is special if the first 31 bytes are zero and the last byte is
    /// smaller than than `0b10000` (16). In other words, special is defined as an address
    /// that matches the following regex: `^0x0{63}[0-9a-f]$`. In short form this means
    /// the addresses in the range from `0x0` to `0xf` (inclusive) are special.
    ///
    /// For more details see the v1 address standard defined as part of AIP-40:
    /// <https://github.com/aptos-foundation/AIPs/blob/main/aips/aip-40.md>
    #[must_use]
    pub fn is_special(&self) -> bool {
        (self.0).0[..H256::BYTES_LEN - 1].iter().all(|x| *x == 0)
            && is_special_byte((self.0).0[H256::BYTES_LEN - 1])
    }
}

const fn is_special_byte(b: u8) -> bool {
    b < 0b10000
}

impl AsRef<[u8]> for AccountAddress {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TryFrom<Vec<u8>> for AccountAddress {
    type Error = InvalidLength;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AccountAddressParseError {
    #[error("hex string must start with a leading 0x")]
    Leading0XRequired,

    #[error(transparent)]
    HexDecode(#[from] hex::FromHexError),

    #[error(
        "the given hex string is not a special address, it must be represented as 0x + 64 chars"
    )]
    LongFormRequiredUnlessSpecial,

    #[error("the given hex string is a special address not in long form, it must be 0x0 to 0xf without padding zeroes")]
    InvalidPaddingZeroes,

    #[error("invalid address length: {0}")]
    InvalidLength(usize),
}

impl Serialize for AccountAddress {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        if serializer.is_human_readable() {
            self.0.serialize(serializer)
        } else {
            // See comment in deserialize.
            serializer.serialize_newtype_struct("AccountAddress", &self.0 .0)
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
            struct Value([u8; H256::BYTES_LEN]);

            let value = Value::deserialize(deserializer)?;
            Ok(AccountAddress(H256(value.0)))
        }
    }
}

impl FromStr for AccountAddress {
    type Err = AccountAddressParseError;

    /// NOTE: This function has strict parsing behavior. For relaxed behavior, please use
    /// the `from_str` function. Where possible, prefer to use `from_str_strict`.
    ///
    /// Create an instance of [`AccountAddress`] by parsing a hex string representation.
    ///
    /// This function allows only the strictest formats defined by AIP-40. In short this
    /// means only the following formats are accepted:
    ///
    /// - LONG
    /// - SHORT for special addresses
    ///
    /// Where:
    ///
    /// - LONG is defined as 0x + 64 hex characters.
    /// - SHORT for special addresses is 0x0 to 0xf inclusive.
    ///
    /// This means the following are not accepted:
    ///
    /// - SHORT for non-special addresses.
    /// - Any address without a leading 0x.
    ///
    /// Learn more about the different address formats by reading AIP-40:
    /// <https://github.com/aptos-foundation/AIPs/blob/main/aips/aip-40.md>.
    fn from_str(s: &str) -> Result<Self, AccountAddressParseError> {
        // Assert the string starts with 0x.
        if !s.starts_with("0x") {
            return Err(AccountAddressParseError::Leading0XRequired);
        }

        let address = hex::decode(&s[2..])?;

        // Check if the address is in LONG form. If it is not, this is only allowed for
        // special addresses, in which case we check it is in proper SHORT form.
        match address.len() {
            H256::BYTES_LEN => Ok(Self(H256(address.try_into().unwrap()))),
            1 => {
                let b = address[0];

                if is_special_byte(b) {
                    return Err(AccountAddressParseError::LongFormRequiredUnlessSpecial);
                }

                // 0x + one hex char is the only valid SHORT form for special addresses.
                if s.len() != 3 {
                    return Err(AccountAddressParseError::InvalidPaddingZeroes);
                }

                let mut address = [0; H256::BYTES_LEN];

                address[H256::BYTES_LEN - 1] = b;

                Ok(Self(H256(address)))
            }
            len => Err(AccountAddressParseError::InvalidLength(len)),
        }
    }
}

impl fmt::Display for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_special() {
            f.write_fmt(format_args!("0x{:x}", self.0 .0[H256::BYTES_LEN - 1]))
        } else {
            self.0.fmt(f)
        }
    }
}
