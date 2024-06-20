use cosmwasm_std::Empty;
use ics008_wasm_client::IbcClient;

use crate::errors::Error;

pub struct NearLightClient;

impl IbcClient for NearLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header;

    type Misbehaviour;

    type ClientState;

    type ConsensusState;

    type Encoding;

    fn verify_membership(
        deps: cosmwasm_std::Deps<Self::CustomQuery>,
        height: unionlabs::ibc::core::client::height::Height,
        delay_time_period: u64,
        delay_block_period: u64,
        proof: Vec<u8>,
        path: unionlabs::ibc::core::commitment::merkle_path::MerklePath,
        value: ics008_wasm_client::StorageState,
    ) -> Result<(), ics008_wasm_client::IbcClientError<Self>> {
        todo!()
    }

    fn verify_header(
        deps: cosmwasm_std::Deps<Self::CustomQuery>,
        env: cosmwasm_std::Env,
        header: Self::Header,
    ) -> Result<(), ics008_wasm_client::IbcClientError<Self>> {
        todo!()
    }

    fn verify_misbehaviour(
        deps: cosmwasm_std::Deps<Self::CustomQuery>,
        env: cosmwasm_std::Env,
        misbehaviour: Self::Misbehaviour,
    ) -> Result<(), ics008_wasm_client::IbcClientError<Self>> {
        todo!()
    }

    fn update_state(
        deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
        env: cosmwasm_std::Env,
        header: Self::Header,
    ) -> Result<
        Vec<unionlabs::ibc::core::client::height::Height>,
        ics008_wasm_client::IbcClientError<Self>,
    > {
        todo!()
    }

    fn update_state_on_misbehaviour(
        deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
        env: cosmwasm_std::Env,
        client_message: Vec<u8>,
    ) -> Result<(), ics008_wasm_client::IbcClientError<Self>> {
        todo!()
    }

    fn check_for_misbehaviour_on_header(
        deps: cosmwasm_std::Deps<Self::CustomQuery>,
        header: Self::Header,
    ) -> Result<bool, ics008_wasm_client::IbcClientError<Self>> {
        todo!()
    }

    fn check_for_misbehaviour_on_misbehaviour(
        deps: cosmwasm_std::Deps<Self::CustomQuery>,
        misbehaviour: Self::Misbehaviour,
    ) -> Result<bool, ics008_wasm_client::IbcClientError<Self>> {
        todo!()
    }

    fn verify_upgrade_and_update_state(
        deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
        upgrade_client_state: Self::ClientState,
        upgrade_consensus_state: Self::ConsensusState,
        proof_upgrade_client: Vec<u8>,
        proof_upgrade_consensus_state: Vec<u8>,
    ) -> Result<(), ics008_wasm_client::IbcClientError<Self>> {
        todo!()
    }

    fn migrate_client_store(
        deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
    ) -> Result<(), ics008_wasm_client::IbcClientError<Self>> {
        todo!()
    }

    fn status(
        deps: cosmwasm_std::Deps<Self::CustomQuery>,
        env: &cosmwasm_std::Env,
    ) -> Result<ics008_wasm_client::Status, ics008_wasm_client::IbcClientError<Self>> {
        todo!()
    }

    fn export_metadata(
        deps: cosmwasm_std::Deps<Self::CustomQuery>,
        env: &cosmwasm_std::Env,
    ) -> Result<
        Vec<unionlabs::ibc::core::client::genesis_metadata::GenesisMetadata>,
        ics008_wasm_client::IbcClientError<Self>,
    > {
        todo!()
    }

    fn timestamp_at_height(
        deps: cosmwasm_std::Deps<Self::CustomQuery>,
        height: unionlabs::ibc::core::client::height::Height,
    ) -> Result<u64, ics008_wasm_client::IbcClientError<Self>> {
        todo!()
    }
}
