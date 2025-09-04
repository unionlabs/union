use core::fmt;

use cosmwasm_std::{StdError, StdResult};
use depolama::Bytes;
use serde::{Deserialize, Serialize};

#[cfg(doc)]
use crate::msg::QueryMsg;
use crate::time::Delay;

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
/// [`TargetAllowedRoles`][crate::state::TargetAllowedRoles] store.
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
/// [`RoleMembers`][crate::state::RoleMembers] store.
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
// TODO: &str
pub struct Selector(String);

impl Selector {
    pub(crate) fn new(selector: impl Into<String>) -> Self {
        Self(selector.into())
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
