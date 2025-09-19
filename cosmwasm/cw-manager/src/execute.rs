use std::cmp;

use cosmwasm_std::Addr;
use depolama::StorageExt;
use sha2::{Digest, Sha256};
use unionlabs_primitives::H256;

use crate::{
    error::ContractError,
    event::{
        operation_executed, role_admin_changed, role_grant_delay_changed, role_granted,
        role_guardian_changed, role_revoked,
    },
    msg::ExecuteMsg,
    query::{
        expiration, get_role_admin, get_role_grant_delay, get_target_admin_delay,
        get_target_function_role, has_role, is_executing, is_target_closed, min_setback, HasRole,
    },
    state::{RoleMembers, Roles, Schedules},
    time::Delay,
    types::{Access, RoleId, Schedule},
    Ctx,
};

/// Check that the caller is authorized to perform the operation.
/// See {AccessManager} description for a detailed breakdown of the authorization logic.
///
/// ```solidity
/// modifier onlyAuthorized()
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L123>
fn only_authorized(ctx: &mut Ctx) -> Result<(), ContractError> {
    _check_authorized(ctx)
}

// =============================================== ROLE MANAGEMENT ===============================================

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L225>
///
/// ```solidity
/// function labelRole(uint64 roleId, string calldata label) public virtual onlyAuthorized
/// ```
pub(crate) fn label_role(role_id: RoleId, label: &str) {
    todo!("figure out how we want to label roles; events like the original solidity implementation or in storage?")
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L233>
///
/// ```solidity
/// function grantRole(uint64 roleId, address account, uint32 executionDelay) public virtual onlyAuthorized
/// ```
pub(crate) fn grant_role(
    ctx: &mut Ctx,
    role_id: RoleId,
    account: &Addr,
    execution_delay: u32,
) -> Result<(), ContractError> {
    only_authorized(ctx)?;

    _grant_role(
        ctx,
        role_id,
        account,
        get_role_grant_delay(ctx, role_id)?,
        execution_delay,
    )?;

    Ok(())
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L238>
///
/// ```solidity
/// function revokeRole(uint64 roleId, address account) public virtual onlyAuthorized
/// ```
pub(crate) fn revoke_role(
    ctx: &mut Ctx,
    role_id: RoleId,
    account: &Addr,
) -> Result<(), ContractError> {
    only_authorized(ctx)?;

    _revoke_role(ctx, role_id, account)?;

    Ok(())
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L243>
///
/// ```solidity
/// function renounceRole(uint64 roleId, address callerConfirmation) public virtual
/// ```
pub(crate) fn renounce_role(
    ctx: &mut Ctx,
    role_id: RoleId,
    caller_confirmation: &Addr,
) -> Result<(), ContractError> {
    if caller_confirmation != ctx.msg_sender() {
        Err(ContractError::AccessManagerBadConfirmation)
    } else {
        _revoke_role(ctx, role_id, caller_confirmation)?;

        Ok(())
    }
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L251>
///
/// ```solidity
/// function setRoleAdmin(uint64 roleId, uint64 admin) public virtual onlyAuthorized
/// ```
pub(crate) fn set_role_admin(
    ctx: &mut Ctx,
    role_id: RoleId,
    admin: RoleId,
) -> Result<(), ContractError> {
    only_authorized(ctx)?;

    _set_role_admin(ctx, role_id, admin)
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L256>
///
/// ```solidity
/// function setRoleGuardian(uint64 roleId, uint64 guardian) public virtual onlyAuthorized
/// ```
pub(crate) fn set_role_guardian(
    ctx: &mut Ctx,
    role_id: RoleId,
    guardian: RoleId,
) -> Result<(), ContractError> {
    only_authorized(ctx)?;

    _set_role_guardian(ctx, role_id, guardian)
}

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L261>
///
/// ```solidity
/// function setGrantDelay(uint64 roleId, uint32 newDelay) public virtual onlyAuthorized
/// ```
pub(crate) fn set_grant_delay(
    ctx: &mut Ctx,
    role_id: RoleId,
    new_delay: u32,
) -> Result<(), ContractError> {
    only_authorized(ctx)?;

    _set_grant_delay(ctx, role_id, new_delay)
}

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
    ctx: &mut Ctx,
    role_id: RoleId,
    account: &Addr,
    grant_delay: u32,
    execution_delay: u32,
) -> Result<bool, ContractError> {
    if role_id == RoleId::PUBLIC_ROLE {
        return Err(ContractError::AccessManagerLockedRole(role_id));
    }

    let new_member = ctx.read::<RoleMembers>(&(role_id, account.clone()))?.since == 0;

    let role_members_key = (role_id, account.clone());

    let since = if new_member {
        let since = ctx.timestamp() + grant_delay as u64;
        ctx.write::<RoleMembers>(
            &role_members_key,
            &Access {
                since,
                delay: Delay::new(execution_delay),
            },
        );
        since
    } else {
        let timestamp = ctx.timestamp();
        ctx.update::<RoleMembers, ContractError, _>(&role_members_key, |access| {
            let (new_delay, since) = access.delay.with_update(timestamp, execution_delay, 0);
            access.delay = new_delay.clone();
            Ok(since)
        })?
    };

    ctx.emit(role_granted(
        role_id,
        account,
        execution_delay,
        since,
        new_member,
    ));

    Ok(new_member)
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
fn _revoke_role(ctx: &mut Ctx, role_id: RoleId, account: &Addr) -> Result<bool, ContractError> {
    if role_id == RoleId::PUBLIC_ROLE {
        return Err(ContractError::AccessManagerLockedRole(role_id));
    }

    match ctx.take::<RoleMembers>(&(role_id, account.clone()))? {
        Some(_) => {
            ctx.emit(role_revoked(role_id, account));
            Ok(true)
        }
        None => Ok(false),
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
fn _set_role_admin(ctx: &mut Ctx, role_id: RoleId, admin: RoleId) -> Result<(), ContractError> {
    if role_id == RoleId::ADMIN_ROLE || role_id == RoleId::PUBLIC_ROLE {
        return Err(ContractError::AccessManagerLockedRole(role_id));
    }

    ctx.update::<Roles, ContractError, _>(&role_id, |role| {
        role.admin = admin;
        Ok(())
    })?;

    ctx.emit(role_admin_changed(role_id, admin));

    Ok(())
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
    ctx: &mut Ctx,
    role_id: RoleId,
    guardian: RoleId,
) -> Result<(), ContractError> {
    if role_id == RoleId::ADMIN_ROLE || role_id == RoleId::PUBLIC_ROLE {
        return Err(ContractError::AccessManagerLockedRole(role_id));
    }

    ctx.update::<Roles, ContractError, _>(&role_id, |role| {
        role.guardian = guardian;
        Ok(())
    })?;

    ctx.emit(role_guardian_changed(role_id, guardian));

    Ok(())
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
fn _set_grant_delay(ctx: &mut Ctx, role_id: RoleId, new_delay: u32) -> Result<(), ContractError> {
    if role_id == RoleId::PUBLIC_ROLE {
        return Err(ContractError::AccessManagerLockedRole(role_id));
    }

    let timestamp = ctx.timestamp();
    let effect = ctx.update::<Roles, ContractError, _>(&role_id, |role| {
        let (new_delay, effect) = role
            .grant_delay
            .with_update(timestamp, new_delay, min_setback());
        role.grant_delay = new_delay.clone();
        Ok(effect)
    })?;

    ctx.emit(role_grant_delay_changed(role_id, new_delay, effect));

    Ok(())
}

// ============================================= FUNCTION MANAGEMENT ==============================================

//     /// ```solidity
//     /// function setTargetFunctionRole(
//     ///     address target,
//     ///     bytes4[] calldata selectors,
//     ///     uint64 roleId
//     /// ) public virtual onlyAuthorized {
//     /// ```
//     ///
//     /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L374-L378>
// fn setTargetFunctionRole(
//     target: &Addr,
//     methods: impl IntoIterator<Item = &Method>,
//     role_id: RoleId
// ) public virtual onlyAuthorized {
//     // for (uint256 i = 0; i < selectors.length; ++i) {
//     //     _setTargetFunctionRole(target, selectors[i], roleId);
//     // }
// }

// /**
//  * @dev Internal version of {setTargetFunctionRole} without access control.
//  *
//  * Emits a {TargetFunctionRoleUpdated} event.
//  */
// fn _setTargetFunctionRole(target: &Addr, method: &Method, role_id: RoleId) internal virtual {
//     _targets[target].allowedRoles[selector] = roleId;
//     emit TargetFunctionRoleUpdated(target, selector, roleId);
// }

// fn setTargetAdminDelay(target: &Addr, uint32 newDelay) public virtual onlyAuthorized {
//     _setTargetAdminDelay(target, newDelay);
// }

// /**
//  * @dev Internal version of {setTargetAdminDelay} without access control.
//  *
//  * Emits a {TargetAdminDelayUpdated} event.
//  */
// fn _setTargetAdminDelay(target: &Addr, uint32 newDelay) internal virtual {
//     uint48 effect;
//     (_targets[target].adminDelay, effect) = _targets[target].adminDelay.withUpdate(newDelay, minSetback());

//     emit TargetAdminDelayUpdated(target, newDelay, effect);
// }

// // =============================================== MODE MANAGEMENT ================================================
// fn setTargetClosed(target: &Addr, bool closed) public virtual onlyAuthorized {
//     _setTargetClosed(target, closed);
// }

// /**
//  * @dev Set the closed flag for a contract. This is an internal setter with no access restrictions.
//  *
//  * Emits a {TargetClosed} event.
//  */
// fn _setTargetClosed(target: &Addr, bool closed) internal virtual {
//     _targets[target].closed = closed;
//     emit TargetClosed(target, closed);
// }

// // ============================================== DELAYED OPERATIONS ==============================================
// fn getSchedule(bytes32 id) public view virtual returns (uint48) {
//     uint48 timepoint = _schedules[id].timepoint;
//     return _isExpired(timepoint) ? 0 : timepoint;
// }

// fn getNonce(bytes32 id) public view virtual returns (uint32) {
//     return _schedules[id].nonce;
// }

// fn schedule(
//     target: &Addr,
//     bytes calldata data,
//     uint48 when
// ) public virtual returns (bytes32 operationId, uint32 nonce) {
//     address caller = _msgSender();

//     // Fetch restrictions that apply to the caller on the targeted function
//     (, uint32 setback) = _canCallExtended(caller, target, data);

//     uint48 minWhen = Time.timestamp() + setback;

//     // If call with delay is not authorized, or if requested timing is too soon, revert
//     if (setback == 0 || (when > 0 && when < minWhen)) {
//         revert AccessManagerUnauthorizedCall(caller, target, _checkSelector(data));
//     }

//     // Reuse variable due to stack too deep
//     when = uint48(Math.max(when, minWhen)); // cast is safe: both inputs are uint48

//     // If caller is authorised, schedule operation
//     operationId = hashOperation(caller, target, data);

//     _checkNotScheduled(operationId);

//     unchecked {
//         // It's not feasible to overflow the nonce in less than 1000 years
//         nonce = _schedules[operationId].nonce + 1;
//     }
//     _schedules[operationId].timepoint = when;
//     _schedules[operationId].nonce = nonce;
//     emit OperationScheduled(operationId, nonce, when, caller, target, data);

//     // Using named return values because otherwise we get stack too deep
// }

// /**
//  * @dev Reverts if the operation is currently scheduled and has not expired.
//  *
//  * NOTE: This fn was introduced due to stack too deep errors in schedule.
//  */
// fn _checkNotScheduled(bytes32 operationId) private view {
//     uint48 prevTimepoint = _schedules[operationId].timepoint;
//     if (prevTimepoint != 0 && !_isExpired(prevTimepoint)) {
//         revert AccessManagerAlreadyScheduled(operationId);
//     }
// }

// // Reentrancy is not an issue because permissions are checked on msg.sender. Additionally,
// // _consumeScheduledOp guarantees a scheduled operation is only executed once.
// // slither-disable-next-line reentrancy-no-eth
// fn execute(target: &Addr, bytes calldata data) public payable virtual returns (uint32) {
//     address caller = _msgSender();

//     // Fetch restrictions that apply to the caller on the targeted function
//     (bool immediate, uint32 setback) = _canCallExtended(caller, target, data);

//     // If call is not authorized, revert
//     if (!immediate && setback == 0) {
//         revert AccessManagerUnauthorizedCall(caller, target, _checkSelector(data));
//     }

//     bytes32 operationId = hashOperation(caller, target, data);
//     uint32 nonce;

//     // If caller is authorised, check operation was scheduled early enough
//     // Consume an available schedule even if there is no currently enforced delay
//     if (setback != 0 || getSchedule(operationId) != 0) {
//         nonce = _consumeScheduledOp(operationId);
//     }

//     // Mark the target and selector as authorised
//     bytes32 executionIdBefore = _executionId;
//     _executionId = _hashExecutionId(target, _checkSelector(data));

//     // Perform call
//     Address.functionCallWithValue(target, data, msg.value);

//     // Reset execute identifier
//     _executionId = executionIdBefore;

//     return nonce;
// }

// fn cancel(address caller, target: &Addr, bytes calldata data) public virtual returns (uint32) {
//     address msgsender = _msgSender();
//     method: &Method = _checkSelector(data);

//     bytes32 operationId = hashOperation(caller, target, data);
//     if (_schedules[operationId].timepoint == 0) {
//         revert AccessManagerNotScheduled(operationId);
//     } else if (caller != msgsender) {
//         // calls can only be canceled by the account that scheduled them, a global admin, or by a guardian of the required role.
//         (bool isAdmin, ) = hasRole(ADMIN_ROLE, msgsender);
//         (bool isGuardian, ) = hasRole(getRoleGuardian(getTargetFunctionRole(target, selector)), msgsender);
//         if (!isAdmin && !isGuardian) {
//             revert AccessManagerUnauthorizedCancel(msgsender, caller, target, selector);
//         }
//     }

//     delete _schedules[operationId].timepoint; // reset the timepoint, keep the nonce
//     uint32 nonce = _schedules[operationId].nonce;
//     emit OperationCanceled(operationId, nonce);

//     return nonce;
// }

// fn consumeScheduledOp(address caller, bytes calldata data) public virtual {
//     target: &Addr = _msgSender();
//     if (IAccessManaged(target).isConsumingScheduledOp() != IAccessManaged.isConsumingScheduledOp.selector) {
//         revert AccessManagerUnauthorizedConsume(target);
//     }
//     _consumeScheduledOp(hashOperation(caller, target, data));
// }

/// Internal variant of {consumeScheduledOp} that operates on bytes32 operationId.
///
/// Returns the nonce of the scheduled operation that is consumed.
///
/// ```solidity
/// function _consumeScheduledOp(bytes32 operationId) internal virtual returns (uint32)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L563>
fn _consume_scheduled_op(ctx: &mut Ctx, operation_id: H256) -> Result<u32, ContractError> {
    let Schedule { timepoint, nonce } = ctx.read::<Schedules>(&operation_id)?;

    if timepoint == 0 {
        return Err(ContractError::AccessManagerNotScheduled(operation_id));
    } else if timepoint > ctx.timestamp() {
        return Err(ContractError::AccessManagerNotReady(operation_id));
    } else if _is_expired(ctx, timepoint) {
        return Err(ContractError::AccessManagerExpired(operation_id));
    }

    // reset the timepoint, keep the nonce
    ctx.write::<Schedules>(
        &operation_id,
        &Schedule {
            timepoint: 0,
            nonce,
        },
    );

    ctx.emit(operation_executed(operation_id, nonce));

    Ok(nonce)
}

/// Hashing function for execute protection.
///
/// ```solidity
/// function hashOperation(address caller, address target, bytes calldata data) public view virtual returns (bytes32)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L582>
fn hash_operation(caller: &Addr, target: &Addr, data: &ExecuteMsg) -> H256 {
    Sha256::digest(format!(
        "{}/{}/{}",
        caller,
        target,
        serde_json_wasm::to_string(data).expect("infallible")
    ))
    .into()
}

// ==================================================== OTHERS ====================================================
// /// @inheritdoc IAccessManager
// function updateAuthority(address target, address newAuthority) public virtual onlyAuthorized {
//     IAccessManaged(target).setAuthority(newAuthority);
// }

// ================================================= ADMIN LOGIC ==================================================

/// Check if the current call is authorized according to admin and roles logic.
///
/// WARNING: Carefully review the considerations of {AccessManaged-restricted} since they apply to this modifier.
///
/// ```solidity
/// function _checkAuthorized() private
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L598>
fn _check_authorized(ctx: &mut Ctx) -> Result<(), ContractError> {
    let (immediate, delay) = _can_call_self(ctx, ctx.msg_sender(), ctx.msg_data())?;

    if !immediate {
        if delay == 0 {
            let (_, required_role, _) = _get_admin_restrictions(ctx, ctx.msg_data())?;
            return Err(ContractError::AccessManagerUnauthorizedAccount {
                msg_sender: ctx.msg_sender().clone(),
                role_id: required_role,
            });
        } else {
            _consume_scheduled_op(
                ctx,
                hash_operation(ctx.msg_sender(), ctx.address_this(), ctx.msg_data()),
            )?;
        }
    }

    Ok(())
}

/// Get the admin restrictions of a given function call based on the function and arguments involved.
///
/// Returns:
///  - bool restricted: does this data match a restricted operation
///  - uint64: which role is this operation restricted to
///  - uint32: minimum delay to enforce for that operation (max between operation's delay and admin's execution delay)
///
/// ```solidity
/// function _getAdminRestrictions(
///     bytes calldata data
/// ) private view returns (bool adminRestricted, uint64 roleAdminId, uint32 executionDelay)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L619-L621>
fn _get_admin_restrictions(
    ctx: &mut Ctx,
    data: &ExecuteMsg,
) -> Result<(bool, RoleId, u32), ContractError> {
    use ExecuteMsg::*;

    match data {
        // Restricted to ADMIN with no delay beside any execution delay the caller may have
        LabelRole { .. }
        | SetRoleAdmin { .. }
        | SetRoleGuardian { .. }
        | SetGrantDelay { .. }
        | SetTargetAdminDelay { .. } => Ok((true, RoleId::ADMIN_ROLE, 0)),

        // Restricted to ADMIN with the admin delay corresponding to the target
        UpdateAuthority { target, .. }
        | SetTargetClosed { target, .. }
        | SetTargetFunctionRole { target, .. } => {
            // First argument is a target.
            // let target = abi.decode(data[0x04:0x24], (address));
            let delay = get_target_admin_delay(ctx, target)?;
            Ok((true, RoleId::ADMIN_ROLE, delay))
        }

        // Restricted to that role's admin with no delay beside any execution delay the caller may have.
        GrantRole { role_id, .. } | RevokeRole { role_id, .. } => {
            Ok((true, get_role_admin(ctx, *role_id)?, 0))
        }

        _ => Ok((
            false,
            get_target_function_role(ctx, ctx.address_this(), &data.selector())?,
            0,
        )),
    }
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

/// A version of {canCall} that checks for restrictions in this contract.
///
/// ```solidity
/// function _canCallSelf(address caller, bytes calldata data) private view returns (bool immediate, uint32 delay)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L685>
fn _can_call_self(
    ctx: &mut Ctx,
    caller: &Addr,
    data: &ExecuteMsg,
) -> Result<(bool, u32), ContractError> {
    if caller == ctx.address_this() {
        // Caller is AccessManager, this means the call was sent through {execute} and it already checked
        // permissions. We verify that the call "identifier", which is set during {execute}, is correct.
        return Ok((is_executing(ctx, ctx.address_this(), &data.selector())?, 0));
    }

    let (admin_restricted, role_id, operation_delay) = _get_admin_restrictions(ctx, data)?;

    // isTargetClosed apply to non-admin-restricted function
    if !admin_restricted && is_target_closed(ctx, ctx.address_this())? {
        return Ok((false, 0));
    }

    let HasRole {
        is_member,
        execution_delay,
    } = has_role(ctx, role_id, caller)?;

    if !is_member {
        return Ok((false, 0));
    }

    let delay = cmp::max(operation_delay, execution_delay);

    Ok((delay == 0, delay))
}

// /**
//  * @dev Returns true if a call with `target` and `selector` is being executed via {executed}.
//  */
// function _isExecuting(address target, bytes4 selector) private view returns (bool) {
//     return _executionId == _hashExecutionId(target, selector);
// }

/// Returns true if a schedule timepoint is past its expiration deadline.
///
/// ```solidity
/// function _isExpired(uint48 timepoint) private view returns (bool)
/// ```
fn _is_expired(ctx: &Ctx, timepoint: u64) -> bool {
    timepoint + (expiration() as u64) <= ctx.timestamp()
}
