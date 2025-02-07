use beacon_api_types::{ChainSpec, Mainnet, Minimal, PresetBaseKind};
use cosmwasm_std::Empty;
use ethereum_light_client_types::{
    ClientState, ConsensusState, Header, LightClientUpdate, Misbehaviour, StorageProof,
};
use ethereum_sync_protocol::{
    utils::{
        compute_slot_at_timestamp, compute_timestamp_at_slot, validate_signature_supermajority,
    },
    validate_light_client_update,
};
use evm_storage_verifier::{
    verify_account_storage_root, verify_storage_absence, verify_storage_proof,
};
use ibc_union_light_client::{IbcClientCtx, IbcClientError};
use ibc_union_msg::lightclient::{Status, VerifyCreationResponseEvent};
use unionlabs::{
    encoding::Bincode,
    ensure,
    ethereum::ibc_commitment_key,
    ibc::core::client::height::Height,
    primitives::{H256, U256},
};

use crate::{errors::Error, verification::VerificationContext};

pub enum EthereumLightClient {}

impl ibc_union_light_client::IbcClient for EthereumLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    type Misbehaviour = Misbehaviour;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type StorageProof = StorageProof;

    type Encoding = Bincode;

    fn verify_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;
        Ok(verify_membership(
            key,
            consensus_state.storage_root,
            storage_proof,
            value,
        )?)
    }

    fn verify_non_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;
        Ok(verify_non_membership(
            key,
            consensus_state.storage_root,
            storage_proof,
        )?)
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64 {
        consensus_state.timestamp
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        client_state.latest_height
    }

    fn get_counterparty_chain_id(client_state: &Self::ClientState) -> String {
        client_state.chain_id.to_string()
    }

    fn status(client_state: &Self::ClientState) -> Status {
        if client_state.frozen_height.height() != 0 {
            Status::Frozen
        } else {
            Status::Active
        }
    }

    fn verify_creation(
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
    ) -> Result<Option<Vec<VerifyCreationResponseEvent>>, IbcClientError<EthereumLightClient>> {
        Ok(None)
    }

    fn verify_header(
        ctx: IbcClientCtx<Self>,
        header: Header,
        _caller: cosmwasm_std::Addr,
    ) -> Result<(u64, Self::ClientState, Self::ConsensusState), IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(header.trusted_height.height())?;

        match client_state.chain_spec {
            PresetBaseKind::Minimal => {
                verify_header::<Minimal>(&ctx, client_state, consensus_state, header)
            }
            PresetBaseKind::Mainnet => {
                verify_header::<Mainnet>(&ctx, client_state, consensus_state, header)
            }
        }
        .map_err(Into::into)
    }

    fn misbehaviour(
        ctx: IbcClientCtx<Self>,
        misbehaviour: Self::Misbehaviour,
    ) -> Result<Self::ClientState, IbcClientError<Self>> {
        let consensus_state =
            ctx.read_self_consensus_state(misbehaviour.trusted_height.height())?;

        let mut client_state = ctx.read_self_client_state()?;

        match client_state.chain_spec {
            PresetBaseKind::Minimal => {
                verify_misbehaviour::<Minimal>(&ctx, &client_state, consensus_state, misbehaviour)?
            }
            PresetBaseKind::Mainnet => {
                verify_misbehaviour::<Mainnet>(&ctx, &client_state, consensus_state, misbehaviour)?
            }
        }

        client_state.frozen_height = Height::new(1);

        Ok(client_state)
    }
}

pub fn verify_membership(
    key: Vec<u8>,
    storage_root: H256,
    storage_proof: StorageProof,
    value: Vec<u8>,
) -> Result<(), Error> {
    check_commitment_key(
        H256::try_from(&key).map_err(|_| Error::InvalidCommitmentKeyLength(key))?,
        storage_proof.key,
    )?;

    let value = H256::try_from(&value).map_err(|_| Error::InvalidCommitmentValueLength(value))?;

    let proof_value = H256::from(storage_proof.value.to_be_bytes());

    if value != proof_value {
        return Err(Error::StoredValueMismatch {
            expected: value,
            stored: proof_value,
        });
    }

    verify_storage_proof(
        storage_root,
        storage_proof.key,
        &rlp::encode(&storage_proof.value),
        storage_proof.proof,
    )
    .map_err(Error::VerifyStorageProof)
}

