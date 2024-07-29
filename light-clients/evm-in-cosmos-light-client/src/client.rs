use cosmwasm_std::{Deps, DepsMut, Env};
use ethereum_light_client::client::{do_verify_membership, do_verify_non_membership};
use ics008_wasm_client::{
    storage_utils::{
        read_client_state, read_consensus_state, save_consensus_state, update_client_state,
    },
    IbcClient, IbcClientError, Status, StorageState,
};
use ics23::ibc_api::SDK_SPECS;
use unionlabs::{
    cosmwasm::wasm::union::custom_query::{
        query_client_state, query_consensus_state, UnionCustomQuery,
    },
    encoding::{DecodeAs, EncodeAs, Proto},
    ibc::{
        core::{
            client::{genesis_metadata::GenesisMetadata, height::Height},
            commitment::merkle_path::MerklePath,
        },
        lightclients::{
            cometbls,
            ethereum::{self, storage_proof::StorageProof},
            evm_in_cosmos::{
                client_state::ClientState, consensus_state::ConsensusState, header::Header,
            },
            wasm,
        },
    },
    ics24::{ClientConsensusStatePath, Path},
};

use crate::errors::Error;

type WasmClientState = wasm::client_state::ClientState<ClientState>;
type WasmConsensusState = wasm::consensus_state::ConsensusState<ConsensusState>;
type WasmL1ConsensusState =
    wasm::consensus_state::ConsensusState<cometbls::consensus_state::ConsensusState>;
type WasmL1ClientState = wasm::client_state::ClientState<cometbls::client_state::ClientState>;
type WasmL2ConsensusState =
    wasm::consensus_state::ConsensusState<ethereum::consensus_state::ConsensusState>;

pub struct EvmInCosmosLightClient;

impl IbcClient for EvmInCosmosLightClient {
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
            )
            .map_err(Error::EthereumLightClient)?,
            StorageState::Empty => do_verify_non_membership(
                path,
                storage_root,
                client_state.data.ibc_commitment_slot,
                storage_proof,
            )
            .map_err(Error::EthereumLightClient)?,
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
        let client_consensus_state_path = Path::ClientConsensusState(ClientConsensusStatePath {
            client_id: client_state.data.l2_client_id.parse().unwrap(),
            height: Height {
                revision_number: 0,
                revision_height: header.l2_slot,
            },
        });
        // The ethereum consensus state is stored in proto-encoded wasm-wrapped form.
        let normalized_l2_consensus_state = WasmL2ConsensusState {
            data: header.l2_consensus_state,
        };
        // Verify inclusion of the ethereum consensus state against union.
        ics23::ibc_api::verify_membership(
            &header.l2_inclusion_proof,
            &SDK_SPECS,
            &l1_consensus_state.data.app_hash,
            &[
                b"ibc".to_vec(),
                client_consensus_state_path.to_string().into_bytes(),
            ],
            normalized_l2_consensus_state.encode_as::<Proto>(),
        )
        .map_err(Error::VerifyL2Membership)?;
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
            client_state.data.latest_slot = updated_height.revision_height;
            update_client_state::<Self>(
                deps.branch(),
                client_state,
                updated_height.revision_height,
            );
        }

        let consensus_state = WasmConsensusState {
            data: ConsensusState {
                evm_state_root: header.l2_consensus_state.state_root,
                ibc_storage_root: header.l2_consensus_state.storage_root,
                timestamp: header.l2_consensus_state.timestamp,
            },
        };
        save_consensus_state::<Self>(deps, consensus_state, &updated_height);
        Ok(vec![updated_height])
    }

    fn update_state_on_misbehaviour(
        _deps: DepsMut<Self::CustomQuery>,
        _env: Env,
        _client_message: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        panic!("impossible; misbehavior check is done on the l1 light client.")
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

    fn status(deps: Deps<Self::CustomQuery>, env: &Env) -> Result<Status, IbcClientError<Self>> {
        let client_state: WasmClientState = read_client_state(deps)?;
        let l1_client_state = query_client_state::<WasmL1ClientState>(
            deps,
            env,
            client_state.data.l1_client_id.clone(),
        )
        .map_err(Error::CustomQuery)?;

        if l1_client_state.data.frozen_height != Height::default() {
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
