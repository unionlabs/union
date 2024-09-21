use std::io::Write as _;

use cosmwasm_std::Deps;
use hex_literal::hex;
use ics008_wasm_client::{
    storage_utils::{
        read_client_state, read_consensus_state, save_client_state, save_consensus_state,
    },
    IbcClient, IbcClientError, StorageState,
};
use sha2::{Digest, Sha256};
use sha3::Sha3_256;
use unionlabs::{
    aptos::{
        account::AccountAddress, ledger_info::LedgerInfoWithSignatures,
        sparse_merkle_proof::SparseMerkleProof, transaction_info::TransactionInfo,
    },
    cosmwasm::wasm::union::custom_query::{query_consensus_state, UnionCustomQuery},
    encoding::{DecodeAs as _, EncodeAs as _, Proto},
    hash::H256,
    ibc::{
        core::{
            client::height::Height, commitment::merkle_path::MerklePath,
            connection::connection_end::ConnectionEnd,
        },
        lightclients::{
            ethereum,
            movement::{
                client_state::ClientState, consensus_state::ConsensusState, header::Header,
            },
            wasm,
        },
    },
    uint::U256,
};

use crate::errors::Error;

type WasmClientState = wasm::client_state::ClientState<ClientState>;
type WasmConsensusState = wasm::consensus_state::ConsensusState<ConsensusState>;
type WasmL1ConsensusState =
    wasm::consensus_state::ConsensusState<ethereum::consensus_state::ConsensusState>;

#[derive(rlp::RlpEncodable)]
struct BlockCommitment {
    pub height: U256,
    pub commitment: U256,
    pub block_id: U256,
}

pub struct MovementLightClient;

impl IbcClient for MovementLightClient {
    type Error = Error;

    type CustomQuery = UnionCustomQuery;

    type Header = Header;

    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type Encoding = Proto;

    fn verify_membership(
        deps: Deps<Self::CustomQuery>,
        height: Height,
        _delay_time_period: u64,
        _delay_block_period: u64,
        proof: Vec<u8>,
        mut path: MerklePath,
        value: StorageState,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state: WasmConsensusState =
            read_consensus_state(deps, &height)?.ok_or(Error::ConsensusStateNotFound(height))?;
        let client_state: WasmClientState = read_client_state(deps)?;

        let path = path.key_path.pop().ok_or(Error::EmptyIbcPath)?;

        match value {
            StorageState::Occupied(value) => do_verify_membership(
                path,
                consensus_state.data.state_root,
                client_state.data.table_handle,
                proof,
                value,
            ),
            StorageState::Empty => unimplemented!(),
        }
    }

    fn verify_header(
        _deps: Deps<Self::CustomQuery>,
        _env: cosmwasm_std::Env,
        header: Header,
    ) -> Result<(), IbcClientError<Self>> {
        aptos_verifier::verify_tx_state(
            &header.tx_proof,
            *header
                .state_proof
                .latest_ledger_info()
                .commit_info
                .executed_state_id
                .get(),
            header.tx_index,
        )
        .map_err(Into::<Error>::into)?;

        // TODO(aeryz): make sure the given state_proof_hash_proof.key matches the correct slot

        // NOTE(aeryz): FOR AUDITORS and NERDS:
        // Movement is currently using an internal eth node to settle on, which we don't have access to get the proofs.
        // Hence, the following checks are disabled and the client is made permissioned until we have access to the proofs.

        // let client_state: WasmClientState = read_client_state(deps)?;

        // let l1_consensus_state = query_consensus_state::<WasmL1ConsensusState>(
        //     deps,
        //     &env,
        //     client_state.data.l1_client_id.to_string(),
        //     header.l1_height,
        // )
        // .map_err(Error::CustomQuery)?;

        // let expected_commitment = BlockCommitment {
        //     height: header.new_height.into(),
        //     commitment: U256::from_be_bytes(header.state_proof.hash()),
        //     block_id: U256::from_be_bytes(header.state_proof.latest_ledger_info().commit_info.id.0),
        // };

        // ethereum_verifier::verify::verify_account_storage_root(
        //     l1_consensus_state.data.state_root,
        //     &client_state.data.l1_contract_address,
        //     &header.settlement_contract_proof.proof,
        //     &header.settlement_contract_proof.storage_root,
        // )
        // .unwrap();

        // ethereum_verifier::verify::verify_storage_proof(
        //     header.settlement_contract_proof.storage_root,
        //     header.state_proof_hash_proof.key,
        //     &rlp::encode(&expected_commitment),
        //     header.state_proof_hash_proof.proof,
        // )
        // .unwrap();

        Ok(())
    }

