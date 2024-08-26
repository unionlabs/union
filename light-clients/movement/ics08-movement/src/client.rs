use cosmwasm_std::{Deps, Empty};
use ics008_wasm_client::{IbcClient, IbcClientError, StorageState};
use unionlabs::{
    encoding::Proto,
    ibc::{
        core::{client::height::Height, commitment::merkle_path::MerklePath},
        lightclients::movement::{
            client_state::ClientState, consensus_state::ConsensusState, header::Header,
        },
    },
};

use crate::errors::Error;

type WasmClientState = unionlabs::ibc::lightclients::wasm::client_state::ClientState<ClientState>;
type WasmConsensusState =
    unionlabs::ibc::lightclients::wasm::consensus_state::ConsensusState<ConsensusState>;

pub struct MovementLightClient;

impl IbcClient for MovementLightClient {
    type Error = Error;

    type CustomQuery = Empty;

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
        header: Self::Header,
    ) -> Result<(), IbcClientError<Self>> {
        todo!()
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
