use cosmwasm_std::{Addr, Deps, DepsMut, Env, Event, MessageInfo};
use depolama::StorageExt;

use crate::{
    error::{self, ContractError},
    query::{is_executing, min_setback},
    state::{RoleMembers, Roles},
    time::Delay,
    types::{Access, Method, RoleId},
};

    // =============================================== ROLE MANAGEMENT ===============================================

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
/// Emits a [`role_granted`] event.
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
    deps: DepsMut,
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
        == 0;

    let since;

    let role_members_key = (role_id, account.clone());

    if new_member {
        since = env.block.time.seconds() + grant_delay as u64;
        deps.storage.write::<RoleMembers>(
            &role_members_key,
            &Access {
                since,
                delay: Delay::new(execution_delay),
            },
        );
    } else {
        let new_delay;
        (new_delay, since) =
            deps.storage
                .update::<RoleMembers, ContractError, _>(&role_members_key, |access| {
                    let (new_delay, since) = access.delay.with_update(env, execution_delay, 0);
                    access.delay = new_delay;
                    Ok((new_delay, since))
                })?;
    }

    Ok((new_member, todo!("RoleGranted()")))
}

// Internal version of [`revoke_role`] without access control. This logic is also used by {renounceRole}. Returns true if the role was previously granted.
///
/// Emits a [`role_revoked`] event if the account had the role.
///
/// ```solidity
/// function _revokeRole(uint64 roleId, address account) internal virtual returns (bool)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L305>
fn _revoke_role(
    deps: DepsMut,
    role_id: RoleId,
    account: &Addr,
) -> Result<(bool, Option<Event>), ContractError> {
    if role_id == RoleId::PUBLIC_ROLE {
        return Err(ContractError::AccessManagerLockedRole(role_id));
    }

    match deps
        .storage
        .take::<RoleMembers>(&(role_id, account.clone()))?
    {
        Some(_) => Ok((true, todo!("RoleGranted()"))),
        None => Ok((false, None)),
    }
}

/// Internal version of [`set_role_admin`] without access control.
///
/// Emits a [`role_admin_changed`] event.
///
/// NOTE: Setting the admin role as the `PUBLIC_ROLE` is allowed, but it will effectively allow
/// anyone to set grant or revoke such role.
///
/// ```solidity
/// function _setRoleAdmin(uint64 roleId, uint64 admin) internal virtual
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L328>
fn _set_role_admin(deps: DepsMut, role_id: RoleId, admin: RoleId) -> Result<Event, ContractError> {
    if role_id == RoleId::ADMIN_ROLE || role_id == RoleId::PUBLIC_ROLE {
        return Err(ContractError::AccessManagerLockedRole(role_id));
    }

    deps.storage
        .update::<Roles, ContractError, _>(&role_id, |role| {
            role.admin = admin;
            Ok(())
        });

    Ok(todo!("RoleAdminChanged(roleId, admin)"))
}

/// Internal version of [`set_role_guardian`] without access control.
///
/// Emits a [`role_guardian_changed`] event.
///
/// NOTE: Setting the guardian role as the `PUBLIC_ROLE` is allowed, but it will effectively allow
/// anyone to cancel any scheduled operation for such role.
///
/// ```solidity
/// function _setRoleGuardian(uint64 roleId, uint64 guardian) internal virtual
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L346>
fn _set_role_guardian(
    deps: DepsMut,
    role_id: RoleId,
    guardian: RoleId,
) -> Result<Event, ContractError> {
    if role_id == RoleId::ADMIN_ROLE || role_id == RoleId::PUBLIC_ROLE {
        return Err(ContractError::AccessManagerLockedRole(role_id));
    }

    deps.storage
        .update::<Roles, ContractError, _>(&role_id, |role| {
            role.guardian = guardian;
            Ok(())
        });

    Ok(todo!("RoleGuardianChanged(roleId, guardian)"));
}

