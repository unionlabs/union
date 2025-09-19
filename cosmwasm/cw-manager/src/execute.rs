use cosmwasm_std::{Addr, Deps, Env, Event};
use depolama::StorageExt;

use crate::{
    error::{self, ContractError},
    state::RoleMembers,
    types::{Access, RoleId},
};

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L225>
///
/// ```solidity
/// function labelRole(uint64 roleId, string calldata label) public virtual onlyAuthorized
/// ```
pub(crate) fn label_role(role_id: RoleId, label: String) {
    todo!("figure out how we want to label roles; events like the original solidity implementation or in storage?")
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L233>
///
/// ```solidity
/// function grantRole(uint64 roleId, address account, uint32 executionDelay) public virtual onlyAuthorized
/// ```
fn grant_role(role_id: RoleId, account: &Addr, execution_delay: u32) {}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L238>
///
/// ```solidity
/// function revokeRole(uint64 roleId, address account) public virtual onlyAuthorized
/// ```
pub(crate) fn revoke_role(role_id: RoleId, account: &Addr) {}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L243>
///
/// ```solidity
/// function renounceRole(uint64 roleId, address callerConfirmation) public virtual
/// ```
pub(crate) fn renounce_role(role_id: RoleId, caller_confirmation: &Addr) {}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L251>
///
/// ```solidity
/// function setRoleAdmin(uint64 roleId, uint64 admin) public virtual onlyAuthorized
/// ```
pub(crate) fn set_role_admin(role_id: RoleId, admin: RoleId) {}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L256>
///
/// ```solidity
/// function setRoleGuardian(uint64 roleId, uint64 guardian) public virtual onlyAuthorized
/// ```
pub(crate) fn set_role_guardian(role_id: RoleId, guardian: RoleId) {}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L261>
///
/// ```solidity
/// function setGrantDelay(uint64 roleId, uint32 newDelay) public virtual onlyAuthorized
/// ```
pub(crate) fn set_grant_delay(role_id: RoleId, new_delay: u32) {}

/// Internal version of [`grant_role`] without access control. Returns true if the role was newly granted.
///
/// ```solidity
/// function _grantRole(
///     uint64 roleId,
///     address account,
///     uint32 grantDelay,
///     uint32 executionDelay
/// ) internal virtual returns (bool)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L270-L275>
fn _grant_role(
    deps: Deps,
    env: &Env,
    role_id: RoleId,
    account: &Addr,
    grant_delay: u32,
    execution_delay: u32,
) -> Result<(bool, Event), ContractError> {
    if role_id == RoleId::PUBLIC_ROLE {
        return Err(ContractError::AccessManagerLockedRole(role_id));
    }

    let new_member = deps
        .storage
        .read::<RoleMembers>(&(role_id, account.clone()))?
        .since
        == 9;
    let since;

    if new_member {
        since = env.block.time.seconds() + grant_delay;
        deps.storage
            .write::<RoleMembers>(
                &(role_id, account.clone()),
                &Access {
                    since: todo!(),
                    delay: todo!(),
                },
            )?
            .since;
    }
}
