use crate::{
    client_state::tendermint_to_cometbls_client_state,
    consensus_state::{tendermint_to_cometbls_consensus_state, TrustedConsensusState},
    context::LightClientContext,
    errors::Error,
    eth_encoding::{
        encode_cometbls_client_state, encode_cometbls_consensus_state, generate_commitment_key,
    },
    header::Header as EthHeader,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{read_client_state, read_consensus_state, save_consensus_state, update_client_state},
    update::apply_light_client_update,
};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdError, StdResult,
};
use ethereum_verifier::{
    primitives::{ExecutionAddress, Hash32, Slot},
    validate_light_client_update, verify_account_storage_root, verify_storage_proof,
};
use ibc::core::ics24_host::Path;
use prost::Message;
use protos::union::ibc::lightclients::ethereum::v1::{Header as RawEthHeader, Proof, StorageProof};
use sha3::Digest;
use ssz_rs::prelude::*;
use std::str::FromStr;
use wasm_light_client_types::{
    decode_client_state_to_concrete_state, decode_consensus_state_to_concrete_state,
    msg::{ClientMessage, ContractResult, Height, MerklePath, Status, StatusResponse},
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
    deps: DepsMut,
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
                let header: EthHeader = RawEthHeader::decode(header.data.as_slice())
                    .map_err(|_| Error::decode("when converting proto header to header in update"))?
                    .try_into()?;
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
///
/// To optimize the ethereum contracts for gas efficiency, we are storing little amount
/// of data possible in custom formats. For this reason, value that we get from the host
/// chain is sometimes different than what stored in Ethereum.
/// - `Connection`: both `value` and stored data is protobuf encoded `ConnectionEnd`.
/// - `ClientState`: `value` is protobuf encoded `wasm.ClientState` wrapped in `Any`. It's `data`
/// field is protobuf encoded `tendermint.ClientState` wrapped in `Any`. Ethereum stores eth abi
/// encoded `cometbls.ClientState`.
/// - `ConsensusState`: `value` is protobuf encoded `wasm.ConsensusState` wrapped in `Any`. It's
/// `data` field is protobuf encoded `tendermint.ConsensusState` wrapped in `Any`. Ethereum stores
/// an eth abi encoded optimized version of `cometbls.ConsensusState`.
pub fn verify_membership(
    deps: Deps,
    height: Height,
    _delay_time_period: u64,
    _delay_block_period: u64,
    proof: Binary,
    path: MerklePath,
    value: Binary,
) -> Result<ContractResult, Error> {
    let (_, consensus_state) =
        read_consensus_state(deps, height.try_into().map_err(|_| Error::InvalidHeight)?)?.ok_or(
            Error::ConsensusStateNotFound(height.revision_number, height.revision_height),
        )?;
    let (_, client_state) = read_client_state(deps)?;

    let path = Path::from_str(
        path.key_path
            .last()
            .ok_or(Error::InvalidPath("path is empty".into()))?,
    )
    .map_err(|e| Error::InvalidPath(e.to_string()))?;

    // This storage root is verified during the header update, so we don't need to verify it again.
    let storage_root = Hash32::try_from(consensus_state.storage_root.as_bytes()).map_err(|_| {
        Error::decode("consensus state has invalid `storage_root` (must be 32 bytes)")
    })?;

    let storage_proof = {
        let mut proofs = StorageProof::decode(proof.0.as_slice())
            .map_err(|_| Error::decode("when decoding storage proof"))?
            .proofs;
        if proofs.len() > 1 {
            return Err(Error::BatchingProofsNotSupported);
        }
        proofs.pop().ok_or(Error::EmptyProof)?
    };

    do_verify_membership(
        path,
        storage_root,
        client_state.counterparty_commitment_slot,
        storage_proof,
        value,
    )?;

    Ok(ContractResult::valid(None))
}

pub fn do_verify_membership(
    path: Path,
    storage_root: Hash32,
    counterparty_commitment_slot: Slot,
    storage_proof: Proof,
    value: Binary,
) -> Result<(), Error> {
    let raw_value = match path {
        Path::ClientState(_) => {
            let cometbls_client_state = tendermint_to_cometbls_client_state(
                decode_client_state_to_concrete_state(value.0.as_slice())?,
            );

            encode_cometbls_client_state(cometbls_client_state)?
        }
        Path::ClientConsensusState(_) => {
            let cometbls_consensus_state = tendermint_to_cometbls_consensus_state(
                decode_consensus_state_to_concrete_state(value.0.as_slice())?,
            );

            encode_cometbls_consensus_state(cometbls_consensus_state)?
        }
        Path::Connection(_)
        | Path::ChannelEnd(_)
        | Path::Commitment(_)
        | Path::Ack(_)
        | Path::SeqRecv(_) => value.0.as_slice().to_vec(),
        p => {
            return Err(Error::InvalidPath(format!(
                "path type not supported for membership verification: {p:?}"
            )))
        }
    };

    // Data MUST be stored to the commitment path that is defined in ICS23.
    if generate_commitment_key(path.to_string(), counterparty_commitment_slot) != storage_proof.key
    {
        return Err(Error::InvalidCommitmentKey);
    }

    // We store the hash of the data, not the data itself to the commitments map.
    let stored_value = rlp::encode(
        &(sha3::Keccak256::new()
            .chain_update(raw_value)
            .finalize()
            .as_slice()),
    );

    verify_storage_proof(
        storage_root,
        &storage_proof.key,
        &stored_value,
        &storage_proof.proof,
    )
    .map_err(|e| Error::Verification(e.to_string()))
}

pub fn update_header(mut deps: DepsMut, header: EthHeader) -> Result<ContractResult, Error> {
    let trusted_sync_committee = header.trusted_sync_committee;
    let (wasm_consensus_state, mut consensus_state) =
        read_consensus_state(deps.as_ref(), trusted_sync_committee.height)?.ok_or(
            Error::ConsensusStateNotFound(
                trusted_sync_committee.height.revision_number(),
                trusted_sync_committee.height.revision_height(),
            ),
        )?;

    let trusted_consensus_state = TrustedConsensusState::new(
        consensus_state.clone(),
        trusted_sync_committee.sync_committee,
        trusted_sync_committee.is_next,
    )?;

    let consensus_update = header.consensus_update;

    let account_update = header.account_update;
    let timestamp = header.timestamp;

    let (wasm_client_state, mut client_state) = read_client_state(deps.as_ref())?;

    let ctx = LightClientContext::new(&client_state, trusted_consensus_state);

    validate_light_client_update::<LightClientContext>(
        &ctx,
        consensus_update.clone(),
        (timestamp
            .into_tm_time()
            .ok_or(Error::TimestampNotSet)?
            .unix_timestamp() as u64
            - client_state.genesis_time)
            / client_state.seconds_per_slot
            + client_state.fork_parameters.genesis_slot,
        client_state.genesis_validators_root,
    )
    .map_err(|e| Error::Verification(e.to_string()))?;

    let proof_data = account_update.proofs.get(0).ok_or(Error::EmptyProof)?;

    let address: ExecutionAddress = proof_data.address.as_slice().try_into().unwrap();
    let storage_root = proof_data.storage_hash.as_slice().try_into().unwrap();

    verify_account_storage_root(
        consensus_update
            .finalized_header
            .execution
            .state_root
            .clone(),
        &address,
        &proof_data.proof,
        &storage_root,
    )
    .map_err(|e| Error::Verification(e.to_string()))?;

    apply_light_client_update::<LightClientContext>(
        &mut client_state,
        &mut consensus_state,
        consensus_update,
        storage_root,
    )?;

    update_client_state(deps.branch(), wasm_client_state, client_state);
    save_consensus_state(deps, wasm_consensus_state, consensus_state)?;

    Ok(ContractResult::valid(None))
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Status {} => to_binary(&query_status()),
    }
}

