use cosmwasm_std::{Binary, Deps, DepsMut, Env};
use ethabi::ethereum_types::U256 as ethabi_U256;
use ethereum_verifier::{
    compute_sync_committee_period_at_slot, compute_timestamp_at_slot, primitives::Slot,
    validate_light_client_update, verify_account_storage_root, verify_storage_absence,
    verify_storage_proof,
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
    ethereum::H256,
    ibc::{
        core::client::height::Height,
        lightclients::ethereum::{
            client_state::ClientState, consensus_state::ConsensusState, header::Header,
            proof::Proof, storage_proof::StorageProof,
        },
    },
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
        let consensus_state: WasmConsensusState = read_consensus_state(deps, &height)?.ok_or(
            Error::ConsensusStateNotFound(height.revision_number, height.revision_height),
        )?;
        let client_state: WasmClientState = read_client_state(deps)?;

        let path = path
            .key_path
            .pop()
            .ok_or(Error::InvalidPath("path is empty".into()))?;

        // This storage root is verified during the header update, so we don't need to verify it again.
        let storage_root = consensus_state.data.storage_root;

        let storage_proof = {
            let mut proofs = StorageProof::try_from_proto_bytes(&proof.0)
                .map_err(|e| Error::decode(format!("when decoding storage proof: {e:#?}")))?
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
                Error::ConsensusStateNotFound(
                    trusted_sync_committee.trusted_height.revision_number,
                    trusted_sync_committee.trusted_height.revision_height,
                ),
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
        )
        .map_err(|e| Error::Verification(e.to_string()))?;

        let proof_data = header
            .account_update
            .proofs
            .get(0)
            .ok_or(Error::EmptyProof)?;

        verify_account_storage_root(
            header.consensus_update.attested_header.execution.state_root,
            &proof_data
                .key
                .as_slice()
                .try_into()
                .map_err(|_| Error::InvalidProofFormat)?,
            &proof_data.proof,
            proof_data
                .value
                .as_slice()
                .try_into()
                .map_err(|_| Error::InvalidProofFormat)?,
        )
        .map_err(|e| Error::Verification(e.to_string()))?;

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
                Error::ConsensusStateNotFound(
                    trusted_sync_committee.trusted_height.revision_number,
                    trusted_sync_committee.trusted_height.revision_height,
                ),
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

            let storage_root = account_update
                .proofs
                .get(0)
                .ok_or(Error::EmptyProof)?
                .value
                .as_slice()
                .try_into()
                .map_err(|_| Error::InvalidProofFormat)?;
            consensus_state.data.storage_root = storage_root;

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
            let storage_root = header
                .account_update
                .proofs
                .get(0)
                .ok_or(Error::EmptyProof)?
                .value
                .as_slice()
                .try_into()
                .map_err(|_| Error::InvalidProofFormat)?;
            if consensus_state.data.storage_root != storage_root {
                return Err(Error::StorageRootMismatch);
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
    counterparty_commitment_slot: Slot,
    storage_proof: Proof,
    value: Vec<u8>,
) -> Result<(), Error> {
    check_commitment_key(path, counterparty_commitment_slot, &storage_proof.key)?;

    // We store the hash of the data, not the data itself to the commitments map.
    let expected_value_hash = sha3::Keccak256::new().chain_update(value).finalize();

    let expected_value = ethabi_U256::from_big_endian(&expected_value_hash);

    let proof_value = ethabi_U256::from_big_endian(storage_proof.value.as_slice());

    if expected_value != proof_value {
        return Err(Error::stored_value_mismatch(
            expected_value_hash,
            storage_proof.value.as_slice(),
        ));
    }

    verify_storage_proof(
        storage_root,
        &storage_proof.key,
        &rlp::encode(&storage_proof.value.as_slice()),
        &storage_proof.proof,
    )
    .map_err(|e| Error::Verification(e.to_string()))
}

/// Verifies that no value is committed at `path` in the counterparty light client's storage.
fn do_verify_non_membership(
    path: String,
    storage_root: H256,
    counterparty_commitment_slot: Slot,
    storage_proof: Proof,
) -> Result<(), Error> {
    check_commitment_key(path, counterparty_commitment_slot, &storage_proof.key)?;

    if verify_storage_absence(storage_root, &storage_proof.key, &storage_proof.proof)
        .map_err(|e| Error::Verification(e.to_string()))?
    {
        Ok(())
    } else {
        Err(Error::CounterpartyStorageNotNil)
    }
}

fn check_commitment_key(
    path: String,
    counterparty_commitment_slot: Slot,
    key: &[u8],
) -> Result<(), Error> {
    let expected_commitment_key = generate_commitment_key(path, counterparty_commitment_slot);

    // Data MUST be stored to the commitment path that is defined in ICS23.
    if expected_commitment_key != key {
        Err(Error::invalid_commitment_key(expected_commitment_key, key))
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

#[cfg(test)]
mod test {
    use std::{fs, marker::PhantomData, path};

    use cosmwasm_std::{
        testing::{mock_env, MockApi, MockQuerier, MockQuerierCustomHandlerResult, MockStorage},
        OwnedDeps, SystemResult, Timestamp,
    };
    use ethereum_verifier::crypto::{eth_aggregate_public_keys, fast_aggregate_verify};
    use hex_literal::hex;
    use ics008_wasm_client::storage_utils::save_client_state;
    use prost::Message;
    use unionlabs::{
        bls::BlsPublicKey,
        ethereum_consts_traits::Mainnet,
        ibc::{
            core::commitment::merkle_root::MerkleRoot,
            lightclients::{cometbls, ethereum},
        },
        proof::{ClientConsensusStatePath, ClientStatePath, ConnectionPath},
        IntoProto,
    };

    use super::*;

    /// These values are obtained by uploading a dummy contract with the necessary types to the devnet and
    /// reading the values by `eth_getProof` RPC call.
    // const CLIENT_STATE_PROOF_KEY: &[u8] =
    //     &hex!("b35cad2b263a62faaae30d8b3f51201fea5501d2df17d59a3eef2751403e684f");
    // const CLIENT_STATE_PROOF_VALUE: &[u8] =
    //     &hex!("272c7c82ac0f0adbfe4ae30614165bf3b94d49754ce8c1955cc255dcc829b5");
    // const CLIENT_STATE_PROOF: [&[u8]; 2] = [
    //     &hex!("f871808080a0b9f6e8d11cf768b8034f04b8b2ab45bb5ca792e1c6e3929cf8222a885631ffac808080808080808080a0f7202a06e8dc011d3123f907597f51546fe03542551af2c9c54d21ba0fbafc7280a0d1797d071b81705da736e39e75f1186c8e529ba339f7a7d12a9b4fafe33e43cc80"),
    //     &hex!("f842a03a8c7f353aebdcd6b56a67cd1b5829681a3c6e1695282161ab3faa6c3666d4c3a09f272c7c82ac0f0adbfe4ae30614165bf3b94d49754ce8c1955cc255dcc829b5")
    // ];
    // /// Storage root of the contract at the time that this proof is obtained.
    // const CLIENT_STATE_STORAGE_ROOT: H256 = H256(hex!(
    //     "5634f342b966b609cdd8d2f7ed43bb94702c9e83d4e974b08a3c2b8205fd85e3"
    // ));
    // const CLIENT_STATE_WASM_CODE_ID: &[u8] =
    //     &hex!("B41F9EE164A6520C269F8928A1F3264A6F983F27478CB3A2251B77A65E0CEFBF");

    const CONSENSUS_STATE_PROOF_KEY: &[u8] =
        &hex!("9f22934f38bf5512b9c33ed55f71525c5d129895aad5585a2624f6c756c1c101");
    const CONSENSUS_STATE_PROOF_VALUE: &[u8] =
        &hex!("504adb89d4e609110eebf79183a10b9a4788a797d973c0ba0504e7a97fc1daa6");
    const CONSENSUS_STATE_PROOF: [&[u8]; 2] = [
        &hex!("f871808080a0b9f6e8d11cf768b8034f04b8b2ab45bb5ca792e1c6e3929cf8222a885631ffac808080808080808080a0f7202a06e8dc011d3123f907597f51546fe03542551af2c9c54d21ba0fbafc7280a0d1797d071b81705da736e39e75f1186c8e529ba339f7a7d12a9b4fafe33e43cc80"),
        &hex!("f843a036210c27d08bc29676360b820acc6de648bb730808a3a7d36a960f6869ac4a3aa1a0504adb89d4e609110eebf79183a10b9a4788a797d973c0ba0504e7a97fc1daa6")
    ];
    /// Storage root of the contract at the time that this proof is obtained.
    const CONSENSUS_STATE_STORAGE_ROOT: H256 = H256(hex!(
        "5634f342b966b609cdd8d2f7ed43bb94702c9e83d4e974b08a3c2b8205fd85e3"
    ));
    const CONSENSUS_STATE_CONTRACT_MERKLE_ROOT: H256 = H256(hex!(
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    ));
    const CONSENSUS_STATE_NEXT_VALIDATORS_HASH: H256 = H256(hex!(
        "B41F9EE164A6520C269F8928A1F3264A6F983F27478CB3A2251B77A65E0CEFBF"
    ));

    const CONNECTION_END_PROOF_KEY: &[u8] =
        &hex!("8e80b902df24e0c324c454fcd01ae0c92966a3f6fe4d1809e7fb75043b6549db");
    const CONNECTION_END_PROOF_VALUE: &[u8] =
        &hex!("9ac95d1087518963f797142524b3c6c273bb74297c076c00b02ed129bcb4cfc0");
    const CONNECTION_END_PROOF: [&[u8]; 2] = [
        &hex!("f871808080a01c44ba4a3ade71a6b527cb53c3f2dd91606f91cd119fd74e85208b1d13096739808080808080808080a0f7202a06e8dc011d3123f907597f51546fe03542551af2c9c54d21ba0fbafc7280a0771904c17414dbc0741f3d1fce0d2709d4f73418020b9b4961e4cb3ec6f46ac280"),
        &hex!("f843a0320fddcfabb459601044296253eed7d7cb53d9a8a3e46b1f7db5115be261c419a1a09ac95d1087518963f797142524b3c6c273bb74297c076c00b02ed129bcb4cfc0")
    ];
    /// Storage root of the contract at the time that this proof is obtained.
    const CONNECTION_END_STORAGE_ROOT: H256 = H256(hex!(
        "78c3bf305b31e5f903d623b0b0023bfa764208429d3ecc0f8e61df44b643981d"
    ));

    const NON_MEMBERSHIP_STORAGE_ROOT: H256 = H256(hex!(
        "9e352a10c5a38c301ee06c22a90f0971b679985b2ca6dd66aca224bd7a9957c1"
    ));
    const NON_MEMBERSHIP_PROOF_KEY: &[u8] =
        &hex!("b35cad2b263a62faaae30d8b3f51201fea5501d2df17d59a3eef2751403e684f");
    const NON_MEMBERSHIP_PROOF: [&[u8]; 1] = [
        &hex!("f838a120df6966c971051c3d54ec59162606531493a51404a002842f56009d7e5cf4a8c79594be68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed"),
    ];

    const WASM_CLIENT_ID_PREFIX: &str = "08-wasm";
    const ETHEREUM_CLIENT_ID_PREFIX: &str = "10-ethereum";
    const IBC_KEY_PREFIX: &str = "ibc";
    const INITIAL_CONSENSUS_STATE_HEIGHT: Height = Height {
        revision_number: 0,
        revision_height: 3577184,
    };

    lazy_static::lazy_static! {
        static ref UPDATE_FILES: Vec<path::PathBuf> = {
            let mut update_files = vec![];
            for entry in fs::read_dir(UPDATES_DIR_PATH).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.file_name().is_some() {
                    update_files.push(path);
                }
            }

            update_files.sort();
            update_files
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

        let updates: Vec<ethereum::header::Header<Mainnet>> = (*UPDATE_FILES)
            .iter()
            .map(|f| serde_json::from_str(&fs::read_to_string(f).unwrap()).unwrap())
            .collect::<Vec<_>>();

        for update in updates {
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
                    wasm_consensus_state.data.storage_root.into_bytes(),
                    update.account_update.proofs[0].value,
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

                let res = fast_aggregate_verify(
                    pubkeys.iter().collect::<Vec<&BlsPublicKey>>().as_slice(),
                    message.as_ref(),
                    &signature.0.clone().try_into().unwrap(),
                );

                SystemResult::Ok(cosmwasm_std::ContractResult::Ok::<Binary>(
                    serde_json::to_vec(&res.is_ok()).unwrap().into(),
                ))
            }
            CustomQuery::Aggregate { public_keys } => {
                let pubkey = eth_aggregate_public_keys(
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

        let update = serde_json::from_str::<ethereum::header::Header<Mainnet>>(
            &fs::read_to_string(&*UPDATE_FILES[0]).unwrap(),
        )
        .unwrap();

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

    // TODO: the proof is no longer valid as we removed `trust_level` from the type
    // #[test]
    // fn membership_verification_works_for_client_state() {
    //     let proof = Proof {
    //         key: CLIENT_STATE_PROOF_KEY.into(),
    //         value: CLIENT_STATE_PROOF_VALUE.into(),
    //         proof: CLIENT_STATE_PROOF.into_iter().map(Into::into).collect(),
    //     };

    //     let storage_root = CLIENT_STATE_STORAGE_ROOT.clone();

    //     let client_state = cometbls::client_state::ClientState {
    //         chain_id: "ibc-0".to_string(),
    //         trusting_period: Duration::new(1814400, 0).unwrap(),
    //         unbonding_period: Duration::new(1814400, 0).unwrap(),
    //         max_clock_drift: Duration::new(40, 0).unwrap(),
    //         frozen_height: Height {
    //             revision_number: 0,
    //             revision_height: 0,
    //         },
    //     };

    //     let wasm_client_state = protos::ibc::lightclients::wasm::v1::ClientState {
    //         data: client_state.into_proto_bytes(),
    //         code_id: CLIENT_STATE_WASM_CODE_ID.into(),
    //         latest_height: Some(protos::ibc::core::client::v1::Height {
    //             revision_number: 0,
    //             revision_height: 1,
    //         }),
    //     };

    //     let any_client_state = protos::google::protobuf::Any {
    //         type_url: "/ibc.lightclients.wasm.v1.ClientState".into(),
    //         value: wasm_client_state.encode_to_vec(),
    //     };

    //     do_verify_membership(
    //         ClientStatePath {
    //             client_id: ClientId::new("10-ethereum-0".into()).unwrap(),
    //         }
    //         .to_string(),
    //         storage_root,
    //         3,
    //         proof,
    //         any_client_state.encode_to_vec(),
    //     )
    //     .expect("Membership verification of client state failed");
    // }

    #[test]
    fn membership_verification_works_for_consensus_state() {
        let proof = Proof {
            key: CONSENSUS_STATE_PROOF_KEY.into(),
            value: CONSENSUS_STATE_PROOF_VALUE.into(),
            proof: CONSENSUS_STATE_PROOF.into_iter().map(Into::into).collect(),
        };

        let storage_root = CONSENSUS_STATE_STORAGE_ROOT.clone();

        let consensus_state = cometbls::consensus_state::ConsensusState {
            root: MerkleRoot {
                hash: CONSENSUS_STATE_CONTRACT_MERKLE_ROOT.clone(),
            },
            next_validators_hash: CONSENSUS_STATE_NEXT_VALIDATORS_HASH.clone(),
        };

        let wasm_consensus_state = protos::ibc::lightclients::wasm::v1::ConsensusState {
            data: consensus_state.into_proto_bytes(),
            timestamp: 1684400046,
        };

        let any_consensus_state = protos::google::protobuf::Any {
            type_url: "/ibc.lightclients.wasm.v1.ConsensusState".into(),
            value: wasm_consensus_state.encode_to_vec(),
        };

        do_verify_membership(
            ClientConsensusStatePath {
                client_id: format!("{ETHEREUM_CLIENT_ID_PREFIX}-0"),
                height: Height {
                    revision_number: 0,
                    revision_height: 1,
                },
            }
            .to_string(),
            storage_root,
            3,
            proof,
            any_consensus_state.encode_to_vec(),
        )
        .expect("Membership verification of consensus state failed");
    }

    fn prepare_connection_end() -> (
        Proof,
        H256,
        protos::ibc::core::connection::v1::ConnectionEnd,
    ) {
        let proof = Proof {
            key: CONNECTION_END_PROOF_KEY.into(),
            value: CONNECTION_END_PROOF_VALUE.into(),
            proof: CONNECTION_END_PROOF.into_iter().map(Into::into).collect(),
        };

        let storage_root = CONNECTION_END_STORAGE_ROOT.clone();

        let connection_end = protos::ibc::core::connection::v1::ConnectionEnd {
            client_id: format!("{ETHEREUM_CLIENT_ID_PREFIX}-0"),
            versions: vec![protos::ibc::core::connection::v1::Version {
                identifier: "1".into(),
                features: vec!["ORDER_ORDERED".into(), "ORDER_UNORDERED".into()],
            }],
            state: 1,
            counterparty: Some(protos::ibc::core::connection::v1::Counterparty {
                client_id: format!("{WASM_CLIENT_ID_PREFIX}-0"),
                connection_id: Default::default(),
                prefix: Some(protos::ibc::core::commitment::v1::MerklePrefix {
                    key_prefix: IBC_KEY_PREFIX.as_bytes().to_vec(),
                }),
            }),
            delay_period: 0,
        };

        (proof, storage_root, connection_end)
    }

    #[test]
    fn membership_verification_works_for_connection_end() {
        let (proof, storage_root, connection_end) = prepare_connection_end();

        do_verify_membership(
            ConnectionPath {
                connection_id: "connection-0".parse().unwrap(),
            }
            .to_string(),
            storage_root,
            3,
            proof,
            connection_end.encode_to_vec(),
        )
        .expect("Membership verification of connection end failed");
    }

    #[test]
    fn membership_verification_fails_for_incorrect_proofs() {
        let (mut proof, storage_root, connection_end) = prepare_connection_end();

        let proofs = vec![
            {
                let mut proof = proof.clone();
                proof.value[10] = u8::MAX - proof.value[10]; // Makes sure that produced value is always valid and different
                proof
            },
            {
                let mut proof = proof.clone();
                proof.key[5] = u8::MAX - proof.key[5];
                proof
            },
            {
                proof.proof[0][10] = u8::MAX - proof.proof[0][10];
                proof
            },
        ];

        for proof in proofs {
            assert!(do_verify_membership(
                ConnectionPath {
                    connection_id: "connection-0".parse().unwrap(),
                }
                .to_string(),
                storage_root.clone(),
                3,
                proof,
                connection_end.encode_to_vec(),
            )
            .is_err());
        }
    }

    #[test]
    fn membership_verification_fails_for_incorrect_storage_root() {
        let (proof, mut storage_root, connection_end) = prepare_connection_end();

        storage_root.0[10] = u8::MAX - storage_root.0[10];

        assert!(do_verify_membership(
            ConnectionPath {
                connection_id: "connection-0".parse().unwrap(),
            }
            .to_string(),
            storage_root,
            3,
            proof,
            connection_end.encode_to_vec(),
        )
        .is_err());
    }

    #[test]
    fn membership_verification_fails_for_incorrect_data() {
        let (proof, storage_root, mut connection_end) = prepare_connection_end();

        connection_end.client_id = "incorrect-client-id".into();

        assert!(do_verify_membership(
            ConnectionPath {
                connection_id: "connection-0".parse().unwrap(),
            }
            .to_string(),
            storage_root,
            3,
            proof,
            connection_end.encode_to_vec(),
        )
        .is_err());
    }

    #[test]
    fn non_membership_verification_works() {
        let proof = Proof {
            key: NON_MEMBERSHIP_PROOF_KEY.into(),
            value: vec![0x0],
            proof: NON_MEMBERSHIP_PROOF.into_iter().map(Into::into).collect(),
        };

        let storage_root = NON_MEMBERSHIP_STORAGE_ROOT.clone();

        do_verify_non_membership(
            ClientStatePath {
                client_id: format!("{ETHEREUM_CLIENT_ID_PREFIX}-0"),
            }
            .to_string(),
            storage_root,
            3,
            proof,
        )
        .expect("Membership verification of client state failed");
    }

    #[test]
    fn non_membership_verification_fails_when_value_not_empty() {
        let (proof, storage_root, _) = prepare_connection_end();

        assert_eq!(
            do_verify_non_membership(
                ConnectionPath {
                    connection_id: "connection-0".parse().unwrap(),
                }
                .to_string(),
                storage_root,
                3,
                proof,
            ),
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
