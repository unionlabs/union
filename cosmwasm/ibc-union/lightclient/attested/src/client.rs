use attested_light_client_types::{ClientState, ConsensusState, Header, StorageProof};
use cosmwasm_std::{ensure, Addr, Deps, Empty};
use depolama::StorageExt;
use ibc_union_light_client::{
    spec::{Status, Timestamp},
    ClientCreationResult, IbcClient, IbcClientCtx, IbcClientError, StateUpdate,
};
use unionlabs::{encoding::Bincode, primitives::Bytes};

use crate::{
    errors::Error,
    state::{Attestations, HeightTimestamps},
    types::{AttestationKey, AttestationValue},
};

pub enum AttestedLightClient {}

impl IbcClient for AttestedLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    type Misbehaviour = ();

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type StorageProof = StorageProof;

    type Encoding = Bincode;

    fn verify_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        StorageProof {}: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        verify_attestation(
            ctx.deps,
            height,
            key.into(),
            AttestationValue::Existence(value.into()),
        )
        .map_err(Into::into)
    }

    fn verify_non_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        StorageProof {}: Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>> {
        verify_attestation(ctx.deps, height, key.into(), AttestationValue::NonExistence)
            .map_err(Into::into)
    }

    fn verify_header(
        ctx: IbcClientCtx<Self>,
        _caller: Addr,
        header: Self::Header,
        _relayer: Addr,
    ) -> Result<StateUpdate<Self>, IbcClientError<Self>> {
        verify_header(ctx.deps, ctx.read_self_client_state()?, header).map_err(Into::into)
    }

    fn misbehaviour(
        _ctx: IbcClientCtx<Self>,
        _caller: Addr,
        _misbehaviour: Self::Misbehaviour,
        _relayer: Addr,
    ) -> Result<Self::ClientState, IbcClientError<Self>> {
        Err(Error::NoMisbehaviourInAttestedClient.into())
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
    ) -> Result<ClientCreationResult<Self>, IbcClientError<AttestedLightClient>> {
        Ok(ClientCreationResult::new())
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> Timestamp {
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

pub fn verify_header(
    deps: Deps,
    client_state: ClientState,
    header: Header,
) -> Result<StateUpdate<AttestedLightClient>, Error> {
    let ClientState::V1(mut client_state) = client_state;

    let Header { height, timestamp } = header;

    let attested_timestamp = deps.storage.read::<HeightTimestamps>(&height)?;

    ensure!(
        attested_timestamp == timestamp,
        Error::InvalidTimestamp {
            height,
            attested_timestamp,
            timestamp
        }
    );

    let mut update = StateUpdate::new(height, ConsensusState { timestamp });

    if header.height > client_state.latest_height {
        client_state.latest_height = header.height;
        update = update.overwrite_client_state(ClientState::V1(client_state));
    }

    Ok(update)
}

pub fn verify_attestation(
    deps: Deps,
    height: u64,
    key: Bytes,
    value: AttestationValue,
) -> Result<(), Error> {
    use AttestationValue::*;

    let attested = deps
        .storage
        .maybe_read::<Attestations>(&AttestationKey {
            height,
            key: key.clone(),
        })?
        .ok_or_else(|| Error::AttestationNotFound {
            height,
            key: key.clone(),
        })?;

    match (attested, value) {
        // membership
        (Existence(attested), Existence(value)) => {
            ensure!(
                value == attested,
                Error::InvalidAttestedValue {
                    height,
                    key,
                    attested: Existence(attested),
                    value: Existence(value),
                }
            );

            Ok(())
        }

        // non-membership
        (NonExistence, NonExistence) => Ok(()),

        // invalid
        (attested @ Existence(_), value @ NonExistence)
        | (attested @ NonExistence, value @ Existence(_)) => Err(Error::InvalidAttestedValue {
            height,
            key,
            attested,
            value,
        }),
    }
}
