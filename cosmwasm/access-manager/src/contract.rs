use std::cmp;

#[cfg(doc)]
use access_manager_types::manager::{event::*, msg::QueryMsg};
use access_manager_types::{
    Access, CanCall, FullAccess, HasRole, Role, RoleId, Schedule, Selector, TargetConfig, managed,
    manager::{
        error::AccessManagerError,
        event::{
            OperationCanceled, OperationExecuted, OperationScheduled, RoleAdminChanged,
            RoleGrantDelayChanged, RoleGranted, RoleGuardianChanged, RoleLabel, RoleRevoked,
            TargetAdminDelayUpdated, TargetClosed, TargetFunctionRoleUpdated,
        },
        msg::ExecuteMsg,
    },
    time::{Delay, UnpackedDelay},
};
use cosmwasm_std::{Addr, StdError, SubMsg, WasmMsg, from_json, wasm_execute};
use depolama::StorageExt;
use sha2::{Digest, Sha256};
use unionlabs_primitives::H256;

use crate::{
    EXECUTE_REPLY_ID,
    context::{ExecCtx, HasStorage, IExecCtx, IQueryCtx, QueryCtx},
    error::ContractError,
    state::{ExecutionIdStack, RoleMembers, Roles, Schedules, TargetAllowedRoles, Targets},
};

/// Check that the caller is authorized to perform the operation.
/// See [`access-manager`][crate] description for a detailed breakdown of the authorization logic.
///
/// ```solidity
/// modifier onlyAuthorized()
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L123>
#[inline]
fn only_authorized(ctx: &mut ExecCtx) -> Result<(), ContractError> {
    _check_authorized(ctx)
}

// ======================================= ROLE MANAGEMENT =======================================

/// See [`ExecuteMsg::LabelRole`].
pub(crate) fn label_role(ctx: &mut ExecCtx, role_id: RoleId, label: &str) {
    // TODO: figure out how we want to label roles; events like the original solidity implementation
    // or in storage?
    ctx.emit(RoleLabel { role_id, label });
}

/// See [`ExecuteMsg::GrantRole`].
pub(crate) fn grant_role(
    ctx: &mut ExecCtx,
    role_id: RoleId,
    account: &Addr,
    execution_delay: u32,
) -> Result<(), ContractError> {
    only_authorized(ctx)?;

    _grant_role(
        ctx,
        role_id,
        account,
        get_role_grant_delay(ctx.query_ctx(), role_id)?,
        execution_delay,
    )?;

    Ok(())
}

/// See [`ExecuteMsg::RevokeRole`].
pub(crate) fn revoke_role(
    ctx: &mut ExecCtx,
    role_id: RoleId,
    account: &Addr,
) -> Result<(), ContractError> {
    only_authorized(ctx)?;

    _revoke_role(ctx, role_id, account)?;

    Ok(())
}

/// See [`ExecuteMsg::RenounceRole`].
pub(crate) fn renounce_role(
    ctx: &mut ExecCtx,
    role_id: RoleId,
    caller_confirmation: &Addr,
) -> Result<(), ContractError> {
    if caller_confirmation == ctx.msg_sender() {
        _revoke_role(ctx, role_id, caller_confirmation)?;

        Ok(())
    } else {
        Err(AccessManagerError::AccessManagerBadConfirmation.into())
    }
}

/// See [`ExecuteMsg::SetRoleAdmin`].
pub(crate) fn set_role_admin(
    ctx: &mut ExecCtx,
    role_id: RoleId,
    admin: RoleId,
) -> Result<(), ContractError> {
    only_authorized(ctx)?;

    _set_role_admin(ctx, role_id, admin)
}

/// See [`ExecuteMsg::SetRoleGuardian`].
pub(crate) fn set_role_guardian(
    ctx: &mut ExecCtx,
    role_id: RoleId,
    guardian: RoleId,
) -> Result<(), ContractError> {
    only_authorized(ctx)?;

    _set_role_guardian(ctx, role_id, guardian)
}

/// See [`ExecuteMsg::SetGrantDelay`].
pub(crate) fn set_grant_delay(
    ctx: &mut ExecCtx,
    role_id: RoleId,
    new_delay: u32,
) -> Result<(), ContractError> {
    only_authorized(ctx)?;

    _set_grant_delay(ctx, role_id, new_delay)
}

