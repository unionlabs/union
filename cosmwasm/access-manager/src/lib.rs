//! CosmWasm implementation of [OpenZeppelin][oz]'s [`AccessManager.sol`][am].
//!
//! `access-manager` is a central contract to store the permissions of a system.
//!
//! A smart contract under the control of an `access-manager` instance is known as a target, and
//! will implement the `access-managed` messages, be connected to this contract as its manager and
//! use the `Restricted<T>` wrapper on a subset of it's `ExecuteMsg` selected to be permissioned.
//! Note that any variants without this setup won't be effectively restricted.
//!
//! The restriction rules for such functions are defined in terms of "roles" identified by a
//! [`RoleId`] and scoped by target ([`Addr`][cosmwasm_std::Addr]) and function selectors
//! ([`Selector`][access_manager_types::Selector]). These roles are stored in this contract and can
//! be configured by admins ([`RoleId::ADMIN_ROLE`] members) after a delay (see
//! [`QueryMsg::GetTargetAdminDelay`]).
//!
//! For each target contract, admins can configure the following without any delay:
//!
//! - The target's [`QueryMsg::Authority`][access_manager_types::managed::msg::QueryMsg::Authority]
//!   via [`ExecuteMsg::UpdateAuthority`].
//! - Close or open a target via [`ExecuteMsg::SetTargetClosed`] keeping the permissions intact.
//! - The roles that are allowed (or disallowed) to call a given function (identified by its
//!   selector) through [`ExecuteMsg::SetTargetAdminDelay`].
//!
//! By default every address is member of the [`RoleId::PUBLIC_ROLE`] and every target function is
//! restricted to the [`RoleId::ADMIN_ROLE`] until configured otherwise. Additionally, each role has
//! the following configuration options restricted to this manager's admins:
//!
//! - A role's admin role via [`ExecuteMsg::SetRoleAdmin`] who can grant or revoke roles.
//! - A role's guardian role via [`ExecuteMsg::SetRoleGuardian`] who's allowed to cancel operations.
//! - A delay in which a role takes effect after being granted through
//!   [`ExecuteMsg::SetGrantDelay`].
//! - A delay of any target's admin action via [`ExecuteMsg::SetTargetAdminDelay`].
//! - A role label for discoverability purposes with [`ExecuteMsg::LabelRole`].
//!
//! Any account can be added and removed into any number of these roles by using the
//! [`ExecuteMsg::GrantRole`] and [`ExecuteMsg::RevokeRole`] functions restricted to each role's
//! admin (see [`QueryMsg::GetRoleAdmin`]).
//!
//! Since all the permissions of the managed system can be modified by the admins of this instance,
//! it is expected that they will be highly secured (e.g., a multisig or a well-configured DAO).
//!
//! # Implementation Differences
//!
//! This implementation attempts to be a faithful, 1:1 reimplementation of the original Solidity
//! source code. The exact structure of the functions, entrypoints, type and parameter names, and
//! business logic have been preserved as much as possible. There are however some instances where
//! this is not possible, due to fundamental differences between the EVM/Solidity and CosmWasm/Rust:
//!
//! - Entrypoints in CosmWasm contracts work significantly differently than in Solidity. In
//!   solidity, `public` and `external` functions are the "entrypoints" to a contract, identified by
//!   their [selector], whereas in CosmWasm, two entrypoints `execute` and `query` are used for all
//!   calls and queries respectively. Within these functions, the calldata passed to the contract is
//!   deserialized as JSON. Typically, for a contract with multiple "methods", an enum is used, with
//!   the default [serde externally tagged enum representation][et]. To reliably handle the
//!   target/selector pattern of the original implementation, we enforce this
//!   `ExecuteMsg`/`QueryMsg` enum pattern to be used by all contracts that will be targets of this
//!   manager. See [`access_manager_types::managed`] for more information.
//! - Storage in CosmWasm functions quite differently than in the EVM. In Solidity, it is possible
//!   to embed a mapping directly in a struct that is stored in storage, which allows for
//!   multi-level deferred storage access. To emulate this behaviour in CosmWasm, a separate storage
//!   item is used explicitly in these cases. See [`state`] and [`access_manager_types`] for
//!   examples.
//! - CosmWasm does not allow for synchronous cross-contract calls, and instead uses a
//!   submessage/reply pattern. As such, nested executions can not be run inline. In the original
//!   Solidity implementation, `_executionId` is used to track the currently executing call, which
//!   is set immediately before executing the subcall and reset immediately after it. To emulate
//!   this behaviour in CosmWasm, we instead use a list (see [`ExecutionIdStack`]) and pop the id in
//!   the reply handler.
//! - CosmWasm does not support modifiers, so modifiers are instead implemented as standalone
//!   functions that are called manually within functions they are applied to in the original
//!   Solidity implementation.
//! - Due to the limitations of Solidity, tuples are often used to emulate enums; this however
//!   results in invalid states being representable. In order to reduce the chance of bugs, and to
//!   make the code more idiomatic and easier to understand, these tuples have been replaced with
//!   enums wherever possible.
//!
//! [oz]: https://www.openzeppelin.com
//! [am]: https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol
//! [et]: https://serde.rs/enum-representations.html#externally-tagged
//! [selector]: https://docs.soliditylang.org/en/latest/abi-spec.html#function-selector

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![warn(clippy::pedantic)]
#![allow(
    clippy::used_underscore_items,
    clippy::missing_errors_doc,
    clippy::enum_glob_use,
    clippy::doc_markdown
)]
#![cfg_attr(not(test), warn(clippy::unwrap_used))]
#![cfg_attr(test, allow(clippy::too_many_lines))]

