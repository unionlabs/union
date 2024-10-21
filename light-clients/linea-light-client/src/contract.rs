use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response};
use ics008_wasm_client::{
    define_cosmwasm_light_client_contract,
    storage_utils::{save_proto_client_state, save_proto_consensus_state},
    CustomQueryOf, InstantiateMsg,
};
use linea_light_client_types::ClientState;
use protos::ibc::lightclients::wasm::v1::{
    ClientState as ProtoClientState, ConsensusState as ProtoConsensusState,
};
use unionlabs::encoding::{DecodeAs, Proto};

use crate::{client::LineaLightClient, errors::Error};

#[entry_point]
pub fn instantiate(
    mut deps: DepsMut<CustomQueryOf<LineaLightClient>>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Error> {
    let client_state =
        ClientState::decode_as::<Proto>(&msg.client_state).map_err(Error::ClientStateDecode)?;

    save_proto_consensus_state::<LineaLightClient>(
        deps.branch(),
        ProtoConsensusState {
            data: msg.consensus_state.into(),
        },
        &client_state.l1_latest_height,
    );
    save_proto_client_state::<LineaLightClient>(
        deps,
        ProtoClientState {
            data: msg.client_state.into(),
            checksum: msg.checksum.into(),
            latest_height: Some(client_state.l1_latest_height.into()),
        },
    );
    Ok(Response::default())
}

define_cosmwasm_light_client_contract!(LineaLightClient, Linea);