/// Internal version of [`grant_role`] without access control. Returns true if the role was newly
/// granted.
///
/// Emits a [`RoleGranted`] event.
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
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L270>
pub(crate) fn _grant_role(
    ctx: &mut ExecCtx,
    role_id: RoleId,
    account: &Addr,
    grant_delay: u32,
    execution_delay: u32,
) -> Result<bool, ContractError> {
    if role_id == RoleId::PUBLIC_ROLE {
        return Err(AccessManagerError::AccessManagerLockedRole(role_id).into());
    }

    let new_member = ctx
        .storage()
        .maybe_read::<RoleMembers>(&(role_id, account.clone()))?
        .is_none();

    let role_members_key = (role_id, account.clone());

    let since = if new_member {
        let since = ctx.timestamp() + u64::from(grant_delay);
        ctx.storage().write::<RoleMembers>(
            &role_members_key,
            &Access {
                since,
                delay: Delay::new(execution_delay),
            },
        );
        since
    } else {
        let timestamp = ctx.timestamp();
        ctx.storage()
            .update::<RoleMembers, ContractError, _>(&role_members_key, |access| {
                let (new_delay, since) = access.delay.with_update(timestamp, execution_delay, 0);
                access.delay = new_delay.clone();
                Ok(since)
            })?
    };

    ctx.emit(RoleGranted {
        role_id,
        account,
        delay: execution_delay,
        since,
        new_member,
    });

    Ok(new_member)
}

// Internal version of [`revoke_role`] without access control. This logic is also used by
// {renounceRole}. Returns true if the role was previously granted.
///
/// Emits a [`RoleRevoked`] event if the account had the role.
///
/// ```solidity
/// function _revokeRole(uint64 roleId, address account) internal virtual returns (bool)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L305>
fn _revoke_role(ctx: &mut ExecCtx, role_id: RoleId, account: &Addr) -> Result<bool, ContractError> {
    if role_id == RoleId::PUBLIC_ROLE {
        return Err(AccessManagerError::AccessManagerLockedRole(role_id).into());
    }

    match ctx
        .storage()
        .take::<RoleMembers>(&(role_id, account.clone()))?
    {
        Some(_) => {
            ctx.emit(RoleRevoked { role_id, account });
            Ok(true)
        }
        None => Ok(false),
    }
}

/// Internal version of [`set_role_admin`] without access control.
///
/// Emits a [`RoleAdminChanged`] event.
///
/// NOTE: Setting the admin role as the [`RoleId::PUBLIC_ROLE`] is allowed, but it will effectively
/// allow anyone to set grant or revoke such role.
///
/// ```solidity
/// function _setRoleAdmin(uint64 roleId, uint64 admin) internal virtual
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L328>
fn _set_role_admin(ctx: &mut ExecCtx, role_id: RoleId, admin: RoleId) -> Result<(), ContractError> {
    if role_id == RoleId::ADMIN_ROLE || role_id == RoleId::PUBLIC_ROLE {
        return Err(AccessManagerError::AccessManagerLockedRole(role_id).into());
    }

    ctx.storage()
        .upsert::<Roles, ContractError>(&role_id, |role| {
            Ok(Role {
                admin,
                ..role.unwrap_or_default()
            })
        })?;

    ctx.emit(RoleAdminChanged { role_id, admin });

    Ok(())
}

/// Internal version of [`set_role_guardian`] without access control.
///
/// Emits a [`RoleGuardianChanged`] event.
///
/// NOTE: Setting the guardian role as the [`RoleId::PUBLIC_ROLE`] is allowed, but it will
/// effectively allow anyone to cancel any scheduled operation for such role.
///
/// ```solidity
/// function _setRoleGuardian(uint64 roleId, uint64 guardian) internal virtual
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L346>
fn _set_role_guardian(
    ctx: &mut ExecCtx,
    role_id: RoleId,
    guardian: RoleId,
) -> Result<(), ContractError> {
    if role_id == RoleId::ADMIN_ROLE || role_id == RoleId::PUBLIC_ROLE {
        return Err(AccessManagerError::AccessManagerLockedRole(role_id).into());
    }

    ctx.storage()
        .upsert::<Roles, ContractError>(&role_id, |role| {
            Ok(Role {
                guardian,
                ..role.unwrap_or_default()
            })
        })?;

    ctx.emit(RoleGuardianChanged { role_id, guardian });

    Ok(())
}

