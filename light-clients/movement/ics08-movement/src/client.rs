use cosmwasm_std::Deps;
use ics008_wasm_client::{
    storage_utils::{
        read_client_state, read_consensus_state, save_client_state, save_consensus_state,
    },
    IbcClient, IbcClientError, StorageState,
};
use unionlabs::{
    aptos::{
        account::AccountAddress, ledger_info::LedgerInfoWithSignatures,
        transaction_info::TransactionInfo,
    },
    cosmwasm::wasm::union::custom_query::{query_consensus_state, UnionCustomQuery},
    encoding::{DecodeAs as _, Proto},
    hash::H256,
    ibc::{
        core::{client::height::Height, commitment::merkle_path::MerklePath},
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
            header
                .state_proof
                .latest_ledger_info()
                .commit_info
                .executed_state_id,
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
            state_root: tx_info.state_checkpoint_hash.unwrap(), // TODO(aeryz): we always need this, no need to make this not an option
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
    _path: String,
    state_root: H256,
    _table_handle: AccountAddress,
    proof: Vec<u8>,
    value: Vec<u8>,
) -> Result<(), IbcClientError<MovementLightClient>> {
    aptos_verifier::verify_existence_proof(
        &proof,
        state_root,
        H256::default(),
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
        r#"{"latest_li_w_sigs":{"V0":{"ledger_info":{"commit_info":{"epoch":1,"round":0,"id":"fc0873da8bbf7864d4903be28e465f4113567eb2cfaa424a7667036aed1cac45","executed_state_id":"98fb503a94537d1171e1714b1bfab89ef845971af5bad1f519e0ecdc4f9149a0","version":35,"timestamp_usecs":1726236671627972,"next_epoch_state":{"epoch":1,"verifier":{"validator_infos":[{"address":"d1126ce48bd65fb72190dbd9a6eaa65ba973f1e1664ac0cfba4db1d071fd0c36","public_key":"0x86fb211f41a07c6399ccc6ab3a8fe568fb0f574ce1b811896c44c6da4f267d543c6cac9fb8f4e9b92a3b809eefb91cbd","voting_power":100000000}]}}},"consensus_data_hash":"0000000000000000000000000000000000000000000000000000000000000000"},"signatures":{"validator_bitmask":{"inner":[]},"sig":null}}},"epoch_changes":{"ledger_info_with_sigs":[],"more":false}}"#,
    ).unwrap();
    let tx_proof: aptos_types::proof::TransactionInfoWithProof = serde_json::from_str(
        r#"{"ledger_info_to_transaction_info_proof":{"siblings":["b42727e6621d5bff6be27a15fa825eab64b96efa76ac5d52357517f01cdb4e3e","5f53374f457df39c6f289208b6209897d2a39d646d778f17611e48616ba8b5f1","414343554d554c41544f525f504c414345484f4c4445525f4841534800000000","414343554d554c41544f525f504c414345484f4c4445525f4841534800000000","414343554d554c41544f525f504c414345484f4c4445525f4841534800000000","639f58c8ed99ed8fdcd367773494111c2bff8815a19b7a67031e354870e8e3cd"],"phantom":null},"transaction_info":{"V0":{"gas_used":0,"status":"Success","transaction_hash":"e4b7f6418ee475f23bd2a131a2d3acf3b84e2a25aed3ced019a8a03e5f71afe6","event_root_hash":"414343554d554c41544f525f504c414345484f4c4445525f4841534800000000","state_change_hash":"afb6e14fe47d850fd0a7395bcfb997ffacf4715e0f895cc162c218e4a7564bc6","state_checkpoint_hash":"f613b73d73585f056003b9d4b60f99b7cf65932ec7a1841b171825b350442d7e","state_cemetery_hash":null}}}"#,
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
    for i in 0..257 {
        let res = aptos_verifier::verify_tx_state(
            &tx_proof,
            state_proof
                .latest_ledger_info()
                .commit_info
                .executed_state_id,
            i,
        );
        if res.is_ok() {
            println!("res: {} {}", res.is_ok(), i);
        }
    }
}
