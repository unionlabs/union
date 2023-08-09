use std::str::FromStr;

use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdError,
};
use ethabi::ethereum_types::U256 as ethabi_U256;
use ethereum_verifier::{
    primitives::{ExecutionAddress, Slot},
    validate_light_client_update, verify_account_storage_root, verify_storage_proof,
};
use ibc::core::ics24_host::Path;
use sha3::Digest;
use unionlabs::{
    ethereum::H256,
    ethereum_consts_traits::ChainSpec,
    ibc::{
        core::client::height::Height,
        lightclients::ethereum::{header::Header, proof::Proof, storage_proof::StorageProof},
    },
    TryFromProto,
};
use wasm_light_client_types::msg::{
    ClientMessage, ContractResult, MerklePath, Status, StatusResponse,
};

use crate::{
    consensus_state::TrustedConsensusState,
    context::LightClientContext,
    custom_query::{query_aggregate_public_keys, CustomQuery, VerificationContext},
    errors::Error,
    eth_encoding::generate_commitment_key,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{read_client_state, read_consensus_state, save_consensus_state, update_client_state},
    update::apply_light_client_update,
    Config,
};

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, Error> {
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut<CustomQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
    let result = match msg {
        ExecuteMsg::VerifyMembership {
            height,
            delay_time_period,
            delay_block_period,
            proof,
            path,
            value,
        } => verify_membership(
            deps.as_ref(),
            height,
            delay_time_period,
            delay_block_period,
            proof,
            path,
            value,
        ),
        ExecuteMsg::UpdateState {
            client_message: ClientMessage { header, .. },
        } => {
            if let Some(header) = header {
                let header =
                    Header::<Config>::try_from_proto_bytes(&header.data).map_err(|err| {
                        Error::decode(format!(
                            "when converting proto header to header in update: {err:#?}"
                        ))
                    })?;
                update_header(deps, header)
            } else {
                Err(StdError::not_found("Not implemented").into())
            }
        }
        _ => Ok(ContractResult::valid(None)),
    }?;

    Ok(Response::default().set_data(result.encode()?))
}

/// Verifies if the `value` is committed at `path` in the counterparty light client.
pub fn verify_membership(
    deps: Deps<CustomQuery>,
    height: Height,
    _delay_time_period: u64,
    _delay_block_period: u64,
    proof: Binary,
    path: MerklePath,
    value: Binary,
) -> Result<ContractResult, Error> {
    let consensus_state = read_consensus_state(deps, &height)?.ok_or(
        Error::ConsensusStateNotFound(height.revision_number, height.revision_height),
    )?;
    let client_state = read_client_state(deps)?;

    let path = Path::from_str(
        path.key_path
            .last()
            .ok_or(Error::InvalidPath("path is empty".into()))?,
    )
    .map_err(|e| Error::InvalidPath(e.to_string()))?;

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

    do_verify_membership(
        path,
        storage_root,
        client_state.data.counterparty_commitment_slot,
        storage_proof,
        value,
    )?;

    Ok(ContractResult::valid(None))
}

