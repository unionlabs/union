use cosmwasm_std::{Binary, Deps, DepsMut, Env};
use ethereum_verifier::{
    compute_sync_committee_period_at_slot, compute_timestamp_at_slot, validate_light_client_update,
    verify_account_storage_root, verify_storage_absence, verify_storage_proof,
};
use ics008_wasm_client::{
    storage_utils::{
        read_client_state, read_consensus_state, save_client_state, save_consensus_state,
        update_client_state,
    },
    ContractResult, IbcClient, QueryResponse, Status, StorageState,
};
use sha3::Digest;
use unionlabs::{
    hash::H256,
    ibc::{
        core::client::height::Height,
        lightclients::ethereum::{
            client_state::ClientState, consensus_state::ConsensusState, header::Header,
            proof::Proof, storage_proof::StorageProof,
        },
    },
    uint::U256,
    TryFromProto,
};

use crate::{
    consensus_state::TrustedConsensusState,
    context::LightClientContext,
    custom_query::{CustomQuery, VerificationContext},
    errors::Error,
    eth_encoding::generate_commitment_key,
    Config,
};

type WasmClientState = unionlabs::ibc::lightclients::wasm::client_state::ClientState<ClientState>;
type WasmConsensusState =
    unionlabs::ibc::lightclients::wasm::consensus_state::ConsensusState<ConsensusState>;

pub struct EthereumLightClient;

impl IbcClient for EthereumLightClient {
    type Error = Error;

    type CustomQuery = CustomQuery;

    type Header = Header<Config>;

    // TODO(aeryz): See #588
    type Misbehaviour = ();

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    fn verify_membership(
        deps: Deps<Self::CustomQuery>,
        height: Height,
        _delay_time_period: u64,
        _delay_block_period: u64,
        proof: Binary,
        mut path: ics008_wasm_client::MerklePath,
        value: ics008_wasm_client::StorageState,
    ) -> Result<ics008_wasm_client::ContractResult, Self::Error> {
        let consensus_state: WasmConsensusState =
            read_consensus_state(deps, &height)?.ok_or(Error::ConsensusStateNotFound(height))?;
        let client_state: WasmClientState = read_client_state(deps)?;

        let path = path.key_path.pop().ok_or(Error::EmptyIbcPath)?;

        // This storage root is verified during the header update, so we don't need to verify it again.
        let storage_root = consensus_state.data.storage_root;

        let storage_proof = {
            let mut proofs = StorageProof::try_from_proto_bytes(&proof.0)
                .map_err(|e| Error::DecodeFromProto {
                    reason: format!("when decoding storage proof: {e:#?}"),
                })?
                .proofs;
            if proofs.len() > 1 {
                return Err(Error::BatchingProofsNotSupported);
            }
            proofs.pop().ok_or(Error::EmptyProof)?
        };

        match value {
            StorageState::Occupied(value) => do_verify_membership(
                path,
                storage_root,
                client_state.data.counterparty_commitment_slot,
                storage_proof,
                value,
            )?,
            StorageState::Empty => do_verify_non_membership(
                path,
                storage_root,
                client_state.data.counterparty_commitment_slot,
                storage_proof,
            )?,
        }

        Ok(ContractResult::valid(None))
    }

    fn verify_header(
        deps: Deps<Self::CustomQuery>,
        env: Env,
        header: Self::Header,
    ) -> Result<ics008_wasm_client::ContractResult, Self::Error> {
        let trusted_sync_committee = header.trusted_sync_committee;
        let wasm_consensus_state =
            read_consensus_state(deps, &trusted_sync_committee.trusted_height)?.ok_or(
                Error::ConsensusStateNotFound(trusted_sync_committee.trusted_height),
            )?;

        let trusted_consensus_state = TrustedConsensusState::new(
            deps,
            wasm_consensus_state.data,
            trusted_sync_committee.sync_committee,
        )?;

        let wasm_client_state = read_client_state(deps)?;
        let ctx = LightClientContext::new(&wasm_client_state.data, trusted_consensus_state);

        // NOTE(aeryz): Ethereum consensus-spec says that we should use the slot
        // at the current timestamp.
        let current_slot = (env.block.time.seconds() - wasm_client_state.data.genesis_time)
            / wasm_client_state.data.seconds_per_slot
            + wasm_client_state.data.fork_parameters.genesis_slot;

        validate_light_client_update::<LightClientContext<Config>, VerificationContext>(
            &ctx,
            header.consensus_update.clone(),
            current_slot,
            wasm_client_state.data.genesis_validators_root.clone(),
            VerificationContext { deps },
        )?;

        let proof_data = header.account_update.account_proof;

        verify_account_storage_root(
            header.consensus_update.attested_header.execution.state_root,
            &proof_data.contract_address,
            &proof_data.proof,
            &proof_data.storage_root,
        )?;

        Ok(ContractResult::valid(None))
    }

