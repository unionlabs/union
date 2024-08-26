use cosmwasm_std::{Deps, Empty};
use ics008_wasm_client::{
    storage_utils::read_client_state, IbcClient, IbcClientError, StorageState,
};
use unionlabs::{
    cosmwasm::wasm::union::custom_query::{query_consensus_state, UnionCustomQuery},
    encoding::Proto,
    hash::{H160, H256},
    ibc::{
        core::{client::height::Height, commitment::merkle_path::MerklePath},
        lightclients::{
            ethereum,
            movement::{
                client_state::ClientState, consensus_state::ConsensusState, header::Header,
            },
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

pub struct MovementLightClient;

impl IbcClient for MovementLightClient {
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
        delay_time_period: u64,
        delay_block_period: u64,
        proof: Vec<u8>,
        path: MerklePath,
        value: StorageState,
    ) -> Result<(), IbcClientError<Self>> {
        todo!()
    }

    fn verify_header(
        deps: Deps<Self::CustomQuery>,
        env: cosmwasm_std::Env,
        header: Header,
    ) -> Result<(), IbcClientError<Self>> {
        let client_state: WasmClientState = read_client_state(deps)?;

        let l1_consensus_state = query_consensus_state::<WasmL1ConsensusState>(
            deps,
            &env,
            client_state.data.l1_client_id.to_string(),
            header.l1_height,
        )
        .map_err(Error::CustomQuery)?;

        aptos_verifier::verify_tx_state(
            &header.tx_proof,
            header
                .state_proof
                .latest_ledger_info()
                .commit_info
                .executed_state_id,
            17, // TODO(aeryz): this should be configurable
        )
        .map_err(Into::<Error>::into)?;

        // TODO(aeryz): make sure the given state_proof_hash_proof.key matches the correct slot

        ethereum_verifier::verify::verify_account_storage_root(
            l1_consensus_state.data.state_root,
            &client_state.data.l1_contract_address,
            &header.settlement_contract_proof.proof,
            &header.settlement_contract_proof.storage_root,
        )
        .unwrap();

        ethereum_verifier::verify::verify_storage_proof(
            header.settlement_contract_proof.storage_root,
            header.state_proof_hash_proof.key,
            &header.state_proof.hash(),
            header.state_proof_hash_proof.proof,
        )
        .unwrap();

        Ok(())
    }

    fn verify_misbehaviour(
        deps: Deps<Self::CustomQuery>,
        env: cosmwasm_std::Env,
        misbehaviour: Self::Misbehaviour,
    ) -> Result<(), IbcClientError<Self>> {
        todo!()
    }

    fn update_state(
        deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
        env: cosmwasm_std::Env,
        header: Self::Header,
    ) -> Result<Vec<Height>, IbcClientError<Self>> {
        todo!()
    }

    fn update_state_on_misbehaviour(
        deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
        env: cosmwasm_std::Env,
        client_message: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        todo!()
    }

    fn check_for_misbehaviour_on_header(
        deps: Deps<Self::CustomQuery>,
        header: Self::Header,
    ) -> Result<bool, IbcClientError<Self>> {
        todo!()
    }

    fn check_for_misbehaviour_on_misbehaviour(
        deps: Deps<Self::CustomQuery>,
        misbehaviour: Self::Misbehaviour,
    ) -> Result<bool, IbcClientError<Self>> {
        todo!()
    }

    fn verify_upgrade_and_update_state(
        deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
        upgrade_client_state: Self::ClientState,
        upgrade_consensus_state: Self::ConsensusState,
        proof_upgrade_client: Vec<u8>,
        proof_upgrade_consensus_state: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        todo!()
    }

    fn migrate_client_store(
        deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
    ) -> Result<(), IbcClientError<Self>> {
        todo!()
    }

    fn status(
        deps: Deps<Self::CustomQuery>,
        env: &cosmwasm_std::Env,
    ) -> Result<ics008_wasm_client::Status, IbcClientError<Self>> {
        todo!()
    }

    fn export_metadata(
        deps: Deps<Self::CustomQuery>,
        env: &cosmwasm_std::Env,
    ) -> Result<
        Vec<unionlabs::ibc::core::client::genesis_metadata::GenesisMetadata>,
        IbcClientError<Self>,
    > {
        todo!()
    }

    fn timestamp_at_height(
        deps: Deps<Self::CustomQuery>,
        height: Height,
    ) -> Result<u64, IbcClientError<Self>> {
        todo!()
    }
}
