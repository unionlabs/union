use std::io::Read;

use cometbls_light_client_types::{ChainId, ClientState, ConsensusState};
use cosmwasm_std::{
    testing::{message_info, mock_dependencies, mock_env},
    to_json_binary,
};
use union_ibc::contract::{
    events::{self, attribute},
    execute, instantiate,
};
use union_ibc_msg::{
    lightclient::QueryMsg as LightClientQueryMsg,
    msg::{ExecuteMsg, InitMsg, MsgCreateClient, MsgRegisterClient},
};
use unionlabs::{
    bytes::Bytes,
    encoding::{Decode, EncodeAs, EthAbi},
    ethereum::keccak256,
    hash::hash_v2::{Base64, HexUnprefixed},
    ibc::core::{client::height::Height, commitment::merkle_root::MerkleRoot},
};

use super::*;

const SENDER: &str = "admin_sender";
const CLIENT_TYPE: &str = "cometbls";
const RELAYER: &str = "relayer";

// Client state starting values
const CHAIN_ID: &str = "test-chain";
const TRUSTING_PERIOD: u64 = 86400;
const MAX_CLOCK_DRIFT: u64 = 300;
const FROZEN_HEIGHT: Height = Height::new(0);
const LATEST_HEIGHT: Height = Height::new(100);
const CONTRACT_ADDRESS_SEED: &str = "test-contract";

// Consensus state starting values
const TIMESTAMP: u64 = 1337;
const APP_HASH_SEED: &str = "app";
const NEXT_VALIDATOR_HASH_SEED: &str = "validators";

fn misbehaviour_common(trusting_period: u64) -> u32 {
    let mut deps = mock_dependencies();

    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        InitMsg {},
    )
    .expect("instantiate ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Encode client state
    let client_state = ClientState {
        chain_id: ChainId::from_string(CHAIN_ID).expect("is valid chain ID"),
        trusting_period,
        max_clock_drift: trusting_period,
        frozen_height: FROZEN_HEIGHT,
        latest_height: Height::new(99),
        contract_address: keccak256(CONTRACT_ADDRESS_SEED),
    };
    let client_state_bytes = Bytes::from(client_state.clone().encode_as::<EthAbi>());

    // Encode consensus state
    let consensus_state = ConsensusState {
        timestamp: TIMESTAMP,
        app_hash: MerkleRoot {
            hash: keccak256(APP_HASH_SEED).into_encoding::<Base64>(),
        },
        next_validators_hash: keccak256(NEXT_VALIDATOR_HASH_SEED).into_encoding::<HexUnprefixed>(),
    };
    let consensus_state_bytes = Bytes::from(consensus_state.clone().encode_as::<EthAbi>());

    // Register client type
    let msg = MsgRegisterClient {
        client_type: CLIENT_TYPE.to_owned(),
        client_address: mock_addr(CLIENT_TYPE).into_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::RegisterClient(msg),
    )
    .expect("register client ok");

    // Create client
    let msg = MsgCreateClient {
        client_type: CLIENT_TYPE.to_owned(),
        client_state_bytes,
        consensus_state_bytes: consensus_state_bytes.clone(),
        relayer: mock_addr(RELAYER).into_string(),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::CreateClient(msg),
    )
    .expect("create client ok");
    let client_id: u32 = res
        .events
        .iter()
        .find(|event| event.ty.eq(events::client::CREATE))
        .expect("create client event exists")
        .attributes
        .iter()
        .find(|attribute| attribute.key.eq(events::attribute::CLIENT_ID))
        .expect("client type attribute exists")
        .value
        .parse()
        .expect("client type string is u32");

    dbg!(client_id);

    let client_state = ClientState {
        latest_height: Height::new(100),
        ..client_state
    };
    let client_state_bytes = Bytes::from(client_state.clone().encode_as::<EthAbi>());
    // Create client
    let msg = MsgCreateClient {
        client_type: CLIENT_TYPE.to_owned(),
        client_state_bytes,
        consensus_state_bytes,
        relayer: mock_addr(RELAYER).into_string(),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::CreateClient(msg),
    )
    .expect("create client ok");
    let client_id = res
        .events
        .iter()
        .find(|event| event.ty.eq(events::client::CREATE))
        .expect("create client event exists")
        .attributes
        .iter()
        .find(|attribute| attribute.key.eq(events::attribute::CLIENT_ID))
        .expect("client type attribute exists")
        .value
        .parse()
        .expect("client type string is u32");

    client_id
}

#[test]
fn create_client_success() {
    let mut deps = mock_dependencies();

    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        InitMsg {},
    )
    .expect("instantiate ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Encode client state
    let client_state = ClientState {
        chain_id: ChainId::from_string(CHAIN_ID).expect("is valid chain ID"),
        trusting_period: TRUSTING_PERIOD,
        max_clock_drift: MAX_CLOCK_DRIFT,
        frozen_height: FROZEN_HEIGHT,
        latest_height: LATEST_HEIGHT,
        contract_address: keccak256(CONTRACT_ADDRESS_SEED),
    };
    let client_state_bytes = Bytes::from(client_state.clone().encode_as::<EthAbi>());

    // Encode consensus state
    let consensus_state = ConsensusState {
        timestamp: TIMESTAMP,
        app_hash: MerkleRoot {
            hash: keccak256(APP_HASH_SEED).into_encoding::<Base64>(),
        },
        next_validators_hash: keccak256(NEXT_VALIDATOR_HASH_SEED).into_encoding::<HexUnprefixed>(),
    };
    let consensus_state_bytes = Bytes::from(consensus_state.clone().encode_as::<EthAbi>());

    // Register client type
    let msg = MsgRegisterClient {
        client_type: CLIENT_TYPE.to_owned(),
        client_address: mock_addr(CLIENT_TYPE).into_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::RegisterClient(msg),
    )
    .expect("register client ok");

    // Create client
    let msg = MsgCreateClient {
        client_type: CLIENT_TYPE.to_owned(),
        client_state_bytes,
        consensus_state_bytes,
        relayer: mock_addr(RELAYER).into_string(),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::CreateClient(msg),
    )
    .expect("create client ok");
    let client_id: u32 = res
        .events
        .iter()
        .find(|event| event.ty.eq(events::client::CREATE))
        .expect("create client event exists")
        .attributes
        .iter()
        .find(|attribute| attribute.key.eq(events::attribute::CLIENT_ID))
        .expect("client type attribute exists")
        .value
        .parse()
        .expect("client type string is u32");

    // Verify client state was stored
    let stored_client_state = <ClientState as Decode<EthAbi>>::decode(
        &union_ibc::state::CLIENT_STATES
            .load(&deps.storage, client_id)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(stored_client_state, client_state);

    // Verify consensus state was stored
    let stored_consensus_state = <ConsensusState as Decode<EthAbi>>::decode(
        &union_ibc::state::CLIENT_CONSENSUS_STATES
            .load(&deps.storage, (client_id, 1))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(stored_consensus_state, consensus_state);
}

#[test]
fn misbehaviour_freezes_client() {
    todo!("Mock CometBLS client and zkp verifier")
}