    fn verify_misbehaviour(
        _deps: Deps<Self::CustomQuery>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<ics008_wasm_client::ContractResult, Self::Error> {
        Ok(ContractResult::invalid("Not implemented".to_string()))
    }

    fn update_state(
        mut deps: DepsMut<Self::CustomQuery>,
        _env: Env,
        header: Self::Header,
    ) -> Result<ics008_wasm_client::ContractResult, Self::Error> {
        let trusted_sync_committee = header.trusted_sync_committee;
        let trusted_height = trusted_sync_committee.trusted_height;

        let mut consensus_state: WasmConsensusState =
            read_consensus_state(deps.as_ref(), &trusted_sync_committee.trusted_height)?.ok_or(
                Error::ConsensusStateNotFound(trusted_sync_committee.trusted_height),
            )?;

        let mut client_state: WasmClientState = read_client_state(deps.as_ref())?;
        let consensus_update = header.consensus_update;
        let account_update = header.account_update;

        let store_period =
            compute_sync_committee_period_at_slot::<Config>(consensus_state.data.slot);
        let update_finalized_period = compute_sync_committee_period_at_slot::<Config>(
            consensus_update.attested_header.beacon.slot,
        );

        match consensus_state.data.next_sync_committee {
            None if update_finalized_period != store_period => {
                return Err(Error::StorePeriodMustBeEqualToFinalizedPeriod)
            }
            None => {
                consensus_state.data.next_sync_committee = consensus_update
                    .next_sync_committee
                    .map(|c| c.aggregate_pubkey);
            }
            Some(ref next_sync_committee) if update_finalized_period == store_period + 1 => {
                consensus_state.data.current_sync_committee = next_sync_committee.clone();
                consensus_state.data.next_sync_committee = consensus_update
                    .next_sync_committee
                    .map(|c| c.aggregate_pubkey);
            }
            _ => {}
        }

        // Some updates can be only for updating the sync committee, therefore the slot number can be
        // smaller. We don't want to save a new state if this is the case.
        let updated_height = core::cmp::max(
            trusted_height.revision_height,
            consensus_update.attested_header.beacon.slot,
        );

        if consensus_update.attested_header.beacon.slot > consensus_state.data.slot {
            consensus_state.data.slot = consensus_update.attested_header.beacon.slot;

            consensus_state.data.storage_root = account_update.account_proof.storage_root;

            consensus_state.timestamp = compute_timestamp_at_slot::<Config>(
                client_state.data.genesis_time,
                consensus_update.attested_header.beacon.slot,
            );
            if client_state.data.latest_slot < consensus_update.attested_header.beacon.slot {
                client_state.data.latest_slot = consensus_update.attested_header.beacon.slot;
                update_client_state(deps.branch(), client_state, updated_height);
            }
        }

        save_consensus_state(
            deps,
            consensus_state,
            &Height {
                revision_number: trusted_height.revision_number,
                revision_height: updated_height,
            },
        );

        Ok(ContractResult::valid(None))
    }

    fn update_state_on_misbehaviour(
        deps: DepsMut<Self::CustomQuery>,
        env: Env,
        _client_message: ics008_wasm_client::ClientMessage,
    ) -> Result<ics008_wasm_client::ContractResult, Self::Error> {
        let mut client_state: WasmClientState = read_client_state(deps.as_ref())?;
        client_state.data.frozen_height = Some(Height {
            revision_number: client_state.latest_height.revision_number,
            revision_height: env.block.height,
        });
        save_client_state(deps, client_state);

        Ok(ContractResult::valid(None))
    }

    fn check_for_misbehaviour_on_header(
        deps: Deps<Self::CustomQuery>,
        header: Self::Header,
    ) -> Result<ContractResult, Self::Error> {
        let height = Height::new(0, header.consensus_update.attested_header.beacon.slot);

        if let Some(consensus_state) =
            read_consensus_state::<CustomQuery, ConsensusState>(deps, &height)?
        {
            // New header is given with the same height but the storage roots don't match.
            if consensus_state.data.storage_root != header.account_update.account_proof.storage_root
            {
                return Err(Error::StorageRootMismatch {
                    expected: consensus_state.data.storage_root,
                    found: header.account_update.account_proof.storage_root,
                });
            }

            if consensus_state.data.slot != header.consensus_update.attested_header.beacon.slot {
                return Err(Error::SlotCannotBeModified);
            }

            // Next sync committee for a consensus height can be set if it is not being set
            // previously, but it cannot be changed or unset after being set.
            if let (Some(lhs), Some(rhs)) = (
                consensus_state.data.next_sync_committee,
                header.consensus_update.next_sync_committee,
            ) {
                if lhs != rhs.aggregate_pubkey {
                    return Err(Error::NextSyncCommitteeCannotBeModified);
                }
            }

            // NOTE(aeryz): we don't check the timestamp here since it is calculated based on the
            // client state and the slot number during update.
        }

        // TODO(#605): Do we need to check whether this header's timestamp is between
        // the next and the previous consensus state in terms of height?

        Ok(ContractResult::valid(None))
    }

    fn check_for_misbehaviour_on_misbehaviour(
        _deps: Deps<Self::CustomQuery>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<ContractResult, Self::Error> {
        Ok(ContractResult::invalid("Not implemented".to_string()))
    }

    fn verify_upgrade_and_update_state(
        _deps: DepsMut<Self::CustomQuery>,
        _upgrade_client_state: WasmClientState,
        _upgrade_consensus_state: WasmConsensusState,
        _proof_upgrade_client: Binary,
        _proof_upgrade_consensus_state: Binary,
    ) -> Result<ContractResult, Self::Error> {
        Ok(ContractResult::invalid("Not implemented".to_string()))
    }

    fn check_substitute_and_update_state(
        _deps: Deps<Self::CustomQuery>,
    ) -> Result<ics008_wasm_client::ContractResult, Self::Error> {
        Ok(ContractResult::invalid("Not implemented".to_string()))
    }

    fn status(deps: Deps<Self::CustomQuery>, env: &Env) -> Result<QueryResponse, Self::Error> {
        let client_state: WasmClientState = read_client_state(deps)?;

        if client_state.data.frozen_height.is_some() {
            return Ok(Status::Frozen.into());
        }

        let Some(consensus_state) =
            read_consensus_state::<CustomQuery, ConsensusState>(deps, &client_state.latest_height)?
        else {
            return Ok(Status::Expired.into());
        };

        if is_client_expired(
            consensus_state.timestamp,
            client_state.data.trusting_period,
            env.block.time.seconds(),
        ) {
            return Ok(Status::Expired.into());
        }

        Ok(Status::Active.into())
    }

    fn export_metadata(
        _deps: Deps<Self::CustomQuery>,
        _env: &Env,
    ) -> Result<QueryResponse, Self::Error> {
        Ok(QueryResponse {
            status: String::new(),
            genesis_metadata: vec![],
        })
    }
}

fn do_verify_membership(
    path: String,
    storage_root: H256,
    counterparty_commitment_slot: U256,
    storage_proof: Proof,
    value: Vec<u8>,
) -> Result<(), Error> {
    check_commitment_key(
        path,
        counterparty_commitment_slot,
        H256(storage_proof.key.to_big_endian()),
    )?;

    // We store the hash of the data, not the data itself to the commitments map.
    let expected_value_hash = H256::from(sha3::Keccak256::new().chain_update(value).finalize());

    let proof_value = H256::from(storage_proof.value.to_big_endian());

    if expected_value_hash != proof_value {
        return Err(Error::StoredValueMismatch {
            expected: expected_value_hash,
            stored: proof_value,
        });
    }

    verify_storage_proof(
        storage_root,
        H256(storage_proof.key.to_big_endian()),
        &rlp::encode(&storage_proof.value.to_big_endian().as_ref()),
        &storage_proof.proof,
    )
    .map_err(Into::into)
}

/// Verifies that no value is committed at `path` in the counterparty light client's storage.
fn do_verify_non_membership(
    path: String,
    storage_root: H256,
    counterparty_commitment_slot: U256,
    storage_proof: Proof,
) -> Result<(), Error> {
    check_commitment_key(
        path,
        counterparty_commitment_slot,
        H256(storage_proof.key.to_big_endian()),
    )?;

    if verify_storage_absence(
        storage_root,
        H256(storage_proof.key.to_big_endian()),
        &storage_proof.proof,
    )? {
        Ok(())
    } else {
        Err(Error::CounterpartyStorageNotNil)
    }
}

fn check_commitment_key(
    path: String,
    counterparty_commitment_slot: U256,
    key: H256,
) -> Result<(), Error> {
    let expected_commitment_key = generate_commitment_key(path, counterparty_commitment_slot);

    // Data MUST be stored to the commitment path that is defined in ICS23.
    if expected_commitment_key != key {
        Err(Error::InvalidCommitmentKey {
            expected: expected_commitment_key,
            found: key,
        })
    } else {
        Ok(())
    }
}

fn is_client_expired(
    consensus_state_timestamp: u64,
    trusting_period: u64,
    current_block_time: u64,
) -> bool {
    consensus_state_timestamp + trusting_period < current_block_time
}

#[cfg(all(test, feature = "mainnet"))]
mod test {
    use std::{cmp::Ordering, fs, marker::PhantomData};

    use cosmwasm_std::{
        testing::{mock_env, MockApi, MockQuerier, MockQuerierCustomHandlerResult, MockStorage},
        HexBinary, OwnedDeps, SystemResult, Timestamp,
    };
    use ethereum_verifier::crypto::{
        eth_aggregate_public_keys_unchecked, fast_aggregate_verify_unchecked,
    };
    use ics008_wasm_client::storage_utils::save_client_state;
    use serde::Deserialize;
    use unionlabs::{
        bls::BlsPublicKey,
        ethereum::config::Mainnet,
        ibc::{
            core::connection::connection_end::ConnectionEnd,
            lightclients::{cometbls, ethereum, wasm},
        },
        id::ClientId,
        proof::ConnectionPath,
        IntoProto,
    };

    use super::*;

    #[derive(Deserialize)]
    struct MembershipTest<T> {
        #[serde(with = "unionlabs::ethereum::u256_big_endian_hex")]
        key: U256,
        #[serde(with = "unionlabs::ethereum::u256_big_endian_hex")]
        value: U256,
        #[serde(with = "::serde_utils::hex_string_list")]
        proof: Vec<Vec<u8>>,
        storage_root: H256,
        commitment_path: String,
        commitments_map_slot: U256,
        expected_data: T,
    }

    const INITIAL_CONSENSUS_STATE_HEIGHT: Height = Height {
        revision_number: 0,
        revision_height: 3577152,
    };

    lazy_static::lazy_static! {
        static ref UPDATES: Vec<ethereum::header::Header<Mainnet>> = {
            let mut update_files = vec![];
            for entry in fs::read_dir(UPDATES_DIR_PATH).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.file_name().is_some() {
                    update_files.push(path);
                }
            }

            update_files.sort_by(|lhs, rhs| {
                let lhs = lhs.file_name().unwrap().to_string_lossy().strip_suffix(".json").unwrap().to_string().parse::<u32>().unwrap();
                let rhs = rhs.file_name().unwrap().to_string_lossy().strip_suffix(".json").unwrap().to_string().parse().unwrap();
                if lhs > rhs {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            });

            let mut updates = vec![];
            let mut prev_height = 0;
            for f in update_files {
                let mut data: ethereum::header::Header<Mainnet>= serde_json::from_str(&fs::read_to_string(f).unwrap()).unwrap();
                if prev_height != 0 {
                    data.trusted_sync_committee.trusted_height.revision_height = prev_height;
                }
                prev_height = data.consensus_update.attested_header.beacon.slot;
                updates.push(data);
            }

            updates
        };
    }

    const UPDATES_DIR_PATH: &str = "src/test/updates/";

    #[test]
    fn query_status_returns_active() {
        let mut deps = OwnedDeps::<_, _, _, CustomQuery> {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier::<CustomQuery>::new(&[]).with_custom_handler(custom_query_handler),
            custom_query_type: PhantomData,
        };

        let wasm_client_state: WasmClientState =
            serde_json::from_str(include_str!("./test/client_state.json")).unwrap();

        let wasm_consensus_state: WasmConsensusState =
            serde_json::from_str(include_str!("./test/consensus_state.json")).unwrap();

        save_client_state(deps.as_mut(), wasm_client_state);

        save_consensus_state(
            deps.as_mut(),
            wasm_consensus_state,
            &INITIAL_CONSENSUS_STATE_HEIGHT,
        );

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0);

        assert_eq!(
            EthereumLightClient::status(deps.as_ref(), &env),
            Ok(Status::Active.into())
        );
    }

    #[test]
    fn query_status_returns_frozen() {
        let mut deps = OwnedDeps::<_, _, _, CustomQuery> {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier::<CustomQuery>::new(&[]).with_custom_handler(custom_query_handler),
            custom_query_type: PhantomData,
        };

        let mut wasm_client_state: WasmClientState =
            serde_json::from_str(include_str!("./test/client_state.json")).unwrap();

        wasm_client_state.data.frozen_height = Some(Height {
            revision_number: 1,
            revision_height: 1,
        });

        save_client_state(deps.as_mut(), wasm_client_state);

        assert_eq!(
            EthereumLightClient::status(deps.as_ref(), &mock_env()),
            Ok(Status::Frozen.into())
        );
    }

    #[test]
    fn query_status_returns_expired() {
        let mut deps = OwnedDeps::<_, _, _, CustomQuery> {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier::<CustomQuery>::new(&[]).with_custom_handler(custom_query_handler),
            custom_query_type: PhantomData,
        };

        let mut wasm_client_state: WasmClientState =
            serde_json::from_str(include_str!("./test/client_state.json")).unwrap();

        save_client_state(deps.as_mut(), wasm_client_state.clone());

        // Client returns expired here because it cannot find the consensus state
        assert_eq!(
            EthereumLightClient::status(deps.as_ref(), &mock_env()),
            Ok(Status::Expired.into())
        );

        let wasm_consensus_state: WasmConsensusState =
            serde_json::from_str(include_str!("./test/consensus_state.json")).unwrap();

        save_consensus_state(
            deps.as_mut(),
            wasm_consensus_state.clone(),
            &INITIAL_CONSENSUS_STATE_HEIGHT,
        );

        wasm_client_state.data.trusting_period = 10;
        save_client_state(deps.as_mut(), wasm_client_state.clone());
        let mut env = mock_env();

        env.block.time = Timestamp::from_seconds(
            wasm_client_state.data.trusting_period + wasm_consensus_state.timestamp + 1,
        );
        assert_eq!(
            EthereumLightClient::status(deps.as_ref(), &env),
            Ok(Status::Expired.into())
        );

        env.block.time = Timestamp::from_seconds(
            wasm_client_state.data.trusting_period + wasm_consensus_state.timestamp,
        );
        assert_eq!(
            EthereumLightClient::status(deps.as_ref(), &env),
            Ok(Status::Active.into())
        )
    }

    #[test]
    fn verify_and_update_header_works_with_good_data() {
        let mut deps = OwnedDeps::<_, _, _, CustomQuery> {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier::<CustomQuery>::new(&[]).with_custom_handler(custom_query_handler),
            custom_query_type: PhantomData,
        };

        let wasm_client_state: WasmClientState =
            serde_json::from_str(include_str!("./test/client_state.json")).unwrap();

        let wasm_consensus_state: WasmConsensusState =
            serde_json::from_str(include_str!("./test/consensus_state.json")).unwrap();

        save_client_state(deps.as_mut(), wasm_client_state);
        save_consensus_state(
            deps.as_mut(),
            wasm_consensus_state,
            &INITIAL_CONSENSUS_STATE_HEIGHT,
        );

        for update in &*UPDATES {
            let mut env = mock_env();
            env.block.time = cosmwasm_std::Timestamp::from_seconds(
                update.consensus_update.attested_header.execution.timestamp + 60 * 5,
            );
            EthereumLightClient::check_for_misbehaviour_on_header(deps.as_ref(), update.clone())
                .unwrap();
            EthereumLightClient::verify_header(deps.as_ref(), env.clone(), update.clone()).unwrap();
            EthereumLightClient::update_state(deps.as_mut(), env, update.clone()).unwrap();
            // Consensus state is saved to the updated height.
            if update.consensus_update.attested_header.beacon.slot
                > update.trusted_sync_committee.trusted_height.revision_height
            {
                // It's a finality update
                let wasm_consensus_state: WasmConsensusState = read_consensus_state(
                    deps.as_ref(),
                    &Height {
                        revision_number: 0,
                        revision_height: update.consensus_update.attested_header.beacon.slot,
                    },
                )
                .unwrap()
                .unwrap();
                // Slot is updated.
                assert_eq!(
                    wasm_consensus_state.data.slot,
                    update.consensus_update.attested_header.beacon.slot
                );
                // Storage root is updated.
                assert_eq!(
                    wasm_consensus_state.data.storage_root,
                    update.account_update.account_proof.storage_root,
                );
                // Latest slot is updated.
                // TODO(aeryz): Add cases for `store_period == update_period` and `update_period == store_period + 1`
                let wasm_client_state: WasmClientState = read_client_state(deps.as_ref()).unwrap();
                assert_eq!(
                    wasm_client_state.data.latest_slot,
                    update.consensus_update.attested_header.beacon.slot
                );
            } else {
                // It's a sync committee update
                let updated_height = core::cmp::max(
                    update.trusted_sync_committee.trusted_height.revision_height,
                    update.consensus_update.attested_header.beacon.slot,
                );
                let wasm_consensus_state: WasmConsensusState = read_consensus_state(
                    deps.as_ref(),
                    &Height {
                        revision_number: 0,
                        revision_height: updated_height,
                    },
                )
                .unwrap()
                .unwrap();

                assert_eq!(
                    wasm_consensus_state.data.next_sync_committee.unwrap(),
                    update
                        .consensus_update
                        .next_sync_committee
                        .clone()
                        .unwrap()
                        .aggregate_pubkey
                );
            }
        }
    }

    fn custom_query_handler(query: &CustomQuery) -> MockQuerierCustomHandlerResult {
        match query {
            CustomQuery::AggregateVerify {
                public_keys,
                message,
                signature,
            } => {
                let pubkeys: Vec<BlsPublicKey> = public_keys
                    .iter()
                    .map(|pk| pk.0.clone().try_into().unwrap())
                    .collect();

                let res = fast_aggregate_verify_unchecked(
                    pubkeys.iter().collect::<Vec<&BlsPublicKey>>().as_slice(),
                    message.as_ref(),
                    &signature.0.clone().try_into().unwrap(),
                );

                SystemResult::Ok(cosmwasm_std::ContractResult::Ok::<Binary>(
                    serde_json::to_vec(&res.is_ok()).unwrap().into(),
                ))
            }
            CustomQuery::Aggregate { public_keys } => {
                let pubkey = eth_aggregate_public_keys_unchecked(
                    public_keys
                        .iter()
                        .map(|pk| pk.as_ref().try_into().unwrap())
                        .collect::<Vec<BlsPublicKey>>()
                        .as_slice(),
                )
                .unwrap();

                SystemResult::Ok(cosmwasm_std::ContractResult::Ok::<Binary>(
                    serde_json::to_vec(&Binary(pubkey.into())).unwrap().into(),
                ))
            }
        }
    }

    #[allow(clippy::type_complexity)]
    fn prepare_test_data() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier<CustomQuery>, CustomQuery>,
        ethereum::header::Header<Mainnet>,
        Env,
    ) {
        let mut deps = OwnedDeps::<_, _, _, CustomQuery> {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier::<CustomQuery>::new(&[]).with_custom_handler(custom_query_handler),
            custom_query_type: PhantomData,
        };

        let wasm_client_state: WasmClientState =
            serde_json::from_str(include_str!("./test/client_state.json")).unwrap();

        let wasm_consensus_state: WasmConsensusState =
            serde_json::from_str(include_str!("./test/consensus_state.json")).unwrap();

        save_client_state(deps.as_mut(), wasm_client_state);
        save_consensus_state(
            deps.as_mut(),
            wasm_consensus_state.clone(),
            &INITIAL_CONSENSUS_STATE_HEIGHT,
        );

        let update = UPDATES[0].clone();

        let mut env = mock_env();
        env.block.time =
            cosmwasm_std::Timestamp::from_seconds(wasm_consensus_state.timestamp + 60 * 5);

        (deps, update, env)
    }

