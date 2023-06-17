use std::{
    ops::Deref,
    time::{SystemTime, UNIX_EPOCH},
};

use base64::{prelude::BASE64_STANDARD, Engine};
use bip32::XPrv;
use k256::{ecdsa::Signature, schnorr::signature::Signer};
// use ethers::providers::{Http, Provider};
use lodestar_rpc::types::{
    BeaconHeaderResponse, LightClientBootstrapResponse, LightClientFinalityUpdateData,
    LightClientFinalityUpdateResponse,
};
use prost::Message;
use protos::{
    cosmos::{
        ics23::v1::CompressedNonExistenceProof,
        tx::{
            self,
            v1beta1::{SignDoc, TxBody, TxRaw},
        },
    },
    google::protobuf::Any,
    ibc::{
        core::client::v1::{
            Height, MsgCreateClient, MsgUpdateClient, QueryClientStateRequest,
            QueryClientStateResponse,
        },
        lightclients::wasm::{self, v1::QueryCodeIdsRequest},
    },
    tendermint::rpc::grpc::ResponseBroadcastTx,
    union::ibc::lightclients::ethereum::{
        self,
        v1::{
            AccountUpdate, BeaconBlockHeader, ExecutionPayloadHeader, LightClientUpdate, Proof,
            SyncAggregate, TrustedSyncCommittee,
        },
    },
};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use ripemd::Digest;
use serde_json::{json, Value};
use tendermint_rpc::Client;

use crate::{account_info_of_signer, get_wallet, ETH_BEACON_RPC_API};

pub async fn get_genesis() -> lodestar_rpc::types::GenesisData {
    let client = lodestar_rpc::client::RPCClient::new(ETH_BEACON_RPC_API);
    client.get_genesis().await.unwrap().data
}

pub async fn get_latest_finalized_block() -> serde_json::Value {
    const API: &str = "eth/v2/debug/beacon/states";
    reqwest::Client::new()
        .get(format!("{ETH_BEACON_RPC_API}/{API}/finalized"))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap()
        .get("data")
        .unwrap()
        .clone()
}

