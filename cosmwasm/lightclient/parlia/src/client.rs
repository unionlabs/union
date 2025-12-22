use cosmwasm_std::{Addr, Empty};
use ethereum_light_client_types::StorageProof;
use ibc_union_light_client::{
    ClientCreationResult, IbcClient, IbcClientCtx, IbcClientError, StateUpdate,
    spec::{Status, Timestamp},
};
use parlia_light_client_types::{ClientState, ClientStateV1, ConsensusState, Header, Misbehaviour};
use parlia_types::Valset;
use parlia_verifier::VerificationContext;
use unionlabs::{
    encoding::Bincode,
    primitives::{H384, H768},
};

use crate::{errors::Error, store::ValsetStore};

pub struct ParliaLightClient;

impl IbcClient for ParliaLightClient {
    type Error = Error;

    type Header = Header;

    type Misbehaviour = Misbehaviour;

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

    fn status(
        _ctx: IbcClientCtx<Self>,
        ClientState::V1(_client_state): &Self::ClientState,
    ) -> Status {
        // TODO: Re-enable these checks before we go to mainnet
        Status::Active
        // if client_state.frozen_height == 0 {
        //     let consensus_state = ctx
        //         .read_self_consensus_state(client_state.latest_height)
        //         .unwrap();

        //     if consensus_state
        //         .timestamp
        //         .plus_duration(client_state.unbond_period)
        //         .expect("should be ok")
        //         < Timestamp::from_nanos(ctx.env.block.time.nanos())
        //     {
        //         Status::Expired
        //     } else {
        //         Status::Active
        //     }
        // } else {
        //     Status::Frozen
        // }
    }

    fn verify_creation(
        _caller: Addr,
        client_state: &Self::ClientState,
        consensus_state: &Self::ConsensusState,
        _relayer: Addr,
    ) -> Result<ClientCreationResult<Self>, IbcClientError<Self>> {
        let ClientState::V1(mut client_state) = client_state.clone();

        let Some(initial_valset) = client_state.initial_valset.take() else {
            return Err(Error::NoInitialValset.into());
        };

        Ok(ClientCreationResult::new()
            .overwrite_client_state(ClientState::V1(client_state))
            .add_storage_write::<ValsetStore>(
                consensus_state.valset_epoch_block_number,
                initial_valset,
            ))
    }

    fn verify_header(
        ctx: IbcClientCtx<Self>,
        _caller: Addr,
        header: Self::Header,
        _relayer: Addr,
    ) -> Result<StateUpdate<Self>, IbcClientError<Self>> {
        let ClientState::V1(mut client_state) = ctx.read_self_client_state()?;

        // verify attestation
        let (verified_header, maybe_epoch_rotation_data) = parlia_verifier::verify_header(
            &header.chain,
            client_state.unbond_period,
            header.trusted_valset_epoch_number,
            CwContext { ctx: &ctx },
        )
        .map_err(Into::<Error>::into)?;

        // verify ibc storage root
        evm_storage_verifier::verify_account_storage_root(
            verified_header.state_root,
            &client_state.ibc_contract_address,
            header.ibc_account_proof.proof,
            &header.ibc_account_proof.storage_root,
        )
        .map_err(Into::<Error>::into)?;

        let update_height = verified_header.number.try_into().expect("impossible");

        let mut consensus_state = ConsensusState {
            valset_epoch_block_number: header.trusted_valset_epoch_number,
            timestamp: verified_header.full_timestamp(),
            state_root: verified_header.state_root,
            ibc_storage_root: header.ibc_account_proof.storage_root,
        };

        // valset rotated
        let state_update = if let Some((new_trusted_valset_block_number, new_trusted_valset)) =
            maybe_epoch_rotation_data
        {
            consensus_state.valset_epoch_block_number = new_trusted_valset_block_number;

            StateUpdate::new(update_height, consensus_state).add_storage_write::<ValsetStore>(
                new_trusted_valset_block_number,
                new_trusted_valset,
            )
        } else {
            StateUpdate::new(update_height, consensus_state)
        };

        if client_state.latest_height < update_height {
            client_state.latest_height = update_height;
            Ok(state_update.overwrite_client_state(ClientState::V1(client_state)))
        } else {
            Ok(state_update)
        }
    }

