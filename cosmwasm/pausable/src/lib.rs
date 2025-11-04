//! CosmWasm implementation of [OpenZeppelin][oz]'s [`Pausable.sol`][pausable].
//!
//! [oz]: https://www.openzeppelin.com
//! [pausable]: https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/Pausable.sol

#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::doc_markdown)]
#![cfg_attr(not(test), warn(clippy::unwrap_used))]

use cosmwasm_std::{Binary, Deps, DepsMut, MessageInfo, Response, to_json_binary};
use depolama::StorageExt;
use serde::{Deserialize, Serialize};

use crate::{
    error::ContractError,
    event::{Paused, Unpaused},
    msg::{Pausable, PausableQuery},
    state::IsPaused,
};

pub mod error;
pub mod event;
pub mod msg;
pub mod state;

#[cfg(test)]
mod tests;

pub fn execute(
    deps: DepsMut,
    info: &MessageInfo,
    msg: &Pausable,
) -> Result<Response, ContractError> {
    match msg {
        Pausable::Pause {} => pause(deps, info),
        Pausable::Unpause {} => unpause(deps, info),
    }
}

pub fn query(deps: Deps, msg: &PausableQuery) -> Result<Binary, ContractError> {
    match msg {
        PausableQuery::Paused {} => Ok(to_json_binary(&is_paused(deps)?)?),
    }
}

/// See [`ExecuteMsg::Pause`];
#[expect(
    clippy::needless_pass_by_value,
    reason = "DepsMut should be passed by value"
)]
pub fn pause(deps: DepsMut, info: &MessageInfo) -> Result<Response, ContractError> {
    require_not_paused(deps.as_ref())?;

    deps.storage.write_item::<IsPaused>(&());

    Ok(Response::new().add_event(Paused {
        account: &info.sender,
    }))
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "DepsMut should be passed by value"
)]
pub fn unpause(deps: DepsMut, info: &MessageInfo) -> Result<Response, ContractError> {
    require_paused(deps.as_ref())?;

    deps.storage.delete_item::<IsPaused>();

    Ok(Response::new().add_event(Unpaused {
        account: &info.sender,
    }))
}

/// See [`QueryMsg::Paused`].
pub fn is_paused(deps: Deps) -> Result<bool, ContractError> {
    Ok(deps.storage.maybe_read_item::<IsPaused>()?.is_some())
}

/// Throws if the contract is paused.
///
/// ```solidity
/// function _requireNotPaused() internal view virtual;
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/Pausable.sol#L74>
pub fn require_not_paused(deps: Deps) -> Result<(), ContractError> {
    if is_paused(deps)? {
        Err(ContractError::ExpectedPause)
    } else {
        Ok(())
    }
}

/// Throws if the contract is not paused.
///
/// ```solidity
/// function _requirePaused() internal view virtual;
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/Pausable.sol#L83>
pub fn require_paused(deps: Deps) -> Result<(), ContractError> {
    if is_paused(deps)? {
        Ok(())
    } else {
        Err(ContractError::EnforcedPause)
    }
}

/// Wrapper to make an entrypoint callable only when the contract is not paused.
///
/// Requirements:
///
/// - The contract must not be paused.
///
/// # Usage
///
/// This type is intended to be used directly in a contract's `ExecuteMsg`. It can wrap either a
/// single variant, or a nested enum containing multiple entrypoints that should be pausable.
///
/// ```rust
/// # use pausable::error::ContractError;
/// use cosmwasm_std::{Response, MessageInfo, DepsMut, Env};
/// use serde::{Deserialize, Serialize};
/// use pausable::{WhenNotPaused, msg::Pausable};
///
/// #[derive(Serialize, Deserialize)]
/// enum ExecuteMsg {
///     PublicAction(WhenNotPaused<PublicAction>),
///     #[serde(untagged)]
///     Pausable(Pausable),
/// }
///
/// #[derive(Serialize, Deserialize)]
/// struct PublicAction {}
///
/// fn execute(
///     deps: DepsMut,
///     env: Env,
///     info: MessageInfo,
///     msg: ExecuteMsg,
/// ) -> Result<Response, ContractError> {
///     match msg {
///         ExecuteMsg::PublicAction(msg) => {
///             let public_action = msg.ensure_not_paused(deps.as_ref())?;
///             // snip
///             # let _ = public_action;
///             # todo!()
///         },
///         // access management is not part of this contract, and must be done manually
///         ExecuteMsg::Pausable(msg) => pausable::execute(deps, &info, &msg),
///     }
/// }
/// ```
///
/// <div class="warning">
///
/// This is not intended to wrap the entire `ExecuteMsg` of a contract, since [`Pausable`] does it's
/// own checks.
///
/// </div>
///
/// ```solidity
/// modifier whenNotPaused()
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/Pausable.sol#L47>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
#[must_use = "consume this type with WhenNotPaused::ensure_not_paused to enforce execution invariants"]
pub struct WhenNotPaused<T>(T);

impl<T> WhenNotPaused<T> {
    pub fn ensure_not_paused(self, deps: Deps) -> Result<T, ContractError> {
        require_not_paused(deps).map(|()| self.0)
    }
}

/// Wrapper to make an entrypoint callable only when the contract is paused.
///
/// Requirements:
///
/// - The contract must be paused.
///
/// # Usage
///
/// This type is intended to be used directly in a contract's `ExecuteMsg`. It can wrap either a
/// single variant, or a nested enum containing multiple entrypoints that should be pausable.
///
/// ```rust
/// # use pausable::error::ContractError;
/// use cosmwasm_std::{Response, MessageInfo, DepsMut, Env};
/// use serde::{Deserialize, Serialize};
/// use pausable::{WhenPaused, msg::Pausable};
///
/// #[derive(Serialize, Deserialize)]
/// enum ExecuteMsg {
///     AdminAction(WhenPaused<AdminAction>),
///     #[serde(untagged)]
///     Pausable(Pausable),
/// }
///
/// #[derive(Serialize, Deserialize)]
/// struct AdminAction {}
///
/// fn execute(
///     deps: DepsMut,
///     env: Env,
///     info: MessageInfo,
///     msg: ExecuteMsg,
/// ) -> Result<Response, ContractError> {
///     match msg {
///         ExecuteMsg::AdminAction(msg) => {
///             let admin_action = msg.ensure_paused(deps.as_ref())?;
///             // snip
///             # let _ = admin_action;
///             # todo!()
///         },
///         // access management is not part of this contract, and must be done manually
///         ExecuteMsg::Pausable(msg) => pausable::execute(deps, &info, &msg),
///     }
/// }
/// ```
///
/// <div class="warning">
///
/// This is not intended to wrap the entire `ExecuteMsg` of a contract, since [`Pausable`] does it's
/// own checks.
///
/// </div>
///
/// ```solidity
/// modifier whenPaused()
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/Pausable.sol#L59>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
#[must_use = "consume this type with WhenPaused::ensure_paused to enforce execution invariants"]
pub struct WhenPaused<T>(T);

impl<T> WhenPaused<T> {
    pub fn ensure_paused(self, deps: Deps) -> Result<T, ContractError> {
        require_paused(deps).map(|()| self.0)
    }
}
