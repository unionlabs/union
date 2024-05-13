use cosmwasm_std::{Deps, DepsMut, Env};
use gnark_mimc::new_mimc_constants_bls12_377;
use ics008_wasm_client::{
    storage_utils::{
        read_client_state, read_consensus_state, save_client_state, save_consensus_state,
        update_client_state,
    },
    IbcClient, IbcClientError, Status, StorageState,
};
use sha3::Digest;
use unionlabs::{
    cosmwasm::wasm::union::custom_query::{query_consensus_state, UnionCustomQuery},
    encoding::{Decode, EncodeAs, EthAbi, Proto},
    google::protobuf::any::Any,
    hash::H256,
    ibc::{
        core::{
            client::{genesis_metadata::GenesisMetadata, height::Height},
            commitment::merkle_path::MerklePath,
        },
        lightclients::{
            cometbls,
            linea::{client_state::ClientState, consensus_state::ConsensusState, header::Header},
            wasm,
        },
    },
    ics24::Path,
    linea::{
        account::ZkAccount,
        proof::{InclusionProof, NonInclusionProof},
    },
    uint::U256,
};

use crate::{errors::Error, eth_encoding::generate_commitment_key};

type WasmClientState = unionlabs::ibc::lightclients::wasm::client_state::ClientState<ClientState>;
type WasmConsensusState =
    unionlabs::ibc::lightclients::wasm::consensus_state::ConsensusState<ConsensusState>;
type WasmL1ConsensusState = unionlabs::ibc::lightclients::wasm::consensus_state::ConsensusState<
    unionlabs::ibc::lightclients::ethereum::consensus_state::ConsensusState,
>;

pub struct LineaLightClient;

impl IbcClient for LineaLightClient {
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

        match value {
            StorageState::Occupied(value) => {
                let inclusion_proof =
                    InclusionProof::decode(&proof).map_err(Error::StorageProofDecode)?;
                do_verify_membership(
                    path,
                    storage_root,
                    client_state.data.l2_ibc_contract_commitment_slot,
                    inclusion_proof,
                    value,
                )?
            }
            StorageState::Empty => {
                let noninclusion_proof =
                    NonInclusionProof::decode(&proof).map_err(Error::StorageProofDecode)?;
                do_verify_non_membership(
                    path,
                    storage_root,
                    client_state.data.l2_ibc_contract_commitment_slot,
                    noninclusion_proof,
                )?
            }
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
        linea_verifier::verify_header(
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

        let updated_height = Height {
            revision_number: client_state.latest_height.revision_number,
            revision_height: header.l1_height.revision_height,
        };

        if client_state.latest_height < header.l1_height {
            client_state.data.l1_latest_height = updated_height;
            update_client_state::<Self>(
                deps.branch(),
                client_state,
                updated_height.revision_height,
            );
        }

        // TODO: perhaps force the proof to be an actual inclusion proof off-chain
        let account = match header.l2_ibc_contract_proof {
            unionlabs::linea::proof::MerkleProof::Inclusion(inclusion) => {
                // Guaranteed to success as we previously verified the proof
                // which involved decoding and hashing the account.
                ZkAccount::decode(inclusion.proof.value).expect("impossible")
            }
            unionlabs::linea::proof::MerkleProof::NonInclusion(_) => {
                return Err(Error::InvalidL2AccountProof.into())
            }
        };

        let consensus_state = WasmConsensusState {
            data: ConsensusState {
                ibc_storage_root: account.storage_root,
                // must be nanos
                timestamp: 1_000_000_000 * header.l2_timestamp,
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
    storage_proof: InclusionProof,
    raw_value: Vec<u8>,
) -> Result<(), Error> {
    // TODO: handle error
    let key = storage_proof.key.try_into().unwrap();

    check_commitment_key(&path, ibc_commitment_slot, key)?;

    let path = path
        .parse::<Path<String, Height>>()
        .map_err(Error::PathParse)?;

    let canonical_value = match path {
        Path::ClientState(_) => {
            Any::<cometbls::client_state::ClientState>::decode(raw_value.as_ref())
                .map_err(Error::CometblsClientStateDecode)?
                .0
                .encode_as::<EthAbi>()
        }
        Path::ClientConsensusState(_) => Any::<
            wasm::consensus_state::ConsensusState<cometbls::consensus_state::ConsensusState>,
        >::decode(raw_value.as_ref())
        .map_err(Error::CometblsConsensusStateDecode)?
        .0
        .data
        .encode_as::<EthAbi>(),
        _ => raw_value,
    };

    // We store the hash of the data, not the data itself to the commitments map.
    let expected_value_hash = H256::from(
        sha3::Keccak256::new()
            .chain_update(canonical_value)
            .finalize(),
    );

    // TODO: handle error
    let proof_value = H256(storage_proof.proof.value.clone().try_into().unwrap());

    if expected_value_hash != proof_value {
        return Err(Error::StoredValueMismatch {
            expected: expected_value_hash,
            stored: proof_value,
        });
    }

    linea_zktrie::verify::verify_inclusion_and_key::<H256>(
        &new_mimc_constants_bls12_377(),
        storage_proof.leaf_index,
        &storage_proof.proof,
        storage_root,
        key,
    )?;

    Ok(())
}

/// Verifies that no value is committed at `path` in the counterparty light client's storage.
fn do_verify_non_membership(
    path: String,
    storage_root: H256,
    ibc_commitment_slot: U256,
    noninclusion_proof: NonInclusionProof,
) -> Result<(), Error> {
    // TODO: handle error
    let key = noninclusion_proof.key.clone().try_into().unwrap();

    check_commitment_key(&path, ibc_commitment_slot, key)?;

    linea_zktrie::verify::verify_noninclusion::<H256>(
        &new_mimc_constants_bls12_377(),
        &noninclusion_proof,
        storage_root,
        key,
    )?;

    Ok(())
}

fn check_commitment_key(path: &str, ibc_commitment_slot: U256, key: H256) -> Result<(), Error> {
    let expected_commitment_key = generate_commitment_key(path, ibc_commitment_slot);

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
