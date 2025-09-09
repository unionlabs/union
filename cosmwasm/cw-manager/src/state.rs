use cosmwasm_std::{Addr, StdError, StdResult};
use depolama::{
    key::KeyCodecViaEncoding, value::ValueCodecViaEncoding, Bytes, KeyCodec, Prefix,
    RawAddrEncoding, Store,
};
use serde::{Deserialize, Serialize};
use unionlabs_encoding::{Bincode, DecodeAs, EncodeAs};

use crate::{execute::Method, time::Delay};

/// The admin of this manager contract. This is the only address that is able to add or remove role permissions.
pub enum Admin {}
impl Store for Admin {
    const PREFIX: Prefix = Prefix::new(b"admin");
    type Key = ();
    type Value = Addr;
}
impl ValueCodecViaEncoding for Admin {
    type Encoding = RawAddrEncoding;
}

/// ```solidity
/// mapping(address target => TargetConfig mode) private _targets;
/// ```
pub enum Targets {}
impl Store for Targets {
    const PREFIX: Prefix = Prefix::new(b"targets");
    type Key = Addr;
    type Value = TargetConfig;
}
impl KeyCodecViaEncoding for Targets {
    type Encoding = RawAddrEncoding;
}
impl ValueCodecViaEncoding for Targets {
    type Encoding = Bincode;
}

/// ```solidity
/// mapping(bytes4 selector => uint64 roleId) allowedRoles;
/// ```
///
/// This is the `allowedRoles` field of the original solidity struct.
pub enum TargetAllowedRoles {}
impl Store for TargetAllowedRoles {
    const PREFIX: Prefix = Prefix::new(b"target_allowed_roles");
    // target address, method
    type Key = (Addr, Method);
    type Value = RoleId;
}
impl KeyCodec<(Addr, Method)> for TargetAllowedRoles {
    fn encode_key((addr, method): &(Addr, Method)) -> Bytes {
        (addr.clone().into_string(), method)
            .encode_as::<Bincode>()
            .into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<(Addr, Method)> {
        let (addr, method) = <(String, Method)>::decode_as::<Bincode>(raw)
            .map_err(|e| StdError::generic_err(format!("unable to decode: {e:?}")))?;

        Ok((Addr::unchecked(addr), method))
    }
}
impl ValueCodecViaEncoding for TargetAllowedRoles {
    type Encoding = Bincode;
}

/// ```solidity
/// mapping(uint64 roleId => Role) private _roles;
/// ```
pub enum Roles {}
impl Store for Roles {
    const PREFIX: Prefix = Prefix::new(b"roles");
    type Key = RoleId;
    type Value = Role;
}
impl KeyCodec<RoleId> for Roles {
    fn encode_key(key: &RoleId) -> Bytes {
        key.to_be_bytes().into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<RoleId> {
        RoleId::try_from_be_bytes(raw)
    }
}
impl ValueCodecViaEncoding for Roles {
    type Encoding = Bincode;
}

/// Members of the role.
///
/// ```solidity
/// mapping(address user => Access access) members;
/// ```
///
/// This is the `members` field of the original solidity struct.
pub enum RoleMembers {}
impl Store for RoleMembers {
    const PREFIX: Prefix = Prefix::new(b"role_members");
    type Key = (RoleId, Addr);
    type Value = Access;
}
impl KeyCodec<(RoleId, Addr)> for RoleMembers {
    fn encode_key(key: &(RoleId, Addr)) -> Bytes {
        key.0
            .to_be_bytes()
            .into_iter()
            .chain(key.1.clone().into_string().into_bytes())
            .collect()
    }

    fn decode_key(raw: &Bytes) -> StdResult<(RoleId, Addr)> {
        if raw.len() < 8 {
            Err(StdError::generic_err(format!(
                "invalid key: expected at least 8 bytes, found {} (raw: {raw})",
                raw.len()
            )))
        } else {
            let role_id = RoleId::from_be_bytes(raw[..8].try_into().expect("valid"))?;
            let addr = Addr::unchecked(
                String::from_utf8(raw[8..].to_vec())
                    .map_err(|e| StdError::generic_err(format!("invalid addr: {e}")))?,
            );
            Ok((role_id, addr))
        }
    }
}
impl ValueCodecViaEncoding for RoleMembers {
    type Encoding = Bincode;
}

/// ```solidity
/// mapping(bytes32 operationId => Schedule) private _schedules;
/// ```
pub enum Schedules {}

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
    pub admin: u64,
    /// Guardian who can cancel operations targeting functions that need this role.
    pub guardian: u64,
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