/// Verifies that no value is committed at `path` in the counterparty light client's storage.
pub fn verify_non_membership(
    key: Vec<u8>,
    storage_root: H256,
    storage_proof: StorageProof,
) -> Result<(), Error> {
    check_commitment_key(
        H256::try_from(&key).map_err(|_| Error::InvalidCommitmentKeyLength(key))?,
        storage_proof.key,
    )?;

    if verify_storage_absence(storage_root, storage_proof.key, &storage_proof.proof)
        .map_err(Error::VerifyStorageAbsence)?
    {
        Ok(())
    } else {
        Err(Error::CounterpartyStorageNotNil)
    }
}

pub fn check_commitment_key(path: H256, key: U256) -> Result<(), Error> {
    let expected_commitment_key = ibc_commitment_key(path);

    if expected_commitment_key != key {
        Err(Error::InvalidCommitmentKey {
            expected: expected_commitment_key,
            found: key,
        })
    } else {
        Ok(())
    }
}

pub fn verify_header<C: ChainSpec>(
    ctx: &IbcClientCtx<EthereumLightClient>,
    client_state: ClientState,
    consensus_state: ConsensusState,
    header: Header,
) -> Result<(u64, ClientState, ConsensusState), Error> {
    // NOTE(aeryz): Ethereum consensus-spec says that we should use the slot
    // at the current timestamp.
    let current_slot =
        compute_slot_at_timestamp::<C>(client_state.genesis_time, ctx.env.block.time.seconds())
            .ok_or(Error::IntegerOverflow)?;

    let (current_sync_committee, next_sync_committee) =
        header.consensus_update.currently_trusted_sync_committee();

    validate_light_client_update::<C, _>(
        &header.consensus_update.clone().into(),
        current_sync_committee,
        next_sync_committee,
        current_slot,
        consensus_state.slot,
        client_state.genesis_validators_root,
        &client_state.fork_parameters,
        VerificationContext { deps: ctx.deps },
    )
    .map_err(Error::ValidateLightClient)?;

    let update_data = header.consensus_update.update_data();

    // check whether at least 2/3 of the sync committee signed
    ensure(
        validate_signature_supermajority(&update_data.sync_aggregate.sync_committee_bits),
        Error::NotEnoughSignatures,
    )?;

    let proof_data = &header.ibc_account_proof;

    verify_account_storage_root(
        update_data.finalized_header.execution.state_root,
        &client_state.ibc_contract_address,
        &proof_data.proof,
        &proof_data.storage_root,
    )
    .map_err(Error::VerifyAccountStorageRoot)?;

    update_state::<C>(client_state, consensus_state, header)
}

fn update_state<C: ChainSpec>(
    mut client_state: ClientState,
    mut consensus_state: ConsensusState,
    header: Header,
) -> Result<(u64, ClientState, ConsensusState), Error> {
    let trusted_height = header.trusted_height;

    let consensus_update = header.consensus_update.update_data();

    if let LightClientUpdate::EpochChange(update) = &header.consensus_update {
        consensus_state.current_sync_committee = consensus_state.next_sync_committee;
        consensus_state.next_sync_committee = update.next_sync_committee.aggregate_pubkey;
    }

    // TODO(aeryz): we should ditch this functionality as it complicates the light client and we don't use it
    // Some updates can be only for updating the sync committee, therefore the slot number can be
    // smaller. We don't want to save a new state if this is the case.
    let updated_height = core::cmp::max(
        trusted_height.height(),
        consensus_update.finalized_header.execution.block_number,
    );

    if consensus_update.finalized_header.beacon.slot > consensus_state.slot {
        consensus_state.slot = consensus_update.finalized_header.beacon.slot;

        consensus_state.state_root = consensus_update.finalized_header.execution.state_root;
        consensus_state.storage_root = header.ibc_account_proof.storage_root;

        // Normalize to nanoseconds to be ibc-go compliant
        consensus_state.timestamp = compute_timestamp_at_slot::<C>(
            client_state.genesis_time,
            consensus_update.finalized_header.beacon.slot,
        ) * 1_000_000_000;

        if client_state.latest_height < consensus_update.finalized_header.execution.block_number {
            client_state.latest_height = consensus_update.finalized_header.execution.block_number;
        }
    }

    Ok((updated_height, client_state, consensus_state))
}