/// Internal version of {setGrantDelay} without access control.
///  
/// Emits a {RoleGrantDelayChanged} event.
///
/// ```solidity
/// function _setGrantDelay(uint64 roleId, uint32 newDelay) internal virtual
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L361>
fn _set_grant_delay(
    deps: DepsMut,
    env: &Env,
    role_id: RoleId,
    new_delay: u32,
) -> Result<Event, ContractError> {
    if role_id == RoleId::PUBLIC_ROLE {
        return Err(ContractError::AccessManagerLockedRole(role_id));
    }

    deps.storage
        .update::<Roles, ContractError, _>(&role_id, |role| {
            let (new_delay, effect) = role.grant_delay.with_update(env, new_delay, min_setback());
            role.grant_delay = new_delay.clone();
            Ok((new_delay, effect))
        });

    Ok(todo!("RoleGrantDelayChanged(role_id, newDelay, effect);"))
}

    // ============================================= FUNCTION MANAGEMENT ==============================================

    /// ```solidity
    /// function setTargetFunctionRole(
    ///     address target,
    ///     bytes4[] calldata selectors,
    ///     uint64 roleId
    /// ) public virtual onlyAuthorized {
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L374-L378>
fn setTargetFunctionRole(
    target: &Addr,
    methods: impl IntoIterator<Item = &Method>,
    role_id: RoleId
) public virtual onlyAuthorized {
    // for (uint256 i = 0; i < selectors.length; ++i) {
    //     _setTargetFunctionRole(target, selectors[i], roleId);
    // }
}

/**
 * @dev Internal version of {setTargetFunctionRole} without access control.
 *
 * Emits a {TargetFunctionRoleUpdated} event.
 */
fn _setTargetFunctionRole(target: &Addr, method: &Method, role_id: RoleId) internal virtual {
    _targets[target].allowedRoles[selector] = roleId;
    emit TargetFunctionRoleUpdated(target, selector, roleId);
}

fn setTargetAdminDelay(target: &Addr, uint32 newDelay) public virtual onlyAuthorized {
    _setTargetAdminDelay(target, newDelay);
}

/**
 * @dev Internal version of {setTargetAdminDelay} without access control.
 *
 * Emits a {TargetAdminDelayUpdated} event.
 */
fn _setTargetAdminDelay(target: &Addr, uint32 newDelay) internal virtual {
    uint48 effect;
    (_targets[target].adminDelay, effect) = _targets[target].adminDelay.withUpdate(newDelay, minSetback());

    emit TargetAdminDelayUpdated(target, newDelay, effect);
}

// =============================================== MODE MANAGEMENT ================================================
fn setTargetClosed(target: &Addr, bool closed) public virtual onlyAuthorized {
    _setTargetClosed(target, closed);
}

/**
 * @dev Set the closed flag for a contract. This is an internal setter with no access restrictions.
 *
 * Emits a {TargetClosed} event.
 */
fn _setTargetClosed(target: &Addr, bool closed) internal virtual {
    _targets[target].closed = closed;
    emit TargetClosed(target, closed);
}

// ============================================== DELAYED OPERATIONS ==============================================
fn getSchedule(bytes32 id) public view virtual returns (uint48) {
    uint48 timepoint = _schedules[id].timepoint;
    return _isExpired(timepoint) ? 0 : timepoint;
}

fn getNonce(bytes32 id) public view virtual returns (uint32) {
    return _schedules[id].nonce;
}

fn schedule(
    target: &Addr,
    bytes calldata data,
    uint48 when
) public virtual returns (bytes32 operationId, uint32 nonce) {
    address caller = _msgSender();

    // Fetch restrictions that apply to the caller on the targeted function
    (, uint32 setback) = _canCallExtended(caller, target, data);

    uint48 minWhen = Time.timestamp() + setback;

    // If call with delay is not authorized, or if requested timing is too soon, revert
    if (setback == 0 || (when > 0 && when < minWhen)) {
        revert AccessManagerUnauthorizedCall(caller, target, _checkSelector(data));
    }

    // Reuse variable due to stack too deep
    when = uint48(Math.max(when, minWhen)); // cast is safe: both inputs are uint48

    // If caller is authorised, schedule operation
    operationId = hashOperation(caller, target, data);

    _checkNotScheduled(operationId);

    unchecked {
        // It's not feasible to overflow the nonce in less than 1000 years
        nonce = _schedules[operationId].nonce + 1;
    }
    _schedules[operationId].timepoint = when;
    _schedules[operationId].nonce = nonce;
    emit OperationScheduled(operationId, nonce, when, caller, target, data);

    // Using named return values because otherwise we get stack too deep
}

/**
 * @dev Reverts if the operation is currently scheduled and has not expired.
 *
 * NOTE: This fn was introduced due to stack too deep errors in schedule.
 */
fn _checkNotScheduled(bytes32 operationId) private view {
    uint48 prevTimepoint = _schedules[operationId].timepoint;
    if (prevTimepoint != 0 && !_isExpired(prevTimepoint)) {
        revert AccessManagerAlreadyScheduled(operationId);
    }
}

