use access_manager_types::{Access, Role, RoleId, Schedule, Selector, TargetConfig};
use cosmwasm_std::{Addr, StdError, StdResult};
use depolama::{
    key::KeyCodecViaEncoding, value::ValueCodecViaEncoding, Bytes, KeyCodec, Prefix,
    RawAddrEncoding, Store,
};
use unionlabs_encoding::{Bincode, DecodeAs, EncodeAs};
use unionlabs_primitives::H256;

/// The admin of this manager contract. This is the only address that is able to add or remove role
/// permissions.
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
    type Key = (Addr, Box<Selector>);
    type Value = RoleId;
}
impl KeyCodec<(Addr, Box<Selector>)> for TargetAllowedRoles {
    fn encode_key((addr, method): &(Addr, Box<Selector>)) -> Bytes {
        (addr.clone().into_string(), method)
            .encode_as::<Bincode>()
            .into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<(Addr, Box<Selector>)> {
        let (addr, method) = <(String, Box<Selector>)>::decode_as::<Bincode>(raw)
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
impl Store for Schedules {
    const PREFIX: Prefix = Prefix::new(b"schedules");
    type Key = H256;
    type Value = Schedule;
}
impl KeyCodec<H256> for Schedules {
    fn encode_key(key: &H256) -> Bytes {
        key.into_bytes()
    }

    fn decode_key(raw: &Bytes) -> StdResult<H256> {
        raw.as_ref()
            .try_into()
            .map_err(|e| StdError::generic_err(format!("invalid key: {e}")))
    }
}
impl ValueCodecViaEncoding for Schedules {
    type Encoding = Bincode;
}

/// Used to identify operations that are currently being executed via
/// [`ExecuteMsg::Execute`][crate::msg::ExecuteMsg::Execute]. This should be transient storage when
/// supported by `CosmWasm`.
///
/// ```solidity
/// bytes32 private _executionId;
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L117>
pub enum ExecutionIdStack {}
impl Store for ExecutionIdStack {
    const PREFIX: Prefix = Prefix::new(b"execution_id");
    type Key = ();
    type Value = Vec<H256>;
}
impl ValueCodecViaEncoding for ExecutionIdStack {
    type Encoding = Bincode;
}