pub fn verify_misbehaviour<C: ChainSpec>(
    ctx: &IbcClientCtx<EthereumLightClient>,
    client_state: &ClientState,
    consensus_state: ConsensusState,
    misbehaviour: Misbehaviour,
) -> Result<(), Error> {
    // There is no point to check for misbehaviour when the headers are not for the same height
    let (slot_1, slot_2) = (
        misbehaviour
            .update_1
            .update_data()
            .finalized_header
            .beacon
            .slot,
        misbehaviour
            .update_2
            .update_data()
            .finalized_header
            .beacon
            .slot,
    );
    ensure(
        slot_1 == slot_2,
        Error::MisbehaviourCannotExist(slot_1, slot_2),
    )?;

    let current_slot =
        compute_slot_at_timestamp::<C>(client_state.genesis_time, ctx.env.block.time.seconds())
            .ok_or(Error::IntegerOverflow)?;

    let (current_sync_committee, next_sync_committee) =
        misbehaviour.update_1.currently_trusted_sync_committee();

    // Make sure both headers would have been accepted by the light client
    validate_light_client_update::<C, VerificationContext>(
        &misbehaviour.update_1.clone().into(),
        current_sync_committee,
        next_sync_committee,
        current_slot,
        consensus_state.slot,
        client_state.genesis_validators_root,
        &client_state.fork_parameters,
        VerificationContext { deps: ctx.deps },
    )
    .map_err(Error::ValidateLightClient)?;

    let (current_sync_committee, next_sync_committee) =
        misbehaviour.update_2.currently_trusted_sync_committee();

    validate_light_client_update::<C, VerificationContext>(
        &misbehaviour.update_2.clone().into(),
        current_sync_committee,
        next_sync_committee,
        current_slot,
        consensus_state.slot,
        client_state.genesis_validators_root,
        &client_state.fork_parameters,
        VerificationContext { deps: ctx.deps },
    )
    .map_err(Error::ValidateLightClient)?;

    Ok(())
}

// #[cfg(test)]
// mod test {
//     use std::{cmp::Ordering, fs, marker::PhantomData};

//     use cosmwasm_std::{
//         testing::{mock_env, MockApi, MockQuerier, MockStorage},
//         OwnedDeps, Timestamp,
//     };
//     use ics008_wasm_client::storage_utils::{
//         consensus_db_key, read_subject_consensus_state, HOST_CLIENT_STATE_KEY,
//         SUBJECT_CLIENT_STORE_PREFIX, SUBSTITUTE_CLIENT_STORE_PREFIX,
//     };
//     use serde::Deserialize;
//     use unionlabs::{
//         encoding::{Encode, EncodeAs},
//         ethereum::config::Mainnet,
//         google::protobuf::any::Any,
//         ibc::{core::connection::connection_end::ConnectionEnd, lightclients::ethereum},
//     };

//     use super::*;
//     use crate::{client::test_utils::custom_query_handler, errors::Error};

//     #[derive(Deserialize)]
//     struct MembershipTest<T> {
//         #[serde(with = "unionlabs::uint::u256_big_endian_hex")]
//         key: U256,
//         #[serde(with = "unionlabs::uint::u256_big_endian_hex")]
//         value: U256,
//         #[serde(with = "::serde_utils::hex_string_list")]
//         proof: Vec<Vec<u8>>,
//         storage_root: H256,
//         commitment_path: String,
//         commitments_map_slot: U256,
//         expected_data: T,
//     }

//     const INITIAL_CONSENSUS_STATE_HEIGHT: Height = Height {
//         revision_number: 0,
//         revision_height: 3577152,
//     };

//     const INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT: Height = Height {
//         revision_number: 0,
//         revision_height: 3577200,
//     };

//     lazy_static::lazy_static! {
//         static ref UPDATES: Vec<ethereum::header::Header<Mainnet>> = {
//             let mut update_files = vec![];
//             for entry in fs::read_dir(UPDATES_DIR_PATH).unwrap() {
//                 let entry = entry.unwrap();
//                 let path = entry.path();
//                 if path.file_name().is_some() {
//                     update_files.push(path);
//                 }
//             }