// Reentrancy is not an issue because permissions are checked on msg.sender. Additionally,
// _consumeScheduledOp guarantees a scheduled operation is only executed once.
// slither-disable-next-line reentrancy-no-eth
fn execute(target: &Addr, bytes calldata data) public payable virtual returns (uint32) {
    address caller = _msgSender();

    // Fetch restrictions that apply to the caller on the targeted function
    (bool immediate, uint32 setback) = _canCallExtended(caller, target, data);

    // If call is not authorized, revert
    if (!immediate && setback == 0) {
        revert AccessManagerUnauthorizedCall(caller, target, _checkSelector(data));
    }

    bytes32 operationId = hashOperation(caller, target, data);
    uint32 nonce;

    // If caller is authorised, check operation was scheduled early enough
    // Consume an available schedule even if there is no currently enforced delay
    if (setback != 0 || getSchedule(operationId) != 0) {
        nonce = _consumeScheduledOp(operationId);
    }

    // Mark the target and selector as authorised
    bytes32 executionIdBefore = _executionId;
    _executionId = _hashExecutionId(target, _checkSelector(data));

    // Perform call
    Address.functionCallWithValue(target, data, msg.value);

    // Reset execute identifier
    _executionId = executionIdBefore;

    return nonce;
}

fn cancel(address caller, target: &Addr, bytes calldata data) public virtual returns (uint32) {
    address msgsender = _msgSender();
    method: &Method = _checkSelector(data);

    bytes32 operationId = hashOperation(caller, target, data);
    if (_schedules[operationId].timepoint == 0) {
        revert AccessManagerNotScheduled(operationId);
    } else if (caller != msgsender) {
        // calls can only be canceled by the account that scheduled them, a global admin, or by a guardian of the required role.
        (bool isAdmin, ) = hasRole(ADMIN_ROLE, msgsender);
        (bool isGuardian, ) = hasRole(getRoleGuardian(getTargetFunctionRole(target, selector)), msgsender);
        if (!isAdmin && !isGuardian) {
            revert AccessManagerUnauthorizedCancel(msgsender, caller, target, selector);
        }
    }

    delete _schedules[operationId].timepoint; // reset the timepoint, keep the nonce
    uint32 nonce = _schedules[operationId].nonce;
    emit OperationCanceled(operationId, nonce);

    return nonce;
}

fn consumeScheduledOp(address caller, bytes calldata data) public virtual {
    target: &Addr = _msgSender();
    if (IAccessManaged(target).isConsumingScheduledOp() != IAccessManaged.isConsumingScheduledOp.selector) {
        revert AccessManagerUnauthorizedConsume(target);
    }
    _consumeScheduledOp(hashOperation(caller, target, data));
}

/**
 * @dev Internal variant of {consumeScheduledOp} that operates on bytes32 operationId.
 *
 * Returns the nonce of the scheduled operation that is consumed.
 */
fn _consumeScheduledOp(bytes32 operationId) internal virtual returns (uint32) {
    uint48 timepoint = _schedules[operationId].timepoint;
    uint32 nonce = _schedules[operationId].nonce;

    if (timepoint == 0) {
        revert AccessManagerNotScheduled(operationId);
    } else if (timepoint > Time.timestamp()) {
        revert AccessManagerNotReady(operationId);
    } else if (_isExpired(timepoint)) {
        revert AccessManagerExpired(operationId);
    }

    delete _schedules[operationId].timepoint; // reset the timepoint, keep the nonce
    emit OperationExecuted(operationId, nonce);

    return nonce;
}

fn hashOperation(address caller, target: &Addr, bytes calldata data) public view virtual returns (bytes32) {
    return keccak256(abi.encode(caller, target, data));
}

// ==================================================== OTHERS ====================================================
/// @inheritdoc IAccessManager
function updateAuthority(address target, address newAuthority) public virtual onlyAuthorized {
    IAccessManaged(target).setAuthority(newAuthority);
}

// ================================================= ADMIN LOGIC ==================================================
/**
 * @dev Check if the current call is authorized according to admin and roles logic.
 *
 * WARNING: Carefully review the considerations of {AccessManaged-restricted} since they apply to this modifier.
 */
