use std::num::NonZeroU32;

use cosmwasm_std::{
    entry_point, from_json, Addr, Binary, Coin, DepsMut, Env, MessageInfo, Response, StdError,
    WasmMsg,
};
use frissitheto::{UpgradeError, UpgradeMsg};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct InstantiateMsg {}

fn init() -> Result<Response, Error> {
    Ok(Response::default())
}

#[entry_point]
pub fn instantiate(
    mut deps: DepsMut,
    _: Env,
    _: MessageInfo,
    _: InstantiateMsg,
) -> Result<Response, Error> {
    frissitheto::init_state_version(&mut deps, const { NonZeroU32::new(1).unwrap() })
        .expect("infallible, instantiate can only be called once; qed;");
    init()
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    OnZkgm { message: Binary },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct Msg {
    pub contract: Addr,
    pub message: Binary,
    pub funds: Vec<Coin>,
}

#[entry_point]
pub fn execute(_: DepsMut, _: Env, _: MessageInfo, msg: ExecuteMsg) -> Result<Response, Error> {
    match msg {
        ExecuteMsg::OnZkgm { message } => {
            let Msg {
                contract,
                message,
                funds,
            } = from_json::<Msg>(message)?;

            Ok(Response::new().add_message(WasmMsg::Execute {
                contract_addr: contract.into(),
                msg: message,
                funds,
            }))
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct MigrateMsg {}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    _: Env,
    msg: UpgradeMsg<InstantiateMsg, MigrateMsg>,
) -> Result<Response, Error> {
    msg.run(
        deps,
        |_, _| {
            let res = init()?;
            Ok((res, None))
        },
        |_, _, _| Ok((Response::default(), None)),
    )
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    StdError(#[from] StdError),

    #[error(transparent)]
    Migrate(#[from] UpgradeError),
}
