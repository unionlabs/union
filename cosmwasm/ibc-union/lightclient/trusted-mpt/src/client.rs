use cosmwasm_std::{Addr, Empty};
use ethereum_light_client_types::StorageProof;
use ibc_union_light_client::{
    ClientCreationResult, IbcClient, IbcClientCtx, IbcClientError, StateUpdate,
};
use ibc_union_msg::lightclient::Status;
use trusted_mpt_light_client_types::{ClientState, ConsensusState, Header};
use unionlabs::encoding::Bincode;

use crate::errors::Error;

pub enum MptTrustedLightClient {}

impl IbcClient for MptTrustedLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type StorageProof = StorageProof;

    type Encoding = Bincode;

    fn verify_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;
        Ok(ethereum_light_client::client::verify_membership(
            key,
            consensus_state.storage_root,
            storage_proof,
            value,
        )
        .map_err(Into::<Error>::into)?)
    }

    fn verify_non_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;
        Ok(ethereum_light_client::client::verify_non_membership(
            key,
            consensus_state.storage_root,
            storage_proof,
        )
        .map_err(Into::<Error>::into)?)
    }

    fn verify_header(
        ctx: IbcClientCtx<Self>,
        caller: Addr,
        header: Self::Header,
        _relayer: Addr,
    ) -> Result<StateUpdate<Self>, IbcClientError<Self>> {
        let ClientState::V1(mut client_state) = ctx.read_self_client_state()?;
        if !client_state
            .whitelisted_relayers
            .contains(&caller.to_string())
        {
            return Err(Error::Unauthorized(caller).into());
        }

        // We still verify the account storage root since we only trust `state_root`
        evm_storage_verifier::verify_account_storage_root(
            header.state_root,
            &client_state.ibc_contract_address,
            &header.ibc_account_proof.proof,
            &header.ibc_account_proof.storage_root,
        )
        .map_err(Error::InvalidContractAddressProof)?;

        let mut update = StateUpdate::new(
            header.height,
            ConsensusState {
                state_root: header.state_root,
                storage_root: header.ibc_account_proof.storage_root,
                timestamp: header.timestamp,
            },
        );

        if header.height > client_state.latest_height {
            client_state.latest_height = header.height;
            update = update.overwrite_client_state(ClientState::V1(client_state));
        }

        Ok(update)
    }

    fn misbehaviour(
        _ctx: IbcClientCtx<Self>,
        _caller: Addr,
        _misbehaviour: Self::Misbehaviour,
        _relayer: Addr,
    ) -> Result<Self::ClientState, IbcClientError<Self>> {
        Err(Error::NoMisbehaviourInTrustedClient.into())
    }

    fn status(ctx: IbcClientCtx<Self>, _client_state: &Self::ClientState) -> Status {
        let _ = ctx;

        Status::Active
    }

    fn verify_creation(
        _caller: Addr,
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
        _relayer: Addr,
    ) -> Result<ClientCreationResult<Self>, IbcClientError<MptTrustedLightClient>> {
        Ok(ClientCreationResult::new())
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64 {
        consensus_state.timestamp
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        let ClientState::V1(client_state) = client_state;
        client_state.latest_height
    }

    fn get_counterparty_chain_id(client_state: &Self::ClientState) -> String {
        let ClientState::V1(client_state) = client_state;
        client_state.chain_id.to_string()
    }
}
