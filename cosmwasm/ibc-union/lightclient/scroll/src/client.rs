use cosmwasm_std::Empty;
use ethereum_light_client::client::{check_commitment_key, EthereumLightClient};
use ethereum_light_client_types::StorageProof;
use ibc_union_light_client::{IbcClient, IbcClientCtx, IbcClientError};
use ibc_union_msg::lightclient::Status;
use scroll_codec::batch_header::BatchHeaderV3;
use scroll_light_client_types::{ClientState, ConsensusState, Header};
use unionlabs::{encoding::Proto, primitives::H256};

use crate::errors::Error;

pub enum ScrollLightClient {}

impl IbcClient for ScrollLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type StorageProof = StorageProof;

    type Encoding = Proto;

    fn verify_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;

        check_commitment_key(
            H256::try_from(&key).map_err(|_| Error::InvalidCommitmentKeyLength(key))?,
            storage_proof.key,
        )
        .map_err(Into::<Error>::into)?;

        let value =
            H256::try_from(&value).map_err(|_| Error::InvalidCommitmentValueLength(value))?;

        let proof_value = H256::from(storage_proof.value.to_be_bytes());

        if value != proof_value {
            return Err(Error::StoredValueMismatch {
                expected: value,
                stored: proof_value,
            }
            .into());
        }

        scroll_verifier::verify_zktrie_storage_proof(
            consensus_state.ibc_storage_root,
            storage_proof.key.to_be_bytes().into(),
            storage_proof.value.to_be_bytes().as_ref(),
            &storage_proof.proof,
        )
        .map_err(Into::<Error>::into)?;

        Ok(())
    }

    fn verify_non_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;
        check_commitment_key(
            H256::try_from(&key).map_err(|_| Error::InvalidCommitmentKeyLength(key))?,
            storage_proof.key,
        )
        .map_err(Into::<Error>::into)?;

        scroll_verifier::verify_zktrie_storage_absence(
            consensus_state.ibc_storage_root,
            H256::new(storage_proof.key.to_be_bytes()),
            &storage_proof.proof,
        )
        .map_err(Into::<Error>::into)?;

        todo!()
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64 {
        consensus_state.timestamp
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        client_state.latest_slot
    }

    fn status(client_state: &Self::ClientState) -> Status {
        if client_state.frozen_height.height() == 0 {
            Status::Active
        } else {
            Status::Frozen
        }
    }

    fn verify_creation(
        _client_state: &ClientState,
        _consensus_state: &ConsensusState,
    ) -> Result<(), IbcClientError<Self>> {
        Ok(())
    }

    fn verify_header(
        ctx: IbcClientCtx<Self>,
        header: Header,
    ) -> Result<(u64, Self::ClientState, Self::ConsensusState), IbcClientError<Self>> {
        let mut client_state = ctx.read_self_client_state()?;
        verify_header(&ctx, &client_state, &header)?;

        let batch_header =
            BatchHeaderV3::decode(&header.batch_header).map_err(Error::BatchHeaderDecode)?;

        let timestamp = batch_header.last_block_timestamp;

        let updated_height = header.l1_height.height();

        if client_state.latest_slot < updated_height {
            client_state.latest_slot = updated_height;
        }

        let consensus_state = ConsensusState {
            state_root: header.l2_state_root_proof.value.to_be_bytes().into(),
            // must be nanos
            timestamp: 1_000_000_000 * timestamp,
            ibc_storage_root: header.l2_ibc_account_proof.storage_root,
        };

        Ok((updated_height, client_state, consensus_state))
    }

    fn misbehaviour(
        _ctx: IbcClientCtx<Self>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<Self::ClientState, IbcClientError<Self>> {
        todo!()
    }
}

pub fn verify_header(
    ctx: &IbcClientCtx<ScrollLightClient>,
    client_state: &ClientState,
    header: &Header,
) -> Result<(), Error> {
    let l1_consensus_state = ctx.read_consensus_state::<EthereumLightClient>(
        client_state.l1_client_id,
        header.l1_height.height(),
    )?;
    scroll_verifier::verify_header(client_state, header, l1_consensus_state.state_root)?;
    Ok(())
}

