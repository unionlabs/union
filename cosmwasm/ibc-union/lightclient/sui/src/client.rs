use cosmwasm_std::Empty;
use ibc_union_light_client::{ClientCreationResult, IbcClient, IbcClientError, StateUpdate};
use ibc_union_msg::lightclient::Status;
use sui_light_client_types::{
    client_state::ClientState, consensus_state::ConsensusState, header::Header,
};
use unionlabs::encoding::Bincode;

use crate::error::Error;

pub enum SuiLightClient {}

impl IbcClient for SuiLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type StorageProof = Header;

    type Encoding = Bincode;

    fn verify_membership(
        _ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _height: u64,
        _key: Vec<u8>,
        _storage_proof: Self::StorageProof,
        _value: Vec<u8>,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        todo!()
    }

    fn verify_non_membership(
        _ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _height: u64,
        _key: Vec<u8>,
        _storage_proof: Self::StorageProof,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        todo!()
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64 {
        consensus_state.timestamp
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        let ClientState::V1(client_state) = client_state;
        client_state.latest_checkpoint
    }

    fn get_counterparty_chain_id(_client_state: &Self::ClientState) -> String {
        todo!()
    }

    fn status(
        _ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _client_state: &Self::ClientState,
    ) -> Status {
        Status::Active
    }

    fn verify_creation(
        _caller: cosmwasm_std::Addr,
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
        _relayer: cosmwasm_std::Addr,
    ) -> Result<ClientCreationResult<Self>, IbcClientError<Self>> {
        Ok(ClientCreationResult::new())
    }

    fn verify_header(
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _caller: cosmwasm_std::Addr,
        _header: Self::Header,
        _relayer: cosmwasm_std::Addr,
    ) -> Result<StateUpdate<Self>, IbcClientError<Self>> {
        let ClientState::V1(client_state) = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(client_state.latest_checkpoint)?;
        Ok(StateUpdate::new(10, consensus_state))
    }

    fn misbehaviour(
        _ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _caller: cosmwasm_std::Addr,
        _misbehaviour: Self::Misbehaviour,
        _relayer: cosmwasm_std::Addr,
    ) -> Result<Self::ClientState, ibc_union_light_client::IbcClientError<Self>> {
        todo!()
    }
}
