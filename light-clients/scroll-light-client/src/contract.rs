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
    ibc::{core::client::height::Height, lightclients::scroll::client_state::ClientState},
    TryFromProto,
};

use crate::{client::ScrollLightClient, errors::Error};

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
        &Height {
            revision_number: 0,
            revision_height: client_state.latest_batch_index,
        },
    );
    save_proto_client_state(
        deps,
        ProtoClientState {
            data: msg.client_state.into(),
            checksum: msg.checksum.into(),
            latest_height: Some(
                Height {
                    revision_number: 0,
                    revision_height: client_state.latest_batch_index,
                }
                .into(),
            ),
        },
    );
    Ok(Response::default())
}

define_cosmwasm_light_client_contract!(ScrollLightClient, Scroll);
