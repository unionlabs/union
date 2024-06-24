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
    ibc::lightclients::cometbls::client_state::ClientState,
};

use crate::{client::NearLightClient, errors::Error};

#[entry_point]
pub fn instantiate(
    mut deps: DepsMut<CustomQueryOf<NearLightClient>>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Error> {
    unimplemented!()
}

define_cosmwasm_light_client_contract!(NearLightClient, Near);
