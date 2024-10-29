use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response};
use ics008_wasm_client::{
    storage_utils::{save_proto_client_state, save_proto_consensus_state},
    CustomQueryOf, InstantiateMsg,
};
use movement_light_client_types::ClientState;
use protos::ibc::lightclients::wasm::v1::{
    ClientState as ProtoClientState, ConsensusState as ProtoConsensusState,
};
use unionlabs::{
    encoding::{DecodeAs, Proto},
    ibc::core::client::height::Height,
};

use crate::{client::MovementLightClient, errors::Error};

// NOTE(aeryz): the fact that the host module forces the light clients to store and use the wasm wrapping
// in the client state makes this code kinda messy. But this is going to be resolved in the future versions
// of IBC (probably v9). When that feature is implemented, we can move this to the ics008 macro.
#[entry_point]
#[allow(unused)]
pub fn instantiate(
    mut deps: DepsMut<CustomQueryOf<MovementLightClient>>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Error> {
    let client_state =
        ClientState::decode_as::<Proto>(&msg.client_state).map_err(Error::ClientStateDecode)?;

    save_proto_consensus_state::<MovementLightClient>(
        deps.branch(),
        ProtoConsensusState {
            data: msg.consensus_state.into(),
        },
        &Height {
            revision_number: 0,
            revision_height: client_state.latest_block_num,
        },
    );
    save_proto_client_state::<MovementLightClient>(
        deps,
        ProtoClientState {
            data: msg.client_state.into(),
            checksum: msg.checksum.into(),
            latest_height: Some(
                Height {
                    revision_number: 0,
                    revision_height: client_state.latest_block_num,
                }
                .into(),
            ),
        },
    );
    Ok(Response::default())
}

ics008_wasm_client::define_cosmwasm_light_client_contract!(MovementLightClient, Movement);
