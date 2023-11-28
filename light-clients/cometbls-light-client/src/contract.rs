use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response};
use ics008_wasm_client::{
    storage_utils::{save_client_state, save_consensus_state},
    IbcClient, QueryMsg, SudoMsg,
};
use serde::{Deserialize, Serialize};
use unionlabs::{
    ibc::{
        core::client::height::Height,
        lightclients::{
            cometbls::client_state::ClientState,
            ethereum::consensus_state::ConsensusState,
            wasm::{
                client_state::ClientState as WasmClientState,
                consensus_state::ConsensusState as WasmConsensusState,
            },
        },
    },
    TryFromProto,
};

use crate::{client::CometblsLightClient, errors::Error};

#[derive(Debug, Serialize, Deserialize)]
pub struct InputClientState {
    pub data: Binary,
    pub checksum: Binary,
    pub latest_height: Height,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstantiateMsg {
    client_state: InputClientState,
    consensus_state: WasmConsensusState<Binary>,
}

#[entry_point]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Error> {
    // TODO(aeryz): remove unwrap
    let client_state = ClientState::try_from_proto_bytes(&msg.client_state.data).unwrap();
    let consensus_state = ConsensusState::try_from_proto_bytes(&msg.consensus_state.data).unwrap();

    save_consensus_state(
        deps.branch(),
        WasmConsensusState::<ConsensusState> {
            data: consensus_state,
        },
        &client_state.latest_height,
    );
    save_client_state(
        deps,
        WasmClientState::<ClientState> {
            data: client_state,
            checksum: msg.client_state.checksum.as_slice().try_into().unwrap(),
            latest_height: msg.client_state.latest_height,
        },
    );
    Ok(Response::default())
}

#[entry_point]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, Error> {
    let result = CometblsLightClient::sudo(deps, env, msg)?;
    Ok(Response::default().set_data(result))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<QueryResponse, Error> {
    CometblsLightClient::query(deps, env, msg)
}
