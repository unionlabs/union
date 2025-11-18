use cosmwasm_std::{Addr, Coin, Deps, DepsMut, Env, Event, MessageInfo, QuerierWrapper, Storage};
use serde::Serialize;

#[must_use = "call ExecCtx::events() to consume this object"]
pub struct ExecCtx<'info, 'deps> {
    events: Vec<Event>,
    deps: DepsMut<'deps>,
    env: &'info Env,
    info: &'info MessageInfo,
    data: String,
}

#[derive(Clone, Copy)]
pub struct QueryCtx<'info, 'deps> {
    deps: Deps<'deps>,
    env: &'info Env,
}

impl<'info, 'deps> QueryCtx<'info, 'deps> {
    #[must_use]
    pub fn new(deps: Deps<'deps>, env: &'info Env) -> Self {
        Self { deps, env }
    }
}

impl<'info, 'deps> ExecCtx<'info, 'deps> {
    /// Create a new [`ExecCtx`].
    ///
    /// # Panics
    ///
    /// This function will panic if the provided `data`'s serialization fails.
    pub fn new(
        deps: DepsMut<'deps>,
        env: &'info Env,
        info: &'info MessageInfo,
        data: &impl Serialize,
    ) -> Self {
        Self {
            events: vec![],
            deps,
            info,
            env,
            data: serde_json_wasm::to_string(data).expect("infallible"),
        }
    }

    #[must_use = "events must be emitted with Response::add_events()"]
    pub fn events(self) -> Vec<Event> {
        self.events
    }
}

pub(crate) trait IQueryCtx<'info> {
    fn querier(&self) -> QuerierWrapper<'_>;
    fn address_this(&self) -> &'info Addr;
    fn timestamp(&self) -> u64;
}

pub(crate) trait IExecCtx<'info> {
    fn emit(&mut self, event: impl Into<Event>);
    fn msg_sender(&self) -> &'info Addr;
    fn msg_data(&self) -> String;
    fn value(&self) -> &[Coin];
    fn query_ctx<'a>(&'a self) -> QueryCtx<'info, 'a>;
}

impl<'info> IQueryCtx<'info> for ExecCtx<'info, '_> {
    fn querier(&self) -> QuerierWrapper<'_> {
        self.deps.querier
    }

    fn address_this(&self) -> &'info Addr {
        &self.env.contract.address
    }

    fn timestamp(&self) -> u64 {
        self.env.block.time.seconds()
    }
}

impl<'info> IExecCtx<'info> for ExecCtx<'info, '_> {
    fn emit(&mut self, event: impl Into<Event>) {
        self.events.push(event.into());
    }

    fn msg_sender(&self) -> &'info Addr {
        &self.info.sender
    }

    fn msg_data(&self) -> String {
        self.data.clone()
    }

    fn value(&self) -> &[Coin] {
        &self.info.funds
    }

    fn query_ctx<'a>(&'a self) -> QueryCtx<'info, 'a> {
        QueryCtx::new(self.deps.as_ref(), self.env)
    }
}

impl<'info> IQueryCtx<'info> for QueryCtx<'info, '_> {
    fn querier(&self) -> QuerierWrapper<'_> {
        self.deps.querier
    }

    fn address_this(&self) -> &'info Addr {
        &self.env.contract.address
    }

    fn timestamp(&self) -> u64 {
        self.env.block.time.seconds()
    }
}

pub(crate) trait HasStorage<'storage> {
    type Storage: 'storage;

    fn storage(self) -> Self::Storage;
}

impl<'deps: 'call, 'call> HasStorage<'call> for &'call mut ExecCtx<'_, 'deps> {
    type Storage = &'call mut dyn Storage;

    fn storage(self) -> Self::Storage {
        self.deps.storage
    }
}

impl<'deps> HasStorage<'deps> for QueryCtx<'_, 'deps> {
    type Storage = &'deps dyn Storage;

    fn storage(self) -> Self::Storage {
        self.deps.storage
    }
}
