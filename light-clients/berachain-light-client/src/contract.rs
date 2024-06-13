use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response};
use ics008_wasm_client::{
    define_cosmwasm_light_client_contract,
    storage_utils::{save_proto_client_state, save_proto_consensus_state},
    InstantiateMsg,
};
use protos::ibc::lightclients::wasm::v1::{
    ClientState as ProtoClientState, ConsensusState as ProtoConsensusState,
};
use unionlabs::{
    cosmwasm::wasm::union::custom_query::UnionCustomQuery,
    encoding::{DecodeAs, Proto},
    ibc::lightclients::berachain::client_state::ClientState,
};

use crate::{client::BerachainLightClient, errors::Error};

// NOTE(aeryz): the fact that the host module forces the light clients to store and use the wasm wrapping
// in the client state makes this code kinda messy. But this is going to be resolved in the future versions
// of IBC (probably v9). When that feature is implemented, we can move this to the ics008 macro.
#[entry_point]
pub fn instantiate(
    mut deps: DepsMut<'_, UnionCustomQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Error> {
    let client_state =
        ClientState::decode_as::<Proto>(&msg.client_state).map_err(Error::ClientStateDecode)?;

    save_proto_consensus_state::<BerachainLightClient>(
        deps.branch(),
        ProtoConsensusState {
            data: msg.consensus_state,
        },
        &client_state.latest_height,
    );
    save_proto_client_state::<BerachainLightClient>(
        deps,
        ProtoClientState {
            data: msg.client_state,
            checksum: msg.checksum,
            latest_height: Some(client_state.latest_height.into()),
        },
    );
    Ok(Response::default())
}

define_cosmwasm_light_client_contract!(BerachainLightClient, Berachain);
