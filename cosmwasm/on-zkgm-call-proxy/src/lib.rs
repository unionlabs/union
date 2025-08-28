use std::num::NonZeroU32;

use cosmwasm_std::{
    entry_point, from_json, Addr, Coin, DepsMut, Env, MessageInfo, Response, StdError, WasmMsg,
};
use frissitheto::{UpgradeError, UpgradeMsg};
use ibc_union_spec::ChannelId;
use serde::{Deserialize, Serialize};
use unionlabs_primitives::{Bytes, U256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ExecuteMsg {
    OnZkgm {
        caller: Addr,
        path: U256,
        source_channel_id: ChannelId,
        destination_channel_id: ChannelId,
        sender: Bytes,
        message: Bytes,
        relayer: Addr,
        relayer_msg: Bytes,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct Msg {
    pub contract: Addr,
    pub message: Bytes,
    pub funds: Vec<Coin>,
}

#[entry_point]
pub fn execute(_: DepsMut, _: Env, _: MessageInfo, msg: ExecuteMsg) -> Result<Response, Error> {
    match msg {
        ExecuteMsg::OnZkgm {
            caller: _,
            path: _,
            source_channel_id: _,
            destination_channel_id: _,
            sender: _,
            message,
            relayer: _,
            relayer_msg: _,
        } => {
            let Msg {
                contract,
                message,
                funds,
            } = from_json::<Msg>(message)?;

            Ok(Response::new().add_message(WasmMsg::Execute {
                contract_addr: contract.into(),
                msg: message.to_vec().into(),
                funds,
            }))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
        |_, InstantiateMsg {}| {
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

    #[error("migration error: {0}")]
    Migrate(#[from] UpgradeError),
}