pub async fn create_wasm_client() -> tendermint_rpc::endpoint::broadcast::tx_commit::Response {
    println!("[ i ] Creating a new wasm client..");

    let alice = get_wallet();
    let alice_pk = alice.public_key().public_key().to_bytes().to_vec();

    let mut client = wasm::v1::query_client::QueryClient::connect("tcp://0.0.0.0:9090")
        .await
        .unwrap();

    let genesis = lodestar_rpc::client::RPCClient::new(ETH_BEACON_RPC_API)
        .get_genesis()
        .await
        .unwrap()
        .data;

    let latest_finalized_block = get_latest_finalized_block().await;

    println!(
        "{}",
        serde_json::to_string_pretty(&latest_finalized_block).unwrap()
    );

    let latest_slot = latest_finalized_block["slot"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    let eth_client_state = ethereum::v1::ClientState {
        genesis_validators_root: genesis.genesis_validators_root.as_bytes().to_vec(),
        genesis_time: genesis.genesis_time.0,
        fork_parameters: Some(ethereum::v1::ForkParameters {
            genesis_fork_version: vec![0, 0, 0, 1],
            genesis_slot: 0,
            altair: Some(ethereum::v1::Fork {
                version: vec![1, 0, 0, 1],
                epoch: 0,
            }),
            bellatrix: Some(ethereum::v1::Fork {
                version: vec![2, 0, 0, 1],
                epoch: 0,
            }),
            capella: Some(ethereum::v1::Fork {
                version: vec![3, 0, 0, 1],
                epoch: 0,
            }),
            eip4844: Some(ethereum::v1::Fork {
                version: vec![4, 0, 0, 0],
                epoch: u64::MAX,
            }),
        }),
        seconds_per_slot: 6,
        slots_per_epoch: 8,
        epochs_per_sync_committee_period: 8,
        trusting_period: 100000000,
        latest_slot: dbg!(latest_slot),
        min_sync_committee_participants: 0,
        trust_level: Some(ethereum::v1::Fraction {
            numerator: 1,
            denominator: 3,
        }),
        frozen_height: None,
        counterparty_commitment_slot: 3,
    };

    let trusted_header = lodestar_rpc::client::RPCClient::new(ETH_BEACON_RPC_API)
        .get_beacon_header_by_slot(dbg!(ethereum_consensus::types::U64(latest_slot)))
        .await
        .unwrap()
        .data;

    // dbg!(&trusted_header);

    let bootstrap = reqwest::get(dbg!(format!(
        "http://0.0.0.0:9596/eth/v1/beacon/light_client/bootstrap/0x{}",
        trusted_header.root
    )))
    .await
    .unwrap()
    .json::<LightClientBootstrapResponse<32, 256, 32>>()
    // .json::<Value>()
    .await
    .unwrap()
    .data;

    dbg!(&bootstrap);

    let eth_consensus_state = ethereum::v1::ConsensusState {
        slot: bootstrap.header.beacon.slot.0,
        storage_root: vec![1, 2, 3],
        timestamp: bootstrap.header.execution.timestamp.0,
        current_sync_committee: bootstrap.current_sync_committee.aggregate_pubkey.to_vec(),
        next_sync_committee: vec![],
    };

    let msg = Any {
        type_url: "/ibc.core.client.v1.MsgCreateClient".into(),
        value: MsgCreateClient {
            signer: signer_from_pk(&alice_pk),
            client_state: Some(Any {
                type_url: "/ibc.lightclients.wasm.v1.ClientState".into(),
                value: wasm::v1::ClientState {
                    data: eth_client_state.encode_to_vec(),
                    code_id: ethers::utils::hex::decode(dbg!(
                        &client
                            .code_ids(QueryCodeIdsRequest { pagination: None })
                            .await
                            .unwrap()
                            .into_inner()
                            .code_ids
                            .first()
                            .unwrap()[1..]
                    ))
                    .unwrap(),
                    latest_height: Some(Height {
                        revision_number: 1,
                        revision_height: latest_slot,
                    }),
                }
                .encode_to_vec(),
            }),
            consensus_state: Some(Any {
                type_url: "/ibc.lightclients.wasm.v1.ConsensusState".into(),
                value: wasm::v1::ConsensusState {
                    data: eth_consensus_state.encode_to_vec(),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                }
                .encode_to_vec(),
            }),
        }
        .encode_to_vec(),
    };

    broadcast_tx_commit([msg].to_vec()).await
}

// // ethereum::v1::ClientState
// pub async fn get_wasm_code() -> u64 {
//     // let msg_client = MsgClient::connect("").await.unwrap();

//     // msg_client.store_code(request)

//     let mut client =
//         protos::ibc::core::client::v1::query_client::QueryClient::connect("tcp://0.0.0.0:9090")
//             .await
//             .unwrap();

//     let QueryClientStateResponse {
//         client_state: Some(client_state),
//         proof: _,
//         proof_height: _,
//     } = client
//         .client_state(QueryClientStateRequest {
//             client_id: WASM_CLIENT_ID.into(),
//         })
//         .await
//         .unwrap()
//         .into_inner()
//     else {
//         panic!()
//     };

//     assert_eq!(
//         client_state.type_url,
//         "/ibc.lightclients.wasm.v1.ClientState"
//     );

//     let wasm_client_state = wasm::v1::ClientState::decode(&*client_state.value).unwrap();

//     // let eth_client_state = ethereum::v1::ClientState::decode(&*wasm_client_state.data).unwrap();

//     dbg!(wasm_client_state)
//         .latest_height
//         .unwrap()
//         .revision_height

//     // (height, dbg!(eth_client_state))

//     // let mut query_client = QueryClient::connect("").await.unwrap();

//     // let code_ids = query_client
//     //     .code_ids(QueryCodeIdsRequest { pagination: None })
//     //     .await
//     //     .unwrap()
//     //     .into_inner()
//     //     .code_ids;

//     // query_client
//     //     .code(QueryCodeRequest {
//     //         code_id: code_ids[0],
//     //     })
//     //     .await
//     //     .unwrap()
//     //     .into_inner()
//     //     .code;
// }

async fn wasm_client() -> wasm::v1::query_client::QueryClient<tonic::transport::Channel> {
    wasm::v1::query_client::QueryClient::connect("tcp://0.0.0.0:9090")
        .await
        .unwrap()
}

// pub async fn query_for_wasm_light_client() -> QueryClientStateResponse {
//     protos::ibc::core::client::v1::query_client::QueryClient::connect("tcp://0.0.0.0:9090")
//         .await
//         .unwrap()
//         .client_state(QueryClientStateRequest {
//             client_id: WASM_CLIENT_ID.into(),
//         })
//         .await
//         .unwrap()
//         .into_inner()
// }

#[allow(clippy::upper_case_acronyms)]
pub type LCFUR = LightClientFinalityUpdateResponse<32, 256, 32>;

// pub async fn update_wasm_client(sequence: u64) {
//     let wasm_client_state = query_for_wasm_light_client().await;

//     let trusted_slot = ethereum::v1::ClientState::decode(
//         &*wasm::v1::ClientState::decode(&*wasm_client_state.client_state.unwrap().value)
//             .unwrap()
//             .data,
//     )
//     .unwrap()
//     .latest_slot;

//     let finality_update = loop {
//         let finality_update =
//             reqwest::get("http://0.0.0.0:9596/eth/v1/beacon/light_client/finality_update")
//                 .await
//                 .unwrap()
//                 .json::<LCFUR>()
//                 .await
//                 .unwrap();

//         let slot = finality_update.data.finalized_header.beacon.slot.0;

//         eprintln!(">>>>>>>>>>> current: {trusted_slot} Update to: {slot}");

//         if slot > trusted_slot {
//             break finality_update;
//         }

//         tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
//     };

//     // dbg!(&finality_update);

//     let trusted_block = reqwest::get(format!(
//         "http://0.0.0.0:9596/eth/v1/beacon/headers/{trusted_slot}",
//     ))
//     .await
//     .unwrap()
//     .json::<BeaconHeaderResponse>()
//     .await
//     .unwrap();

//     let bootstrap: LightClientBootstrapResponse<32, 256, 32> = serde_json::from_value(
//         dbg!(
//             reqwest::get(dbg!(format!(
//                 "http://0.0.0.0:9596/eth/v1/beacon/light_client/bootstrap/0x{}",
//                 trusted_block.data.root
//             )))
//             .await
//             .unwrap()
//             .json::<Value>()
//             // .json::<LightClientBootstrapResponse<32, 256, 32>>()
//             .await
//         )
//         .unwrap(),
//     )
//     .unwrap();

//     // dbg!(&bootstrap);

//     // arbitrary data
//     let account_update = AccountUpdate { proofs: vec![] };

//     let wasm_header = wasm::v1::Header {
//         data: ethereum::v1::Header {
//             trusted_sync_committee: Some(TrustedSyncCommittee {
//                 trusted_height: Some(Height {
//                     revision_number: 0,
//                     revision_height: bootstrap.data.header.beacon.slot.0,
//                 }),
//                 sync_committee: Some(ethereum::v1::SyncCommittee {
//                     pubkeys: bootstrap
//                         .data
//                         .current_sync_committee
//                         .pubkeys
//                         .iter()
//                         .map(|x| x.0.iter().copied().collect())
//                         .collect(),
//                     aggregate_pubkey: bootstrap
//                         .data
//                         .current_sync_committee
//                         .aggregate_pubkey
//                         .iter()
//                         .copied()
//                         .collect(),
//                 }),
//                 is_next: false,
//             }),
//             consensus_update: Some({
//                 let LightClientFinalityUpdateData {
//                     attested_header,
//                     finalized_header,
//                     finality_branch,
//                     sync_aggregate,
//                     signature_slot,
//                 } = finality_update.data;

//                 ethereum::v1::LightClientUpdate {
//                     attested_header: Some(translate_header(attested_header)),
//                     next_sync_committee: None,
//                     next_sync_committee_branch: vec![],
//                     finalized_header: Some(translate_header(finalized_header)),
//                     finality_branch: finality_branch
//                         .iter()
//                         .map(|x| x.as_bytes().to_vec())
//                         .collect(),
//                     sync_aggregate: Some(SyncAggregate {
//                         sync_committee_bits: sync_aggregate
//                             .sync_committee_bits
//                             .deref()
//                             .as_bitslice()
//                             .to_bitvec()
//                             .into_vec(),
//                         sync_committee_signature: sync_aggregate
//                             .sync_committee_signature
//                             .iter()
//                             .copied()
//                             .collect(),
//                     }),
//                     signature_slot: signature_slot.0,
//                 }
//             }),
//             account_update: Some(account_update),
//             timestamp: SystemTime::now()
//                 .duration_since(UNIX_EPOCH)
//                 .unwrap()
//                 .as_secs(),
//         }
//         .encode_to_vec(),
//         height: Some(Height {
//             revision_number: 0,
//             revision_height: 10_000,
//         }),
//     };

//     let alice = get_wallet();

//     let alice_pk = alice.public_key().public_key().to_bytes().to_vec();

//     dbg!(&alice_pk);

//     let messages = [Any {
//         type_url: "/ibc.core.client.v1.MsgUpdateClient".to_string(),
//         value: MsgUpdateClient {
//             client_id: WASM_CLIENT_ID.to_string(),
//             client_message: Some(protos::google::protobuf::Any {
//                 type_url: "/ibc.lightclients.wasm.v1.Header".to_string(),
//                 value: wasm_header.encode_to_vec(),
//             }),
//             signer: signer_from_pk(&alice_pk),
//         }
//         .encode_to_vec(),
//     }]
//     .to_vec();

//     broadcast_tx_commit(messages).await;
// }

pub async fn broadcast_tx_commit(
    messages: Vec<Any>,
) -> tendermint_rpc::endpoint::broadcast::tx_commit::Response {
    let alice = get_wallet();
    let alice_pk = alice.public_key().public_key().to_bytes().to_vec();

    let account = account_info_of_signer(&alice).await;

    let sign_doc = SignDoc {
        body_bytes: TxBody {
            messages,
            memo: "".into(),
            timeout_height: 123_123_123,
            extension_options: vec![],
            non_critical_extension_options: vec![],
        }
        .encode_to_vec(),
        auth_info_bytes: tx::v1beta1::AuthInfo {
            signer_infos: [tx::v1beta1::SignerInfo {
                public_key: Some(Any {
                    type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
                    value: alice_pk.encode_to_vec(),
                }),
                mode_info: Some(tx::v1beta1::ModeInfo {
                    sum: Some(tx::v1beta1::mode_info::Sum::Single(
                        tx::v1beta1::mode_info::Single {
                            mode: tx::signing::v1beta1::SignMode::Direct.into(),
                        },
                    )),
                }),
                sequence: account.sequence,
            }]
            .to_vec(),
            fee: Some(tx::v1beta1::Fee {
                amount: vec![protos::cosmos::base::v1beta1::Coin {
                    denom: "stake".to_string(),
                    amount: "1".to_string(),
                }],
                gas_limit: 5_000_000,
                payer: "".to_string(),
                granter: "".to_string(),
            }),
            tip: None,
        }
        .encode_to_vec(),
        chain_id: "union-devnet-1".to_string(),
        account_number: 0,
    };

    let alice_signature =
        Signer::<Signature>::try_sign(alice.private_key(), &sign_doc.encode_to_vec())
            .unwrap()
            .to_vec();

    let tx_raw = TxRaw {
        body_bytes: sign_doc.body_bytes,
        auth_info_bytes: sign_doc.auth_info_bytes,
        signatures: [alice_signature].to_vec(),
    };

    let tm_client = tendermint_rpc::HttpClient::new("http://0.0.0.0:26657").unwrap();

    let response = tm_client
        .broadcast_tx_commit(tx_raw.encode_to_vec())
        .await
        .unwrap();

    response
}

pub fn signer_from_pk(alice_pk: &Vec<u8>) -> String {
    subtle_encoding::bech32::encode(
        "union",
        ripemd::Ripemd160::new()
            .chain_update(sha2::Sha256::new().chain_update(alice_pk).finalize())
            .finalize(),
    )
}

pub fn signer_from_sk(sk: &XPrv) -> String {
    subtle_encoding::bech32::encode(
        "union",
        ripemd::Ripemd160::new()
            .chain_update(
                sha2::Sha256::new()
                    .chain_update(sk.public_key().public_key().to_bytes())
                    .finalize(),
            )
            .finalize(),
    )
}

fn translate_header(
    header: ethereum_consensus::capella::LightClientHeader<256, 32>,
) -> ethereum::v1::LightClientHeader {
    ethereum::v1::LightClientHeader {
        beacon: Some(BeaconBlockHeader {
            slot: header.beacon.slot.0,
            proposer_index: header.beacon.proposer_index.0,
            parent_root: header.beacon.parent_root.as_bytes().to_vec(),
            state_root: header.beacon.state_root.as_bytes().to_vec(),
            body_root: header.beacon.body_root.as_bytes().to_vec(),
        }),
        execution: Some(ExecutionPayloadHeader {
            parent_hash: header.execution.parent_hash.as_bytes().to_vec(),
            fee_recipient: header.execution.fee_recipient.0.to_vec(),
            state_root: header.execution.state_root.as_bytes().to_vec(),
            receipts_root: header.execution.receipts_root.as_bytes().to_vec(),
            logs_bloom: header.execution.logs_bloom.iter().copied().collect(),
            prev_randao: header.execution.prev_randao.as_bytes().to_vec(),
            block_number: header.execution.block_number.0,
            gas_limit: header.execution.gas_limit.0,
            gas_used: header.execution.gas_used.0,
            timestamp: header.execution.timestamp.0,
            extra_data: header.execution.extra_data.iter().copied().collect(),
            base_fee_per_gas: header.execution.base_fee_per_gas.to_bytes_le(),
            block_hash: header.execution.block_hash.as_bytes().to_vec(),
            transactions_root: header.execution.transactions_root.as_bytes().to_vec(),
            withdrawals_root: header.execution.withdrawals_root.as_bytes().to_vec(),
        }),
        execution_branch: header
            .execution_branch
            .iter()
            .map(|x| x.0.to_vec())
            .collect(),
    }
}
