use cosmwasm_std::Empty;
use ibc_union_light_client::IbcClient;
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
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        todo!()
    }

    fn verify_non_membership(
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        todo!()
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64 {
        todo!()
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        todo!()
    }

    fn get_counterparty_chain_id(client_state: &Self::ClientState) -> String {
        todo!()
    }

    fn status(
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        client_state: &Self::ClientState,
    ) -> ibc_union_msg::lightclient::Status {
        todo!()
    }

    fn verify_creation(
        caller: cosmwasm_std::Addr,
        client_state: &Self::ClientState,
        consensus_state: &Self::ConsensusState,
        relayer: cosmwasm_std::Addr,
    ) -> Result<
        ibc_union_light_client::ClientCreationResult<Self>,
        ibc_union_light_client::IbcClientError<Self>,
    > {
        todo!()
    }

    fn verify_header(
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        caller: cosmwasm_std::Addr,
        header: Self::Header,
        relayer: cosmwasm_std::Addr,
    ) -> Result<
        ibc_union_light_client::StateUpdate<Self>,
        ibc_union_light_client::IbcClientError<Self>,
    > {
        todo!()
    }

    fn misbehaviour(
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        caller: cosmwasm_std::Addr,
        misbehaviour: Self::Misbehaviour,
        relayer: cosmwasm_std::Addr,
    ) -> Result<Self::ClientState, ibc_union_light_client::IbcClientError<Self>> {
        todo!()
    }
}