use std::num::NonZero;

use access_manager_types::{
    RoleId,
    manager::msg::{ExecuteMsg, InitMsg, MigrateMsg, QueryMsg},
};
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, SubMsg, to_json_binary,
};
use depolama::StorageExt;
use frissitheto::UpgradeMsg;
use serde::Serialize;

use crate::{
    context::{ExecCtx, HasStorage, QueryCtx},
    contract::{
        _grant_role, can_call, cancel, consume_scheduled_op, expiration, get_access, get_nonce,
        get_role_admin, get_role_grant_delay, get_role_guardian, get_role_labels, get_schedule,
        get_target_admin_delay, get_target_function_role, grant_role, has_role, hash_operation,
        is_target_closed, label_role, min_setback, renounce_role, revoke_role, schedule,
        set_grant_delay, set_role_admin, set_role_guardian, set_target_admin_delay,
        set_target_closed, set_target_function_role, update_authority,
    },
    error::ContractError,
    state::ExecutionIdStack,
};

pub mod context;
pub mod contract;
pub mod error;
pub mod state;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests;

pub const EXECUTE_REPLY_ID: u64 = 1;

/// ```solidity
/// constructor(address initialAdmin)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L128>
pub fn init(deps: DepsMut, env: &Env, msg: &InitMsg) -> Result<Response, ContractError> {
    let InitMsg { initial_admin } = msg;

    let info = MessageInfo {
        sender: initial_admin.clone(),
        funds: vec![],
    };

    let mut ctx = ExecCtx::new(
        deps,
        env,
        &info,
        // this can technically be whatever, just needs to be passed through
        &(),
    );

    ctx.storage().write_item::<ExecutionIdStack>(&vec![]);

    _grant_role(&mut ctx, RoleId::ADMIN_ROLE, initial_admin, 0, 0)?;

    Ok(Response::new().add_events(ctx.events()))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
#[expect(clippy::needless_pass_by_value, reason = "required for entry_point")]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    _: MessageInfo,
    msg: InitMsg,
) -> Result<Response, ContractError> {
    frissitheto::init_state_version(&mut deps, const { <NonZero<u32>>::new(1).unwrap() })?;

    init(deps, &env, &msg)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
#[expect(clippy::needless_pass_by_value, reason = "required for entry_point")]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let mut msgs: Vec<SubMsg> = vec![];
    let mut ctx = ExecCtx::new(deps, &env, &info, &msg);
    let mut response_data = None;

    match &msg {
        ExecuteMsg::LabelRole { role_id, label } => {
            label_role(&mut ctx, *role_id, label)?;
        }
        ExecuteMsg::GrantRole {
            role_id,
            account,
            execution_delay,
        } => {
            grant_role(&mut ctx, *role_id, account, *execution_delay)?;
        }
        ExecuteMsg::RevokeRole { role_id, account } => {
            revoke_role(&mut ctx, *role_id, account)?;
        }
        ExecuteMsg::RenounceRole {
            role_id,
            caller_confirmation,
        } => {
            renounce_role(&mut ctx, *role_id, caller_confirmation)?;
        }
        ExecuteMsg::SetRoleAdmin { role_id, admin } => {
            set_role_admin(&mut ctx, *role_id, *admin)?;
        }
        ExecuteMsg::SetRoleGuardian { role_id, guardian } => {
            set_role_guardian(&mut ctx, *role_id, *guardian)?;
        }
        ExecuteMsg::SetGrantDelay {
            role_id,
            grant_delay,
        } => {
            set_grant_delay(&mut ctx, *role_id, *grant_delay)?;
        }
        ExecuteMsg::SetTargetAdminDelay { target, new_delay } => {
            set_target_admin_delay(&mut ctx, target, *new_delay)?;
        }
        ExecuteMsg::SetTargetClosed { target, closed } => {
            set_target_closed(&mut ctx, target, *closed)?;
        }
        ExecuteMsg::SetTargetFunctionRole {
            target,
            selectors,
            role_id,
        } => {
            set_target_function_role(&mut ctx, target, selectors.iter().map(|e| &**e), *role_id)?;
        }
        ExecuteMsg::UpdateAuthority {
            target,
            new_authority,
        } => {
            let msg = update_authority(&mut ctx, target, new_authority)?;

            msgs.push(msg);
        }
        ExecuteMsg::Schedule { target, data, when } => {
            let (operation_id, nonce) = schedule(&mut ctx, target, data, *when)?;

            response_data = Some(json((operation_id, nonce)));
        }
        ExecuteMsg::Cancel {
            caller,
            target,
            data,
        } => {
            let nonce = cancel(&mut ctx, caller, target, data)?;

            response_data = Some(json(nonce));
        }
        ExecuteMsg::Execute { target, data } => {
            let (msg, nonce) = contract::execute(&mut ctx, target, data)?;

            msgs.push(msg);
            response_data = Some(json(nonce));
        }
        ExecuteMsg::ConsumeScheduledOp { caller, data } => {
            consume_scheduled_op(&mut ctx, caller, data)?;
        }
    }

    let mut res = Response::new()
        .add_submessages(msgs)
        .add_events(ctx.events());

    if let Some(data) = response_data {
        res = res.set_data(data);
    }

    Ok(res)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
#[expect(clippy::needless_pass_by_value, reason = "required for entry_point")]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    let ctx = QueryCtx::new(deps, &env);

    match msg {
        QueryMsg::AdminRole {} => Ok(json(RoleId::ADMIN_ROLE)),
        QueryMsg::PublicRole {} => Ok(json(RoleId::PUBLIC_ROLE)),
        QueryMsg::CanCall {
            selector,
            target,
            caller,
        } => can_call(ctx, &caller, &target, &selector).map(json),
        QueryMsg::Expiration {} => Ok(json(expiration())),
        QueryMsg::MinSetback {} => Ok(json(min_setback())),
        QueryMsg::IsTargetClosed { target } => is_target_closed(ctx, &target).map(json),
        QueryMsg::GetTargetFunctionRole { target, selector } => {
            get_target_function_role(ctx, &target, &selector).map(json)
        }
        QueryMsg::GetTargetAdminDelay { target } => get_target_admin_delay(ctx, &target).map(json),
        QueryMsg::GetRoleAdmin { role_id } => get_role_admin(ctx, role_id).map(json),
        QueryMsg::GetRoleGuardian { role_id } => get_role_guardian(ctx, role_id).map(json),
        QueryMsg::GetRoleGrantDelay { role_id } => get_role_grant_delay(ctx, role_id).map(json),
        QueryMsg::GetAccess { role_id, account } => get_access(ctx, role_id, &account).map(json),
        QueryMsg::HasRole { role_id, account } => has_role(ctx, role_id, &account).map(json),
        QueryMsg::GetSchedule { id } => get_schedule(ctx, id).map(json),
        QueryMsg::GetNonce { id } => get_nonce(ctx, id).map(json),
        QueryMsg::HashOperation {
            caller,
            target,
            data,
        } => Ok(json(hash_operation(&caller, &target, &data))),
        QueryMsg::GetRoleLabels { role_ids } => get_role_labels(ctx, &role_ids).map(json),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
#[expect(clippy::needless_pass_by_value, reason = "required for entry_point")]
#[expect(clippy::missing_panics_doc, reason = "internal invariant")]
pub fn reply(deps: DepsMut, _: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply {
        Reply {
            id: EXECUTE_REPLY_ID,
            result,
            ..
        } => {
            result
                .into_result()
                .map_err(|why| StdError::generic_err(format!("execution failed: {why}")))?;

            deps.storage
                .update_item::<ExecutionIdStack, ContractError, _>(|stack| {
                    stack
                        .pop()
                        .expect("execution stack should not be empty on reply; qed;");
                    Ok(())
                })?;

            Ok(Response::new())
        }
        _ => Err(StdError::generic_err(format!("unknown reply: {reply:?}")).into()),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
#[expect(clippy::needless_pass_by_value, reason = "required for entry_point")]
pub fn migrate(
    deps: DepsMut,
    env: Env,
    msg: UpgradeMsg<InitMsg, MigrateMsg>,
) -> Result<Response, ContractError> {
    msg.run(
        deps,
        |deps, msg| {
            let res = init(deps, &env, &msg)?;
            Ok((res, None))
        },
        |_, _, _| Ok((Response::default(), None)),
    )
}

#[track_caller]
#[inline]
fn json(t: impl Serialize) -> Binary {
    to_json_binary(&t).expect("serialization of access manager types is infallible; qed;")
}