/// Internal version of [`set_grant_delay`] without access control.
///  
/// Emits a [`RoleGrantDelayChanged`] event.
///
/// ```solidity
/// function _setGrantDelay(uint64 roleId, uint32 newDelay) internal virtual
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L361>
fn _set_grant_delay(
    ctx: &mut ExecCtx,
    role_id: RoleId,
    new_delay: u32,
) -> Result<(), ContractError> {
    if role_id == RoleId::PUBLIC_ROLE {
        return Err(AccessManagerError::AccessManagerLockedRole(role_id).into());
    }

    let timestamp = ctx.timestamp();

    let mut effect = 0;

    ctx.storage()
        .upsert::<Roles, ContractError>(&role_id, |role| {
            let role = role.unwrap_or_default();

            let new_grant_delay: Delay;
            (new_grant_delay, effect) =
                role.grant_delay
                    .with_update(timestamp, new_delay, min_setback());

            Ok(Role {
                grant_delay: new_grant_delay,
                ..role
            })
        })?;

    ctx.emit(RoleGrantDelayChanged {
        role_id,
        delay: new_delay,
        since: effect,
    });

    Ok(())
}

// ===================================== FUNCTION MANAGEMENT ======================================

/// See [`ExecuteMsg::SetTargetFunctionRole`].
pub(crate) fn set_target_function_role<'a>(
    ctx: &mut ExecCtx,
    target: &Addr,
    selectors: impl IntoIterator<Item = &'a Selector>,
    role_id: RoleId,
) -> Result<(), ContractError> {
    only_authorized(ctx)?;

    for selector in selectors {
        _set_target_function_role(ctx, target, selector, role_id)?;
    }

    Ok(())
}

/// Internal version of [`set_target_function_role`] without access control.
///
/// Emits a [`TargetFunctionRoleUpdated`] event.
fn _set_target_function_role(
    ctx: &mut ExecCtx,
    target: &Addr,
    selector: &Selector,
    role_id: RoleId,
) -> Result<(), ContractError> {
    if selector.is_internal() {
        return Err(ContractError::AccessManager(
            AccessManagerError::InternalSelector(selector.to_owned()),
        ));
    }

    ctx.storage()
        .write::<TargetAllowedRoles>(&(target.clone(), selector.to_owned()), &role_id);

    ctx.emit(TargetFunctionRoleUpdated {
        target,
        selector,
        role_id,
    });

    Ok(())
}

/// See [`ExecuteMsg::SetTargetAdminDelay`].
pub(crate) fn set_target_admin_delay(
    ctx: &mut ExecCtx,
    target: &Addr,
    new_delay: u32,
) -> Result<(), ContractError> {
    only_authorized(ctx)?;

    _set_target_admin_delay(ctx, target, new_delay)?;

    Ok(())
}

/// Internal version of [`set_target_admin_delay`] without access control.
///
/// Emits a [`TargetAdminDelayUpdated`] event.
///
/// ```solidity
/// function _setTargetAdminDelay(address target, uint32 newDelay) internal virtual
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L404>
fn _set_target_admin_delay(
    ctx: &mut ExecCtx,
    target: &Addr,
    new_delay: u32,
) -> Result<(), ContractError> {
    let timestamp = ctx.timestamp();

    let mut effect = 0;

    ctx.storage()
        .upsert::<Targets, ContractError>(target, |maybe_target_config| {
            let mut target_config = maybe_target_config.unwrap_or_default();

            let delay;

            (delay, effect) =
                target_config
                    .admin_delay
                    .with_update(timestamp, new_delay, min_setback());

            target_config.admin_delay = delay;

            Ok(target_config)
        })?;

    ctx.emit(TargetAdminDelayUpdated {
        target,
        delay: new_delay,
        since: effect,
    });

    Ok(())
}

// ======================================= MODE MANAGEMENT ========================================

/// See [`ExecuteMsg::SetTargetClosed`].
pub(crate) fn set_target_closed(
    ctx: &mut ExecCtx,
    target: &Addr,
    closed: bool,
) -> Result<(), ContractError> {
    only_authorized(ctx)?;

    _set_target_closed(ctx, target, closed)?;

    Ok(())
}

