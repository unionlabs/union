use beacon_api_types::{ExecutionPayloadHeaderSsz, Mainnet};
use berachain_light_client_types::{ClientState, ConsensusState, Header};
use cosmwasm_std::Empty;
use ethereum_light_client_types::StorageProof;
use ibc_union_light_client::IbcClient;
use ibc_union_msg::lightclient::{Status, VerifyCreationResponseEvent};
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
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
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
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;
        ethereum_light_client::client::verify_non_membership(
            key,
            consensus_state.storage_root,
            storage_proof,
        )
        .map_err(Into::<Error>::into)?;
        Ok(())
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64 {
        consensus_state.timestamp
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        client_state.latest_height
    }

    fn get_counterparty_chain_id(client_state: &Self::ClientState) -> String {
        client_state.chain_id.to_string()
    }

    fn status(_client_state: &Self::ClientState) -> Status {
        // FIXME: expose the ctx to this call to allow threading this call to L1
        // client. generally, we want to thread if a client is an L2 so always
        // provide the ctx?
        Status::Active
    }

    fn verify_creation(
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
    ) -> Result<
        Option<Vec<VerifyCreationResponseEvent>>,
        ibc_union_light_client::IbcClientError<Self>,
    > {
        Ok(None)
    }

    // TODO: rearrange to avoid the clones
    fn verify_header(
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        header: Self::Header,
        _caller: cosmwasm_std::Addr,
    ) -> Result<
        (u64, Self::ClientState, Self::ConsensusState),
        ibc_union_light_client::IbcClientError<Self>,
    > {
        let mut client_state = ctx.read_self_client_state()?;

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
            ExecutionPayloadHeaderSsz::<Mainnet>::try_from(header.execution_header.clone())
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
        if client_state.latest_height < update_height {
            client_state.latest_height = update_height;
        }
        let new_consensus_state = ConsensusState {
            timestamp: header.execution_header.timestamp,
            state_root: header.execution_header.state_root,
            storage_root: header.account_proof.storage_root,
        };

        Ok((update_height, client_state, new_consensus_state))
    }

    fn misbehaviour(
        _ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<Self::ClientState, ibc_union_light_client::IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }
}