    #[test]
    fn verify_header_fails_when_sync_committee_aggregate_pubkey_is_incorrect() {
        let (deps, mut update, env) = prepare_test_data();

        let mut pubkey = update
            .trusted_sync_committee
            .sync_committee
            .get()
            .aggregate_pubkey
            .clone();
        pubkey.0[0] += 1;
        update
            .trusted_sync_committee
            .sync_committee
            .get_mut()
            .aggregate_pubkey = pubkey;
        assert!(EthereumLightClient::verify_header(deps.as_ref(), env, update).is_err());
    }

    #[test]
    fn verify_header_fails_when_finalized_header_execution_branch_merkle_is_invalid() {
        let (deps, mut update, env) = prepare_test_data();
        update.consensus_update.finalized_header.execution_branch[0].0[0] += 1;
        assert!(EthereumLightClient::verify_header(deps.as_ref(), env, update).is_err());
    }

    #[test]
    fn verify_header_fails_when_finality_branch_merkle_is_invalid() {
        let (deps, mut update, env) = prepare_test_data();
        update.consensus_update.finality_branch[0].0[0] += 1;
        assert!(EthereumLightClient::verify_header(deps.as_ref(), env, update).is_err());
    }

    #[test]
    fn gen_commitment_key() {
        let key = generate_commitment_key(
            ConnectionPath {
                connection_id: unionlabs::validated::Validated::new("connection-100".into())
                    .unwrap(),
            }
            .to_string(),
            U256::from(0),
        );

        println!("KEY: {}", hex::encode(key));
    }