/// Set the closed flag for a contract. This is an internal setter with no access restrictions.
///
/// Emits a [`TargetClosed`] event.
///
/// ```solidity
/// function _setTargetClosed(address target, bool closed) internal virtual
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L422>
fn _set_target_closed(ctx: &mut ExecCtx, target: &Addr, closed: bool) -> Result<(), ContractError> {
    ctx.storage()
        .upsert::<Targets, ContractError>(target, |maybe_target_config| {
            Ok(TargetConfig {
                closed,
                ..maybe_target_config.unwrap_or_default()
            })
        })?;

    ctx.emit(TargetClosed { target, closed });

    Ok(())
}

// ====================================== DELAYED OPERATIONS ======================================

/// See [`QueryMsg::GetSchedule`].
pub(crate) fn get_schedule(ctx: QueryCtx, id: H256) -> Result<u64, ContractError> {
    let timepoint = ctx
        .storage()
        .maybe_read::<Schedules>(&id)?
        .map(|s| s.timepoint)
        .unwrap_or_default();

    Ok(if _is_expired(ctx, timepoint) {
        0
    } else {
        timepoint
    })
}

/// See [`QueryMsg::GetNonce`].
pub(crate) fn get_nonce(ctx: QueryCtx, id: H256) -> Result<u32, ContractError> {
    Ok(ctx
        .storage()
        .maybe_read::<Schedules>(&id)?
        .map(|s| s.nonce)
        .unwrap_or_default())
}

/// See [`ExecuteMsg::Schedule`].
pub(crate) fn schedule(
    ctx: &mut ExecCtx,
    target: &Addr,
    data: &str,
    when: u64,
) -> Result<(H256, u32), ContractError> {
    let caller = ctx.msg_sender();

    // Fetch restrictions that apply to the caller on the targeted function
    let CanCall {
        allowed: _,
        delay: setback,
    } = _can_call_extended(ctx, caller, target, data)?;

    let min_when = ctx.timestamp() + u64::from(setback);

    // If call with delay is not authorized, or if requested timing is too soon, revert
    if setback == 0 || (when > 0 && when < min_when) {
        return Err(AccessManagerError::AccessManagerUnauthorizedCall {
            caller: caller.clone(),
            target: target.clone(),
            selector: _check_selector(data)?.to_owned(),
        }
        .into());
    }

    let when = cmp::max(when, min_when);

    // If caller is authorized, schedule operation
    let operation_id = hash_operation(caller, target, data);

    let maybe_schedule = ctx.storage().maybe_read::<Schedules>(&operation_id)?;

    // inlined _checkNotScheduled
    if let Some(Schedule {
        timepoint: prev_timepoint,
        ..
    }) = maybe_schedule
        && !_is_expired(ctx.query_ctx(), prev_timepoint)
    {
        return Err(AccessManagerError::AccessManagerAlreadyScheduled(operation_id).into());
    }

    let schedule = Schedule {
        timepoint: when,
        nonce: maybe_schedule.map(|s| s.nonce).unwrap_or_default() + 1,
    };

    ctx.storage().write::<Schedules>(&operation_id, &schedule);

    ctx.emit(OperationScheduled {
        operation_id,
        nonce: schedule.nonce,
        schedule: when,
        caller,
        target,
        data,
    });

    Ok((operation_id, schedule.nonce))
}

/// See [`ExecuteMsg::Execute`].
// NOTE: Reentrancy is not an issue because permissions are checked on info.sender. Additionally,
// _consume_scheduled_op guarantees a scheduled operation is only executed once.
pub(crate) fn execute(
    ctx: &mut ExecCtx,
    target: &Addr,
    data: &str,
) -> Result<(SubMsg, Option<u32>), ContractError> {
    let caller = ctx.msg_sender();

    // Fetch restrictions that apply to the caller on the targeted function
    let CanCall {
        allowed: immediate,
        delay: setback,
    } = _can_call_extended(ctx, caller, target, data)?;

    // If call is not authorized, revert
    if !immediate && setback == 0 {
        return Err(AccessManagerError::AccessManagerUnauthorizedCall {
            caller: caller.clone(),
            target: target.clone(),
            selector: _check_selector(data)?.to_owned(),
        }
        .into());
    }

    let operation_id = hash_operation(caller, target, data);
    let mut nonce = None;

    // If caller is authorized, check operation was scheduled early enough
    // Consume an available schedule even if there is no currently enforced delay
    if setback != 0 || get_schedule(ctx.query_ctx(), operation_id)? != 0 {
        nonce = Some(_consume_scheduled_op(ctx, operation_id)?);
    }

    // Mark the target and selector as authorized
    ctx.storage()
        .update_item::<ExecutionIdStack, ContractError, _>(|stack| {
            // this gets popped in the reply handler
            stack.push(_hash_execution_id(target, _check_selector(data)?));

            Ok(())
        })?;

    Ok((
        SubMsg::reply_on_success(
            WasmMsg::Execute {
                contract_addr: target.to_string(),
                msg: data.as_bytes().into(),
                funds: ctx.value().to_vec(),
            },
            EXECUTE_REPLY_ID,
        ),
        nonce,
    ))
}