// #[cfg(test)]
// mod test {
//     use cosmwasm_std::{
//         testing::{MockApi, MockQuerier, MockStorage},
//         DepsMut, OwnedDeps,
//     };
//     use ics008_wasm_client::{
//         storage_utils::{
//             consensus_db_key, read_subject_client_state, HOST_CLIENT_STATE_KEY,
//             SUBJECT_CLIENT_STORE_PREFIX, SUBSTITUTE_CLIENT_STORE_PREFIX,
//         },
//         IbcClient,
//     };
//     use unionlabs::{
//         cosmwasm::wasm::union::custom_query::UnionCustomQuery,
//         encoding::{EncodeAs, Proto},
//         google::protobuf::any::Any,
//         primitives::{H160, H256},
//         ibc::core::client::height::Height,
//         primitives::U256,
//     };

//     use super::{
//         ClientState, ConsensusState, ScrollLightClient, WasmClientState, WasmConsensusState,
//     };
//     use crate::errors::Error;

//     const INITIAL_CONSENSUS_STATE_HEIGHT: Height = Height {
//         revision_number: 0,
//         revision_height: 950,
//     };

//     const INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT: Height = Height {
//         revision_number: 0,
//         revision_height: 970,
//     };

//     #[allow(clippy::type_complexity)]
//     fn mock_dependencies(
//     ) -> OwnedDeps<MockStorage, MockApi, MockQuerier<UnionCustomQuery>, UnionCustomQuery> {
//         OwnedDeps::<_, _, _, UnionCustomQuery> {
//             storage: MockStorage::default(),
//             api: MockApi::default(),
//             querier: MockQuerier::<UnionCustomQuery>::new(&[]),
//             custom_query_type: std::marker::PhantomData,
//         }
//     }

//     fn create_client_state(
//         l1_client_id: String,
//         chain_id: U256,
//         latest_slot: u64,
//         height: Height,
//         frozen_height: Height,
//     ) -> WasmClientState {
//         WasmClientState {
//             data: ClientState {
//                 l1_client_id,
//                 chain_id,
//                 latest_slot,
//                 frozen_height,
//                 latest_batch_index_slot: U256::from(10),
//                 l2_contract_address: H160::default(),
//                 l2_finalized_state_roots_slot: U256::from(10),
//                 l2_committed_batches_slot: U256::from(10),
//                 ibc_contract_address: H160::default(),
//             },
//             latest_height: height,
//             checksum: H256::default(),
//         }
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
//         let deps = mock_dependencies();

//         let subject_client_state = create_client_state(
//             "client_1".to_string(),
//             U256::from(1),
//             INITIAL_CONSENSUS_STATE_HEIGHT.revision_height,
//             INITIAL_CONSENSUS_STATE_HEIGHT,
//             Height::default(),
//         );
//         let substitute_client_state = create_client_state(
//             "client_1".to_string(),
//             U256::from(1),
//             INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT.revision_height,
//             INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT,
//             Height::default(),
//         );

//         let subject_consensus_state = WasmConsensusState {
//             data: ConsensusState {
//                 state_root: H256::default(),
//                 timestamp: 1000,
//                 ibc_storage_root: H256::default(),
//             },
//         };
//         let substitute_consensus_state = WasmConsensusState {
//             data: ConsensusState {
//                 state_root: H256::default(),
//                 timestamp: 2000,
//                 ibc_storage_root: H256::default(),
//             },
//         };

//         (
//             deps,
//             subject_client_state,
//             subject_consensus_state,
//             substitute_client_state,
//             substitute_consensus_state,
//         )
//     }

//     #[test]
//     fn migrate_client_store_succeeds_with_valid_data() {
//         let (
//             mut deps,
//             mut subject_client_state,
//             subject_consensus_state,
//             mut substitute_client_state,
//             substitute_consensus_state,
//         ) = prepare_migrate_tests();

//         subject_client_state.data.frozen_height = Height {
//             revision_number: 0,
//             revision_height: 1000,
//         };

