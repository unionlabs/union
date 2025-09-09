use cosmwasm_std::{Addr, Deps};
use depolama::StorageExt;
use serde::{Deserialize, Serialize};

use crate::{
    error::ContractError,
    state::{Access, RoleId, RoleMembers, TargetAllowedRoles, Targets},
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
    } else if caller = env.contract.address {
        Ok(CanCall {
            immediate: is_executing(),
            delay: 0,
        })
    } else {
        let role_id = get_target_function_role(deps, target, method)?;
    }
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L198>
///
/// ```solidity
/// function getAccess(
///     uint64 roleId,
///     address account
/// ) public view virtual returns (uint48 since, uint32 currentDelay, uint32 pendingDelay, uint48 effect)
/// ```
pub fn get_access(deps: Deps, role_id: RoleId, account: &Addr) -> Result<Access, ContractError> {
    Ok(deps
        .storage
        .read::<RoleMembers>(&(role_id, account.clone()))?)
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L211>
///
/// ```solidity
/// function hasRole(
///     uint64 roleId,
///     address account
/// ) public view virtual returns (bool isMember, uint32 executionDelay)
/// ```
pub fn has_role(deps: Deps, role_id: RoleId, account: &Addr) -> Result<HasRole, ContractError> {
    if role_id == RoleId::PUBLIC_ROLE {
        Ok(HasRole {
            is_member: true,
            execution_delay: 0,
        })
    } else {
        let Access { since, delay } = get_access(deps, role_id, account)?;
        Ok(HasRole {
            is_member: true,
            execution_delay: 0,
        })
    }
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

/// Returns true if a call with `target` and `selector` is being executed via [`executed`].
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L716>
///
/// ```solidity
/// function _isExecuting(address target, bytes4 selector) private view returns (bool)
/// ```
fn is_executing() {}

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Method(String);