// TODO(aeryz): Status needs to be stored and fetched
fn query_status() -> StatusResponse {
    StatusResponse {
        status: Status::Active.to_string(),
        genesis_metadata: vec![],
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::state::{save_wasm_client_state, save_wasm_consensus_state};
    use cosmwasm_std::{
        testing::{mock_dependencies, MockApi, MockQuerier, MockStorage},
        Empty, OwnedDeps,
    };
    use ethereum_verifier::crypto::BlsPublicKey;
    use ibc::{
        core::{
            ics02_client::client_type::ClientType,
            ics24_host::{
                identifier::ClientId,
                path::{ClientConsensusStatePath, ClientStatePath},
            },
        },
        Height as IbcHeight,
    };
    use protos::ibc::core::client::v1::Height as ProtoHeight;
    use protos::{
        google::protobuf::{Any, Duration, Timestamp},
        ibc::{
            core::commitment::v1::MerkleRoot,
            lightclients::{
                tendermint::v1::{
                    ClientState as TmClientState, ConsensusState as TmConsensusState, Fraction,
                },
                wasm::v1::{ClientState as WasmClientState, ConsensusState as WasmConsensusState},
            },
        },
    };

    #[test]
    fn update_works_with_good_data() {
        let mut deps = mock_dependencies();

        let wasm_client_state: WasmClientState =
            serde_json::from_str(include_str!("./test/client_state.json")).unwrap();

        let wasm_consensus_state: WasmConsensusState =
            serde_json::from_str(include_str!("./test/consensus_state.json")).unwrap();

        save_wasm_client_state(deps.as_mut(), &wasm_client_state);
        save_wasm_consensus_state(
            deps.as_mut(),
            &wasm_consensus_state,
            &IbcHeight::new(0, 1328).unwrap(),
        );

        let updates: &[RawEthHeader] = &[
            serde_json::from_str(include_str!("./test/sync_committee_update_1.json")).unwrap(),
            serde_json::from_str(include_str!("./test/finality_update_1.json")).unwrap(),
            serde_json::from_str(include_str!("./test/sync_committee_update_2.json")).unwrap(),
            serde_json::from_str(include_str!("./test/finality_update_2.json")).unwrap(),
            serde_json::from_str(include_str!("./test/finality_update_3.json")).unwrap(),
            serde_json::from_str(include_str!("./test/finality_update_4.json")).unwrap(),
        ];

        for update in updates {
            let update: EthHeader = update.clone().try_into().unwrap();
            update_header(deps.as_mut(), update.clone()).unwrap();
            // Consensus state is saved to the updated height.
            if update.consensus_update.finalized_header.beacon.slot
                > update.trusted_sync_committee.height.revision_height()
            {
                // It's a finality update
                let (_, consensus_state) = read_consensus_state(
                    deps.as_ref(),
                    IbcHeight::new(0, update.consensus_update.finalized_header.beacon.slot).unwrap(),
                )
                .unwrap()
                .unwrap();
                // Slot is updated.
                assert_eq!(
                    consensus_state.slot,
                    update.consensus_update.finalized_header.beacon.slot
                );
                // Storage root is updated.
                assert_eq!(
                    consensus_state.storage_root.as_bytes(),
                    update.account_update.proofs[0].storage_hash,
                );
                // Latest slot is updated.
                // TODO(aeryz): Add cases for `store_period == update_period` and `update_period == store_period + 1`
                let (_, client_state) = read_client_state(deps.as_ref()).unwrap();
                assert_eq!(
                    client_state.latest_slot,
                    update.consensus_update.finalized_header.beacon.slot
                );
            } else {
                // It's a sync committee update
                let (_, consensus_state) =
                    read_consensus_state(deps.as_ref(), update.trusted_sync_committee.height)
                        .unwrap()
                        .unwrap();

                assert_eq!(
                    consensus_state.next_sync_committee.unwrap(),
                    update
                        .consensus_update
                        .next_sync_committee
                        .unwrap()
                        .aggregate_public_key
                );
            }
        }
    }

    fn prepare_for_fail_tests() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
        EthHeader,
    ) {
        let mut deps = mock_dependencies();

        let wasm_client_state: WasmClientState =
            serde_json::from_str(include_str!("./test/client_state.json")).unwrap();

        let wasm_consensus_state: WasmConsensusState =
            serde_json::from_str(include_str!("./test/consensus_state.json")).unwrap();

        save_wasm_client_state(deps.as_mut(), &wasm_client_state);
        save_wasm_consensus_state(
            deps.as_mut(),
            &wasm_consensus_state,
            &IbcHeight::new(0, 1328).unwrap(),
        );

        let update: EthHeader = serde_json::from_str::<RawEthHeader>(include_str!(
            "./test/sync_committee_update_1.json"
        ))
        .unwrap()
        .try_into()
        .unwrap();

        (deps, update)
    }

    #[test]
    fn update_fails_when_sync_committee_aggregate_pubkey_is_incorrect() {
        let (mut deps, mut update) = prepare_for_fail_tests();

        let mut pubkey: BlsPublicKey = update
            .trusted_sync_committee
            .sync_committee
            .aggregate_public_key
            .clone();
        pubkey[0] += 1;
        update
            .trusted_sync_committee
            .sync_committee
            .aggregate_public_key = pubkey;
        assert!(update_header(deps.as_mut(), update).is_err());
    }

    #[test]
    fn update_fails_when_finalized_header_execution_branch_merkle_is_invalid() {
        let (mut deps, mut update) = prepare_for_fail_tests();
        update.consensus_update.finalized_header.execution_branch[0][0] += 1;
        assert!(update_header(deps.as_mut(), update).is_err());
    }

    #[test]
    fn update_fails_when_finality_branch_merkle_is_invalid() {
        let (mut deps, mut update) = prepare_for_fail_tests();
        update.consensus_update.finality_branch[0][0] += 1;
        assert!(update_header(deps.as_mut(), update).is_err());
    }

    #[test]
    fn membership_verification_works_for_client_state() {
        let proof = Proof {
                key: hex::decode(
                    "b35cad2b263a62faaae30d8b3f51201fea5501d2df17d59a3eef2751403e684f",
                )
                .unwrap(),
                value: hex::decode(
                    "e83b93381f2e43cc03cb7e823317b5dd1854d02abccd5c2fa96a59888ddcd602"
                ).unwrap(),
                proof: vec![
                    hex::decode("f871808080a08e048ecf26a7dc2320ef294e3def6e4e8d52934299986e0268f43fa426b283b5808080808080808080a0f7202a06e8dc011d3123f907597f51546fe03542551af2c9c54d21ba0fbafc7280a0771904c17414dbc0741f3d1fce0d2709d4f73418020b9b4961e4cb3ec6f46ac280").unwrap(),
                    hex::decode("f843a03a8c7f353aebdcd6b56a67cd1b5829681a3c6e1695282161ab3faa6c3666d4c3a1a0e83b93381f2e43cc03cb7e823317b5dd1854d02abccd5c2fa96a59888ddcd602").unwrap(),
                ],
        };

        let storage_root =
            hex::decode("1223777f330ea4734b0a4ac964def8242a3bc17421c929494837d591bf1d33c4")
                .unwrap();

        let client_state = TmClientState {
            chain_id: "ibc-0".to_string(),
            trust_level: Some(Fraction {
                numerator: 1,
                denominator: 3,
            }),
            trusting_period: Some(Duration {
                seconds: 1814400,
                nanos: 0,
            }),
            unbonding_period: Some(Duration {
                seconds: 1814400,
                nanos: 0,
            }),
            max_clock_drift: Some(Duration {
                seconds: 40,
                nanos: 0,
            }),
            frozen_height: Some(ProtoHeight {
                revision_number: 0,
                revision_height: 0,
            }),
            latest_height: Some(ProtoHeight {
                revision_number: 0,
                revision_height: 1,
            }),
            ..Default::default()
        };

        let any_client_state = Any {
            type_url: "/ibc.lightclients.tendermint.v1.ClientState".into(),
            value: client_state.encode_to_vec(),
        };

        let wasm_client_state = WasmClientState {
            data: any_client_state.encode_to_vec(),
            ..Default::default()
        };

        let any_client_state = Any {
            type_url: "/ibc.lightclients.wasm.v1.ClientState".into(),
            value: wasm_client_state.encode_to_vec(),
        };

        do_verify_membership(
            ClientStatePath::new(&ClientId::new(ClientType::new("10-ethereum".into()), 0).unwrap())
                .into(),
            Hash32::try_from(storage_root.as_slice()).unwrap(),
            3,
            proof,
            any_client_state.encode_to_vec().into(),
        )
        .unwrap();
    }

    #[test]
    fn membership_verification_works_for_consensus_state() {
        let proof = Proof {
                key: hex::decode(
                    "9f22934f38bf5512b9c33ed55f71525c5d129895aad5585a2624f6c756c1c101",
                )
                .unwrap(),
                value: hex::decode(
                    "3a62ff0d63f377098f870ab7a320b5d6eb2f8cd70c766c987ea2d4aa74125e8a"
                ).unwrap(),
                proof: vec![
                    hex::decode("f871808080a08e048ecf26a7dc2320ef294e3def6e4e8d52934299986e0268f43fa426b283b5808080808080808080a0f7202a06e8dc011d3123f907597f51546fe03542551af2c9c54d21ba0fbafc7280a0361d1cfb583d3e591e6f9b9114dc891a989736d2da999a0cd9333fe42bb99b1180").unwrap(),
                    hex::decode("f843a036210c27d08bc29676360b820acc6de648bb730808a3a7d36a960f6869ac4a3aa1a03a62ff0d63f377098f870ab7a320b5d6eb2f8cd70c766c987ea2d4aa74125e8a").unwrap(),
                ],
        };

        let storage_root =
            hex::decode("034ad1c5701b0c51653c0529b1d1873365cb6ff8d262888b25876f3631ad52e3")
                .unwrap();

        let consensus_state = TmConsensusState {
            timestamp: Some(Timestamp {
                seconds: 1684400046,
                nanos: 0,
            }),
            root: Some(MerkleRoot {
                hash: hex::decode(
                    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                )
                .unwrap(),
            }),
            next_validators_hash: hex::decode(
                "B41F9EE164A6520C269F8928A1F3264A6F983F27478CB3A2251B77A65E0CEFBF",
            )
            .unwrap(),
        };

        let any_consensus_state = Any {
            type_url: "/ibc.lightclients.tendermint.v1.ConsensusState".into(),
            value: consensus_state.encode_to_vec(),
        };

        let wasm_consensus_state = WasmConsensusState {
            data: any_consensus_state.encode_to_vec(),
            ..Default::default()
        };

        let any_consensus_state = Any {
            type_url: "/ibc.lightclients.wasm.v1.ClientState".into(),
            value: wasm_consensus_state.encode_to_vec(),
        };

        do_verify_membership(
            ClientConsensusStatePath::new(
                &ClientId::new(ClientType::new("10-ethereum".into()), 0).unwrap(),
                &IbcHeight::new(0, 1).unwrap(),
            )
            .into(),
            Hash32::try_from(storage_root.as_slice()).unwrap(),
            3,
            proof,
            any_consensus_state.encode_to_vec().into(),
        )
        .unwrap();
    }
}