//         substitute_client_state.data.frozen_height = Height::default();

//         save_states_to_migrate_store(
//             deps.as_mut(),
//             &subject_client_state,
//             &substitute_client_state,
//             &subject_consensus_state,
//             &substitute_consensus_state,
//         );

//         let original_subject_client_state: WasmClientState =
//             read_subject_client_state::<ScrollLightClient>(deps.as_ref()).unwrap();

//         assert_eq!(
//             original_subject_client_state.data.frozen_height,
//             Height {
//                 revision_number: 0,
//                 revision_height: 1000,
//             }
//         );

//         // Perform migration
//         let result = ScrollLightClient::migrate_client_store(deps.as_mut());

//         // Assert success, print error if any
//         if let Err(ref e) = result {
//             println!("Migration failed with error: {:?}", e);
//         }
//         // Assert success
//         assert!(result.is_ok());
//         let updated_subject_client_state: WasmClientState =
//             read_subject_client_state::<ScrollLightClient>(deps.as_ref()).unwrap();
//         assert_eq!(
//             updated_subject_client_state.data.frozen_height,
//             Height::default()
//         );

//         assert_eq!(
//             updated_subject_client_state.latest_height,
//             substitute_client_state.latest_height
//         );
//     }
//     #[test]
//     fn migrate_client_store_fails_when_substitute_client_frozen() {
//         let (
//             mut deps,
//             subject_client_state,
//             subject_consensus_state,
//             mut substitute_client_state,
//             substitute_consensus_state,
//         ) = prepare_migrate_tests();

//         // Make the substitute client frozen
//         substitute_client_state.data.frozen_height = Height {
//             revision_number: 0,
//             revision_height: 100,
//         };

//         // Save the states into the mock storage
//         save_states_to_migrate_store(
//             deps.as_mut(),
//             &subject_client_state,
//             &substitute_client_state,
//             &subject_consensus_state,
//             &substitute_consensus_state,
//         );

//         // Perform migration
//         let result = ScrollLightClient::migrate_client_store(deps.as_mut());

//         // Assert failure
//         assert_eq!(result, Err(Error::SubstituteClientFrozen.into()));
//     }

//     #[test]
//     fn migrate_client_store_fails_when_fields_differ() {
//         let (
//             mut deps,
//             subject_client_state,
//             subject_consensus_state,
//             mut substitute_client_state,
//             substitute_consensus_state,
//         ) = prepare_migrate_tests();

//         // Alter the chain_id in the substitute client state
//         substitute_client_state.data.chain_id = U256::from(999);

//         // Save the states into the mock storage
//         save_states_to_migrate_store(
//             deps.as_mut(),
//             &subject_client_state,
//             &substitute_client_state,
//             &subject_consensus_state,
//             &substitute_consensus_state,
//         );

//         // Perform migration
//         let result = ScrollLightClient::migrate_client_store(deps.as_mut());

//         // Assert failure
//         assert_eq!(result, Err(Error::MigrateFieldsChanged.into()));
//     }
//     #[test]
//     fn migrate_client_store_fails_when_substitute_consensus_not_found() {
//         let (
//             mut deps,
//             subject_client_state,
//             subject_consensus_state,
//             mut substitute_client_state,
//             _substitute_consensus_state, // we won't save this to storage
//         ) = prepare_migrate_tests();

//         // modify latest height to a height where the consensus state is not found
//         substitute_client_state.latest_height = Height {
//             revision_number: 0,
//             revision_height: 15,
//         };

//         // Save only the client states and subject consensus state (skip substitute consensus state)
//         save_states_to_migrate_store(
//             deps.as_mut(),
//             &subject_client_state,
//             &substitute_client_state,
//             &subject_consensus_state,
//             &subject_consensus_state, // Reusing subject consensus intentionally
//         );

//         // Perform migration
//         let result = ScrollLightClient::migrate_client_store(deps.as_mut());

//         // Assert failure
//         assert_eq!(
//             result,
//             Err(Error::ConsensusStateNotFound(substitute_client_state.latest_height).into())
//         );
//     }
// }
