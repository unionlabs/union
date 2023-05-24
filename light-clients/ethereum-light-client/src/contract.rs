use crate::{
    client_state::tendermint_to_cometbls_client_state,
    consensus_state::{tendermint_to_cometbls_consensus_state, TrustedConsensusState},
    errors::Error,
    eth_encoding::{
        encode_cometbls_client_state, encode_cometbls_consensus_state, generate_commitment_key,
    },
    eth_types::{ExecutionUpdateInfo, LightClientHeader},
    header::Header as EthHeader,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{read_client_state, read_consensus_state, save_consensus_state, update_client_state},
    update::apply_updates,
};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdError, StdResult,
};
use ethereum_consensus::{
    capella,
    compute::{compute_epoch_at_slot, hash_tree_root},
    context::ChainContext,
    execution::{EXECUTION_PAYLOAD_BLOCK_NUMBER_INDEX, EXECUTION_PAYLOAD_STATE_ROOT_INDEX},
    merkle::is_valid_merkle_branch,
    sync_protocol::EXECUTION_PAYLOAD_INDEX,
    types::H256,
};
use ethereum_light_client_verifier::consensus::SyncProtocolVerifier;
use ibc::core::ics24_host::Path;
use prost::Message;
use protos::ibc::lightclients::ethereum::v1::{Header as RawEthHeader, StorageProof};
use sha3::Digest;
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

    let raw_value = match path {
        Path::Connection(_) => value.0.as_slice().to_vec(),
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
        p => {
            return Err(Error::InvalidPath(format!(
                "path type not supported: {p:?}"
            )))
        }
    };

    let storage_proof = StorageProof::decode(proof.0.as_slice())
        .map_err(|_| Error::decode("when decoding storage proof"))?;

    let proof = storage_proof.proof[0].clone();

    let root = H256::from_slice(consensus_state.storage_root.as_bytes());

    let expected_key = generate_commitment_key(
        path.to_string(),
        client_state.counterparty_connection_state_slot.0,
    );

    if hex::encode(&expected_key) != &proof.key[2..] {
        return Err(Error::InvalidCommitmentKey);
    }

    let mut value = sha3::Keccak256::new();
    value.update(&raw_value);
    let value = rlp::encode(&value.finalize().to_vec());

    client_state
        .execution_verifier
        .verify_membership(
            root,
            &expected_key,
            &value,
            proof
                .proof
                .iter()
                .map(|p| hex::decode(&p[2..]).map_err(|_| Error::DecodeError("".into())))
                .collect::<Result<Vec<_>, _>>()?,
        )
        .map_err(|e| Error::Verification(e.to_string()))?;
    Ok(ContractResult::valid(None))
}

pub fn update_header(mut deps: DepsMut, header: EthHeader) -> Result<ContractResult, Error> {
    let trusted_sync_committee = header.trusted_sync_committee;
    let (wasm_consensus_state, consensus_state) =
        read_consensus_state(deps.as_ref(), trusted_sync_committee.height)?.ok_or(
            Error::ConsensusStateNotFound(
                trusted_sync_committee.height.revision_number(),
                trusted_sync_committee.height.revision_height(),
            ),
        )?;

    let trusted_consensus_state = TrustedConsensusState::new(
        consensus_state,
        trusted_sync_committee.sync_committee,
        trusted_sync_committee.is_next,
    )?;

    let consensus_update = header.consensus_update;

    let execution_update = {
        let execution_payload_header = consensus_update.0.finalized_header.execution.clone();
        let (_, state_root_branch) = capella::gen_execution_payload_fields_proof(
            &execution_payload_header,
            &[EXECUTION_PAYLOAD_STATE_ROOT_INDEX],
        )
        .map_err(|_| Error::CannotGenerateProof)?;
        let (_, block_number_branch) = capella::gen_execution_payload_fields_proof(
            &execution_payload_header,
            &[EXECUTION_PAYLOAD_BLOCK_NUMBER_INDEX],
        )
        .map_err(|_| Error::CannotGenerateProof)?;
        ExecutionUpdateInfo {
            state_root: execution_payload_header.state_root,
            state_root_branch,
            block_number: execution_payload_header.block_number,
            block_number_branch,
        }
    };

    let account_update = header.account_update;
    let timestamp = header.timestamp;

    let (wasm_client_state, client_state) = read_client_state(deps.as_ref())?;
    let ctx = client_state.build_context(timestamp)?;

    is_valid_light_client_header(&ctx, &consensus_update.finalized_header)?;

    client_state
        .consensus_verifier
        .validate_updates(
            &ctx,
            &trusted_consensus_state,
            &consensus_update,
            &execution_update,
        )
        .map_err(|e| Error::Verification(e.to_string()))?;

    client_state
        .consensus_verifier
        .ensure_relevant_update(&ctx, &trusted_consensus_state, &consensus_update)
        .map_err(|e| Error::Verification(e.to_string()))?;

    let (new_client_state, new_consensus_state) = apply_updates(
        &ctx,
        &client_state,
        &trusted_consensus_state,
        consensus_update,
        execution_update,
        account_update,
    )?;

    update_client_state(deps.branch(), wasm_client_state, new_client_state);
    save_consensus_state(deps, wasm_consensus_state, new_consensus_state)?;

    Ok(ContractResult::valid(None))
}

