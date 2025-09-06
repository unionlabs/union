use cosmwasm_std::{
    entry_point, from_json, Addr, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo, Response,
    StdError, StdResult,
};
use depolama::{self, StorageExt};
use frissitheto::{UpgradeError, UpgradeMsg};
use ucs03_zkgmable::Zkgmable;

use crate::{
    msg::{ExecuteMsg, InitMsg, MigrateMsg},
    state::{Admins, Zkgm},
    types::{Admin, LocalAdmin, RemoteAdmin},
};

pub mod msg;
pub mod state;
pub mod types;

fn ensure_remote_admin(deps: Deps, info: &MessageInfo, admin: &RemoteAdmin) -> Result<(), Error> {
    // for remote admins, first ensure that info.sender is zkgm
    if info.sender != deps.storage.read_item::<Zkgm>()? {
        return Err(Error::OnlyZkgm {
            sender: info.sender.clone(),
        });
    }

    deps.storage
        .maybe_read::<Admins>(&Admin::Remote(admin.clone()))?
        .ok_or_else(|| Error::OnlyAdmin {
            sender: info.sender.clone(),
        })
}

fn ensure_local_admin_or_self(deps: Deps, env: &Env, info: &MessageInfo) -> Result<String, Error> {
    // allow reentrant calls into this contract
    if info.sender != env.contract.address {
        let local_admin = Admin::Local(LocalAdmin {
            address: info.sender.to_string(),
        });

        deps.storage
            .maybe_read::<Admins>(&local_admin)?
            .ok_or_else(|| Error::OnlyAdmin {
                sender: info.sender.clone(),
            })?;

        Ok(local_admin.to_string())
    } else {
        Ok("self".to_owned())
    }
}

fn init(deps: DepsMut, msg: InitMsg) -> Result<Response, Error> {
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

    Ok(Response::default())
}

#[entry_point]
pub fn instantiate(_: DepsMut, _: Env, _: MessageInfo, _: ()) -> StdResult<Response> {
    panic!("this contract cannot be instantiated directly, but must be migrated from an existing instantiated contract.");
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
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

            Ok(Response::new().add_events(maybe_event))
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
        ExecuteMsg::Zkgmable(Zkgmable::OnIntentZkgm(_)) => Err(Error::IntentsNotSupported),
    }
}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    _: Env,
    msg: UpgradeMsg<InitMsg, MigrateMsg>,
) -> Result<Response, Error> {
    msg.run(
        deps,
        |deps, init_msg| {
            let res = init(deps, init_msg)?;
            Ok((res, None))
        },
        |_, _, _| Ok((Response::default(), None)),
    )
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    StdError(#[from] StdError),

    #[error("migration error: {0}")]
    Migrate(#[from] UpgradeError),

    #[error("sender {sender} is a configured admin")]
    OnlyAdmin { sender: Addr },

    #[error("sender {sender} is not zkgm")]
    OnlyZkgm { sender: Addr },

    #[error("intents are not supported")]
    IntentsNotSupported,
}