/// See [`ExecuteMsg::Cancel`].
pub(crate) fn cancel(
    ctx: &mut ExecCtx,
    caller: &Addr,
    target: &Addr,
    data: &str,
) -> Result<u32, ContractError> {
    let msgsender = ctx.msg_sender();
    let selector = _check_selector(data)?;

    let operation_id = hash_operation(caller, target, data);
    let schedule = ctx
        .storage()
        .maybe_read::<Schedules>(&operation_id)?
        .ok_or(AccessManagerError::AccessManagerNotScheduled(operation_id))?;

    if schedule.timepoint == 0 {
        return Err(AccessManagerError::AccessManagerNotScheduled(operation_id).into());
    } else if caller != msgsender {
        // calls can only be canceled by the account that scheduled them, a global admin, or by a
        // guardian of the required role.
        let is_admin = has_role(ctx.query_ctx(), RoleId::ADMIN_ROLE, msgsender)?.is_member;
        let is_guardian = has_role(
            ctx.query_ctx(),
            get_role_guardian(
                ctx.query_ctx(),
                get_target_function_role(ctx.query_ctx(), target, selector)?,
            )?,
            msgsender,
        )?
        .is_member;

        if !is_admin && !is_guardian {
            return Err(AccessManagerError::AccessManagerUnauthorizedCancel {
                msg_sender: msgsender.clone(),
                caller: caller.clone(),
                target: target.clone(),
                selector: selector.to_owned(),
            }
            .into());
        }
    }

    // reset the timepoint, keep the nonce
    ctx.storage().write::<Schedules>(
        &operation_id,
        &Schedule {
            timepoint: 0,
            ..schedule
        },
    );

    ctx.emit(OperationCanceled {
        operation_id,
        nonce: schedule.nonce,
    });

    Ok(schedule.nonce)
}

/// See [`ExecuteMsg::ConsumeScheduledOp`].
pub(crate) fn consume_scheduled_op(
    ctx: &mut ExecCtx,
    caller: &Addr,
    data: &str,
) -> Result<(), ContractError> {
    let target = ctx.msg_sender();

    // idrk what's going on here ngl
    let selector = ctx.querier().query_wasm_smart::<Box<Selector>>(
        target,
        &managed::msg::QueryMsg::IsConsumingScheduledOp {},
    )?;
    if (&*selector != managed::msg::QueryMsg::IsConsumingScheduledOp {}.selector()) {
        return Err(AccessManagerError::AccessManagerUnauthorizedConsume {
            target: target.clone(),
        }
        .into());
    }

    _consume_scheduled_op(ctx, hash_operation(caller, target, data))?;

    Ok(())
}

/// Internal variant of [`consume_scheduled_op`] that operates on bytes32 operationId.
///
/// Returns the nonce of the scheduled operation that is consumed.
///
/// ```solidity
/// function _consumeScheduledOp(bytes32 operationId) internal virtual returns (uint32)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L563>
fn _consume_scheduled_op(ctx: &mut ExecCtx, operation_id: H256) -> Result<u32, ContractError> {
    let Schedule { timepoint, nonce } = ctx
        .storage()
        .maybe_read::<Schedules>(&operation_id)?
        .unwrap_or_default();

    if timepoint == 0 {
        return Err(AccessManagerError::AccessManagerNotScheduled(operation_id).into());
    } else if timepoint > ctx.timestamp() {
        return Err(AccessManagerError::AccessManagerNotReady(operation_id).into());
    } else if _is_expired(ctx.query_ctx(), timepoint) {
        return Err(AccessManagerError::AccessManagerExpired(operation_id).into());
    }

    // reset the timepoint, keep the nonce
    ctx.storage().write::<Schedules>(
        &operation_id,
        &Schedule {
            timepoint: 0,
            nonce,
        },
    );

    ctx.emit(OperationExecuted {
        operation_id,
        nonce,
    });

    Ok(nonce)
}

