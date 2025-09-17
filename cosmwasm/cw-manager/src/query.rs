use cosmwasm_std::{Addr, Deps, Env};
use depolama::StorageExt;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use unionlabs_primitives::H256;

use crate::{
    error::ContractError,
    state::{ExecutionId, RoleMembers, Roles, TargetAllowedRoles, Targets},
    time::UnpackedDelay,
    types::{Method, RoleId},
};

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L139>
///
/// ```solidity
/// function canCall(
///     address caller,
///     address target,
///     bytes4 selector
/// ) public view virtual returns (bool immediate, uint32 delay)
/// ```
pub fn can_call(
    deps: Deps,
    env: &Env,
    caller: &Addr,
    target: &Addr,
    method: Method,
) -> Result<CanCall, ContractError> {
    if is_target_closed(deps, target)? {
        Ok(CanCall {
            immediate: false,
            delay: 0,
        })
    } else if caller == env.contract.address {
        // Caller is AccessManager, this means the call was sent through {execute} and it already checked
        // permissions. We verify that the call "identifier", which is set during {execute}, is correct.
        Ok(CanCall {
            immediate: is_executing(deps, target, &method)?,
            delay: 0,
        })
    } else {
        let role_id = get_target_function_role(deps, target, &method)?;
        let HasRole {
            is_member,
            execution_delay,
        } = has_role(deps, env, role_id, caller)?;
        if is_member {
            Ok(CanCall {
                immediate: execution_delay == 0,
                delay: execution_delay,
            })
        } else {
            Ok(CanCall {
                immediate: false,
                delay: 0,
            })
        }
    }
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L158>
pub fn expiration() -> u32 {
    // 1 week
    7 * 24 * 60 * 60
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L163>
pub fn min_setback() -> u32 {
    // 5 days
    5 * 24 * 60 * 60
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L168>
///
/// ```solidity
/// function isTargetClosed(address target) public view virtual returns (bool)
/// ```
pub fn is_target_closed(deps: Deps, target: &Addr) -> Result<bool, ContractError> {
    Ok(deps.storage.read::<Targets>(target)?.closed)
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L173>
///
/// ```solidity
/// function getTargetFunctionRole(address target, bytes4 selector) public view virtual returns (uint64)
/// ```
pub fn get_target_function_role(
    deps: Deps,
    target: &Addr,
    method: &Method,
) -> Result<RoleId, ContractError> {
    Ok(deps
        .storage
        .read::<TargetAllowedRoles>(&(target.clone(), method.clone()))?)
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L178>
///
/// ```solidity
/// function getTargetAdminDelay(address target) public view virtual returns (uint32)
/// ```
pub fn get_target_admin_delay(deps: Deps, env: &Env, target: &Addr) -> Result<u32, ContractError> {
    Ok(deps.storage.read::<Targets>(target)?.admin_delay.get(env))
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L183>
///
/// ```solidity
/// function getRoleAdmin(uint64 roleId) public view virtual returns (uint64)
/// ```
pub fn get_role_admin(deps: Deps, role_id: RoleId) -> Result<RoleId, ContractError> {
    Ok(deps.storage.read::<Roles>(&role_id)?.admin)
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L188>
///
/// ```solidity
/// function getRoleGuardian(uint64 roleId) public view virtual returns (uint64)
/// ```
pub fn get_role_guardian(deps: Deps, role_id: RoleId) -> Result<RoleId, ContractError> {
    Ok(deps.storage.read::<Roles>(&role_id)?.guardian)
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L193>
///
/// ```solidity
/// function getRoleGrantDelay(uint64 roleId) public view virtual returns (uint32)
/// ```
pub fn get_role_grant_delay(deps: Deps, env: &Env, role_id: RoleId) -> Result<u32, ContractError> {
    Ok(deps.storage.read::<Roles>(&role_id)?.grant_delay.get(env))
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L198>
///
/// ```solidity
/// function getAccess(
///     uint64 roleId,
///     address account
/// ) public view virtual returns (uint48 since, uint32 currentDelay, uint32 pendingDelay, uint48 effect)
/// ```
pub fn get_access(
    deps: Deps,
    env: &Env,
    role_id: RoleId,
    account: &Addr,
) -> Result<(u64, UnpackedDelay), ContractError> {
    let access = deps
        .storage
        .read::<RoleMembers>(&(role_id, account.clone()))?;

    Ok((access.since, access.delay.get_full(env)))
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L211>
///
/// ```solidity
/// function hasRole(
///     uint64 roleId,
///     address account
/// ) public view virtual returns (bool isMember, uint32 executionDelay)
/// ```
pub fn has_role(
    deps: Deps,
    env: &Env,
    role_id: RoleId,
    account: &Addr,
) -> Result<HasRole, ContractError> {
    if role_id == RoleId::PUBLIC_ROLE {
        Ok(HasRole {
            is_member: true,
            execution_delay: 0,
        })
    } else {
        let (since, delay) = get_access(deps, env, role_id, account)?;
        Ok(HasRole {
            is_member: since != 0 && since <= env.block.time.seconds(),
            execution_delay: delay.value_before,
        })
    }
}

/// Returns true if a call with `target` and `selector` is being executed via [`executed`].
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L716>
///
/// ```solidity
/// function _isExecuting(address target, bytes4 selector) private view returns (bool)
/// ```
fn is_executing(deps: Deps, target: &Addr, method: &Method) -> Result<bool, ContractError> {
    Ok(deps
        .storage
        .maybe_read_item::<ExecutionId>()?
        .is_some_and(|id| id == hash_execution_id(target, method)))
}

/// Hashing function for execute protection.
///
/// ```solidity
/// function _hashExecutionId(address target, bytes4 selector) private pure returns (bytes32)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L737>
fn hash_execution_id(target: &Addr, method: &Method) -> H256 {
    sha2::Sha256::digest(format!("{target}:{method}")).into()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanCall {
    pub immediate: bool,
    pub delay: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HasRole {
    pub is_member: bool,
    pub execution_delay: u32,
}