//             update_files.sort_by(|lhs, rhs| {
//                 let lhs = lhs.file_name().unwrap().to_string_lossy().strip_suffix(".json").unwrap().to_string().parse::<u32>().unwrap();
//                 let rhs = rhs.file_name().unwrap().to_string_lossy().strip_suffix(".json").unwrap().to_string().parse().unwrap();
//                 if lhs > rhs {
//                     Ordering::Greater
//                 } else {
//                     Ordering::Less
//                 }
//             });

//             let mut updates = vec![];
//             let mut prev_height = 0;
//             for f in update_files {
//                 let mut data: ethereum::header::Header<Mainnet>= serde_json::from_str(&fs::read_to_string(f).unwrap()).unwrap();
//                 if prev_height != 0 {
//                     data.trusted_sync_committee.trusted_height.revision_height = prev_height;
//                 }
//                 prev_height = data.consensus_update.attested_header.beacon.slot;
//                 updates.push(data);
//             }

//             updates
//         };
//     }

//     const UPDATES_DIR_PATH: &str = "src/test/updates/";

//     #[test]
//     fn query_status_returns_active() {
//         let mut deps = OwnedDeps::<_, _, _, UnionCustomQuery> {
//             storage: MockStorage::default(),
//             api: MockApi::default(),
//             querier: MockQuerier::<UnionCustomQuery>::new(&[])
//                 .with_custom_handler(custom_query_handler),
//             custom_query_type: PhantomData,
//         };

//         let wasm_client_state: WasmClientState =
//             serde_json::from_str(include_str!("./test/client_state.json")).unwrap();

//         let wasm_consensus_state: WasmConsensusState =
//             serde_json::from_str(include_str!("./test/consensus_state.json")).unwrap();

//         save_client_state::<EthereumLightClient>(deps.as_mut(), wasm_client_state);

//         save_consensus_state::<EthereumLightClient>(
//             deps.as_mut(),
//             wasm_consensus_state,
//             &INITIAL_CONSENSUS_STATE_HEIGHT,
//         );

//         let mut env = mock_env();
//         env.block.time = Timestamp::from_seconds(0);

//         assert_eq!(
//             EthereumLightClient::status(deps.as_ref(), &env),
//             Ok(Status::Active)
//         );
//     }

//     #[test]
//     fn query_status_returns_frozen() {
//         let mut deps = OwnedDeps::<_, _, _, UnionCustomQuery> {
//             storage: MockStorage::default(),
//             api: MockApi::default(),
//             querier: MockQuerier::<UnionCustomQuery>::new(&[])
//                 .with_custom_handler(custom_query_handler),
//             custom_query_type: PhantomData,
//         };

//         let mut wasm_client_state: WasmClientState =
//             serde_json::from_str(include_str!("./test/client_state.json")).unwrap();

//         wasm_client_state.data.frozen_height = FROZEN_HEIGHT;

//         save_client_state::<EthereumLightClient>(deps.as_mut(), wasm_client_state);

//         assert_eq!(
//             EthereumLightClient::status(deps.as_ref(), &mock_env()),
//             Ok(Status::Frozen)
//         );
//     }

//     #[test]
//     fn verify_and_update_header_works_with_good_data() {
//         let mut deps = OwnedDeps::<_, _, _, UnionCustomQuery> {
//             storage: MockStorage::default(),
//             api: MockApi::default(),
//             querier: MockQuerier::<UnionCustomQuery>::new(&[])
//                 .with_custom_handler(custom_query_handler),
//             custom_query_type: PhantomData,
//         };

//         let wasm_client_state: WasmClientState =
//             serde_json::from_str(include_str!("./test/client_state.json")).unwrap();

//         let wasm_consensus_state: WasmConsensusState =
//             serde_json::from_str(include_str!("./test/consensus_state.json")).unwrap();

//         save_client_state::<EthereumLightClient>(deps.as_mut(), wasm_client_state);
//         save_consensus_state::<EthereumLightClient>(
//             deps.as_mut(),
//             wasm_consensus_state,
//             &INITIAL_CONSENSUS_STATE_HEIGHT,
//         );