fn _checkAuthorized(info: &MessageInfo) -> Result<(), ContractError> {
    let caller = info.sender;
    (bool immediate, uint32 delay) = _canCallSelf(caller, _msgData());
    if (!immediate) {
        if (delay == 0) {
            (, uint64 requiredRole, ) = _getAdminRestrictions(_msgData());
            revert AccessManagerUnauthorizedAccount(caller, requiredRole);
        } else {
            _consumeScheduledOp(hashOperation(caller, address(this), _msgData()));
        }
    }
}

/**
 * @dev Get the admin restrictions of a given function call based on the function and arguments involved.
 *
 * Returns:
 * - bool restricted: does this data match a restricted operation
 * - uint64: which role is this operation restricted to
 * - uint32: minimum delay to enforce for that operation (max between operation's delay and admin's execution delay)
 */
fn _getAdminRestrictions(
deps: Deps,
    method: &Method
) private view returns (bool adminRestricted, uint64 roleAdminId, uint32 executionDelay) {
    if (data.length < 4) {
        return (false, 0, 0);
    }

    bytes4 selector = _checkSelector(data);

    // Restricted to ADMIN with no delay beside any execution delay the caller may have
    if (
        selector == this.labelRole.selector ||
        selector == this.setRoleAdmin.selector ||
        selector == this.setRoleGuardian.selector ||
        selector == this.setGrantDelay.selector ||
        selector == this.setTargetAdminDelay.selector
    ) {
        return (true, ADMIN_ROLE, 0);
    }

    // Restricted to ADMIN with the admin delay corresponding to the target
    if (
        selector == this.updateAuthority.selector ||
        selector == this.setTargetClosed.selector ||
        selector == this.setTargetFunctionRole.selector
    ) {
        // First argument is a target.
        address target = abi.decode(data[0x04:0x24], (address));
        uint32 delay = getTargetAdminDelay(target);
        return (true, ADMIN_ROLE, delay);
    }

    // Restricted to that role's admin with no delay beside any execution delay the caller may have.
    if (selector == this.grantRole.selector || selector == this.revokeRole.selector) {
        // First argument is a roleId.
        uint64 roleId = abi.decode(data[0x04:0x24], (uint64));
        return (true, getRoleAdmin(roleId), 0);
    }

    return (false, getTargetFunctionRole(address(this), selector), 0);
}

// // =================================================== HELPERS ====================================================
// /**
//  * @dev An extended version of {canCall} for internal usage that checks {_canCallSelf}
//  * when the target is this contract.
//  *
//  * Returns:
//  * - bool immediate: whether the operation can be executed immediately (with no delay)
//  * - uint32 delay: the execution delay
//  */
// function _canCallExtended(
//     address caller,
//     address target,
//     bytes calldata data
// ) private view returns (bool immediate, uint32 delay) {
//     if (target == address(this)) {
//         return _canCallSelf(caller, data);
//     } else {
//         return data.length < 4 ? (false, 0) : canCall(caller, target, _checkSelector(data));
//     }
// 

// /**
//  * @dev A version of {canCall} that checks for restrictions in this contract.
//  */
fn _canCallSelf(deps: Deps, env: &Env, caller: &Addr, method: &Method) -> Result<(bool, uint32), ContractError> {
    if (caller == &env.contract.address) {
        // Caller is AccessManager, this means the call was sent through {execute} and it already checked
        // permissions. We verify that the call "identifier", which is set during {execute}, is correct.
        return Ok((is_executing(deps, &env.contract.address, method)?, 0));
    }

    (bool adminRestricted, uint64 roleId, uint32 operationDelay) = _getAdminRestrictions(data);

    // isTargetClosed apply to non-admin-restricted function
    if (!adminRestricted && isTargetClosed(address(this))) {
        return (false, 0);
    }

    (bool inRole, uint32 executionDelay) = hasRole(roleId, caller);
    if (!inRole) {
        return (false, 0);
    }

    // downcast is safe because both options are uint32
    delay = uint32(Math.max(operationDelay, executionDelay));
    return (delay == 0, delay);
}

// /**
//  * @dev Returns true if a call with `target` and `selector` is being executed via {executed}.
//  */
// function _isExecuting(address target, bytes4 selector) private view returns (bool) {
//     return _executionId == _hashExecutionId(target, selector);
// }

// /**
//  * @dev Returns true if a schedule timepoint is past its expiration deadline.
//  */
// function _isExpired(uint48 timepoint) private view returns (bool) {
//     return timepoint + expiration() <= Time.timestamp();
// }
