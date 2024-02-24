use cosmwasm_std::{Deps, DepsMut, Env, QueryRequest};
use ics008_wasm_client::{
    storage_utils::{
        read_client_state, read_consensus_state, save_client_state, save_consensus_state,
        update_client_state,
    },
    IbcClient, Status, StorageState,
};
use scroll_verifier::{
    scroll_verify_header, scroll_verify_zktrie_storage_absence, scroll_verify_zktrie_storage_proof,
};
use sha3::Digest;
use unionlabs::{
    encoding::Proto,
    google::protobuf::any::Any,
    hash::H256,
    ibc::{
        core::{
            client::{genesis_metadata::GenesisMetadata, height::Height},
            commitment::merkle_path::MerklePath,
        },
        lightclients::{
            cometbls,
            ethereum::{proof::Proof, storage_proof::StorageProof},
            scroll::{client_state::ClientState, consensus_state::ConsensusState, header::Header},
            wasm,
        },
    },
    proof::Path,
    uint::U256,
    IntoEthAbi, TryFromProto,
};

use crate::{errors::Error, eth_encoding::generate_commitment_key};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CustomQuery {
    ConsensusState { client_id: String, height: Height },
}
impl cosmwasm_std::CustomQuery for CustomQuery {}

type WasmClientState = unionlabs::ibc::lightclients::wasm::client_state::ClientState<ClientState>;
type WasmConsensusState =
    unionlabs::ibc::lightclients::wasm::consensus_state::ConsensusState<ConsensusState>;
type WasmL1ConsensusState = unionlabs::ibc::lightclients::wasm::consensus_state::ConsensusState<
    unionlabs::ibc::lightclients::ethereum::consensus_state::ConsensusState,
>;

pub struct ScrollLightClient;

impl IbcClient for ScrollLightClient {
    type Error = Error;