//         for update in &*UPDATES {
//             let mut env = mock_env();
//             env.block.time = cosmwasm_std::Timestamp::from_seconds(
//                 update.consensus_update.attested_header.execution.timestamp + 60 * 5,
//             );
//             EthereumLightClient::check_for_misbehaviour_on_header(deps.as_ref(), update.clone())
//                 .unwrap();
//             EthereumLightClient::verify_header(deps.as_ref(), env.clone(), update.clone()).unwrap();
//             EthereumLightClient::update_state(deps.as_mut(), env, update.clone()).unwrap();
//             // Consensus state is saved to the updated height.
//             if update.consensus_update.attested_header.beacon.slot
//                 > update.trusted_sync_committee.trusted_height.revision_height
//             {
//                 // It's a finality update
//                 let wasm_consensus_state: WasmConsensusState =
//                     read_consensus_state::<EthereumLightClient>(
//                         deps.as_ref(),
//                         &Height {
//                             revision_number: 0,
//                             revision_height: update.consensus_update.attested_header.beacon.slot,
//                         },
//                     )
//                     .unwrap()
//                     .unwrap();
//                 // Slot is updated.
//                 assert_eq!(
//                     wasm_consensus_state.data.slot,
//                     update.consensus_update.attested_header.beacon.slot
//                 );
//                 // Storage root is updated.
//                 assert_eq!(
//                     wasm_consensus_state.data.storage_root,
//                     update.account_update.account_proof.storage_root,
//                 );
//                 // Latest slot is updated.
//                 // TODO(aeryz): Add cases for `store_period == update_period` and `update_period == store_period + 1`
//                 let wasm_client_state: WasmClientState =
//                     read_client_state::<EthereumLightClient>(deps.as_ref()).unwrap();
//                 assert_eq!(
//                     wasm_client_state.data.latest_slot,
//                     update.consensus_update.attested_header.beacon.slot
//                 );
//             } else {
//                 // It's a sync committee update
//                 let updated_height = core::cmp::max(
//                     update.trusted_sync_committee.trusted_height.revision_height,
//                     update.consensus_update.attested_header.beacon.slot,
//                 );
//                 let wasm_consensus_state: WasmConsensusState =
//                     read_consensus_state::<EthereumLightClient>(
//                         deps.as_ref(),
//                         &Height {
//                             revision_number: 0,
//                             revision_height: updated_height,
//                         },
//                     )
//                     .unwrap()
//                     .unwrap();

//                 assert_eq!(
//                     wasm_consensus_state.data.next_sync_committee.unwrap(),
//                     update
//                         .consensus_update
//                         .next_sync_committee
//                         .clone()
//                         .unwrap()
//                         .aggregate_pubkey
//                 );
//             }
//         }
//     }

//     #[allow(clippy::type_complexity)]
//     fn prepare_test_data() -> (
//         OwnedDeps<MockStorage, MockApi, MockQuerier<UnionCustomQuery>, UnionCustomQuery>,
//         ethereum::header::Header<Mainnet>,
//         Env,
//     ) {
//         let mut deps = OwnedDeps::<_, _, _, UnionCustomQuery> {
//             storage: MockStorage::default(),
//             api: MockApi::default(),
//             querier: MockQuerier::<UnionCustomQuery>::new(&[])
//                 .with_custom_handler(custom_query_handler),
//             custom_query_type: PhantomData,
//         };

//         let wasm_client_state: WasmClientState =
//             serde_json::from_str(&fs::read_to_string("src/test/client_state.json").unwrap())
//                 .unwrap();

//         let wasm_consensus_state: WasmConsensusState =
//             serde_json::from_str(&fs::read_to_string("src/test/consensus_state.json").unwrap())
//                 .unwrap();

//         save_client_state::<EthereumLightClient>(deps.as_mut(), wasm_client_state);
//         save_consensus_state::<EthereumLightClient>(
//             deps.as_mut(),
//             wasm_consensus_state.clone(),
//             &INITIAL_CONSENSUS_STATE_HEIGHT,
//         );

//         let update = UPDATES[0].clone();

//         let mut env = mock_env();
//         env.block.time =
//             cosmwasm_std::Timestamp::from_seconds(wasm_consensus_state.data.timestamp + 60 * 5);

//         (deps, update, env)
//     }

