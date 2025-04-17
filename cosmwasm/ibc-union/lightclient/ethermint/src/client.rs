use cometbft_types::crypto::public_key::PublicKey;
use cosmwasm_std::{Addr, Empty};
use ethermint_light_client_types::ClientState;
use ibc_union_light_client::{
    spec::Timestamp, ClientCreationResult, IbcClient, IbcClientCtx, IbcClientError, StateUpdate,
};
use ibc_union_msg::lightclient::Status;
use ics23::ibc_api::SDK_SPECS;
use tendermint_light_client::verifier::Ed25519Verifier;
use tendermint_light_client_types::{ConsensusState, Header};
use tendermint_verifier::types::SignatureVerifier;
use unionlabs::{
    encoding::Bincode, ethereum::ibc_commitment_key,
    ibc::core::commitment::merkle_proof::MerkleProof,
};

use crate::errors::Error;

pub struct EthermintLightClient;

impl IbcClient for EthermintLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    // TODO(aeryz): Change this to appropriate misbehavior type when it is implemented
    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type StorageProof = MerkleProof;

    type Encoding = Bincode;

    fn verify_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(height)?;

        ics23::ibc_api::verify_membership(
            &storage_proof,
            &SDK_SPECS,
            &consensus_state.root,
            &[
                client_state.store_key.to_vec(),
                client_state
                    .key_prefix_storage
                    .into_iter()
                    .chain(client_state.ibc_contract_address)
                    .chain(
                        ibc_commitment_key(key.try_into().map_err(Error::InvalidKey)?)
                            .to_be_bytes(),
                    )
                    .collect::<Vec<_>>(),
            ],
            value,
        )
        .map_err(tendermint_light_client::errors::Error::VerifyMembership)
        .map_err(Error::from)?;

        Ok(())
    }

    fn verify_non_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(height)?;

        ics23::ibc_api::verify_non_membership(
            &storage_proof,
            &SDK_SPECS,
            &consensus_state.root,
            &[
                client_state.store_key.to_vec(),
                client_state
                    .key_prefix_storage
                    .into_iter()
                    .chain(
                        ibc_commitment_key(key.try_into().map_err(Error::InvalidKey)?)
                            .to_be_bytes(),
                    )
                    .collect::<Vec<_>>(),
            ],
        )
        .map_err(tendermint_light_client::errors::Error::VerifyMembership)
        .map_err(Error::from)?;

        Ok(())
    }

    fn verify_header(
        ctx: IbcClientCtx<Self>,
        _caller: Addr,
        header: Self::Header,
        _relayer: Addr,
    ) -> Result<StateUpdate<Self>, IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(header.trusted_height.height())?;
        match header.validator_set.validators.first().map(|v| &v.pub_key) {
            Some(PublicKey::Ed25519(_)) => {
                let StateUpdate {
                    height,
                    client_state: tendermint_client_state,
                    consensus_state,
                    ..
                } = tendermint_light_client::client::verify_header(
                    client_state.tendermint_client_state,
                    consensus_state,
                    header,
                    ctx.env.block.time,
                    &SignatureVerifier::new(Ed25519Verifier::new(ctx.deps)),
                )
                .map_err(Error::from)?;
                let state_update = StateUpdate::new(height, consensus_state);
                if let Some(tendermint_client_state) = tendermint_client_state {
                    Ok(state_update.overwrite_client_state(ClientState {
                        tendermint_client_state,
                        ..client_state
                    }))
                } else {
                    Ok(state_update)
                }
            }
            _ => {
                Err(Error::from(tendermint_light_client::errors::Error::InvalidValidatorSet).into())
            }
        }
    }

    fn misbehaviour(
        _ctx: IbcClientCtx<Self>,
        _caller: Addr,
        _misbehaviour: Self::Misbehaviour,
        _relayer: Addr,
    ) -> Result<Self::ClientState, IbcClientError<Self>> {
        Err(Error::from(tendermint_light_client::errors::Error::Unimplemented).into())
    }

    fn status(ctx: IbcClientCtx<Self>, client_state: &Self::ClientState) -> Status {
        let _ = ctx;

        // FIXME: read latest consensus to verify if client expired
        // if is_client_expired(
        //     &consensus_state.timestamp,
        //     client_state.trusting_period,
        //     env.block
        //         .time
        //         .try_into()
        //         .map_err(|_| Error::from(InvalidHostTimestamp(env.block.time)))?,
        // ) {
        //     return Ok(Status::Expired);
        // }
        if client_state
            .tendermint_client_state
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

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> Timestamp {
        Timestamp::from_nanos(consensus_state.timestamp.as_unix_nanos())
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        client_state.tendermint_client_state.latest_height.height()
    }

    fn get_counterparty_chain_id(client_state: &Self::ClientState) -> String {
        client_state.tendermint_client_state.chain_id.clone()
    }

    fn verify_creation(
        _caller: Addr,
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
        _relayer: Addr,
    ) -> Result<ClientCreationResult<Self>, IbcClientError<Self>> {
        Ok(ClientCreationResult::new())
    }
}
