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
    ibc::{core::client::height::Height, lightclients::arbitrum::client_state::ClientState},
};

use crate::{client::ArbitrumLightClient, errors::Error};

#[entry_point]
pub fn instantiate(
    mut deps: DepsMut<CustomQueryOf<ArbitrumLightClient>>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Error> {
    let client_state =
        ClientState::decode_as::<Proto>(&msg.client_state).map_err(Error::ClientStateDecode)?;

    save_proto_consensus_state::<ArbitrumLightClient>(
        deps.branch(),
        ProtoConsensusState {
            data: msg.consensus_state,
        },
        &Height {
            revision_number: 0,
            revision_height: client_state.l1_latest_slot,
        },
    );
    save_proto_client_state::<ArbitrumLightClient>(
        deps,
        ProtoClientState {
            data: msg.client_state,
            checksum: msg.checksum,
            latest_height: Some(
                Height {
                    revision_number: 0,
                    revision_height: client_state.l1_latest_slot,
                }
                .into(),
            ),
        },
    );
    Ok(Response::default())
}

define_cosmwasm_light_client_contract!(ArbitrumLightClient, Arbitrum);
