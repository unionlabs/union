use cosmwasm_std::{Addr, Empty};
use ibc_union_light_client::{
    ClientCreationResult, IbcClient, IbcClientCtx, IbcClientError, StateUpdate, client_impl,
};
use ibc_union_msg::lightclient::{
    VerificationQueryMsg, VerifyCreationResponseEvent, VerifyMembershipQuery,
};
use ibc_union_spec::{
    Status, Timestamp,
    path::{ConsensusStatePath, MembershipProofPath, NON_MEMBERSHIP_COMMITMENT_VALUE},
};
use proof_lens_light_client_types::{ClientState, ConsensusState, Header};
use unionlabs::{encoding::Bincode, ethereum::keccak256};

use crate::{errors::Error, raw_bytes::RawBytes};

pub struct ProofLensLightClient;

impl IbcClient for ProofLensLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type StorageProof = RawBytes;

    type Encoding = Bincode;

    fn verify_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        RawBytes(proof): Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;
        let client_state = ctx.read_self_client_state()?;

        let client_impl =
            client_impl(&*ctx.deps.querier, &ctx.ibc_host, client_state.l1_client_id)?;

        ctx.deps.querier.query_wasm_smart::<()>(
            &client_impl,
            &VerificationQueryMsg::VerifyMembership(VerifyMembershipQuery {
                client_id: client_state.l1_client_id,
                height: consensus_state.l1_height,
                proof,
                path: key.into(),
                value: MembershipProofPath {
                    client_id: client_state.l2_client_id,
                    proof_height: height,
                    path: keccak256(value).into(),
                }
                .key()
                .into(),
            }),
        )?;

        Ok(())
    }

    fn verify_non_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        RawBytes(proof): Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;
        let client_state = ctx.read_self_client_state()?;

        let client_impl =
            client_impl(&*ctx.deps.querier, &ctx.ibc_host, client_state.l1_client_id)?;

        ctx.deps.querier.query_wasm_smart::<()>(
            &client_impl,
            &VerificationQueryMsg::VerifyMembership(VerifyMembershipQuery {
                client_id: client_state.l1_client_id,
                height: consensus_state.l1_height,
                proof,
                path: key.into(),
                value: MembershipProofPath {
                    client_id: client_state.l2_client_id,
                    proof_height: height,
                    path: NON_MEMBERSHIP_COMMITMENT_VALUE.into(),
                }
                .key()
                .into(),
            }),
        )?;

        Ok(())
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> Timestamp {
        consensus_state.timestamp
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        client_state.l2_latest_height
    }

    fn get_counterparty_chain_id(client_state: &Self::ClientState) -> String {
        client_state.l2_chain_id.clone()
    }

    fn status(ctx: IbcClientCtx<Self>, client_state: &Self::ClientState) -> Status {
        let Ok(client_impl) =
            client_impl(&*ctx.deps.querier, &ctx.ibc_host, client_state.l1_client_id)
        else {
            return Status::Frozen;
        };

        ctx.deps
            .querier
            .query_wasm_smart::<Status>(
                &client_impl,
                &ibc_union_msg::query::QueryMsg::GetStatus {
                    client_id: client_state.l1_client_id,
                },
            )
            .unwrap_or(Status::Frozen)
    }

    fn verify_creation(
        _caller: Addr,
        client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
        _relayer: Addr,
    ) -> Result<ClientCreationResult<Self>, IbcClientError<ProofLensLightClient>> {
        Ok(
            ClientCreationResult::new().add_event(VerifyCreationResponseEvent::CreateLensClient {
                l1_client_id: client_state.l1_client_id,
                l2_client_id: client_state.l2_client_id,
                l2_chain_id: client_state.l2_chain_id.clone(),
            }),
        )
    }

    fn verify_header(
        ctx: IbcClientCtx<Self>,
        _caller: Addr,
        header: Self::Header,
        _relayer: Addr,
    ) -> Result<StateUpdate<Self>, IbcClientError<Self>> {
        let mut client_state = ctx.read_self_client_state()?;
        let mut consensus_state = ctx.read_self_consensus_state(header.l2_height)?;

        let client_impl =
            client_impl(&*ctx.deps.querier, &ctx.ibc_host, client_state.l1_client_id)?;

        ctx.deps.querier.query_wasm_smart::<()>(
            &client_impl,
            &VerificationQueryMsg::VerifyMembership(VerifyMembershipQuery {
                client_id: client_state.l1_client_id,
                height: header.l1_height,
                proof: header.l2_consensus_state_proof,
                path: ConsensusStatePath {
                    client_id: client_state.l2_client_id,
                    height: header.l1_height,
                }
                .key()
                .into(),
                value: keccak256(&header.l2_consensus_state).into(),
            }),
        )?;

        consensus_state.l1_height = header.l1_height;
        consensus_state.timestamp = extract_timestamp(
            &header.l2_consensus_state,
            client_state.timestamp_offset.into(),
        );
        consensus_state.raw_l2_consensus_state = header.l2_consensus_state;

        let mut state_update = StateUpdate::new(header.l2_height, consensus_state);

        if header.l2_height > client_state.l2_latest_height {
            client_state.l2_latest_height = header.l2_height;
            state_update = state_update.overwrite_client_state(client_state);
        }

        Ok(state_update)
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

fn extract_timestamp(data: &[u8], offset: usize) -> Timestamp {
    Timestamp::from_nanos(u64::from_be_bytes(
        data[offset..offset + 8]
            .try_into()
            .expect("configure the client correctly"),
    ))
}
