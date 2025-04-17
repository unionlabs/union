use bob_light_client_types::{ClientState, ConsensusState, Header};
use bob_verifier::FINALIZATION_PERIOD_SECONDS;
use cosmwasm_std::{Addr, Empty};
use ethereum_light_client::client::EthereumLightClient;
use ethereum_light_client_types::StorageProof;
use ibc_union_light_client::{
    spec::Timestamp, ClientCreationResult, IbcClient, IbcClientCtx, IbcClientError, StateUpdate,
};
use ibc_union_msg::lightclient::Status;
use unionlabs::encoding::Bincode;

use crate::errors::Error;

pub struct BobLightClient;

impl IbcClient for BobLightClient {
    type Error = Error;

    type Header = Header;

    type Misbehaviour = ();

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type Encoding = Bincode;

    type CustomQuery = Empty;

    type StorageProof = StorageProof;

    fn verify_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;
        ethereum_light_client::client::verify_membership(
            key,
            consensus_state.ibc_storage_root,
            storage_proof,
            value,
        )
        .map_err(Into::<Error>::into)?;
        Ok(())
    }

    fn verify_non_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;
        ethereum_light_client::client::verify_non_membership(
            key,
            consensus_state.ibc_storage_root,
            storage_proof,
        )
        .map_err(Into::<Error>::into)?;
        Ok(())
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> Timestamp {
        consensus_state.timestamp
    }

    fn get_latest_height(ClientState::V1(client_state): &Self::ClientState) -> u64 {
        client_state.latest_height
    }

    fn get_counterparty_chain_id(ClientState::V1(client_state): &Self::ClientState) -> String {
        client_state.chain_id.to_string()
    }

    fn status(ctx: IbcClientCtx<Self>, client_state: &Self::ClientState) -> Status {
        let _ = client_state;
        let _ = ctx;
        // FIXME: expose the ctx to this call to allow threading this call to L1
        // client. generally, we want to thread if a client is an L2 so always
        // provide the ctx?
        Status::Active
    }

    fn verify_creation(
        _caller: Addr,
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
        _relayer: Addr,
    ) -> Result<ClientCreationResult<Self>, IbcClientError<Self>> {
        Ok(ClientCreationResult::new())
    }

    fn verify_header(
        ctx: IbcClientCtx<Self>,
        _caller: Addr,
        header: Self::Header,
        _relayer: Addr,
    ) -> Result<StateUpdate<Self>, IbcClientError<Self>> {
        let ClientState::V1(mut client_state) = ctx.read_self_client_state()?;

        let l1_consensus_state = ctx
            .read_consensus_state::<EthereumLightClient>(
                client_state.l1_client_id,
                header.l1_height,
            )
            .map_err(Into::<Error>::into)?;

        bob_verifier::verify_header(
            &client_state,
            &header,
            l1_consensus_state.state_root,
            ctx.env.block.time.seconds(),
            FINALIZATION_PERIOD_SECONDS,
        )
        .map_err(Into::<Error>::into)?;

        let update_height = header.l2_header.number.try_into().expect("impossible");

        let consensus_state = ConsensusState {
            timestamp: Timestamp::from_nanos(header.l2_header.timestamp),
            state_root: header.l2_header.state_root,
            ibc_storage_root: header.l2_ibc_account_proof.storage_root,
        };

        let state_update = StateUpdate::new(update_height, consensus_state);

        if client_state.latest_height < update_height {
            client_state.latest_height = update_height;
            Ok(state_update.overwrite_client_state(ClientState::V1(client_state)))
        } else {
            Ok(state_update)
        }
    }

    fn misbehaviour(
        _ctx: IbcClientCtx<Self>,
        _caller: Addr,
        _misbehaviour: Self::Misbehaviour,
        _relayer: Addr,
    ) -> Result<Self::ClientState, IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }
}