    #[test]
    fn membership_verification_works_for_client_state() {
        do_membership_test::<
            unionlabs::google::protobuf::any::Any<
                wasm::client_state::ClientState<cometbls::client_state::ClientState>,
            >,
        >("src/test/memberships/valid_client_state.json")
        .expect("Membership verification of client state failed");
    }

    #[test]
    fn membership_verification_works_for_consensus_state() {
        do_membership_test::<
            unionlabs::google::protobuf::any::Any<
                wasm::consensus_state::ConsensusState<cometbls::consensus_state::ConsensusState>,
            >,
        >("src/test/memberships/valid_consensus_state.json")
        .expect("Membership verification of client state failed");
    }
    fn membership_data<T: serde::de::DeserializeOwned>(
        path: &str,
    ) -> (Proof, String, U256, H256, T) {
        let data: MembershipTest<T> =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();

        let proof = Proof {
            key: data.key.into(),
            value: data.value.into(),
            proof: data.proof.into_iter().map(Into::into).collect(),
        };

        (
            proof,
            data.commitment_path,
            data.commitments_map_slot,
            data.storage_root.as_ref().try_into().unwrap(),
            data.expected_data,
        )
    }

    fn do_membership_test<T: serde::de::DeserializeOwned + IntoProto>(
        path: &str,
    ) -> Result<(), Error> {
        let (proof, commitment_path, slot, storage_root, expected_data) =
            membership_data::<T>(path);
        do_verify_membership(
            commitment_path,
            storage_root.as_ref().try_into().unwrap(),
            slot,
            proof,
            expected_data.into_proto_bytes(),
        )
    }