    fn verify_misbehaviour(
        _deps: Deps<Self::CustomQuery>,
        _env: cosmwasm_std::Env,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<(), IbcClientError<Self>> {
        todo!()
    }

    fn update_state(
        mut deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
        _env: cosmwasm_std::Env,
        header: Self::Header,
    ) -> Result<Vec<Height>, IbcClientError<Self>> {
        let mut client_state: WasmClientState = read_client_state(deps.as_ref())?;

        let TransactionInfo::V0(tx_info) = header.tx_proof.transaction_info;

        let consensus_state = ConsensusState {
            state_root: H256(tx_info.state_checkpoint_hash.unwrap().get().clone()), // TODO(aeryz): we always need this, no need to make this not an option
            timestamp: header
                .state_proof
                .latest_ledger_info()
                .commit_info
                .timestamp_usecs,
            state_proof_hash: H256::default(), // TODO(aeryz): im not sure if we need this
        };

        if header.new_height > client_state.data.latest_block_num {
            client_state.data.latest_block_num = header.new_height;
            client_state.latest_height.revision_height = header.new_height;
            save_client_state::<MovementLightClient>(deps.branch(), client_state);
        }

        let update_height = Height {
            revision_number: 0,
            revision_height: header.new_height,
        };

        save_consensus_state::<MovementLightClient>(
            deps,
            WasmConsensusState {
                data: consensus_state,
            },
            &update_height,
        );

        Ok(vec![update_height])
    }

    fn update_state_on_misbehaviour(
        _deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
        _env: cosmwasm_std::Env,
        _client_message: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        todo!()
    }

    fn check_for_misbehaviour_on_header(
        _deps: Deps<Self::CustomQuery>,
        _header: Self::Header,
    ) -> Result<bool, IbcClientError<Self>> {
        todo!()
    }

    fn check_for_misbehaviour_on_misbehaviour(
        _deps: Deps<Self::CustomQuery>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<bool, IbcClientError<Self>> {
        todo!()
    }

    fn verify_upgrade_and_update_state(
        _deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
        _upgrade_client_state: Self::ClientState,
        _upgrade_consensus_state: Self::ConsensusState,
        _proof_upgrade_client: Vec<u8>,
        _proof_upgrade_consensus_state: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        todo!()
    }

    fn migrate_client_store(
        _deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
    ) -> Result<(), IbcClientError<Self>> {
        todo!()
    }

    fn status(
        _deps: Deps<Self::CustomQuery>,
        _env: &cosmwasm_std::Env,
    ) -> Result<ics008_wasm_client::Status, IbcClientError<Self>> {
        Ok(ics008_wasm_client::Status::Active)
    }

    fn export_metadata(
        _deps: Deps<Self::CustomQuery>,
        _env: &cosmwasm_std::Env,
    ) -> Result<
        Vec<unionlabs::ibc::core::client::genesis_metadata::GenesisMetadata>,
        IbcClientError<Self>,
    > {
        todo!()
    }

    fn timestamp_at_height(
        deps: Deps<Self::CustomQuery>,
        height: Height,
    ) -> Result<u64, IbcClientError<Self>> {
        Ok(read_consensus_state::<Self>(deps, &height)?
            .ok_or(Error::ConsensusStateNotFound(height))?
            .data
            .timestamp)
    }
}

fn do_verify_membership(
    path: String,
    state_root: H256,
    table_handle: AccountAddress,
    proof: Vec<u8>,
    value: Vec<u8>,
) -> Result<(), IbcClientError<MovementLightClient>> {
    aptos_verifier::verify_membership(
        &proof,
        state_root.into(),
        &table_handle,
        &bcs::to_bytes(path.as_bytes()).unwrap(),
        value.try_into().unwrap(),
    )
    .unwrap();

    Ok(())
}

#[test]
fn update_test() {
    use aptos_crypto::hash::CryptoHash;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage};
    use hex_literal::hex;

    let deps = cosmwasm_std::OwnedDeps::<MockStorage, MockApi, MockQuerier, UnionCustomQuery> {
        storage: MockStorage::new(),
        api: MockApi::default(),
        querier: MockQuerier::default(),
        custom_query_type: std::marker::PhantomData,
    };

    // let header = Header::decode_as::<Proto>(&hex!("0a021001120210081ae3010ade010ad9010ab40108011a204d9829ca3b429726929ad13123acd90fb5181d76bf92e9b4aafa04b65402144c2220147f9e2ac6513e6ca45b7561536084d4664af93ef727c33befcc4c7f440a45dd282030e6dfa687f4bf88033a610801125d0a5b0a20d1126ce48bd65fb72190dbd9a6eaa65ba973f1e1664ac0cfba4db1d071fd0c3612320a3086fb211f41a07c6399ccc6ab3a8fe568fb0f574ce1b811896c44c6da4f267d543c6cac9fb8f4e9b92a3b809eefb91cbd1880c2d72f122000000000000000000000000000000000000000000000000000000000000000001200120022f9020a201d646962a8526c7b7a1f38eccea47aea233fa952405d4bbb7a14251d526fb5680a200485a4001d0c4a54e7e3188b9a9f1c4ed0472841210e19e597e0f299e8f92f8d0a20966740644714735eeee65f1d3647e2a6635e580c81acce7cafde1c31f9166eac0a20b01d5a56dace2d05b3b065e753680f810577bc168655c6a17a6f057078eff34b0a20defdb4b8c7eefc2121d2658c4daf07a91a9a3e10b91434da1df90518e796ad5b0a205f863421b1b5692c9b2342c04eee4a592bea3f704fae7697c0fb1658d775843312aa01122007722ac0872c8bb56c3521a45a1df816acfcead14b836215ddb853091b6858a31a20414343554d554c41544f525f504c414345484f4c4445525f48415348000000002220afb6e14fe47d850fd0a7395bcfb997ffacf4715e0f895cc162c218e4a7564bc62a20c09ada319e827f0fe51b0d7c9110f6d751d649ec019e2f9b2c60aa874ed112d7322000000000000000000000000000000000000000000000000000000000000000002a440a2000000000000000000000000000000000000000000000000000000000000000001220000000000000000000000000000000000000000000000000000000000000000032220a200000000000000000000000000000000000000000000000000000000000000000380b")).unwrap();

    // let LedgerInfoWithSignatures::V0(ledger_info) = header.tx_proof.latest_li_w_sigs.clone();

    // let bcs_encoded = bcs::to_bytes(&header.tx_proof).unwrap();
    // let bcs_encoded_2 = bcs::to_bytes(&header.state_proof).unwrap();

    // let tx_proof: aptos_types::proof::TransactionInfoWithProof =
    //     bcs::from_bytes(&bcs_encoded).unwrap();
    // let state_proof: aptos_types::state_proof::StateProof =
    //     bcs::from_bytes(&bcs_encoded_2).unwrap();

    let state_proof: aptos_types::state_proof::StateProof = serde_json::from_str(
        r#"{"latest_li_w_sigs":{"V0":{"ledger_info":{"commit_info":{"epoch":1,"round":0,"id":"24a8f5eb283ce35aa9d5cdc6d7274edac99e694612e55422e3918bc04e96e7fd","executed_state_id":"942bf5b28be4893753513176fa8e850f654281be4dfebe60032d435bad3cc14f","version":86,"timestamp_usecs":1726821767444881,"next_epoch_state":{"epoch":1,"verifier":{"validator_infos":[{"address":"d1126ce48bd65fb72190dbd9a6eaa65ba973f1e1664ac0cfba4db1d071fd0c36","public_key":"0x86fb211f41a07c6399ccc6ab3a8fe568fb0f574ce1b811896c44c6da4f267d543c6cac9fb8f4e9b92a3b809eefb91cbd","voting_power":100000000}]}}},"consensus_data_hash":"0000000000000000000000000000000000000000000000000000000000000000"},"signatures":{"validator_bitmask":{"inner":[]},"sig":null}}},"epoch_changes":{"ledger_info_with_sigs":[],"more":false}}"#,
    ).unwrap();
    let tx_proof: aptos_types::proof::TransactionInfoWithProof = serde_json::from_str(
        r#"{"ledger_info_to_transaction_info_proof":{"siblings":["414343554d554c41544f525f504c414345484f4c4445525f4841534800000000","af0f8ad7c7abd9aa78231c74491141392ac75a815b449426b0e70171351a6945","9141f154a62fb34ce7e50a94b8756ed38724fd6d7d8c7248bcfd5fdb2e1ebe6d","414343554d554c41544f525f504c414345484f4c4445525f4841534800000000","a4f543e44132202cdcd4479174ef841c08857349577c71484756b023f0cc8fda","414343554d554c41544f525f504c414345484f4c4445525f4841534800000000","5b2ffcfbd58523fe5edd4f8e07928a758fa52ab815d5d299bc81b10fdc4c0186"],"phantom":null},"transaction_info":{"V0":{"gas_used":0,"status":"Success","transaction_hash":"e61c5d024c2c5392a1b2b4da25092051d8443c7c329221e90c325de0d151cece","event_root_hash":"414343554d554c41544f525f504c414345484f4c4445525f4841534800000000","state_change_hash":"afb6e14fe47d850fd0a7395bcfb997ffacf4715e0f895cc162c218e4a7564bc6","state_checkpoint_hash":"6f2a6adb2c71ebc7c66c35a2a22d1c61cbac3b904bceed84d76fd5647be2d4bc","state_cemetery_hash":null}}}"#,
    ).unwrap();
    println!("tx version: {}", state_proof.latest_ledger_info().version());

    // for i in 0..257 {
    //     let res = tx_proof.ledger_info_to_transaction_info_proof.verify(
    //         state_proof
    //             .latest_ledger_info()
    //             .commit_info()
    //             .executed_state_id(),
    //         tx_proof.transaction_info.hash(),
    //         i,
    //     );
    //     if res.is_ok() {
    //         println!("res: {} {}", res.is_ok(), i);
    //     }
    // }

    // let val_info = &header
    //     .state_proof
    //     .latest_ledger_info()
    //     .commit_info
    //     .next_epoch_state
    //     .as_ref()
    //     .cloned()
    //     .unwrap()
    //     .verifier
    //     .validator_infos[0];

    // let validator_info = aptos_types::validator_verifier::ValidatorConsensusInfo {
    //     address: aptos_types::account_address::AccountAddress::new(val_info.address.0 .0),
    //     public_key: aptos_crypto::bls12381::PublicKey::try_from(
    //         val_info.public_key.pubkey.to_vec().as_slice(),
    //     )
    //     .unwrap(),
    //     voting_power: val_info.voting_power,
    // };

    // panic!(
    //     "first: {}, second: {}",
    //     hex::encode(&bcs::to_bytes(&val_info).unwrap()),
    //     hex::encode(&bcs::to_bytes(&validator_info).unwrap())
    // );

    // let decoded: aptos_crypto::bls12381::PublicKey = bcs::from_bytes(&bcs_encoded).unwrap();

    // MovementLightClient::verify_header(deps.as_ref(), mock_env(), header).unwrap();
    let tx_proof: unionlabs::aptos::transaction_proof::TransactionInfoWithProof =
        bcs::from_bytes(&bcs::to_bytes(&tx_proof).unwrap()).unwrap();
    let state_proof: unionlabs::aptos::state_proof::StateProof =
        bcs::from_bytes(&bcs::to_bytes(&state_proof).unwrap()).unwrap();

    let state_proof = unionlabs::aptos::state_proof::StateProof::decode_as::<Proto>(
        &state_proof.encode_as::<Proto>(),
    )
    .unwrap();

    let tx_proof =
        unionlabs::aptos::transaction_proof::TransactionInfoWithProof::decode_as::<Proto>(
            &tx_proof.encode_as::<Proto>(),
        )
        .unwrap();

    for i in 0..257 {
        let res = aptos_verifier::verify_tx_state(
            &tx_proof,
            state_proof
                .latest_ledger_info()
                .commit_info
                .executed_state_id
                .get()
                .clone(),
            i,
        );
        if res.is_ok() {
            println!("res: {} {}", res.is_ok(), i);
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename = "StateValue")]
enum PersistedStateValue {
    V0(Vec<u8>),
    WithMetadata {
        data: Vec<u8>,
        metadata: PersistedStateValueMetadata,
    },
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename = "StateValueMetadata")]
pub enum PersistedStateValueMetadata {
    V0 {
        deposit: u64,
        creation_time_usecs: u64,
    },
    V1 {
        slot_deposit: u64,
        bytes_deposit: u64,
        creation_time_usecs: u64,
    },
}

#[test]
fn test_mem_ver() {
    // "state_key_hash": "0xa0f3409f6d658f56daa8e092107ee81488d7e99d3dcf0aa2fd54857823a39a9b",
    // "handle": "0x6a95fc19afdc28b79ac96a39151b4cec85b1281d1bc0c246f3d369688439e17a",
    // "key": "0x18636f6e6e656374696f6e732f636f6e6e656374696f6e2d34",
    let value = hex!("206c979155ae4fb94adc1be3e5597fef73ec5ec3bfd941e76301b06ad5f3c8bfdb");

    let state_key_hash = hex!("a0f3409f6d658f56daa8e092107ee81488d7e99d3dcf0aa2fd54857823a39a9b");
    let table_handle = hex!("6a95fc19afdc28b79ac96a39151b4cec85b1281d1bc0c246f3d369688439e17a");
    let key = hex!("18636f6e6e656374696f6e732f636f6e6e656374696f6e2d34");

    let handle = AccountAddress(unionlabs::hash::hash_v2::Hash::new(hex!(
        "6a95fc19afdc28b79ac96a39151b4cec85b1281d1bc0c246f3d369688439e17a"
    )));

    let mut buf = vec![1];
    bcs::serialize_into(&mut buf, &handle).unwrap();
    buf.write_all(&key).unwrap();
    use aptos_crypto::HashValue;

    let hash = Sha3_256::new()
        .chain_update(Sha3_256::new().chain_update("APTOS::StateKey").finalize())
        .chain_update(&buf)
        .finalize()
        .to_vec();

    println!("state key {state_key_hash:?} vs hash {hash:?}");

    let proof: aptos_types::proof::SparseMerkleProof = serde_json::from_str(
        r#"{"leaf":{"key":"a0f3409f6d658f56daa8e092107ee81488d7e99d3dcf0aa2fd54857823a39a9b","value_hash":"e0f1455e9aa503607c640ec3a1686d1c880e8c5764e9f8d835546b64967513c6"},"siblings":["5d7706cc7dd8c67b9f7f591df480fac0cb4072b4fb6e0eb174db30f3dd21e0cf","b4d5bcecf9fe8c498fd47534b58bb884db1d3e3aac199a9d7628b322dfe9ad94","cbf6cd565500808d340a730222b7cd010cf6bf15c66d4a1a242a50d0c18e7cc3","09e8788b148cd7c676984da74ed74662081bacd7e626428acb76ef197f84da83","d4966172dff6b30ad1c8831b00a4915aecc30bf69df8c44d068ece0e1022b19b","c975874bf196e43483b134dbe97d00b4108a18fc3322b9807121986f05a6b3ba","5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000","794524a7d7b3d5010a7d124fedbdf5e841da55290a52809d09911c04c71cee49"]}"#,
    ).unwrap();

    let hash_value = Sha3_256::new()
        .chain_update(Sha3_256::new().chain_update("APTOS::StateValue").finalize())
        .chain_update(
            &bcs::to_bytes(&PersistedStateValue::WithMetadata {
                data: value.to_vec(),
                metadata: PersistedStateValueMetadata::V1 {
                    slot_deposit: 40000,
                    bytes_deposit: 3600,
                    creation_time_usecs: 1726830371849456,
                },
            })
            .unwrap(),
        )
        .finalize()
        .to_vec();

    println!("value: {}", hex::encode(&hash_value));

    proof
        .verify_by_hash(
            HashValue::new(hex!(
                "d9c8ce5f6d8eac96858d81325d886a0928ef1680ef430d270a3b69742fe984e6"
            )),
            HashValue::new(hex!(
                "a0f3409f6d658f56daa8e092107ee81488d7e99d3dcf0aa2fd54857823a39a9b"
            )),
            Some(HashValue::new(hex!(
                "e0f1455e9aa503607c640ec3a1686d1c880e8c5764e9f8d835546b64967513c6"
            ))),
        )
        .unwrap();

    // do_verify_membership(
    //     "connections/connection-4".into(),
    //     hex!("d9c8ce5f6d8eac96858d81325d886a0928ef1680ef430d270a3b69742fe984e6").into(),
    //     handle,
    //     proof.encode_as::<Proto>(),
    //     hex!("e0f1455e9aa503607c640ec3a1686d1c880e8c5764e9f8d835546b64967513c6").to_vec(),
    // );
}
