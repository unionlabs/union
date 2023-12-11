use cosmwasm_std::{entry_point, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response};
use ics008_wasm_client::{
    storage_utils::{save_proto_client_state, save_proto_consensus_state},
    IbcClient, InstantiateMsg, QueryMsg, SudoMsg,
};
use protos::ibc::lightclients::wasm::v1::{
    ClientState as ProtoClientState, ConsensusState as ProtoConsensusState,
};
use unionlabs::{ibc::lightclients::cometbls::client_state::ClientState, TryFromProto};

use crate::{client::CometblsLightClient, errors::Error};

#[entry_point]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Error> {
    let client_state = ClientState::try_from_proto_bytes(&msg.client_state).map_err(|e| {
        Error::DecodeFromProto {
            reason: format!("{:?}", e),
        }
    })?;

    save_proto_consensus_state(
        deps.branch(),
        ProtoConsensusState {
            data: msg.consensus_state.into(),
        },
        &client_state.latest_height,
    );
    save_proto_client_state(
        deps,
        ProtoClientState {
            data: msg.client_state.into(),
            checksum: msg.checksum.unwrap().into(),
            latest_height: Some(client_state.latest_height.into()),
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
