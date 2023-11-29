use cosmwasm_std::{entry_point, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response};
use ics008_wasm_client::{
    storage_utils::{save_proto_client_state, save_proto_consensus_state},
    IbcClient, InstantiateMsg, QueryMsg, SudoMsg,
};

use crate::{client::EthereumLightClient, custom_query::CustomQuery, errors::Error};

#[entry_point]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Error> {
    save_proto_consensus_state(
        deps.branch(),
        msg.consensus_state,
        &msg.client_state
            .latest_height
            .clone()
            .ok_or(Error::DecodeFromProto {
                reason: "`latest_height` is missing".to_string(),
            })?
            .into(),
    );
    save_proto_client_state(deps, msg.client_state);
    Ok(Response::default())
}

#[entry_point]
pub fn sudo(deps: DepsMut<CustomQuery>, env: Env, msg: SudoMsg) -> Result<Response, Error> {
    let result = EthereumLightClient::sudo(deps, env, msg)?;
    Ok(Response::default().set_data(result))
}

#[entry_point]
pub fn query(deps: Deps<CustomQuery>, env: Env, msg: QueryMsg) -> Result<QueryResponse, Error> {
    EthereumLightClient::query(deps, env, msg)
}