//     #[test]
//     fn verify_header_fails_when_sync_committee_aggregate_pubkey_is_incorrect() {
//         let (deps, mut update, env) = prepare_test_data();

//         let mut pubkey = update
//             .trusted_sync_committee
//             .sync_committee
//             .get()
//             .aggregate_pubkey;
//         pubkey.0[0] ^= u8::MAX;
//         update
//             .trusted_sync_committee
//             .sync_committee
//             .get_mut()
//             .aggregate_pubkey = pubkey;
//         assert!(EthereumLightClient::verify_header(deps.as_ref(), env, update).is_err());
//     }

//     #[test]
//     fn verify_header_fails_when_finalized_header_execution_branch_merkle_is_invalid() {
//         let (deps, mut update, env) = prepare_test_data();
//         update.consensus_update.finalized_header.execution_branch[0].get_mut()[0] ^= u8::MAX;
//         assert!(EthereumLightClient::verify_header(deps.as_ref(), env, update).is_err());
//     }

//     #[test]
//     fn verify_header_fails_when_finality_branch_merkle_is_invalid() {
//         let (deps, mut update, env) = prepare_test_data();
//         update.consensus_update.finality_branch[0].get_mut()[0] ^= u8::MAX;
//         assert!(EthereumLightClient::verify_header(deps.as_ref(), env, update).is_err());
//     }

//     // TODO(aeryz): These won't work now since they now eth abi encoded
//     // #[test]
//     // fn membership_verification_works_for_client_state() {
//     //     do_membership_test::<
//     //         unionlabs::google::protobuf::any::Any<
//     //             wasm::client_state::ClientState<cometbls::client_state::ClientState>,
//     //         >,
//     //     >("src/test/memberships/valid_client_state.json")
//     //     .expect("Membership verification of client state failed");
//     // }

//     // #[test]
//     // fn membership_verification_works_for_consensus_state() {
//     //     do_membership_test::<
//     //         unionlabs::google::protobuf::any::Any<
//     //             wasm::consensus_state::ConsensusState<cometbls::consensus_state::ConsensusState>,
//     //         >,
//     //     >("src/test/memberships/valid_consensus_state.json")
//     //     .expect("Membership verification of client state failed");
//     // }

//     fn membership_data<T: serde::de::DeserializeOwned>(
//         path: &str,
//     ) -> (StorageProof, String, U256, H256, T) {
//         let data: MembershipTest<T> =
//             serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();

//         let proof = StorageProof {
//             key: data.key,
//             value: data.value,
//             proof: data.proof.into_iter().map(Into::into).collect(),
//         };

//         (
//             proof,
//             data.commitment_path,
//             data.commitments_map_slot,
//             data.storage_root.as_ref().try_into().unwrap(),
//             data.expected_data,
//         )
//     }

//     fn do_membership_test<T: serde::de::DeserializeOwned + Encode<Proto>>(
//         path: &str,
//     ) -> Result<(), Error> {
//         let (proof, commitment_path, slot, storage_root, expected_data) =
//             membership_data::<T>(path);
//         do_verify_membership(
//             commitment_path,
//             storage_root.as_ref().try_into().unwrap(),
//             slot,
//             proof,
//             expected_data.encode_as::<Proto>(),
//         )
//     }

//     #[test]
//     fn membership_verification_works_for_connection_end() {
//         do_membership_test::<ConnectionEnd>("src/test/memberships/valid_connection_end.json")
//             .expect("Membership verification of client state failed");
//     }

//     #[test]
//     fn membership_verification_fails_for_incorrect_proofs() {
//         let (mut proof, commitment_path, slot, storage_root, connection_end) =
//             membership_data::<ConnectionEnd>("src/test/memberships/valid_connection_end.json");

//         let proofs = vec![
//             {
//                 let mut proof = proof.clone();
//                 proof.key.0 .0[0] ^= u64::MAX;
//                 proof
//             },
//             {
//                 proof.proof[0][10] ^= u8::MAX;
//                 proof
//             },
//         ];

//         for proof in proofs {
//             assert!(do_verify_membership(
//                 commitment_path.clone(),
//                 storage_root,
//                 slot,
//                 proof,
//                 connection_end.clone().encode_as::<Proto>(),
//             )
//             .is_err());
//         }
//     }

