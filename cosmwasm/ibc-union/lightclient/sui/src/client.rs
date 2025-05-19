use cosmwasm_std::{Empty, StdError};
use depolama::{KeyCodec, Prefix, Store, ValueCodec};
use ibc_union_light_client::{
    spec::Timestamp, ClientCreationResult, IbcClient, IbcClientError, StateUpdate,
};
use ibc_union_msg::lightclient::Status;
use sui_light_client_types::{
    client_state::ClientState, committee::Committee, consensus_state::ConsensusState,
    header::Header, storage_proof::StorageProof, U64,
};
use unionlabs::encoding::{Bincode, DecodeAs, EncodeAs};

use crate::{error::Error, verifier::Verifier};

pub enum SuiLightClient {}

impl IbcClient for SuiLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type StorageProof = StorageProof;

    type Encoding = Bincode;

    fn verify_membership(
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        let ClientState::V1(client_state) = ctx.read_self_client_state()?;

        let consensus_state = ctx.read_self_consensus_state(height)?;

        sui_verifier::verify_membership(
            client_state.ibc_commitments_object_id,
            key.into(),
            value.into(),
            storage_proof.object,
            storage_proof.transaction_effects,
            storage_proof.checkpoint_contents,
            consensus_state.content_digest,
        )
        .map_err(Into::<Error>::into)?;

        Ok(())
    }

    fn verify_non_membership(
        _ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _height: u64,
        _key: Vec<u8>,
        _storage_proof: Self::StorageProof,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        Ok(())
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> Timestamp {
        Timestamp::from_nanos(consensus_state.timestamp)
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        let ClientState::V1(client_state) = client_state;
        client_state.latest_checkpoint
    }

    fn get_counterparty_chain_id(client_state: &Self::ClientState) -> String {
        let ClientState::V1(cs) = client_state;
        cs.chain_id.clone()
    }

    fn status(
        _ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _client_state: &Self::ClientState,
    ) -> Status {
        Status::Active
    }

    fn verify_creation(
        _caller: cosmwasm_std::Addr,
        client_state: &ClientState,
        _consensus_state: &ConsensusState,
        _relayer: cosmwasm_std::Addr,
    ) -> Result<ClientCreationResult<Self>, IbcClientError<Self>> {
        let ClientState::V1(client_state) = client_state;

        let Some(initial_committee) = &client_state.initial_committee else {
            return Err(Error::NoInitialCommittee.into());
        };

        let mut client_state = client_state.clone();
        client_state.initial_committee = None;

        Ok(ClientCreationResult::new()
            .overwrite_client_state(ClientState::V1(client_state))
            .add_storage_write::<CommitteeStore>(
                initial_committee.epoch.0,
                initial_committee.clone(),
            ))
    }

    fn verify_header(
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _caller: cosmwasm_std::Addr,
        header: Header,
        _relayer: cosmwasm_std::Addr,
    ) -> Result<StateUpdate<Self>, IbcClientError<Self>> {
        let ClientState::V1(mut client_state) = ctx.read_self_client_state()?;

        let committee = ctx.read_self_storage::<CommitteeStore>(header.checkpoint_summary.epoch)?;

        sui_verifier::verify_checkpoint(
            &committee,
            &header.checkpoint_summary,
            &header.sign_info,
            &Verifier { deps: ctx.deps },
        )
        .map_err(Into::<Error>::into)?;

        let consensus_state = ConsensusState {
            timestamp: header.checkpoint_summary.timestamp_ms * 1_000_000,
            content_digest: header.checkpoint_summary.content_digest,
        };

        let mut state_update =
            StateUpdate::new(header.checkpoint_summary.sequence_number, consensus_state);

        if let Some(epoch_ending) = header.checkpoint_summary.end_of_epoch_data {
            state_update = state_update.add_storage_write::<CommitteeStore>(
                header.checkpoint_summary.epoch + 1,
                Committee {
                    epoch: U64(header.checkpoint_summary.epoch + 1),
                    voting_rights: epoch_ending.next_epoch_committee,
                },
            );
        }

        if client_state.latest_checkpoint > header.checkpoint_summary.sequence_number {
            client_state.latest_checkpoint = header.checkpoint_summary.sequence_number;
            state_update = state_update.overwrite_client_state(ClientState::V1(client_state));
        }

        Ok(state_update)
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

pub enum CommitteeStore {}
impl Store for CommitteeStore {
    const PREFIX: Prefix = Prefix::new(b"committee");

    type Key = u64;
    type Value = Committee;
}

impl KeyCodec<u64> for CommitteeStore {
    fn encode_key(key: &u64) -> depolama::Bytes {
        key.to_be_bytes().into()
    }

    fn decode_key(raw: &depolama::Bytes) -> cosmwasm_std::StdResult<u64> {
        raw.try_into()
            .map_err(|_| {
                StdError::generic_err(format!(
                    "invalid key: expected {N} bytes, found {}: {raw}",
                    raw.len(),
                    N = u64::BITS / 8,
                ))
            })
            .map(u64::from_be_bytes)
    }
}

impl ValueCodec<Committee> for CommitteeStore {
    fn encode_value(value: &Committee) -> depolama::Bytes {
        value.encode_as::<Bincode>().into()
    }

    fn decode_value(raw: &depolama::Bytes) -> cosmwasm_std::StdResult<Committee> {
        Committee::decode_as::<Bincode>(raw).map_err(|e| {
            StdError::generic_err(format!("unable to decode {}: {e}", stringify!($ty)))
        })
    }
}