/// https://github.com/ethereum/consensus-specs/blob/82d6267951ad47cffa1b7b4179eab97b25a99b91/specs/capella/light-client/sync-protocol.md#modified-is_valid_light_client_header
fn is_valid_light_client_header<C: ChainContext>(
    ctx: &C,
    header: &LightClientHeader,
) -> Result<(), Error> {
    let epoch = compute_epoch_at_slot(ctx, header.beacon.slot);

    if epoch < ctx.fork_parameters().capella_fork_epoch {
        Err(Error::InvalidChainVersion)
    } else {
        is_valid_merkle_branch(
            hash_tree_root(header.execution.clone())
                .map_err(|e| Error::Verification(e.to_string()))?,
            &header.execution_branch,
            EXECUTION_PAYLOAD_INDEX,
            header.beacon.body_root.clone(),
        )
        .map_err(|e| Error::Verification(e.to_string()))
    }
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
    use crate::{
        eth_encoding::{abi, SolidityDataType},
        state::{save_wasm_client_state, save_wasm_consensus_state},
    };
    use cosmwasm_std::{
        testing::{mock_dependencies, MockApi, MockQuerier, MockStorage},
        Empty, OwnedDeps,
    };
    use ethereum_consensus::bls::PublicKeyBytes;
    use ibc::Height;
    use protos::ibc::lightclients::wasm::v1::{
        ClientState as WasmClientState, ConsensusState as WasmConsensusState,
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
            &Height::new(0, 1216).unwrap(),
        );

        let updates: [RawEthHeader; 3] = [
            serde_json::from_str(include_str!("./test/valid_light_client_update_1.json")).unwrap(),
            serde_json::from_str(include_str!("./test/valid_light_client_update_2.json")).unwrap(),
            serde_json::from_str(include_str!("./test/valid_light_client_update_3.json")).unwrap(),
        ];

        for update in updates {
            let update: EthHeader = update.try_into().unwrap();
            update_header(deps.as_mut(), update.clone()).unwrap();
            // Consensus state is saved to the updated height.
            let (_, consensus_state) = read_consensus_state(
                deps.as_ref(),
                Height::new(
                    0,
                    update.consensus_update.finalized_header.beacon.slot.into(),
                )
                .unwrap(),
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
                consensus_state.storage_root,
                update.account_update.account_storage_root.0.to_vec().into(),
            );
            // TODO(aeryz): Add cases for `store_period == update_period` and `update_period == store_period + 1`
            let (_, client_state) = read_client_state(deps.as_ref()).unwrap();
            // Latest slot is updated.
            assert_eq!(
                client_state.latest_slot,
                update.consensus_update.finalized_header.beacon.slot
            );
            // Latest execution block number is updated.
            assert_eq!(
                client_state.latest_execution_block_number,
                update
                    .consensus_update
                    .finalized_header
                    .execution
                    .block_number,
            );
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
            &Height::new(0, 1216).unwrap(),
        );

        let update: EthHeader = serde_json::from_str::<RawEthHeader>(include_str!(
            "./test/valid_light_client_update_1.json"
        ))
        .unwrap()
        .try_into()
        .unwrap();

        (deps, update)
    }

    #[test]
    fn update_fails_when_sync_committee_aggregate_pubkey_is_incorrect() {
        let (mut deps, mut update) = prepare_for_fail_tests();

        let mut pubkey: PublicKeyBytes = (*update
            .trusted_sync_committee
            .sync_committee
            .aggregate_pubkey)
            .clone();
        pubkey.0[0] += 1;
        update
            .trusted_sync_committee
            .sync_committee
            .aggregate_pubkey = From::from(pubkey);
        assert!(update_header(deps.as_mut(), update).is_err());
    }

    #[test]
    fn update_fails_when_finalized_header_execution_branch_merkle_is_invalid() {
        let (mut deps, mut update) = prepare_for_fail_tests();
        update.consensus_update.0.finalized_header.execution_branch[0].0[0] += 1;
        assert!(update_header(deps.as_mut(), update).is_err());
    }

    #[test]
    fn update_fails_when_finality_branch_merkle_is_invalid() {
        let (mut deps, mut update) = prepare_for_fail_tests();
        update.consensus_update.0.finality_branch[0].0[0] += 1;
        assert!(update_header(deps.as_mut(), update).is_err());
    }

    #[test]
    fn update_header_works() {}
}