/// See [`QueryMsg::HashOperation`].
pub(crate) fn hash_operation(caller: &Addr, target: &Addr, data: &str) -> H256 {
    Sha256::digest(format!("{caller}/{target}/{data}")).into()
}

// ============================================ OTHERS ============================================

/// See [`ExecuteMsg::UpdateAuthority`].
pub(crate) fn update_authority(
    ctx: &mut ExecCtx,
    target: &Addr,
    new_authority: &Addr,
) -> Result<SubMsg, ContractError> {
    only_authorized(ctx)?;

    Ok(SubMsg::reply_never(wasm_execute(
        target,
        &managed::msg::ExecuteMsg::SetAuthority {
            new_authority: new_authority.clone(),
        },
        vec![],
    )?))
}

// ========================================= ADMIN LOGIC ==========================================

/// Check if the current call is authorized according to admin and roles logic.
///
/// WARNING: Carefully review the considerations of {AccessManaged-restricted} since they apply to
/// this modifier.
///
/// ```solidity
/// function _checkAuthorized() private
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L598>
#[inline]
fn _check_authorized(ctx: &mut ExecCtx) -> Result<(), ContractError> {
    let msg_data = ctx.msg_data();

    let CanCall {
        allowed: immediate,
        delay,
    } = _can_call_self(ctx, ctx.msg_sender(), &msg_data)?;

    if !immediate {
        if delay == 0 {
            let (_, required_role, _) = _get_admin_restrictions(ctx, &msg_data)?;
            return Err(AccessManagerError::AccessManagerUnauthorizedAccount {
                msg_sender: ctx.msg_sender().clone(),
                required_role_id: required_role,
            }
            .into());
        }

        _consume_scheduled_op(
            ctx,
            hash_operation(ctx.msg_sender(), ctx.address_this(), &msg_data),
        )?;
    }

    Ok(())
}

/// Get the admin restrictions of a given function call based on the function and arguments
/// involved.
///
/// Returns:
///  - bool restricted: does this data match a restricted operation
///  - uint64: which role is this operation restricted to
///  - uint32: minimum delay to enforce for that operation (max between operation's delay and
///    admin's execution delay)
///
/// ```solidity
/// function _getAdminRestrictions(
///     bytes calldata data
/// ) private view returns (bool adminRestricted, uint64 roleAdminId, uint32 executionDelay)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L619>
fn _get_admin_restrictions(
    ctx: &mut ExecCtx,
    data: &str,
) -> Result<(bool, RoleId, u32), ContractError> {
    use ExecuteMsg::*;

    match from_json(data) {
        // Restricted to ADMIN with no delay beside any execution delay the caller may have
        Ok(
            LabelRole { .. }
            | SetRoleAdmin { .. }
            | SetRoleGuardian { .. }
            | SetGrantDelay { .. }
            | SetTargetAdminDelay { .. },
        ) => Ok((true, RoleId::ADMIN_ROLE, 0)),

        // Restricted to ADMIN with the admin delay corresponding to the target
        Ok(
            UpdateAuthority { target, .. }
            | SetTargetClosed { target, .. }
            | SetTargetFunctionRole { target, .. },
        ) => {
            let delay = get_target_admin_delay(ctx.query_ctx(), &target)?;
            Ok((true, RoleId::ADMIN_ROLE, delay))
        }

        // Restricted to that role's admin with no delay beside any execution delay the caller may
        // have.
        Ok(GrantRole { role_id, .. } | RevokeRole { role_id, .. }) => {
            Ok((true, get_role_admin(ctx.query_ctx(), role_id)?, 0))
        }

        _ => Ok((
            false,
            get_target_function_role(ctx.query_ctx(), ctx.address_this(), _check_selector(data)?)?,
            0,
        )),
    }
}

/// Extracts the selector from calldata. Returns an error if there is no selector in the provided
/// JSON data.
///
/// ```solidity
/// function _checkSelector(bytes calldata data) private pure returns (bytes4)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L730>
fn _check_selector(data: &str) -> Result<&Selector, ContractError> {
    Selector::extract_from_data(data)
        .map_err(|e| StdError::generic_err(format!("error extracting selector: {e}")).into())
}