    type CustomQuery = CustomQuery;

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
    ) -> Result<(), Self::Error> {
        let consensus_state: WasmConsensusState =
            read_consensus_state(deps, &height)?.ok_or(Error::ConsensusStateNotFound(height))?;
        let client_state: WasmClientState = read_client_state(deps)?;

        let path = path.key_path.pop().ok_or(Error::EmptyIbcPath)?;

        // This storage root is verified during the header update, so we don't need to verify it again.
        let storage_root = consensus_state.data.ibc_storage_root;

        let storage_proof = {
            let mut proofs = StorageProof::try_from_proto_bytes(&proof)
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
        _env: Env,
        header: Self::Header,
    ) -> Result<(), Self::Error> {
        let client_state: WasmClientState = read_client_state(deps)?;
        let l1_consensus_state =
            deps.querier
                .query::<WasmL1ConsensusState>(&QueryRequest::Custom(
                    CustomQuery::ConsensusState {
                        client_id: client_state.data.l1_client_id.clone(),
                        height: header.l1_height,
                    },
                ))?;
        scroll_verify_header(
            client_state.data,
            header,
            l1_consensus_state.data.storage_root,
        )?;
        Ok(())
    }

    fn verify_misbehaviour(
        _deps: Deps<Self::CustomQuery>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<(), Self::Error> {
        panic!("Not implemented")
    }

    fn update_state(
        mut deps: DepsMut<Self::CustomQuery>,
        _env: Env,
        header: Self::Header,
    ) -> Result<Vec<Height>, Self::Error> {
        let mut client_state: WasmClientState = read_client_state(deps.as_ref())?;
        let l1_consensus_state =
            deps.querier
                .query::<WasmL1ConsensusState>(&QueryRequest::Custom(
                    CustomQuery::ConsensusState {
                        client_id: client_state.data.l1_client_id.clone(),
                        height: header.l1_height,
                    },
                ))?;
        if client_state.data.latest_batch_index < header.finalized_proof.batch_index {
            client_state.data.latest_batch_index = header.finalized_proof.batch_index;
            update_client_state(
                deps.branch(),
                client_state,
                header.finalized_proof.batch_index,
            );
        }
        let updated_height = Height {
            revision_number: 0,
            revision_height: header.finalized_proof.batch_index,
        };
        let consensus_state = WasmConsensusState {
            data: ConsensusState {
                batch_index: header.finalized_proof.batch_index,
                ibc_storage_root: header.ibc_account_proof.storage_root,
                timestamp: l1_consensus_state.data.timestamp,
            },
        };
        save_consensus_state(deps, consensus_state, &updated_height);
        Ok(vec![updated_height])
    }

    fn update_state_on_misbehaviour(
        deps: DepsMut<Self::CustomQuery>,
        env: Env,
        _client_message: Vec<u8>,
    ) -> Result<(), Self::Error> {
        let mut client_state: WasmClientState = read_client_state(deps.as_ref())?;
        client_state.data.frozen_height = Height {
            revision_number: client_state.latest_height.revision_number,
            revision_height: env.block.height,
        };
        save_client_state(deps, client_state);
        Ok(())
    }

    fn check_for_misbehaviour_on_header(
        _deps: Deps<Self::CustomQuery>,
        _header: Self::Header,
    ) -> Result<bool, Self::Error> {
        Ok(false)
    }

    fn check_for_misbehaviour_on_misbehaviour(
        _deps: Deps<Self::CustomQuery>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<bool, Self::Error> {
        unimplemented!()
    }

    fn verify_upgrade_and_update_state(
        _deps: DepsMut<Self::CustomQuery>,
        _upgrade_client_state: Self::ClientState,
        _upgrade_consensus_state: Self::ConsensusState,
        _proof_upgrade_client: Vec<u8>,
        _proof_upgrade_consensus_state: Vec<u8>,
    ) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn migrate_client_store(_deps: Deps<Self::CustomQuery>) -> Result<(), Self::Error> {
        // migration from previous client to self, so unimplemented now
        unimplemented!()
    }

    fn status(deps: Deps<Self::CustomQuery>, _env: &Env) -> Result<Status, Self::Error> {
        let client_state: WasmClientState = read_client_state(deps)?;

        if client_state.data.frozen_height != Height::default() {
            return Ok(Status::Frozen);
        }

        let Some(_) = read_consensus_state::<Self::CustomQuery, ConsensusState>(
            deps,
            &client_state.latest_height,
        )?
        else {
            return Ok(Status::Expired);
        };

        Ok(Status::Active)
    }

    fn export_metadata(
        _deps: Deps<Self::CustomQuery>,
        _env: &Env,
    ) -> Result<Vec<GenesisMetadata>, Self::Error> {
        Ok(Vec::new())
    }

    fn timestamp_at_height(
        deps: Deps<Self::CustomQuery>,
        height: Height,
    ) -> Result<u64, Self::Error> {
        Ok(
            read_consensus_state::<Self::CustomQuery, ConsensusState>(deps, &height)?
                .ok_or(Error::ConsensusStateNotFound(height))?
                .data
                .timestamp,
        )
    }
}

fn do_verify_membership(
    path: String,
    storage_root: H256,
    counterparty_commitment_slot: U256,
    storage_proof: Proof,
    raw_value: Vec<u8>,
) -> Result<(), Error> {
    check_commitment_key(
        &path,
        counterparty_commitment_slot,
        H256(storage_proof.key.to_big_endian()),
    )?;

    let path = path
        .parse::<Path<String, Height>>()
        .map_err(|_| Error::UnknownIbcPath(path))?;

    let canonical_value = match path {
        Path::ClientStatePath(_) => {
            Any::<cometbls::client_state::ClientState>::try_from_proto_bytes(raw_value.as_ref())
                .map_err(|e| Error::DecodeFromProto {
                    reason: format!("{e:?}"),
                })?
                .0
                .into_eth_abi_bytes()
        }
        Path::ClientConsensusStatePath(_) => Any::<
            wasm::consensus_state::ConsensusState<cometbls::consensus_state::ConsensusState>,
        >::try_from_proto_bytes(raw_value.as_ref())
        .map_err(|e| Error::DecodeFromProto {
            reason: format!("{e:?}"),
        })?
        .0
        .data
        .into_eth_abi_bytes(),
        _ => raw_value,
    };

    // We store the hash of the data, not the data itself to the commitments map.
    let expected_value_hash = H256::from(
        sha3::Keccak256::new()
            .chain_update(canonical_value)
            .finalize(),
    );

    let proof_value = H256::from(storage_proof.value.to_big_endian());

    if expected_value_hash != proof_value {
        return Err(Error::StoredValueMismatch {
            expected: expected_value_hash,
            stored: proof_value,
        });
    }

    scroll_verify_zktrie_storage_proof(
        storage_root,
        storage_proof.key.to_big_endian().into(),
        &storage_proof.value.to_big_endian().as_ref(),
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
        &path,
        counterparty_commitment_slot,
        H256(storage_proof.key.to_big_endian()),
    )?;
    scroll_verify_zktrie_storage_absence(
        storage_root,
        H256(storage_proof.key.to_big_endian()),
        &storage_proof.proof,
    )?;
    Ok(())
}

fn check_commitment_key(
    path: &str,
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
