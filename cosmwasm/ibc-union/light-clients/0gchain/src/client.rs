use cosmwasm_std::Empty;
use ibc_union_light_client::IbcClient;
use ibc_union_msg::lightclient::Status;
use ics23::ibc_api::SDK_SPECS;
use ogchain_light_client_types::ClientState;
use tendermint_light_client::{client::TendermintLightClient, verifier::Ed25519Verifier};
use tendermint_light_client_types::{ConsensusState, Header};
use tendermint_verifier::types::SignatureVerifier;
use unionlabs::{encoding::Bincode, ibc::core::commitment::merkle_proof::MerkleProof};

use crate::errors::Error;

pub struct OgchainLightClient;

impl IbcClient for OgchainLightClient {
    type Error = Error;

    type Header = Header;

    // TODO(aeryz): Change this to appropriate misbehavior type when it is implemented
    type Misbehaviour = ();

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type Encoding = Bincode;

    type CustomQuery = Empty;

    type StorageProof = MerkleProof;

    fn verify_membership(
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(height)?;

        Ok(ics23::ibc_api::verify_membership(
            &storage_proof,
            &SDK_SPECS,
            &consensus_state.root,
            &[
                b"evm".to_vec(),
                0x2u8
                    .to_le_bytes()
                    .into_iter()
                    .chain(client_state.ibc_contract_address)
                    .chain(key)
                    .collect::<Vec<_>>(),
            ],
            value,
        )
        .map_err(Error::VerifyMembership)?)
    }

    fn verify_non_membership(
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(height)?;

        Ok(ics23::ibc_api::verify_non_membership(
            &storage_proof,
            &SDK_SPECS,
            &consensus_state.root,
            &[
                b"evm".to_vec(),
                0x2u8
                    .to_le_bytes()
                    .into_iter()
                    .chain(client_state.ibc_contract_address)
                    .chain(key)
                    .collect::<Vec<_>>(),
            ],
        )
        .map_err(Error::VerifyMembership)?)
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64 {
        TendermintLightClient::get_timestamp(consensus_state)
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
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        Ok(())
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
        let consensus_state = ctx.read_self_consensus_state(header.trusted_height.height())?;
        let (update_height, consensus_state) = tendermint_light_client::client::verify_header(
            client_state.chain_id.clone(),
            client_state.trusting_period,
            client_state.max_clock_drift,
            client_state.trust_level,
            consensus_state,
            header,
            ctx.env.block.time,
            &SignatureVerifier::new(Ed25519Verifier::new(ctx.deps)),
        )
        .map_err(Error::TmLightClient)?;

        if client_state.latest_height < update_height {
            client_state.latest_height = update_height;
        }

        Ok((update_height, client_state, consensus_state))
    }

    fn misbehaviour(
        _ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<Self::ClientState, ibc_union_light_client::IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }
}