// =========================================== HELPERS ============================================

/// An extended version of [`can_call`] for internal usage that checks [`_can_call_self`]
/// when the target is this contract.
///
/// Returns:
/// - bool immediate: whether the operation can be executed immediately (with no delay)
/// - uint32 delay: the execution delay
///
/// ```solidity
/// function _canCallExtended(
///     address caller,
///     address target,
///     bytes calldata data
/// ) private view returns (bool immediate, uint32 delay)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L670>
fn _can_call_extended(
    ctx: &mut ExecCtx,
    caller: &Addr,
    target: &Addr,
    data: &str,
) -> Result<CanCall, ContractError> {
    if target == ctx.address_this() {
        _can_call_self(ctx, caller, data)
    } else {
        can_call(ctx.query_ctx(), caller, target, _check_selector(data)?)
    }
}

/// A version of [`can_call`] that checks for restrictions in this contract.
///
/// ```solidity
/// function _canCallSelf(address caller, bytes calldata data) private view returns (bool immediate, uint32 delay)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L685>
fn _can_call_self(ctx: &mut ExecCtx, caller: &Addr, data: &str) -> Result<CanCall, ContractError> {
    if caller == ctx.address_this() {
        // Caller is AccessManager, this means the call was sent through `execute` and it already
        // checked permissions. We verify that the call "identifier", which is set during
        // `execute`, is correct.
        return Ok(CanCall {
            allowed: _is_executing(ctx.query_ctx(), ctx.address_this(), _check_selector(data)?)?,
            delay: 0,
        });
    }

    let (admin_restricted, role_id, operation_delay) = _get_admin_restrictions(ctx, data)?;

    // isTargetClosed apply to non-admin-restricted function
    if !admin_restricted && is_target_closed(ctx.query_ctx(), ctx.address_this())? {
        return Ok(CanCall {
            allowed: false,
            delay: 0,
        });
    }

    let HasRole {
        is_member,
        execution_delay,
    } = has_role(ctx.query_ctx(), role_id, caller)?;

    if !is_member {
        return Ok(CanCall {
            allowed: false,
            delay: 0,
        });
    }

    let delay = cmp::max(operation_delay, execution_delay);

    Ok(CanCall {
        allowed: delay == 0,
        delay,
    })
}

/// Returns true if a call with `target` and `selector` is being executed via [`execute`].
///
/// ```solidity
/// function _isExecuting(address target, bytes4 selector) private view returns (bool)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L716>
pub(crate) fn _is_executing(
    ctx: QueryCtx,
    target: &Addr,
    selector: &Selector,
) -> Result<bool, ContractError> {
    Ok(ctx
        .storage()
        .maybe_read_item::<ExecutionIdStack>()?
        .is_some_and(|stack| {
            stack
                .last()
                .is_some_and(|id| id == &_hash_execution_id(target, selector))
        }))
}

/// Returns true if a schedule timepoint is past its expiration deadline.
///
/// ```solidity
/// function _isExpired(uint48 timepoint) private view returns (bool)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L685>
fn _is_expired(ctx: QueryCtx, timepoint: u64) -> bool {
    timepoint + u64::from(expiration()) <= ctx.timestamp()
}

/// See [`QueryMsg::CanCall`].
pub(crate) fn can_call(
    ctx: QueryCtx,
    caller: &Addr,
    target: &Addr,
    selector: &Selector,
) -> Result<CanCall, ContractError> {
    if selector.is_internal() {
        Ok(CanCall {
            allowed: true,
            delay: 0,
        })
    } else if is_target_closed(ctx, target)? {
        Ok(CanCall {
            allowed: false,
            delay: 0,
        })
    } else if caller == ctx.address_this() {
        // Caller is AccessManager, this means the call was sent through `execute` and it already
        // checked permissions. We verify that the call "identifier", which is set during
        // `execute`, is correct.
        Ok(CanCall {
            allowed: _is_executing(ctx, target, selector)?,
            delay: 0,
        })
    } else {
        let role_id = get_target_function_role(ctx, target, selector)?;
        let HasRole {
            is_member,
            execution_delay,
        } = has_role(ctx, role_id, caller)?;
        if is_member {
            Ok(CanCall {
                allowed: execution_delay == 0,
                delay: execution_delay,
            })
        } else {
            Ok(CanCall {
                allowed: false,
                delay: 0,
            })
        }
    }
}

