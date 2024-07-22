use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response};
use ics008_wasm_client::{
    define_cosmwasm_light_client_contract,
    storage_utils::{save_proto_client_state, save_proto_consensus_state},
    CustomQueryOf, InstantiateMsg,
};
use protos::ibc::lightclients::wasm::v1::{
    ClientState as ProtoClientState, ConsensusState as ProtoConsensusState,
};
use unionlabs::{
    encoding::{DecodeAs, Proto},
    ibc::{
        core::client::height::Height,
        lightclients::near::{client_state::ClientState, consensus_state::ConsensusState},
    },
};

use crate::{client::NearLightClient, errors::Error, state::EPOCH_BLOCK_PRODUCERS_MAP};

#[entry_point]
pub fn instantiate(
    mut deps: DepsMut<CustomQueryOf<NearLightClient>>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Error> {
    let client_state =
        ClientState::decode_as::<Proto>(&msg.client_state).map_err(Error::ClientStateDecode)?;

    let consensus_state = ConsensusState::decode_as::<Proto>(&msg.consensus_state)
        .map_err(Error::ConsensusStateDecode)?;

    EPOCH_BLOCK_PRODUCERS_MAP
        .save(
            deps.storage,
            consensus_state.state.epoch_id.0,
            &client_state.initial_block_producers,
        )
        .unwrap();

    save_proto_consensus_state::<NearLightClient>(
        deps.branch(),
        ProtoConsensusState {
            data: msg.consensus_state.into(),
        },
        &Height {
            revision_number: 0,
            revision_height: client_state.latest_height,
        },
    );
    save_proto_client_state::<NearLightClient>(
        deps,
        ProtoClientState {
            data: msg.client_state.into(),
            checksum: msg.checksum.into(),
            latest_height: Some(
                Height {
                    revision_number: 0,
                    revision_height: client_state.latest_height,
                }
                .into(),
            ),
        },
    );
    Ok(Response::default())
}

define_cosmwasm_light_client_contract!(NearLightClient, Near);
