use cosmwasm_std::{StdError, StdResult};
use depolama::Bytes;
use serde::{Deserialize, Serialize};

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
/// Note that the `allowedRoles` field of the original solidity struct is implemented via the [`TargetAllowedRoles`] store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct TargetConfig {
    pub admin_delay: Delay,
    pub closed: bool,
}

/// Structure that stores the details for a role/account pair. This structures fit into a single slot.
///
/// ```solidity
// struct Access {
//     uint48 since;
//     Time.Delay delay;
// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Access {
    /// Timepoint at which the user gets the permission.
    ///
    /// If this is either 0 or in the future, then the role permission is not available.
    pub since: u64,
    /// Delay for execution. Only applies to restricted() / execute() calls.
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
/// Note that the `members` field of the original solidity struct is implemented via the [`RoleMembers`] store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Role {
    /// Admin who can grant or revoke permissions.
    pub admin: RoleId,
    /// Guardian who can cancel operations targeting functions that need this role.
    pub guardian: RoleId,
    /// Delay in which the role takes effect after being granted.
    pub grant_delay: Delay,
}

/// Structure that stores the details for a scheduled operation. This structure fits into a single slot.
///
/// ```solidity
// struct Schedule {
//     uint48 timepoint;
//     uint32 nonce;
// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Schedule {
    /// Moment at which the operation can be executed.
    pub timepoint: u64,
    /// Operation nonce to allow third-party contracts to identify the operation.
    pub nonce: u64,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Serialize, Deserialize, bincode::Encode, bincode::Decode,
)]
pub struct RoleId(#[serde(with = "::serde_utils::string")] u64);

impl RoleId {
    /// The identifier of the admin role. Required to perform most configuration operations including other roles' management and target restrictions.
    ///
    /// ```solidity
    /// uint64 public constant ADMIN_ROLE = type(uint64).min; // 0
    /// ```
    pub const ADMIN_ROLE: Self = Self(u64::MIN);

    /// The identifier of the public role. Automatically granted to all addresses with no delay.
    ///
    /// ```solidity
    // uint64 public constant PUBLIC_ROLE = type(uint64).max; // 2**64-1
    /// ```
    pub const PUBLIC_ROLE: Self = Self(u64::MAX);

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