//     #[test]
//     fn membership_verification_fails_for_incorrect_storage_root() {
//         let (proof, commitment_path, slot, mut storage_root, connection_end) =
//             membership_data::<ConnectionEnd>("src/test/memberships/valid_connection_end.json");

//         storage_root.get_mut()[10] ^= u8::MAX;

//         assert!(do_verify_membership(
//             commitment_path,
//             storage_root,
//             slot,
//             proof,
//             connection_end.encode_as::<Proto>(),
//         )
//         .is_err());
//     }

//     #[test]
//     fn membership_verification_fails_for_incorrect_data() {
//         let (proof, commitment_path, slot, storage_root, mut connection_end) =
//             membership_data::<ConnectionEnd>("src/test/memberships/valid_connection_end.json");

//         connection_end.client_id =
//             unionlabs::validated::Validated::new("08-client-1".into()).unwrap();

//         assert!(do_verify_membership(
//             commitment_path,
//             storage_root,
//             slot,
//             proof,
//             connection_end.encode_as::<Proto>(),
//         )
//         .is_err());
//     }

//     #[test]
//     fn non_membership_verification_works() {
//         let (proof, commitment_path, slot, storage_root, _) =
//             membership_data::<()>("src/test/memberships/valid_non_membership_proof.json");

//         do_verify_non_membership(commitment_path, storage_root, slot, proof)
//             .expect("Membership verification of client state failed");
//     }

//     #[test]
//     fn non_membership_verification_fails_when_value_not_empty() {
//         let (proof, commitment_path, slot, storage_root, _) =
//             membership_data::<ConnectionEnd>("src/test/memberships/valid_connection_end.json");
//         assert_eq!(
//             do_verify_non_membership(commitment_path, storage_root, slot, proof),
//             Err(Error::CounterpartyStorageNotNil)
//         );
//     }

//     #[test]
//     fn update_state_on_misbehaviour_works() {
//         let (mut deps, _, env) = prepare_test_data();

//         EthereumLightClient::update_state_on_misbehaviour(deps.as_mut(), env.clone(), Vec::new())
//             .unwrap();

//         assert_eq!(
//             EthereumLightClient::status(deps.as_ref(), &env),
//             Ok(Status::Frozen)
//         );
//     }

//     fn save_states_to_migrate_store(
//         deps: DepsMut<UnionCustomQuery>,
//         subject_client_state: &WasmClientState,
//         substitute_client_state: &WasmClientState,
//         subject_consensus_state: &WasmConsensusState,
//         substitute_consensus_state: &WasmConsensusState,
//     ) {
//         deps.storage.set(
//             format!("{SUBJECT_CLIENT_STORE_PREFIX}{HOST_CLIENT_STATE_KEY}").as_bytes(),
//             &Any(subject_client_state.clone()).encode_as::<Proto>(),
//         );
//         deps.storage.set(
//             format!(
//                 "{SUBJECT_CLIENT_STORE_PREFIX}{}",
//                 consensus_db_key(&INITIAL_CONSENSUS_STATE_HEIGHT)
//             )
//             .as_bytes(),
//             &Any(subject_consensus_state.clone()).encode_as::<Proto>(),
//         );
//         deps.storage.set(
//             format!("{SUBSTITUTE_CLIENT_STORE_PREFIX}{HOST_CLIENT_STATE_KEY}").as_bytes(),
//             &Any(substitute_client_state.clone()).encode_as::<Proto>(),
//         );
//         deps.storage.set(
//             format!(
//                 "{SUBSTITUTE_CLIENT_STORE_PREFIX}{}",
//                 consensus_db_key(&INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT)
//             )
//             .as_bytes(),
//             &Any(substitute_consensus_state.clone()).encode_as::<Proto>(),
//         );
//     }

