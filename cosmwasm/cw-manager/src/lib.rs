//! CosmWasm implementation of openzeppelin's [`AccessManager.sol`](am).
//!
//! [am]: https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol

use std::ops::RangeBounds;

use cosmwasm_std::{Addr, DepsMut, Env, Event, MessageInfo, Order, Response, StdResult};
use depolama::{StorageExt, Store};

use crate::{
    error::ContractError,
    execute::{
        grant_role, label_role, renounce_role, revoke_role, set_grant_delay, set_role_admin,
        set_role_guardian,
    },
    msg::ExecuteMsg,
};

pub mod error;
pub mod event;
pub mod execute;
pub mod managed;
pub mod msg;
pub mod query;
pub mod state;
pub mod time;
pub mod types;

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let mut events = vec![];
    let mut ctx = Ctx::new(&mut events, deps, &env, &info, &msg);

    match &msg {
        ExecuteMsg::LabelRole { role_id, label } => {
            label_role(*role_id, label);
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
        ExecuteMsg::SetTargetAdminDelay { target, new_delay } => todo!(),
        ExecuteMsg::SetTargetClosed { target, closed } => todo!(),
        ExecuteMsg::SetTargetFunctionRole {
            target,
            selectors,
            role_id,
        } => todo!(),
        ExecuteMsg::UpdateAuthority {
            target,
            new_authority,
        } => todo!(),
    }

    Ok(ctx.into_response())
}

#[must_use]
pub struct Ctx<'a, 'e, 'd> {
    events: &'e mut Vec<Event>,
    deps: DepsMut<'d>,
    env: &'a Env,
    info: &'a MessageInfo,
    data: &'a ExecuteMsg,
}

impl<'a, 'e, 'd> Ctx<'a, 'e, 'd> {
    pub fn new(
        events: &'e mut Vec<Event>,
        deps: DepsMut<'d>,
        env: &'a Env,
        info: &'a MessageInfo,
        data: &'a ExecuteMsg,
    ) -> Self {
        Self {
            events,
            deps,
            info,
            env,
            data,
        }
    }

    pub fn emit(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn address_this(&self) -> &'a Addr {
        &self.info.sender
    }

    pub fn timestamp(&self) -> u64 {
        self.env.block.time.seconds()
    }

    pub fn msg_sender(&self) -> &'a Addr {
        &self.info.sender
    }

    pub fn msg_data(&self) -> &'a ExecuteMsg {
        self.data
    }

    pub fn into_response(self) -> Response {
        Response::new().add_events(self.events.drain(..))
    }
}

impl<'a, 'e, 'deps> StorageExt for Ctx<'a, 'e, 'deps> {
    fn read<S: Store>(&self, k: &S::Key) -> StdResult<S::Value> {
        self.deps.storage.read::<S>(k)
    }

    fn maybe_read<S: Store>(&self, k: &S::Key) -> StdResult<Option<S::Value>> {
        self.deps.storage.maybe_read::<S>(k)
    }

    fn write<S: Store>(&mut self, k: &S::Key, v: &S::Value) {
        self.deps.storage.write::<S>(k, v)
    }

    fn delete<S: Store>(&mut self, k: &S::Key) {
        self.deps.storage.delete::<S>(k)
    }

    fn take<S: Store>(&mut self, k: &S::Key) -> StdResult<Option<S::Value>> {
        self.deps.storage.take::<S>(k)
    }

    fn iter<S: Store>(&self, order: Order) -> impl Iterator<Item = StdResult<(S::Key, S::Value)>> {
        self.deps.storage.iter::<S>(order)
    }

    fn iter_range<S: Store>(
        &self,
        order: Order,
        bounds: impl RangeBounds<S::Key>,
    ) -> impl Iterator<Item = StdResult<(S::Key, S::Value)>> {
        self.deps.storage.iter_range::<S>(order, bounds)
    }
}