    #[test]
    fn membership_verification_works_for_connection_end() {
        do_membership_test::<ConnectionEnd<ClientId, ClientId>>(
            "src/test/memberships/valid_connection_end.json",
        )
        .expect("Membership verification of client state failed");
    }

    #[test]
    fn membership_verification_fails_for_incorrect_proofs() {
        let (mut proof, commitment_path, slot, storage_root, connection_end) =
            membership_data::<ConnectionEnd<ClientId, ClientId>>(
                "src/test/memberships/valid_connection_end.json",
            );

        let proofs = vec![
            {
                let mut proof = proof.clone();
                proof.key = U256(proof.key.0 - U256::from(10).0); // Makes sure that produced value is always valid and different
                proof
            },
            {
                let mut proof = proof.clone();
                proof.key = U256(proof.key.0 - U256::from(1).0);
                proof
            },
            {
                proof.proof[0][10] = u8::MAX - proof.proof[0][10];
                proof
            },
        ];

        for proof in proofs {
            assert!(do_verify_membership(
                commitment_path.clone(),
                storage_root.clone(),
                slot,
                proof,
                connection_end.clone().into_proto_bytes(),
            )
            .is_err());
        }
    }

    #[test]
    fn membership_verification_fails_for_incorrect_storage_root() {
        let (proof, commitment_path, slot, mut storage_root, connection_end) =
            membership_data::<ConnectionEnd<ClientId, ClientId>>(
                "src/test/memberships/valid_connection_end.json",
            );

        storage_root.0[10] = u8::MAX - storage_root.0[10];

        assert!(do_verify_membership(
            commitment_path,
            storage_root,
            slot,
            proof,
            connection_end.into_proto_bytes(),
        )
        .is_err());
    }