    fn misbehaviour(
        ctx: IbcClientCtx<Self>,
        _caller: Addr,
        misbehaviour: Self::Misbehaviour,
        _relayer: Addr,
    ) -> Result<Self::ClientState, IbcClientError<Self>> {
        let ClientState::V1(client_state) = ctx.read_self_client_state()?;

        if misbehaviour.attestation_1.number != misbehaviour.attestation_2.number {
            return Err(Error::MisbehaviourHeadersNotForSameHeight.into());
        }

        if misbehaviour.attestation_1 == misbehaviour.attestation_2 {
            return Err(Error::MisbehaviourHeadersMustBeDifferent.into());
        }

        let frozen_height = misbehaviour
            .source_1
            .number
            .try_into()
            .expect("checked in verify_header; qed;");

        let cw_ctx = CwContext { ctx: &ctx };

        // verify first attestation
        parlia_verifier::verify_header(
            &[
                misbehaviour.source_1,
                misbehaviour.target_1,
                misbehaviour.attestation_1,
            ],
            client_state.unbond_period,
            misbehaviour.trusted_valset_epoch_number,
            cw_ctx.clone(),
        )
        .map_err(Into::<Error>::into)?;

        // verify second attestation
        parlia_verifier::verify_header(
            &[
                misbehaviour.source_2,
                misbehaviour.target_2,
                misbehaviour.attestation_2,
            ],
            client_state.unbond_period,
            misbehaviour.trusted_valset_epoch_number,
            cw_ctx,
        )
        .map_err(Into::<Error>::into)?;

        Ok(ClientState::V1(ClientStateV1 {
            frozen_height,
            ..client_state
        }))
    }
}

#[derive(Clone)]
pub struct CwContext<'a> {
    ctx: &'a IbcClientCtx<'a, ParliaLightClient>,
}

impl VerificationContext for CwContext<'_> {
    type Error = CwContextError;

    fn current_timestamp(&self) -> Timestamp {
        Timestamp::from_nanos(self.ctx.env.block.time.nanos())
    }

    fn get_valset(&self, epoch_block_number: u64) -> Result<Valset, Self::Error> {
        self.ctx
            .read_self_storage::<ValsetStore>(epoch_block_number)
            .map_err(|e| CwContextError::GetValset(Box::new(e)))
    }

    fn verify<'pk>(
        &self,
        public_keys: impl IntoIterator<Item = &'pk H384>,
        msg: &[u8],
        signature: H768,
    ) -> Result<(), Self::Error> {
        const DST_POP_G2: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";

        let pubkey = self.ctx.deps.api.bls12_381_aggregate_g1(
            &public_keys
                .into_iter()
                .flatten()
                .copied()
                .collect::<Vec<_>>(),
        )?;

        let hashed_msg = self.ctx.deps.api.bls12_381_hash_to_g2(
            cosmwasm_std::HashFunction::Sha256,
            msg,
            DST_POP_G2,
        )?;

        let valid = self.ctx.deps.api.bls12_381_pairing_equality(
            &cosmwasm_std::BLS12_381_G1_GENERATOR,
            signature.as_ref(),
            &pubkey,
            &hashed_msg,
        )?;

        if valid {
            Ok(())
        } else {
            Err(CwContextError::InvalidSignature)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CwContextError {
    #[error("invalid signature")]
    InvalidSignature,
    #[error(transparent)]
    VerificationError(#[from] cosmwasm_std::VerificationError),
    // box needed for indirection
    #[error(transparent)]
    GetValset(#[from] Box<IbcClientError<ParliaLightClient>>),
}