//     #[allow(clippy::type_complexity)]
//     fn prepare_migrate_tests() -> (
//         OwnedDeps<MockStorage, MockApi, MockQuerier<UnionCustomQuery>, UnionCustomQuery>,
//         WasmClientState,
//         WasmConsensusState,
//         WasmClientState,
//         WasmConsensusState,
//     ) {
//         (
//             OwnedDeps::<_, _, _, UnionCustomQuery> {
//                 storage: MockStorage::default(),
//                 api: MockApi::default(),
//                 querier: MockQuerier::<UnionCustomQuery>::new(&[])
//                     .with_custom_handler(custom_query_handler),
//                 custom_query_type: PhantomData,
//             },
//             serde_json::from_str(&fs::read_to_string("src/test/client_state.json").unwrap())
//                 .unwrap(),
//             serde_json::from_str(&fs::read_to_string("src/test/consensus_state.json").unwrap())
//                 .unwrap(),
//             serde_json::from_str(
//                 &fs::read_to_string("src/test/substitute_client_state.json").unwrap(),
//             )
//             .unwrap(),
//             serde_json::from_str(
//                 &fs::read_to_string("src/test/substitute_consensus_state.json").unwrap(),
//             )
//             .unwrap(),
//         )
//     }

//     #[test]
//     fn migrate_client_store_works() {
//         let (
//             mut deps,
//             mut wasm_client_state,
//             wasm_consensus_state,
//             substitute_wasm_client_state,
//             substitute_wasm_consensus_state,
//         ) = prepare_migrate_tests();

//         wasm_client_state.data.frozen_height = FROZEN_HEIGHT;

//         save_states_to_migrate_store(
//             deps.as_mut(),
//             &wasm_client_state,
//             &substitute_wasm_client_state,
//             &wasm_consensus_state,
//             &substitute_wasm_consensus_state,
//         );

//         EthereumLightClient::migrate_client_store(deps.as_mut()).unwrap();

//         let wasm_client_state: WasmClientState =
//             read_subject_client_state::<EthereumLightClient>(deps.as_ref()).unwrap();
//         // we didn't miss updating any fields
//         assert_eq!(wasm_client_state, substitute_wasm_client_state);
//         // client is unfrozen
//         assert_eq!(wasm_client_state.data.frozen_height, ZERO_HEIGHT);

//         // the new consensus state is saved under the correct height
//         assert_eq!(
//             read_subject_consensus_state::<EthereumLightClient>(
//                 deps.as_ref(),
//                 &INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT
//             )
//             .unwrap()
//             .unwrap(),
//             substitute_wasm_consensus_state
//         )
//     }

//     #[test]
//     fn migrate_client_store_fails_when_invalid_change() {
//         let (
//             mut deps,
//             wasm_client_state,
//             wasm_consensus_state,
//             substitute_wasm_client_state,
//             substitute_wasm_consensus_state,
//         ) = prepare_migrate_tests();

//         macro_rules! modify_fns {
//             ($param:ident, $($m:expr), + $(,)?) => ([$(|$param: &mut ClientState| $m),+])
//         }

//         let modifications = modify_fns! { s,
//             s.genesis_time ^= u64::MAX,
//             s.genesis_validators_root.get_mut()[0] ^= u8::MAX,
//             s.seconds_per_slot ^= u64::MAX,
//             s.slots_per_epoch ^= u64::MAX,
//             s.epochs_per_sync_committee_period ^= u64::MAX,
//         };

//         for m in modifications {
//             let mut state = substitute_wasm_client_state.clone();
//             m(&mut state.data);

//             save_states_to_migrate_store(
//                 deps.as_mut(),
//                 &wasm_client_state,
//                 &state,
//                 &wasm_consensus_state,
//                 &substitute_wasm_consensus_state,
//             );
//             assert_eq!(
//                 EthereumLightClient::migrate_client_store(deps.as_mut()),
//                 Err(Error::MigrateFieldsChanged.into())
//             );
//         }
//     }

//     #[test]
//     fn migrate_client_store_fails_when_substitute_client_frozen() {
//         let (
//             mut deps,
//             wasm_client_state,
//             wasm_consensus_state,
//             mut substitute_wasm_client_state,
//             substitute_wasm_consensus_state,
//         ) = prepare_migrate_tests();

//         substitute_wasm_client_state.data.frozen_height = FROZEN_HEIGHT;

//         save_states_to_migrate_store(
//             deps.as_mut(),
//             &wasm_client_state,
//             &substitute_wasm_client_state,
//             &wasm_consensus_state,
//             &substitute_wasm_consensus_state,
//         );

//         assert_eq!(
//             EthereumLightClient::migrate_client_store(deps.as_mut()),
//             Err(Error::SubstituteClientFrozen.into())
//         );
//     }
// }