/// See [`QueryMsg::Expiration`].
pub(crate) fn expiration() -> u32 {
    // 1 week
    7 * 24 * 60 * 60
}

/// See [`QueryMsg::MinSetback`].
pub(crate) fn min_setback() -> u32 {
    // 5 days
    5 * 24 * 60 * 60
}

/// See [`QueryMsg::IsTargetClosed`].
pub(crate) fn is_target_closed(ctx: QueryCtx, target: &Addr) -> Result<bool, ContractError> {
    Ok(ctx
        .storage()
        .maybe_read::<Targets>(target)?
        .unwrap_or_default()
        .closed)
}

/// See [`QueryMsg::GetTargetFunctionRole`].
pub(crate) fn get_target_function_role(
    ctx: QueryCtx,
    target: &Addr,
    selector: &Selector,
) -> Result<RoleId, ContractError> {
    if selector.is_internal() {
        return Err(ContractError::AccessManager(
            AccessManagerError::InternalSelector(selector.to_owned()),
        ));
    }

    Ok(ctx
        .storage()
        .maybe_read::<TargetAllowedRoles>(&(target.clone(), selector.to_owned()))?
        .unwrap_or_default())
}

/// See [`QueryMsg::GetTargetAdminDelay`].
pub(crate) fn get_target_admin_delay(ctx: QueryCtx, target: &Addr) -> Result<u32, ContractError> {
    Ok(ctx
        .storage()
        .maybe_read::<Targets>(target)?
        .unwrap_or_default()
        .admin_delay
        .get(ctx.timestamp()))
}

/// See [`QueryMsg::GetRoleAdmin`].
pub(crate) fn get_role_admin(ctx: QueryCtx, role_id: RoleId) -> Result<RoleId, ContractError> {
    Ok(ctx
        .storage()
        .maybe_read::<Roles>(&role_id)?
        .unwrap_or_default()
        .admin)
}

/// See [`QueryMsg::GetRoleGuardian`].
pub(crate) fn get_role_guardian(ctx: QueryCtx, role_id: RoleId) -> Result<RoleId, ContractError> {
    Ok(ctx
        .storage()
        .maybe_read::<Roles>(&role_id)?
        .unwrap_or_default()
        .guardian)
}

/// See [`QueryMsg::GetRoleGrantDelay`].
pub(crate) fn get_role_grant_delay(ctx: QueryCtx, role_id: RoleId) -> Result<u32, ContractError> {
    Ok(ctx
        .storage()
        .maybe_read::<Roles>(&role_id)?
        .unwrap_or_default()
        .grant_delay
        .get(ctx.timestamp()))
}

/// See [`QueryMsg::GetAccess`].
pub(crate) fn get_access(
    ctx: QueryCtx,
    role_id: RoleId,
    account: &Addr,
) -> Result<FullAccess, ContractError> {
    let access = ctx
        .storage()
        .maybe_read::<RoleMembers>(&(role_id, account.clone()))?
        .unwrap_or_default();

    let UnpackedDelay {
        value_before,
        value_after,
        effect_date,
    } = access.delay.get_full(ctx.timestamp());

    Ok(FullAccess {
        since: access.since,
        current_delay: value_before,
        pending_delay: value_after,
        effect: effect_date,
    })
}

/// See [`QueryMsg::HasRole`].
pub(crate) fn has_role(
    ctx: QueryCtx,
    role_id: RoleId,
    account: &Addr,
) -> Result<HasRole, ContractError> {
    if role_id == RoleId::PUBLIC_ROLE {
        Ok(HasRole {
            is_member: true,
            execution_delay: 0,
        })
    } else {
        let FullAccess {
            since,
            current_delay,
            pending_delay: _,
            effect: _,
        } = get_access(ctx, role_id, account)?;
        Ok(HasRole {
            is_member: since != 0 && since <= ctx.timestamp(),
            execution_delay: current_delay,
        })
    }
}

/// Hashing function for execute protection.
///
/// ```solidity
/// function _hashExecutionId(address target, bytes4 selector) private pure returns (bytes32)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L737>
pub(crate) fn _hash_execution_id(target: &Addr, selector: &Selector) -> H256 {
    sha2::Sha256::digest(format!("{target}:{selector}")).into()
}
