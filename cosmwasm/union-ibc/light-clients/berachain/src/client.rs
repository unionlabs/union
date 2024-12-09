use beacon_api_types::{ExecutionPayloadHeaderSsz, Mainnet};
use berachain_light_client_types::{ClientState, ConsensusState, Header};
use cosmwasm_std::Empty;
use ics23::ibc_api::SDK_SPECS;
use tendermint_verifier::types::SignatureVerifier;
use union_ibc_light_client::IbcClient;
use union_ibc_msg::lightclient::Status;
use unionlabs::{
    berachain::LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX,
    encoding::{Bincode, EncodeAs, Ssz},
    ibc::core::commitment::{merkle_proof::MerkleProof, merkle_root::MerkleRoot},
};

use crate::{errors::Error, verifier::Bls12Verifier};

const WASM_PREFIX: &[u8] = b"wasm";
const WASM_STORAGE_PREFIX: u8 = 0x03;

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

    type StorageProof = MerkleProof;

    fn verify_membership(
        ctx: union_ibc_light_client::IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), union_ibc_light_client::IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(height)?;
        Ok(ics23::ibc_api::verify_membership(
            &storage_proof,
            &SDK_SPECS,
            &consensus_state.comet.root,
            &[
                WASM_PREFIX.to_vec(),
                [
                    vec![WASM_STORAGE_PREFIX],
                    client_state.ibc_contract_address.get().to_vec(),
                    key,
                ]
                .concat(),
            ],
            value,
        )
        .map_err(Into::<Error>::into)?)
    }

    fn verify_non_membership(
        ctx: union_ibc_light_client::IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), union_ibc_light_client::IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(height)?;
        Ok(ics23::ibc_api::verify_non_membership(
            &storage_proof,
            &SDK_SPECS,
            &consensus_state.comet.root,
            &[
                WASM_PREFIX.to_vec(),
                [
                    vec![WASM_STORAGE_PREFIX],
                    client_state.ibc_contract_address.get().to_vec(),
                    key,
                ]
                .concat(),
            ],
        )
        .map_err(Into::<Error>::into)?)
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64 {
        consensus_state.timestamp
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        client_state.execution_latest_height
    }

    fn status(client_state: &Self::ClientState) -> Status {
        if client_state
            .comet
            .frozen_height
            .unwrap_or_default()
            .height()
            != 0
        {
            Status::Frozen
        } else {
            Status::Active
        }
    }

    fn verify_creation(
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
    ) -> Result<(), union_ibc_light_client::IbcClientError<Self>> {
        Ok(())
    }

    // TODO: rearrange to avoid the clones
    fn verify_header(
        ctx: union_ibc_light_client::IbcClientCtx<Self>,
        header: Self::Header,
    ) -> Result<
        (u64, Self::ClientState, Self::ConsensusState),
        union_ibc_light_client::IbcClientError<Self>,
    > {
        let mut client_state = ctx.read_self_client_state()?;
        let consensus_state =
            ctx.read_self_consensus_state(header.cometbft_header.trusted_height.height())?;

        // 1. verify that the cometbft consensus transition is correct
        let (_, next_comet_client_state, new_comet_consensus_state) =
            tendermint_light_client::client::verify_header(
                client_state.comet.clone(),
                consensus_state.comet,
                header.cometbft_header.clone(),
                ctx.env.block.time,
                &SignatureVerifier::new(Bls12Verifier { deps: ctx.deps }),
            )
            .map_err(Into::<Error>::into)?;

        // 2. verify that the evm execution header is part of the cometbft consensus state
        ics23::ibc_api::verify_membership(
            &header.execution_header_proof,
            &client_state.comet.proof_specs,
            &MerkleRoot {
                hash: (*header.cometbft_header.signed_header.header.app_hash.get()).into(),
            },
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

        // Create/update consensus/execution states
        let update_height = header.execution_header.block_number;
        if client_state.execution_latest_height < update_height {
            client_state.execution_latest_height = update_height;
        }
        client_state.comet = next_comet_client_state;
        let new_consensus_state = ConsensusState {
            comet: new_comet_consensus_state,
            timestamp: header.execution_header.timestamp,
            storage_root: header.account_proof.storage_root,
        };

        Ok((update_height, client_state, new_consensus_state))
    }

    fn misbehaviour(
        _ctx: union_ibc_light_client::IbcClientCtx<Self>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<Self::ClientState, union_ibc_light_client::IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }
}
