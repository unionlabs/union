use core::fmt;
use std::{borrow::Cow, ptr};

use cosmwasm_std::{StdError, StdResult};
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::value::RawValue;
use unionlabs_primitives::Bytes;

use crate::time::Delay;

pub mod restricted;
pub mod time;

pub mod managed;
pub mod manager;

#[cfg(doc)]
use crate::manager::msg::QueryMsg;

/// Structure that stores the details for a target contract.
///
/// ```solidity
/// struct TargetConfig {
///     mapping(bytes4 selector => uint64 roleId) allowedRoles;
///     Time.Delay adminDelay;
///     bool closed;
/// }
/// ```
///
/// Note that the `allowedRoles` field of the original solidity struct is implemented via the
/// `TargetAllowedRoles` store.
#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, bincode::Encode, bincode::Decode,
)]
pub struct TargetConfig {
    pub admin_delay: Delay,
    pub closed: bool,
}

/// Structure that stores the details for a role/account pair.
///
/// ```solidity
/// struct Access {
///     uint48 since;
///     Time.Delay delay;
/// }
/// ```
#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, bincode::Encode, bincode::Decode,
)]
pub struct Access {
    /// Timepoint at which the user gets the permission.
    ///
    /// If this is either 0 or in the future, then the role permission is not available.
    pub since: u64,
    /// Delay for execution. Only applies to `restricted()`/`execute()` calls.
    pub delay: Delay,
}

/// Structure that stores the details of a role.
///
/// ```solidity
/// struct Role {
///     mapping(address user => Access access) members;
///     uint64 admin;
///     uint64 guardian;
///     Time.Delay grantDelay;
/// }
/// ```
///
/// Note that the `members` field of the original solidity struct is implemented via the
/// `RoleMembers` store.
#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, bincode::Encode, bincode::Decode,
)]
pub struct Role {
    /// Admin who can grant or revoke permissions.
    pub admin: RoleId,
    /// Guardian who can cancel operations targeting functions that need this role.
    pub guardian: RoleId,
    /// Delay in which the role takes effect after being granted.
    pub grant_delay: Delay,
}

/// Structure that stores the details for a scheduled operation.
///
/// ```solidity
/// struct Schedule {
///     uint48 timepoint;
///     uint32 nonce;
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Schedule {
    /// Moment at which the operation can be executed.
    pub timepoint: u64,
    /// Operation nonce to allow third-party contracts to identify the operation.
    // TODO: Newtype
    pub nonce: u32,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, bincode::Encode, bincode::Decode,
)]
pub struct RoleId(#[serde(with = "::serde_utils::string")] u64);

impl fmt::Display for RoleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl RoleId {
    /// See [`QueryMsg::AdminRole`].
    pub const ADMIN_ROLE: Self = Self(u64::MIN);

    /// See [`QueryMsg::PublicRole`].
    pub const PUBLIC_ROLE: Self = Self(u64::MAX);

    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    #[must_use]
    pub fn to_be_bytes(&self) -> [u8; 8] {
        self.0.to_be_bytes()
    }

    pub fn from_be_bytes(raw: [u8; 8]) -> StdResult<Self> {
        Ok(RoleId(u64::from_be_bytes(raw)))
    }

    pub fn try_from_be_bytes(raw: &Bytes) -> StdResult<Self> {
        raw.try_into()
            .map_err(|_| {
                StdError::generic_err(format!(
                    "invalid key: expected 8 bytes, found {}: {raw}",
                    raw.len(),
                ))
            })
            .and_then(RoleId::from_be_bytes)
    }
}

#[derive(Debug, PartialEq, bincode::Encode)]
#[repr(transparent)]
pub struct Selector(str);

impl ToOwned for Selector {
    type Owned = Box<Self>;

    fn to_owned(&self) -> Self::Owned {
        unsafe { Box::from_raw(Box::into_raw(self.0.to_string().into_boxed_str()) as *mut Self) }
    }
}

impl Serialize for Selector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Box<Selector> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Box<str>>::deserialize(deserializer).map(Selector::new_owned)
    }
}

impl<'de> Deserialize<'de> for &'de Selector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <&'de str>::deserialize(deserializer).map(Selector::new)
    }
}

impl<Context> bincode::Decode<Context> for Box<Selector> {
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        <Box<str> as bincode::Decode<Context>>::decode(decoder).map(Selector::new_owned)
    }
}

impl Selector {
    pub fn new(selector: &str) -> &Self {
        unsafe { &*(ptr::from_ref::<str>(selector) as *const Self) }
    }

    pub fn new_owned(selector: Box<str>) -> Box<Self> {
        unsafe { Box::from_raw(Box::into_raw(selector) as *mut Self) }
    }

    pub fn extract(data: &str) -> Result<&Self, serde_json::Error> {
        struct ExtractSelector;

        impl<'de> Visitor<'de> for ExtractSelector {
            type Value = &'de Selector;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "json object with single top level key")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                match map.next_entry::<&'de str, &'de RawValue>()? {
                    Some((selector, _)) => {
                        if map.next_key::<Cow<'de, str>>()?.is_some() {
                            Err(<A::Error as de::Error>::custom("multiple keys found"))
                        } else {
                            Ok(Selector::new(selector))
                        }
                    }
                    None => Err(<A::Error as de::Error>::custom("no key found")),
                }
            }
        }

        serde_json::de::Deserializer::from_str(data).deserialize_map(ExtractSelector)
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanCall {
    // whether or not the caller can call the target *right now*, i.e. without any delay
    pub allowed: bool,
    pub delay: u32,
}

// TODO: Make this an enum, execution_delay is only ever meaningful when is_member is true
// (i think)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HasRole {
    pub is_member: bool,
    pub execution_delay: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FullAccess {
    /// Timestamp at which the account membership becomes valid. 0 means role is not granted.
    pub since: u64,
    /// Current execution delay for the account.
    pub current_delay: u32,
    /// Pending execution delay for the account.
    pub pending_delay: u32,
    /// Timestamp at which the pending execution delay will become active. 0 means no delay update
    /// is scheduled.
    pub effect: u64,
}
