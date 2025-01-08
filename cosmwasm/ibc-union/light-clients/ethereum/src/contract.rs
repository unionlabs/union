#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_json, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, SubMsg,
};
use ethereum_light_client_types::ClientState;
use ibc_union_light_client::{
    msg::{InstantiateMsg, QueryMsg},
    read_consensus_state,
    state::IBC_HOST,
    IbcClientError, CLIENT_STATES,
};
use serde::{Deserialize, Serialize};
use unionlabs::{
    encoding::{Bincode, EncodeAs, EthAbi},
    primitives::{encoding::Base64, Bytes},
};

use crate::client::EthereumLightClient;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, IbcClientError<EthereumLightClient>> {
    ibc_union_light_client::instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    ibc_union_light_client::query::<EthereumLightClient>(deps, env, msg).map_err(Into::into)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrateMsg {
    client_ids: Vec<u32>,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    msg: MigrateMsg,
) -> Result<Response, IbcClientError<EthereumLightClient>> {
    let ibc_host = IBC_HOST.load(deps.storage)?;

    let mut msgs = vec![];

    for client_id in msg.client_ids {
        let client_state_bytes = from_json::<Bytes<Base64>>(
            deps.querier
                .query_wasm_raw(ibc_host.to_string(), CLIENT_STATES.key(client_id).to_vec())?
                .ok_or_else(|| {
                    IbcClientError::Std(cosmwasm_std::StdError::generic_err(format!(
                        "unable to read client state of client {client_id}"
                    )))
                })?,
        )?;

        let client_state: ClientState = bincode_1::deserialize(&client_state_bytes).unwrap();

        let consensus_state = read_consensus_state::<EthereumLightClient>(
            deps.querier,
            &ibc_host,
            client_id,
            client_state.latest_height,
        )?;

        msgs.push(SubMsg::new(CosmosMsg::Wasm(
            cosmwasm_std::WasmMsg::Execute {
                contract_addr: ibc_host.to_string(),
                msg: to_json_binary(&ibc_union_msg::msg::ExecuteMsg::MigrateState(
                    ibc_union_msg::msg::MsgMigrateState {
                        client_id,
                        height: client_state.latest_height,
                        client_state: client_state.encode_as::<Bincode>().into(),
                        consensus_state: consensus_state.encode_as::<EthAbi>().into(),
                    },
                ))
                .unwrap(),
                funds: vec![],
            },
        )))
    }

    Ok(Response::new().add_submessages(msgs))
}
