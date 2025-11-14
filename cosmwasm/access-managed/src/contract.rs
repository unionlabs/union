use access_manager_types::{
    Selector,
    managed::{error::AccessManagedError, event::AuthorityUpdated, msg::QueryMsg},
};
use cosmwasm_std::{Addr, Deps, DepsMut, Event, MessageInfo};
use depolama::StorageExt;

use crate::{
    error::ContractError,
    state::{Authority, ConsumingSchedule},
};

/// See [`QueryMsg::Authority`].
pub(crate) fn authority(deps: Deps) -> Result<Addr, ContractError> {
    deps.storage.read_item::<Authority>().map_err(Into::into)
}

/// See [`ExecuteMsg::SetAuthority`].
pub(crate) fn set_authority(
    deps: DepsMut,
    info: MessageInfo,
    new_authority: Addr,
) -> Result<Event, ContractError> {
    let caller = info.sender;

    if caller != authority(deps.as_ref())? {
        return Err(AccessManagedError::AccessManagedUnauthorized { caller }.into());
    }

    if deps
        .querier
        .query_wasm_contract_info(&new_authority)
        .is_err()
    {
        return Err(AccessManagedError::AccessManagedInvalidAuthority {
            authority: new_authority,
        }
        .into());
    }

    Ok(_set_authority(deps, &new_authority))
}

/// Transfers control to a new authority. Internal function with no access restriction. Allows
/// bypassing the permissions set by the current authority.
///
/// ```solidity
/// function _setAuthority(address newAuthority) internal virtual
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManaged.sol#L86>
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn _set_authority(deps: DepsMut, new_authority: &Addr) -> Event {
    deps.storage.write_item::<Authority>(new_authority);

    AuthorityUpdated {
        authority: new_authority,
    }
    .into()
}

/// See [`ExecuteMsg::IsConsumingScheduledOp`].
pub(crate) fn is_consuming_scheduled_op(deps: Deps) -> Result<&'static Selector, ContractError> {
    if deps.storage.read_item::<ConsumingSchedule>()? {
        Ok(Selector::extract_from_serialize(
            &QueryMsg::IsConsumingScheduledOp {},
        ))
    } else {
        Ok(Selector::new(""))
    }
}