    #[test]
    fn membership_verification_fails_for_incorrect_data() {
        let (proof, commitment_path, slot, storage_root, mut connection_end) =
            membership_data::<ConnectionEnd<ClientId, ClientId>>(
                "src/test/memberships/valid_connection_end.json",
            );

        connection_end.client_id =
            unionlabs::validated::Validated::new("08-client-1".into()).unwrap();

        assert!(do_verify_membership(
            commitment_path,
            storage_root,
            slot,
            proof,
            connection_end.into_proto_bytes(),
        )
        .is_err());
    }

    #[test]
    fn non_membership_verification_works() {
        let (proof, commitment_path, slot, storage_root, _) =
            membership_data::<()>("src/test/memberships/valid_non_membership_proof.json");

        do_verify_non_membership(commitment_path, storage_root, slot, proof)
            .expect("Membership verification of client state failed");
    }

    #[test]
    fn non_membership_verification_fails_when_value_not_empty() {
        let (proof, commitment_path, slot, storage_root, _) =
            membership_data::<ConnectionEnd<ClientId, ClientId>>(
                "src/test/memberships/valid_connection_end.json",
            );
        assert_eq!(
            do_verify_non_membership(commitment_path, storage_root, slot, proof),
            Err(Error::CounterpartyStorageNotNil)
        );
    }

    #[test]
    fn update_state_on_misbehaviour_works() {
        let (mut deps, header, env) = prepare_test_data();

        EthereumLightClient::update_state_on_misbehaviour(
            deps.as_mut(),
            env.clone(),
            ics008_wasm_client::ClientMessage::Header(
                protos::ibc::lightclients::wasm::v1::Header {
                    data: header.into_proto_bytes(),
                    height: None,
                },
            ),
        )
        .unwrap();

        assert_eq!(
            EthereumLightClient::status(deps.as_ref(), &env)
                .unwrap()
                .status,
            Status::Frozen.to_string()
        );
    }
}
