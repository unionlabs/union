use std::sync::Arc;

use cosmwasm_std::{Deps, DepsMut, Env};
use ethereum_light_client::client::{canonicalize_stored_value, check_commitment_key};
use ethers_core::abi::AbiDecode;
use ics008_wasm_client::{
    storage_utils::{
        read_client_state, read_consensus_state, save_client_state, save_consensus_state,
        update_client_state,
    },
    IbcClient, IbcClientError, Status, StorageState,
};
use scroll_codec::{
    batch_header::BatchHeader,
    chunk::{ChunkV0, ChunkV1},
    CommitBatchCall,
};
use unionlabs::{
    cosmwasm::wasm::union::custom_query::{query_consensus_state, UnionCustomQuery},
    encoding::{DecodeAs, Proto},
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

        let call = <CommitBatchCall as AbiDecode>::decode(header.commit_batch_calldata)
            .map_err(|err| Error::CommitBatchDecode(Arc::new(err)))?;

        let timestamp = match BatchHeader::decode(call.parent_batch_header)
            .map_err(Error::BatchHeaderDecode)?
        {
            BatchHeader::V0(_) => {
                call.chunks
                    .last()
                    .map(ChunkV0::decode)
                    .ok_or(Error::EmptyBatch)?
                    .map_err(Error::ChunkV0Decode)?
                    .blocks
            }
            BatchHeader::V1(_) => {
                call.chunks
                    .last()
                    .map(ChunkV1::decode)
                    .ok_or(Error::EmptyBatch)?
                    .map_err(Error::ChunkV1Decode)?
                    .blocks
            }
        }
        .pop()
        .ok_or(Error::EmptyBatch)?
        .timestamp;

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
                ibc_storage_root: header.l2_ibc_account_proof.storage_root,
                // must be nanos
                timestamp: 1_000_000_000 * timestamp,
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

    fn migrate_client_store(_deps: DepsMut<Self::CustomQuery>) -> Result<(), IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
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
