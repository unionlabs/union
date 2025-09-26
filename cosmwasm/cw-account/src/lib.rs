use std::num::NonZeroU32;

use cosmwasm_std::{
    from_json, to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo,
    Order, Response, StdError,
};
use depolama::{self, StorageExt};
use frissitheto::{InitStateVersionError, UpgradeError, UpgradeMsg};
use ucs03_zkgmable::Zkgmable;

use crate::{
    msg::{ExecuteMsg, InitMsg, MigrateMsg, QueryMsg},
    state::{Admins, Zkgm},
    types::{Admin, LocalAdmin, RemoteAdmin},
};

pub mod msg;
pub mod state;
pub mod types;

#[cfg(test)]
mod tests;

pub fn ensure_remote_admin(
    deps: Deps,
    info: &MessageInfo,
    admin: &RemoteAdmin,
) -> Result<(), ContractError> {
    // for remote admins, first ensure that info.sender is zkgm
    if info.sender
        != deps
            .storage
            .maybe_read_item::<Zkgm>()?
            .ok_or(ContractError::ZkgmNotConfigured)?
    {
        return Err(ContractError::OnlyZkgm {
            sender: info.sender.clone(),
        });
    }

    deps.storage
        .maybe_read::<Admins>(&Admin::Remote(admin.clone()))?
        .ok_or_else(|| ContractError::OnlyAdmin {
            sender: Admin::Remote(admin.clone()),
        })
}

pub fn ensure_local_admin_or_self(
    deps: Deps,
    env: &Env,
    info: &MessageInfo,
) -> Result<String, ContractError> {
    // allow reentrant calls into this contract
    if info.sender != env.contract.address {
        let local_admin = Admin::Local(LocalAdmin {
            address: info.sender.to_string(),
        });

        deps.storage
            .maybe_read::<Admins>(&local_admin)?
            .ok_or_else(|| ContractError::OnlyAdmin {
                sender: Admin::Local(LocalAdmin {
                    address: info.sender.to_string(),
                }),
            })?;

        Ok(local_admin.to_string())
    } else {
        Ok("self".to_owned())
    }
}

pub fn init(deps: DepsMut, msg: InitMsg) -> Response {
    match msg {
        InitMsg::Zkgm {
            zkgm,
            path,
            channel_id,
            sender,
        } => {
            deps.storage.write_item::<Zkgm>(&zkgm);
            deps.storage.write::<Admins>(
                &Admin::Remote(RemoteAdmin {
                    address: sender,
                    channel_id,
                    path,
                }),
                &(),
            );
        }
        InitMsg::Local { admin } => {
            deps.storage.write::<Admins>(
                &Admin::Local(LocalAdmin {
                    address: admin.to_string(),
                }),
                &(),
            );
        }
    }

    Response::default()
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: InitMsg,
) -> Result<Response, ContractError> {
    frissitheto::init_state_version(&mut deps, const { NonZeroU32::new(1).unwrap() })?;

    Ok(init(deps, msg))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetZkgm(zkgm) => {
            let actor = ensure_local_admin_or_self(deps.as_ref(), &env, &info)?;

            deps.storage.write_item::<Zkgm>(&zkgm);

            Ok(Response::new().add_event(
                Event::new("set_zkgm")
                    .add_attribute("zkgm", zkgm.to_string())
                    .add_attribute("admin", actor),
            ))
        }
        ExecuteMsg::AddAdmin(new_admin) => {
            let actor = ensure_local_admin_or_self(deps.as_ref(), &env, &info)?;

            deps.storage.write::<Admins>(&new_admin, &());

            Ok(Response::new().add_event(
                Event::new("add_admin")
                    .add_attribute("new_admin", new_admin.to_string())
                    .add_attribute("admin", actor),
            ))
        }
        ExecuteMsg::RemoveAdmin(removed_admin) => {
            let actor = ensure_local_admin_or_self(deps.as_ref(), &env, &info)?;

            let maybe_event = deps
                .storage
                .maybe_read::<Admins>(&removed_admin)?
                .map(|()| {
                    Event::new("remove_admin")
                        .add_attribute("removed_admin", removed_admin.to_string())
                        .add_attribute("admin", actor)
                });

            deps.storage.delete::<Admins>(&removed_admin);

            if deps
                .storage
                .iter::<Admins>(Order::Ascending)
                .collect::<Result<Vec<_>, _>>()?
                .is_empty()
            {
                Err(ContractError::OneAdminRequired)
            } else {
                Ok(Response::new().add_events(maybe_event))
            }
        }
        ExecuteMsg::Dispatch(messages) => {
            let actor = ensure_local_admin_or_self(deps.as_ref(), &env, &info)?;

            Ok(Response::new()
                .add_event(Event::new("dispatch").add_attribute("admin", actor))
                .add_messages(messages))
        }
        ExecuteMsg::Zkgmable(Zkgmable::OnZkgm(on_zkgm)) => {
            let remote_admin = RemoteAdmin {
                address: on_zkgm.sender.clone(),
                channel_id: on_zkgm.destination_channel_id,
                path: on_zkgm.path,
            };

            ensure_remote_admin(deps.as_ref(), &info, &remote_admin)?;

            Ok(Response::new()
                .add_event(Event::new("remote_execute").add_attributes([
                    ("sender", on_zkgm.sender.to_string()),
                    ("channel_id", on_zkgm.destination_channel_id.to_string()),
                    ("path", on_zkgm.path.to_string()),
                ]))
                .add_messages(from_json::<Vec<CosmosMsg>>(&on_zkgm.message)?))
        }
        ExecuteMsg::Zkgmable(Zkgmable::OnIntentZkgm(_)) => Err(ContractError::IntentsNotSupported),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Admins {} => Ok(to_json_binary(
            &deps
                .storage
                .iter::<Admins>(Order::Ascending)
                .map(|r| r.map(|(admin, _)| admin))
                .collect::<Result<Vec<_>, _>>()?,
        )?),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(
    deps: DepsMut,
    _: Env,
    msg: UpgradeMsg<InitMsg, MigrateMsg>,
) -> Result<Response, ContractError> {
    msg.run(
        deps,
        |deps, init_msg| {
            let res = init(deps, init_msg);
            Ok((res, None))
        },
        |_, _, _| Ok((Response::default(), None)),
    )
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("migration error: {0}")]
    Migrate(#[from] UpgradeError),

    #[error("init state version error: {0}")]
    InitStateVersion(#[from] InitStateVersionError),

    #[error("sender {sender} is not a configured admin")]
    OnlyAdmin { sender: Admin },

    #[error("sender {sender} is not zkgm")]
    OnlyZkgm { sender: Addr },

    #[error("intents are not supported")]
    IntentsNotSupported,

    #[error("at least one remote or local admin must be configured")]
    OneAdminRequired,

    #[error("no zkgm address configured")]
    ZkgmNotConfigured,
}
