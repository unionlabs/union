use beacon_api_types::{chain_spec::Mainnet, deneb};
use berachain_light_client_types::{ClientState, ConsensusState, Header};
use cosmwasm_std::{Addr, Empty};
use ethereum_light_client_types::StorageProof;
use ibc_union_light_client::{
    spec::Timestamp, ClientCreationResult, IbcClient, IbcClientCtx, IbcClientError, StateUpdate,
};
use ibc_union_msg::lightclient::Status;
use tendermint_light_client::client::TendermintLightClient;
use unionlabs::{
    berachain::LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX,
    encoding::{Bincode, EncodeAs, Ssz},
};

use crate::errors::Error;

pub struct BerachainLightClient;

impl IbcClient for BerachainLightClient {
    type Error = Error;

    type Header = Header;

    // TODO(aeryz): Change this to appropriate misbehavior type when it is implemented
    type Misbehaviour = Header;

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
            consensus_state.storage_root,
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
            consensus_state.storage_root,
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

        // 1. extract L1 state
        let l1_client_state = ctx
            .read_client_state::<TendermintLightClient>(client_state.l1_client_id)
            .map_err(Into::<Error>::into)?;
        let l1_consensus_state = ctx
            .read_consensus_state::<TendermintLightClient>(
                client_state.l1_client_id,
                header.l1_height.height(),
            )
            .map_err(Into::<Error>::into)?;

        // 2. verify that the evm execution header is part of the cometbft consensus state
        ics23::ibc_api::verify_membership(
            &header.execution_header_proof,
            &l1_client_state.proof_specs,
            &l1_consensus_state.root.hash.into(),
            &[
                b"beacon".to_vec(),
                [LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX].to_vec(),
            ],
            deneb::ExecutionPayloadHeaderSsz::<Mainnet>::try_from(header.execution_header.clone())
                .map_err(Into::<Error>::into)?
                .encode_as::<Ssz>(),
        )
        .map_err(Into::<Error>::into)?;

        // 3. verify that the contract storage root is part of the evm execution header
        evm_storage_verifier::verify_account_storage_root(
            header.execution_header.state_root,
            &client_state.ibc_contract_address,
            &header.account_proof.proof,
            &header.account_proof.storage_root,
        )
        .map_err(Into::<Error>::into)?;

        // 4. update
        let update_height = header.execution_header.block_number;

        let consensus_state = ConsensusState {
            timestamp: Timestamp::from_secs(header.execution_header.timestamp),
            state_root: header.execution_header.state_root,
            storage_root: header.account_proof.storage_root,
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