pub fn do_verify_membership(
    path: Path,
    storage_root: H256,
    counterparty_commitment_slot: Slot,
    storage_proof: Proof,
    value: Binary,
) -> Result<(), Error> {
    let expected_commitment_key =
        generate_commitment_key(path.to_string(), counterparty_commitment_slot);

    // Data MUST be stored to the commitment path that is defined in ICS23.
    if expected_commitment_key != storage_proof.key {
        return Err(Error::invalid_commitment_key(
            expected_commitment_key,
            storage_proof.key,
        ));
    }

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

pub fn update_header<C: ChainSpec>(
    mut deps: DepsMut<CustomQuery>,
    header: Header<C>,
) -> Result<ContractResult, Error> {
    let trusted_sync_committee = header.trusted_sync_committee;
    let mut wasm_consensus_state =
        read_consensus_state(deps.as_ref(), &trusted_sync_committee.trusted_height)?.ok_or(
            Error::ConsensusStateNotFound(
                trusted_sync_committee.trusted_height.revision_number,
                trusted_sync_committee.trusted_height.revision_height,
            ),
        )?;

    let aggregate_public_key = query_aggregate_public_keys(
        deps.as_ref(),
        trusted_sync_committee.sync_committee.pubkeys.clone().into(),
    )?;

    let trusted_height = trusted_sync_committee.trusted_height;

    let trusted_consensus_state = TrustedConsensusState::new(
        wasm_consensus_state.data.clone(),
        trusted_sync_committee.sync_committee,
        aggregate_public_key,
        trusted_sync_committee.is_next,
    )?;

    let consensus_update = header.consensus_update;
    let account_update = header.account_update;
    let timestamp = header.timestamp;

    let mut wasm_client_state = read_client_state(deps.as_ref())?;
    let genesis_validators_root = wasm_client_state.data.genesis_validators_root.clone();
    let ctx = LightClientContext::new(&wasm_client_state.data, trusted_consensus_state);

    validate_light_client_update::<LightClientContext<C>, VerificationContext>(
        &ctx,
        consensus_update.clone(),
        (timestamp - wasm_client_state.data.genesis_time) / wasm_client_state.data.seconds_per_slot
            + wasm_client_state.data.fork_parameters.genesis_slot,
        genesis_validators_root,
        VerificationContext {
            deps: deps.as_ref(),
        },
    )
    .map_err(|e| Error::Verification(e.to_string()))?;

    let proof_data = account_update.proofs.get(0).ok_or(Error::EmptyProof)?;

    let address: ExecutionAddress = proof_data.key.as_slice().try_into().unwrap();
    let storage_root = proof_data.value.as_slice().try_into().unwrap();

    verify_account_storage_root(
        consensus_update
            .attested_header
            .execution
            .state_root
            .clone(),
        &address,
        &proof_data.proof,
        &storage_root,
    )
    .map_err(|e| Error::Verification(e.to_string()))?;

    // Some updates can be only for updating the sync committee, therefore the execution number can be
    // smaller. We don't want to save a new state if this is the case.
    let updated_execution_height = core::cmp::max(
        trusted_height.revision_height,
        consensus_update.attested_header.execution.block_number,
    );

    apply_light_client_update::<LightClientContext<C>>(
        &mut wasm_client_state,
        &mut wasm_consensus_state,
        consensus_update,
        storage_root,
    )?;

    update_client_state(
        deps.branch(),
        wasm_client_state,
        // wasm_client_state.data,
        updated_execution_height,
    );
    save_consensus_state(
        deps,
        wasm_consensus_state,
        // wasm_consensus_state.data,
        updated_execution_height,
    )?;

    Ok(ContractResult::valid(None))
}

#[entry_point]
pub fn query(deps: Deps<CustomQuery>, env: Env, msg: QueryMsg) -> Result<QueryResponse, Error> {
    let response = match msg {
        QueryMsg::Status {} => query_status(deps, &env)?,
    };

    to_binary(&response).map_err(Into::into)
}

fn query_status(deps: Deps<CustomQuery>, env: &Env) -> Result<StatusResponse, Error> {
    let client_state = read_client_state(deps)?;

    if client_state.data.frozen_height.is_some() {
        return Ok(Status::Frozen.into());
    }

    let Some(consensus_state) = read_consensus_state(deps, &client_state.latest_height)? else {
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

fn is_client_expired(
    consensus_state_timestamp: u64,
    trusting_period: u64,
    current_block_time: u64,
) -> bool {
    consensus_state_timestamp + trusting_period < current_block_time
}

#[cfg(test)]
mod test {
    use std::marker::PhantomData;

    use cosmwasm_std::{
        testing::{mock_env, MockApi, MockQuerier, MockQuerierCustomHandlerResult, MockStorage},
        OwnedDeps, SystemResult, Timestamp,
    };
    use ethereum_verifier::crypto::{eth_aggregate_public_keys, fast_aggregate_verify};
    use ibc::{
        core::{
            ics02_client::client_type::ClientType,
            ics24_host::{
                identifier::{ClientId, ConnectionId},
                path::{ClientConsensusStatePath, ClientStatePath, ConnectionPath},
            },
        },
        Height as IbcHeight,
    };
    use prost::Message;
    use unionlabs::{
        bls::BlsPublicKey,
        ethereum_consts_traits::Minimal,
        ibc::{
            core::commitment::merkle_root::MerkleRoot,
            google::protobuf::duration::Duration,
            lightclients::{cometbls, ethereum, tendermint::fraction::Fraction, wasm},
        },
        IntoProto,
    };

    use super::*;
    use crate::state::{save_wasm_client_state, save_wasm_consensus_state};

    /// These values are obtained by uploading a dummy contract with the necessary types to the devnet and
    /// reading the values by `eth_getProof` RPC call.
    const CLIENT_STATE_PROOF_KEY: &str =
        "b35cad2b263a62faaae30d8b3f51201fea5501d2df17d59a3eef2751403e684f";
    const CLIENT_STATE_PROOF_VALUE: &str =
        "272c7c82ac0f0adbfe4ae30614165bf3b94d49754ce8c1955cc255dcc829b5";
    const CLIENT_STATE_PROOF: [&str; 2] = [
        "f871808080a0b9f6e8d11cf768b8034f04b8b2ab45bb5ca792e1c6e3929cf8222a885631ffac808080808080808080a0f7202a06e8dc011d3123f907597f51546fe03542551af2c9c54d21ba0fbafc7280a0d1797d071b81705da736e39e75f1186c8e529ba339f7a7d12a9b4fafe33e43cc80",
        "f842a03a8c7f353aebdcd6b56a67cd1b5829681a3c6e1695282161ab3faa6c3666d4c3a09f272c7c82ac0f0adbfe4ae30614165bf3b94d49754ce8c1955cc255dcc829b5"
    ];
    /// Storage root of the contract at the time that this proof is obtained.
    const CLIENT_STATE_STORAGE_ROOT: &str =
        "5634f342b966b609cdd8d2f7ed43bb94702c9e83d4e974b08a3c2b8205fd85e3";
    const CLIENT_STATE_WASM_CODE_ID: &str =
        "B41F9EE164A6520C269F8928A1F3264A6F983F27478CB3A2251B77A65E0CEFBF";

    const CONSENSUS_STATE_PROOF_KEY: &str =
        "9f22934f38bf5512b9c33ed55f71525c5d129895aad5585a2624f6c756c1c101";
    const CONSENSUS_STATE_PROOF_VALUE: &str =
        "504adb89d4e609110eebf79183a10b9a4788a797d973c0ba0504e7a97fc1daa6";
    const CONSENSUS_STATE_PROOF: [&str; 2] = [
        "f871808080a0b9f6e8d11cf768b8034f04b8b2ab45bb5ca792e1c6e3929cf8222a885631ffac808080808080808080a0f7202a06e8dc011d3123f907597f51546fe03542551af2c9c54d21ba0fbafc7280a0d1797d071b81705da736e39e75f1186c8e529ba339f7a7d12a9b4fafe33e43cc80",
        "f843a036210c27d08bc29676360b820acc6de648bb730808a3a7d36a960f6869ac4a3aa1a0504adb89d4e609110eebf79183a10b9a4788a797d973c0ba0504e7a97fc1daa6"
    ];
    /// Storage root of the contract at the time that this proof is obtained.
    const CONSENSUS_STATE_STORAGE_ROOT: &str =
        "5634f342b966b609cdd8d2f7ed43bb94702c9e83d4e974b08a3c2b8205fd85e3";
    const CONSENSUS_STATE_CONTRACT_MERKLE_ROOT: &str =
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    const CONSENSUS_STATE_NEXT_VALIDATORS_HASH: &str =
        "B41F9EE164A6520C269F8928A1F3264A6F983F27478CB3A2251B77A65E0CEFBF";

    const CONNECTION_END_PROOF_KEY: &str =
        "8e80b902df24e0c324c454fcd01ae0c92966a3f6fe4d1809e7fb75043b6549db";
    const CONNECTION_END_PROOF_VALUE: &str =
        "9ac95d1087518963f797142524b3c6c273bb74297c076c00b02ed129bcb4cfc0";
    const CONNECTION_END_PROOF: [&str; 2] = [
        "f871808080a01c44ba4a3ade71a6b527cb53c3f2dd91606f91cd119fd74e85208b1d13096739808080808080808080a0f7202a06e8dc011d3123f907597f51546fe03542551af2c9c54d21ba0fbafc7280a0771904c17414dbc0741f3d1fce0d2709d4f73418020b9b4961e4cb3ec6f46ac280",
        "f843a0320fddcfabb459601044296253eed7d7cb53d9a8a3e46b1f7db5115be261c419a1a09ac95d1087518963f797142524b3c6c273bb74297c076c00b02ed129bcb4cfc0"
    ];
    /// Storage root of the contract at the time that this proof is obtained.
    const CONNECTION_END_STORAGE_ROOT: &str =
        "78c3bf305b31e5f903d623b0b0023bfa764208429d3ecc0f8e61df44b643981d";

    const WASM_CLIENT_ID_PREFIX: &str = "08-wasm";
    const ETHEREUM_CLIENT_ID_PREFIX: &str = "10-ethereum";
    const IBC_KEY_PREFIX: &str = "ibc";
    const INITIAL_CONSENSUS_STATE_HEIGHT: Height = Height {
        revision_number: 0,
        revision_height: 1328,
    };

    #[test]
    fn query_status_returns_active() {
        let mut deps = OwnedDeps::<_, _, _, CustomQuery> {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier::<CustomQuery>::new(&[]).with_custom_handler(custom_query_handler),
            custom_query_type: PhantomData,
        };

        let wasm_client_state =
            serde_json::from_str(include_str!("./test/client_state.json")).unwrap();

        let wasm_consensus_state =
            serde_json::from_str(include_str!("./test/consensus_state.json")).unwrap();

        save_wasm_client_state(
            deps.as_mut(),
            <_>::try_from_proto(wasm_client_state).unwrap(),
        );

        save_wasm_consensus_state(
            deps.as_mut(),
            <_>::try_from_proto(wasm_consensus_state).unwrap(),
            &INITIAL_CONSENSUS_STATE_HEIGHT,
        );

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0);

        assert_eq!(query_status(deps.as_ref(), &env), Ok(Status::Active.into()));
    }

    #[test]
    fn query_status_returns_frozen() {
        let mut deps = OwnedDeps::<_, _, _, CustomQuery> {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier::<CustomQuery>::new(&[]).with_custom_handler(custom_query_handler),
            custom_query_type: PhantomData,
        };

        let mut wasm_client_state =
            <wasm::client_state::ClientState<ethereum::client_state::ClientState>>::try_from_proto(
                serde_json::from_str(include_str!("./test/client_state.json")).unwrap(),
            )
            .unwrap();

        wasm_client_state.data.frozen_height = Some(Height {
            revision_number: 1,
            revision_height: 1,
        });

        save_wasm_client_state(deps.as_mut(), wasm_client_state);

        assert_eq!(
            query_status(deps.as_ref(), &mock_env()),
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

        let mut wasm_client_state =
            <wasm::client_state::ClientState<ethereum::client_state::ClientState>>::try_from_proto(
                serde_json::from_str(include_str!("./test/client_state.json")).unwrap(),
            )
            .unwrap();

        save_wasm_client_state(deps.as_mut(), wasm_client_state.clone());

        // Client returns expired here because it cannot find the consensus state
        assert_eq!(
            query_status(deps.as_ref(), &mock_env()),
            Ok(Status::Expired.into())
        );

        let wasm_consensus_state = <wasm::consensus_state::ConsensusState<
            ethereum::consensus_state::ConsensusState,
        >>::try_from_proto(
            serde_json::from_str(include_str!("./test/consensus_state.json")).unwrap(),
        )
        .unwrap();

        save_wasm_consensus_state(
            deps.as_mut(),
            wasm_consensus_state.clone(),
            &INITIAL_CONSENSUS_STATE_HEIGHT,
        );

        wasm_client_state.data.trusting_period = 10;
        save_wasm_client_state(deps.as_mut(), wasm_client_state.clone());
        let mut env = mock_env();

        env.block.time = Timestamp::from_seconds(
            wasm_client_state.data.trusting_period + wasm_consensus_state.timestamp + 1,
        );
        assert_eq!(
            query_status(deps.as_ref(), &env),
            Ok(Status::Expired.into())
        );

        env.block.time = Timestamp::from_seconds(
            wasm_client_state.data.trusting_period + wasm_consensus_state.timestamp,
        );
        assert_eq!(query_status(deps.as_ref(), &env), Ok(Status::Active.into()))
    }

    #[test]
    fn update_works_with_good_data() {
        let mut deps = OwnedDeps::<_, _, _, CustomQuery> {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier::<CustomQuery>::new(&[]).with_custom_handler(custom_query_handler),
            custom_query_type: PhantomData,
        };

        let wasm_client_state =
            serde_json::from_str(include_str!("./test/client_state.json")).unwrap();

        let wasm_consensus_state =
            serde_json::from_str(include_str!("./test/consensus_state.json")).unwrap();

        save_wasm_client_state(
            deps.as_mut(),
            <_>::try_from_proto(wasm_client_state).unwrap(),
        );
        save_wasm_consensus_state(
            deps.as_mut(),
            <_>::try_from_proto(wasm_consensus_state).unwrap(),
            &INITIAL_CONSENSUS_STATE_HEIGHT,
        );

        let updates = &[
            ethereum::header::Header::<Minimal>::try_from_proto(
                serde_json::from_str(include_str!("./test/sync_committee_update_1.json")).unwrap(),
            )
            .unwrap(),
            ethereum::header::Header::<Minimal>::try_from_proto(
                serde_json::from_str(include_str!("./test/finality_update_1.json")).unwrap(),
            )
            .unwrap(),
            ethereum::header::Header::<Minimal>::try_from_proto(
                serde_json::from_str(include_str!("./test/sync_committee_update_2.json")).unwrap(),
            )
            .unwrap(),
            ethereum::header::Header::<Minimal>::try_from_proto(
                serde_json::from_str(include_str!("./test/finality_update_2.json")).unwrap(),
            )
            .unwrap(),
            ethereum::header::Header::<Minimal>::try_from_proto(
                serde_json::from_str(include_str!("./test/finality_update_3.json")).unwrap(),
            )
            .unwrap(),
            ethereum::header::Header::<Minimal>::try_from_proto(
                serde_json::from_str(include_str!("./test/finality_update_4.json")).unwrap(),
            )
            .unwrap(),
        ];

        for update in updates {
            update_header(deps.as_mut(), update.clone()).unwrap();
            // Consensus state is saved to the updated height.
            if update.consensus_update.attested_header.beacon.slot
                > update.trusted_sync_committee.trusted_height.revision_height
            {
                // It's a finality update
                let wasm_consensus_state = read_consensus_state(
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
                let wasm_client_state = read_client_state(deps.as_ref()).unwrap();
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
                let wasm_consensus_state = read_consensus_state(
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

    fn prepare_for_fail_tests() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier<CustomQuery>, CustomQuery>,
        ethereum::header::Header<Minimal>,
    ) {
        let mut deps = OwnedDeps::<_, _, _, CustomQuery> {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier::<CustomQuery>::new(&[]).with_custom_handler(custom_query_handler),
            custom_query_type: PhantomData,
        };

        let wasm_client_state: protos::ibc::lightclients::wasm::v1::ClientState =
            serde_json::from_str(include_str!("./test/client_state.json")).unwrap();

        let wasm_consensus_state: protos::ibc::lightclients::wasm::v1::ConsensusState =
            serde_json::from_str(include_str!("./test/consensus_state.json")).unwrap();

        save_wasm_client_state(deps.as_mut(), wasm_client_state.try_into().unwrap());
        save_wasm_consensus_state(
            deps.as_mut(),
            wasm_consensus_state.try_into().unwrap(),
            &INITIAL_CONSENSUS_STATE_HEIGHT,
        );

        let update =
            serde_json::from_str::<protos::union::ibc::lightclients::ethereum::v1::Header>(
                include_str!("./test/sync_committee_update_1.json"),
            )
            .unwrap();

        (deps, update.try_into().unwrap())
    }

    #[test]
    fn update_fails_when_sync_committee_aggregate_pubkey_is_incorrect() {
        let (mut deps, mut update) = prepare_for_fail_tests();

        let mut pubkey = update
            .trusted_sync_committee
            .sync_committee
            .aggregate_pubkey
            .clone();
        pubkey.0[0] += 1;
        update
            .trusted_sync_committee
            .sync_committee
            .aggregate_pubkey = pubkey;
        assert!(update_header(deps.as_mut(), update).is_err());
    }

    #[test]
    fn update_fails_when_finalized_header_execution_branch_merkle_is_invalid() {
        let (mut deps, mut update) = prepare_for_fail_tests();
        update.consensus_update.finalized_header.execution_branch[0].0[0] += 1;
        assert!(update_header(deps.as_mut(), update).is_err());
    }

    #[test]
    fn update_fails_when_finality_branch_merkle_is_invalid() {
        let (mut deps, mut update) = prepare_for_fail_tests();
        update.consensus_update.finality_branch[0].0[0] += 1;
        assert!(update_header(deps.as_mut(), update).is_err());
    }

    #[test]
    fn membership_verification_works_for_client_state() {
        let proof = Proof {
            key: hex::decode(CLIENT_STATE_PROOF_KEY).unwrap(),
            value: hex::decode(CLIENT_STATE_PROOF_VALUE).unwrap(),
            proof: CLIENT_STATE_PROOF
                .iter()
                .map(|p| hex::decode(p).unwrap())
                .collect(),
        };

        let storage_root = hex::decode(CLIENT_STATE_STORAGE_ROOT).unwrap();

        let client_state = cometbls::client_state::ClientState {
            chain_id: "ibc-0".to_string(),
            trust_level: Fraction {
                numerator: 1,
                denominator: 3,
            },
            trusting_period: Duration {
                seconds: 1814400,
                nanos: 0,
            },
            unbonding_period: Duration {
                seconds: 1814400,
                nanos: 0,
            },
            max_clock_drift: Duration {
                seconds: 40,
                nanos: 0,
            },
            frozen_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
        };

        let wasm_client_state = protos::ibc::lightclients::wasm::v1::ClientState {
            data: client_state.into_proto_bytes(),
            code_id: hex::decode(CLIENT_STATE_WASM_CODE_ID).unwrap(),
            latest_height: Some(protos::ibc::core::client::v1::Height {
                revision_number: 0,
                revision_height: 1,
            }),
        };

        let any_client_state = protos::google::protobuf::Any {
            type_url: "/ibc.lightclients.wasm.v1.ClientState".into(),
            value: wasm_client_state.encode_to_vec(),
        };

        do_verify_membership(
            ClientStatePath::new(
                &ClientId::new(ClientType::new(ETHEREUM_CLIENT_ID_PREFIX.into()), 0).unwrap(),
            )
            .into(),
            storage_root.try_into().unwrap(),
            3,
            proof,
            any_client_state.encode_to_vec().into(),
        )
        .expect("Membership verification of client state failed");
    }

    #[test]
    fn membership_verification_works_for_consensus_state() {
        let proof = Proof {
            key: hex::decode(CONSENSUS_STATE_PROOF_KEY).unwrap(),
            value: hex::decode(CONSENSUS_STATE_PROOF_VALUE).unwrap(),
            proof: CONSENSUS_STATE_PROOF
                .iter()
                .map(|p| hex::decode(p).unwrap())
                .collect(),
        };

        let storage_root = hex::decode(CONSENSUS_STATE_STORAGE_ROOT).unwrap();

        let consensus_state = cometbls::consensus_state::ConsensusState {
            root: MerkleRoot {
                hash: hex::decode(CONSENSUS_STATE_CONTRACT_MERKLE_ROOT).unwrap(),
            },
            next_validators_hash: hex::decode(CONSENSUS_STATE_NEXT_VALIDATORS_HASH).unwrap(),
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
            ClientConsensusStatePath::new(
                &ClientId::new(ClientType::new(ETHEREUM_CLIENT_ID_PREFIX.into()), 0).unwrap(),
                &IbcHeight::new(0, 1).unwrap(),
            )
            .into(),
            storage_root.try_into().unwrap(),
            3,
            proof,
            any_consensus_state.encode_to_vec().into(),
        )
        .expect("Membership verification of consensus state failed");
    }

    fn prepare_connection_end() -> (
        Proof,
        Vec<u8>,
        protos::ibc::core::connection::v1::ConnectionEnd,
    ) {
        let proof = Proof {
            key: hex::decode(CONNECTION_END_PROOF_KEY).unwrap(),
            value: hex::decode(CONNECTION_END_PROOF_VALUE).unwrap(),
            proof: CONNECTION_END_PROOF
                .iter()
                .map(|p| hex::decode(p).unwrap())
                .collect(),
        };

        let storage_root = hex::decode(CONNECTION_END_STORAGE_ROOT).unwrap();

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
            ConnectionPath::new(&ConnectionId::new(0)).into(),
            storage_root.try_into().unwrap(),
            3,
            proof,
            connection_end.encode_to_vec().into(),
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
                ConnectionPath::new(&ConnectionId::new(0)).into(),
                storage_root.clone().try_into().unwrap(),
                3,
                proof,
                connection_end.encode_to_vec().into(),
            )
            .is_err());
        }
    }

    #[test]
    fn membership_verification_fails_for_incorrect_storage_root() {
        let (proof, mut storage_root, connection_end) = prepare_connection_end();

        storage_root[10] = u8::MAX - storage_root[10];

        assert!(do_verify_membership(
            ConnectionPath::new(&ConnectionId::new(0)).into(),
            storage_root.try_into().unwrap(),
            3,
            proof,
            connection_end.encode_to_vec().into(),
        )
        .is_err());
    }

    #[test]
    fn membership_verification_fails_for_incorrect_data() {
        let (proof, storage_root, mut connection_end) = prepare_connection_end();

        connection_end.client_id = "incorrect-client-id".into();

        assert!(do_verify_membership(
            ConnectionPath::new(&ConnectionId::new(0)).into(),
            storage_root.try_into().unwrap(),
            3,
            proof,
            connection_end.encode_to_vec().into(),
        )
        .is_err());
    }
}
