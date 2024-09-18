use cosmwasm_std::{Deps, DepsMut, Env};
use ethereum_light_client::client::{canonicalize_stored_value, check_commitment_key};
use ics008_wasm_client::{
    storage_utils::{
        read_client_state, read_consensus_state, save_client_state, save_consensus_state,
        update_client_state, read_subject_client_state, read_substitute_client_state, 
        read_substitute_consensus_state, save_subject_consensus_state, save_subject_client_state
    },
    IbcClient, IbcClientError, Status, StorageState,
};
use scroll_codec::batch_header::BatchHeaderV3;
use unionlabs::{
    cosmwasm::wasm::union::custom_query::{query_consensus_state, UnionCustomQuery},
    encoding::{DecodeAs, Proto},
    ensure,
    ethereum::keccak256,
    hash::H256,
    ibc::{
        core::{
            client::{genesis_metadata::GenesisMetadata, height::Height},
            commitment::merkle_path::MerklePath,
        },
        lightclients::{
            ethereum::{self, storage_proof::StorageProof},
            scroll::{client_state::ClientState, consensus_state::ConsensusState, header::Header},
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

pub struct ScrollLightClient;

impl IbcClient for ScrollLightClient {
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
        value: ics008_wasm_client::StorageState,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state: WasmConsensusState =
            read_consensus_state(deps, &height)?.ok_or(Error::ConsensusStateNotFound(height))?;
        let client_state: WasmClientState = read_client_state(deps)?;

        let path = path.key_path.pop().ok_or(Error::EmptyIbcPath)?;

        // This storage root is verified during the header update, so we don't need to verify it again.
        let storage_root = consensus_state.data.ibc_storage_root;

        let storage_proof =
            StorageProof::decode_as::<Proto>(&proof).map_err(Error::StorageProofDecode)?;

        match value {
            StorageState::Occupied(value) => do_verify_membership(
                path,
                storage_root,
                client_state.data.ibc_commitment_slot,
                storage_proof,
                value,
            )?,
            StorageState::Empty => do_verify_non_membership(
                path,
                storage_root,
                client_state.data.ibc_commitment_slot,
                storage_proof,
            )?,
        }

        Ok(())
    }

    fn verify_header(
        deps: Deps<Self::CustomQuery>,
        env: Env,
        header: Self::Header,
    ) -> Result<(), IbcClientError<Self>> {
        let client_state: WasmClientState = read_client_state(deps)?;
        let l1_consensus_state = query_consensus_state::<WasmL1ConsensusState>(
            deps,
            &env,
            client_state.data.l1_client_id.clone(),
            header.l1_height,
        )
        .map_err(Error::CustomQuery)?;
        scroll_verifier::verify_header(
            client_state.data,
            header,
            l1_consensus_state.data.state_root,
        )
        .map_err(Error::Verify)?;
        Ok(())
    }

    fn verify_misbehaviour(
        _deps: Deps<Self::CustomQuery>,
        _env: Env,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<(), IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }

    fn update_state(
        mut deps: DepsMut<Self::CustomQuery>,
        _env: Env,
        header: Self::Header,
    ) -> Result<Vec<Height>, IbcClientError<Self>> {
        let mut client_state: WasmClientState = read_client_state(deps.as_ref())?;

        let batch_header =
            BatchHeaderV3::decode(&header.batch_header).map_err(Error::BatchHeaderDecode)?;

        let timestamp = batch_header.last_block_timestamp;

        let updated_height = Height {
            revision_number: client_state.latest_height.revision_number,
            revision_height: header.l1_height.revision_height,
        };

        if client_state.latest_height < header.l1_height {
            client_state.data.latest_slot = updated_height.revision_height;
            update_client_state::<Self>(
                deps.branch(),
                client_state,
                updated_height.revision_height,
            );
        }

        let consensus_state = WasmConsensusState {
            data: ConsensusState {
                state_root: header.l2_state_root_proof.value.to_be_bytes().into(),
                // must be nanos
                timestamp: 1_000_000_000 * timestamp,
                ibc_storage_root: header.l2_ibc_account_proof.storage_root,
            },
        };
        save_consensus_state::<Self>(deps, consensus_state, &updated_height);
        Ok(vec![updated_height])
    }

    fn update_state_on_misbehaviour(
        deps: DepsMut<Self::CustomQuery>,
        env: Env,
        _client_message: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        let mut client_state: WasmClientState = read_client_state(deps.as_ref())?;
        client_state.data.frozen_height = Height {
            revision_number: client_state.latest_height.revision_number,
            revision_height: env.block.height,
        };
        save_client_state::<Self>(deps, client_state);
        Ok(())
    }

    fn check_for_misbehaviour_on_header(
        _deps: Deps<Self::CustomQuery>,
        _header: Self::Header,
    ) -> Result<bool, IbcClientError<Self>> {
        Ok(false)
    }

    fn check_for_misbehaviour_on_misbehaviour(
        _deps: Deps<Self::CustomQuery>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<bool, IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }

    fn verify_upgrade_and_update_state(
        _deps: DepsMut<Self::CustomQuery>,
        _upgrade_client_state: Self::ClientState,
        _upgrade_consensus_state: Self::ConsensusState,
        _proof_upgrade_client: Vec<u8>,
        _proof_upgrade_consensus_state: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }

    fn migrate_client_store(mut deps: DepsMut<Self::CustomQuery>) -> Result<(), IbcClientError<Self>> {
        // Read the subject and substitute client states
        let subject_client_state: WasmClientState = read_subject_client_state(deps.as_ref())?;
        let substitute_client_state: WasmClientState = read_substitute_client_state(deps.as_ref())?;

        // Ensure the substitute client is not frozen
        ensure(
            substitute_client_state.data.frozen_height == Height::default(),
            Error::SubstituteClientFrozen,
        )?;

        // Ensure the non-mutable fields match between subject and substitute clients
        ensure(
            check_allowed_fields(&subject_client_state.data, &substitute_client_state.data),
            Error::MigrateFieldsChanged,
        )?;

        // Read the consensus state for the substitute client
        let substitute_consensus_state: WasmConsensusState =
            read_substitute_consensus_state(deps.as_ref(), &substitute_client_state.latest_height)?
                .ok_or(Error::ConsensusStateNotFound(
                    substitute_client_state.latest_height,
                ))?;

        // Save the consensus state to the subject's storage
        save_subject_consensus_state::<Self>(
            deps.branch(),
            substitute_consensus_state,
            &substitute_client_state.latest_height,
        );

        // Save the updated subject client state by unfreezing it
        let mut updated_subject_client_state = subject_client_state;
        updated_subject_client_state.data.frozen_height = Height::default(); // Unfreeze
        updated_subject_client_state.latest_height = substitute_client_state.latest_height;
        
        save_subject_client_state::<Self>(deps, updated_subject_client_state);

        Ok(())
    }

    fn status(deps: Deps<Self::CustomQuery>, _env: &Env) -> Result<Status, IbcClientError<Self>> {
        let client_state: WasmClientState = read_client_state(deps)?;

        if client_state.data.frozen_height != Height::default() {
            return Ok(Status::Frozen);
        }

        let Some(_) = read_consensus_state::<Self>(deps, &client_state.latest_height)? else {
            return Ok(Status::Expired);
        };

        Ok(Status::Active)
    }

    fn export_metadata(
        _deps: Deps<Self::CustomQuery>,
        _env: &Env,
    ) -> Result<Vec<GenesisMetadata>, IbcClientError<Self>> {
        Ok(Vec::new())
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
    storage_root: H256,
    ibc_commitment_slot: U256,
    storage_proof: StorageProof,
    raw_value: Vec<u8>,
) -> Result<(), Error> {
    check_commitment_key(&path, ibc_commitment_slot, storage_proof.key)?;

    // we store the hash of the data, not the data itself to the commitments map
    let expected_value_hash = keccak256(canonicalize_stored_value(path, raw_value)?);

    let proof_value = H256::from(storage_proof.value.to_be_bytes());

    if expected_value_hash != proof_value {
        return Err(Error::StoredValueMismatch {
            expected: expected_value_hash,
            stored: proof_value,
        });
    }

    scroll_verifier::verify_zktrie_storage_proof(
        storage_root,
        storage_proof.key.to_be_bytes().into(),
        storage_proof.value.to_be_bytes().as_ref(),
        &storage_proof.proof,
    )?;

    Ok(())
}

/// Verifies that no value is committed at `path` in the counterparty light client's storage.
fn do_verify_non_membership(
    path: String,
    storage_root: H256,
    ibc_commitment_slot: U256,
    storage_proof: StorageProof,
) -> Result<(), Error> {
    check_commitment_key(&path, ibc_commitment_slot, storage_proof.key)?;

    scroll_verifier::verify_zktrie_storage_absence(
        storage_root,
        H256(storage_proof.key.to_be_bytes()),
        &storage_proof.proof,
    )?;

    Ok(())
}

fn check_allowed_fields(
    subject_client_state: &ClientState,
    substitute_client_state: &ClientState,
) -> bool {
    subject_client_state.l1_client_id == substitute_client_state.l1_client_id
        && subject_client_state.chain_id == substitute_client_state.chain_id
        && subject_client_state.l2_contract_address == substitute_client_state.l2_contract_address
        && subject_client_state.l2_finalized_state_roots_slot == substitute_client_state.l2_finalized_state_roots_slot
        && subject_client_state.l2_committed_batches_slot == substitute_client_state.l2_committed_batches_slot
        && subject_client_state.ibc_contract_address == substitute_client_state.ibc_contract_address
        && subject_client_state.ibc_commitment_slot == substitute_client_state.ibc_commitment_slot
}

#[cfg(all(test))]
mod test {
    use cosmwasm_std::{
        testing::{mock_env, MockApi, MockQuerier, MockStorage},
        OwnedDeps, DepsMut, Storage,
    };
    use ics008_wasm_client::storage_utils::{
        SUBJECT_CLIENT_STORE_PREFIX,  SUBSTITUTE_CLIENT_STORE_PREFIX, HOST_CLIENT_STATE_KEY,
        consensus_db_key, 
        read_subject_consensus_state, save_subject_consensus_state, save_client_state,
        save_consensus_state, read_subject_client_state, read_substitute_client_state,
    };
    use unionlabs::{
        cosmwasm::wasm::union::custom_query::UnionCustomQuery,
        hash::H160,
        ibc::core::client::height::Height,
        uint::U256,
        google::protobuf::any::Any,
        encoding::{DecodeAs, EncodeAs, EthAbi, Proto},
        hash::H256,
    };
    use bincode;
    use std::fs;

    use cosmwasm_std::StdError;
    use ics008_wasm_client::IbcClient;
    use crate::errors::Error;

    use super::{ScrollLightClient, WasmClientState, WasmConsensusState, ClientState, ConsensusState};


    const INITIAL_CONSENSUS_STATE_HEIGHT: Height = Height {
        revision_number: 0,
        revision_height: 950,
    };

    const INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT: Height = Height {
        revision_number: 0,
        revision_height: 970,
    };


    #[allow(clippy::type_complexity)]
    fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, MockQuerier<UnionCustomQuery>, UnionCustomQuery> {
        OwnedDeps::<_, _, _, UnionCustomQuery> {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier::<UnionCustomQuery>::new(&[]),
            custom_query_type: std::marker::PhantomData,
        }
    }

    fn create_client_state(
        l1_client_id: String, 
        chain_id: U256, 
        latest_slot: u64, 
        height: Height,
        frozen_height: Height
    ) -> WasmClientState {
        WasmClientState {
            data: ClientState {
                l1_client_id,
                chain_id,
                latest_slot,
                frozen_height,
                latest_batch_index_slot: U256::from(10),
                l2_contract_address: H160::default(),
                l2_finalized_state_roots_slot: U256::from(10),
                l2_committed_batches_slot: U256::from(10),
                ibc_contract_address: H160::default(),
                ibc_commitment_slot: U256::from(10),
            },
            latest_height: height,
            checksum: H256::default(),
        }
    }
    

    // fn create_client_state() -> WasmClientState {
    //     WasmClientState {
    //         data: ClientState {
    //             l1_client_id: "client_1".to_string(),
    //             chain_id: U256::from(1),
    //             latest_slot: 1000,
    //             frozen_height: Height::default(),
    //             latest_batch_index_slot: U256::from(10),
    //             l2_contract_address: H160::default(),
    //             l2_finalized_state_roots_slot: U256::from(10),
    //             l2_committed_batches_slot: U256::from(10),
    //             ibc_contract_address: H160::default(),
    //             ibc_commitment_slot: U256::from(10),
    //         },
    //         latest_height: Height {
    //             revision_number: 0,
    //             revision_height: 1000,
    //         },
    //         checksum: H256::default(),
    //     }
    // }

    fn save_states_to_migrate_store(
        deps: DepsMut<UnionCustomQuery>,
        subject_client_state: &WasmClientState,
        substitute_client_state: &WasmClientState,
        subject_consensus_state: &WasmConsensusState,
        substitute_consensus_state: &WasmConsensusState,
    ) {
        deps.storage.set(
            format!("{SUBJECT_CLIENT_STORE_PREFIX}{HOST_CLIENT_STATE_KEY}").as_bytes(),
            &Any(subject_client_state.clone()).encode_as::<Proto>(),
        );
        deps.storage.set(
            format!(
                "{SUBJECT_CLIENT_STORE_PREFIX}{}",
                consensus_db_key(&INITIAL_CONSENSUS_STATE_HEIGHT)
            )
            .as_bytes(),
            &Any(subject_consensus_state.clone()).encode_as::<Proto>(),
        );
        deps.storage.set(
            format!("{SUBSTITUTE_CLIENT_STORE_PREFIX}{HOST_CLIENT_STATE_KEY}").as_bytes(),
            &Any(substitute_client_state.clone()).encode_as::<Proto>(),
        );
        deps.storage.set(
            format!(
                "{SUBSTITUTE_CLIENT_STORE_PREFIX}{}",
                consensus_db_key(&INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT)
            )
            .as_bytes(),
            &Any(substitute_consensus_state.clone()).encode_as::<Proto>(),
        );
    }

    fn prepare_migrate_tests() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier<UnionCustomQuery>, UnionCustomQuery>,
        WasmClientState,
        WasmConsensusState,
        WasmClientState,
        WasmConsensusState,
    ) {
        let deps = mock_dependencies();

        let subject_client_state = create_client_state(
            "client_1".to_string(), 
            U256::from(1), 
            INITIAL_CONSENSUS_STATE_HEIGHT.revision_height,
            INITIAL_CONSENSUS_STATE_HEIGHT, 
            Height::default(),
        );
        let substitute_client_state = create_client_state(
            "client_1".to_string(),
            U256::from(1),          
            INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT.revision_height,
            INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT,                  
            Height::default(),
        );
        
        let subject_consensus_state = WasmConsensusState {
            data: ConsensusState {
                state_root: H256::default(),
                timestamp: 1000,
                ibc_storage_root: H256::default(),
            },
        };
        let substitute_consensus_state = WasmConsensusState {
            data: ConsensusState {
                state_root: H256::default(),
                timestamp: 2000,
                ibc_storage_root: H256::default(),
            },
        };

        (
            deps,
            subject_client_state,
            subject_consensus_state,
            substitute_client_state,
            substitute_consensus_state,
        )
    }

    #[test]
    fn migrate_client_store_succeeds_with_valid_data() {
        let (
            mut deps,
            mut subject_client_state,
            subject_consensus_state,
            mut substitute_client_state,
            substitute_consensus_state,
        ) = prepare_migrate_tests();


        subject_client_state.data.frozen_height = Height {
            revision_number: 0,
            revision_height: 1000,
        };

        substitute_client_state.data.frozen_height = Height::default();
        
        save_states_to_migrate_store(
            deps.as_mut(),
            &subject_client_state,
            &substitute_client_state,
            &subject_consensus_state,
            &substitute_consensus_state,
        );
        
        let original_subject_client_state: WasmClientState =
            read_subject_client_state::<ScrollLightClient>(deps.as_ref()).unwrap();
            
        assert_eq!(original_subject_client_state.data.frozen_height, Height {
            revision_number: 0,
            revision_height: 1000,
        });

        // Perform migration
        let result = ScrollLightClient::migrate_client_store(deps.as_mut());

            
        // Assert success, print error if any
        if let Err(ref e) = result {
            println!("Migration failed with error: {:?}", e);
        }
        // Assert success
        assert!(result.is_ok());
        let updated_subject_client_state: WasmClientState =
            read_subject_client_state::<ScrollLightClient>(deps.as_ref()).unwrap();
        assert_eq!(updated_subject_client_state.data.frozen_height, Height::default());

        assert_eq!(
            updated_subject_client_state.latest_height,
            substitute_client_state.latest_height
        );
    }
    #[test]
    fn migrate_client_store_fails_when_substitute_client_frozen() {
        let (
            mut deps,
            subject_client_state,
            subject_consensus_state,
            mut substitute_client_state,
            substitute_consensus_state,
        ) = prepare_migrate_tests();

        // Make the substitute client frozen
        substitute_client_state.data.frozen_height = Height {
            revision_number: 0,
            revision_height: 100,
        };

        // Save the states into the mock storage
        save_states_to_migrate_store(
            deps.as_mut(),
            &subject_client_state,
            &substitute_client_state,
            &subject_consensus_state,
            &substitute_consensus_state,
        );

        // Perform migration
        let result = ScrollLightClient::migrate_client_store(deps.as_mut());

        // Assert failure
        assert_eq!(result, Err(Error::SubstituteClientFrozen.into()));
    }

    #[test]
    fn migrate_client_store_fails_when_fields_differ() {
        let (
            mut deps,
            mut subject_client_state,
            subject_consensus_state,
            mut substitute_client_state,
            substitute_consensus_state,
        ) = prepare_migrate_tests();

        // Alter the chain_id in the substitute client state
        substitute_client_state.data.chain_id = U256::from(999);

        // Save the states into the mock storage
        save_states_to_migrate_store(
            deps.as_mut(),
            &subject_client_state,
            &substitute_client_state,
            &subject_consensus_state,
            &substitute_consensus_state,
        );

        // Perform migration
        let result = ScrollLightClient::migrate_client_store(deps.as_mut());

        // Assert failure
        assert_eq!(result, Err(Error::MigrateFieldsChanged.into()));
    }
    #[test]
    fn migrate_client_store_fails_when_substitute_consensus_not_found() {
        let (
            mut deps,
            subject_client_state,
            subject_consensus_state,
            mut substitute_client_state,
            _substitute_consensus_state,  // we won't save this to storage
        ) = prepare_migrate_tests();

        // modify latest height to a height where the consensus state is not found
        substitute_client_state.latest_height = Height {
            revision_number: 0,
            revision_height: 15,
        };

        // Save only the client states and subject consensus state (skip substitute consensus state)
        save_states_to_migrate_store(
            deps.as_mut(),
            &subject_client_state,
            &substitute_client_state,
            &subject_consensus_state,
            &subject_consensus_state,  // Reusing subject consensus intentionally
        );

        // Perform migration
        let result = ScrollLightClient::migrate_client_store(deps.as_mut());

        // Assert failure
        assert_eq!(result, Err(Error::ConsensusStateNotFound(substitute_client_state.latest_height).into()));
    }


    
}

