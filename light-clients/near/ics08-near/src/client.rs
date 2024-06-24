use cosmwasm_std::{Deps, Empty};
use ics008_wasm_client::{
    storage_utils::{read_client_state, read_consensus_state},
    IbcClient,
};
use near_primitives_core::hash::CryptoHash;
use near_verifier::state_proof::RawStateProof;
use unionlabs::{
    encoding::Proto,
    ibc::{
        core::client::height::Height,
        lightclients::{
            near::{
                client_state::ClientState, consensus_state::ConsensusState, header::Header,
                validator_stake::ValidatorStakeView,
            },
            wasm,
        },
    },
};

use crate::{errors::Error, state::EPOCH_BLOCK_PRODUCERS_MAP};

pub type WasmClientState = wasm::client_state::ClientState<ClientState>;
pub type WasmConsensusState = wasm::consensus_state::ConsensusState<ConsensusState>;

pub struct NearLightClient;

impl IbcClient for NearLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type Encoding = Proto;

    fn verify_membership(
        deps: Deps<Self::CustomQuery>,
        height: Height,
        _delay_time_period: u64,
        _delay_block_period: u64,
        proof: Vec<u8>,
        path: unionlabs::ibc::core::commitment::merkle_path::MerklePath,
        value: ics008_wasm_client::StorageState,
    ) -> Result<(), ics008_wasm_client::IbcClientError<Self>> {
        let proof: RawStateProof = serde_json_wasm::from_slice(&proof).unwrap();
        let consensus_state: WasmConsensusState = read_consensus_state(deps, &height)?
            .ok_or(Error::ConsensusStateNotFound(height.revision_height))?;
        let client_state: WasmClientState = read_client_state(deps)?;
        let key = path.key_path.last().unwrap();
        match value {
            ics008_wasm_client::StorageState::Occupied(value) => near_verifier::verify_state(
                proof,
                &consensus_state.data.chunk_prev_state_root,
                &client_state.data.ibc_account_id,
                key.as_bytes(),
                Some(&value),
            ),
            ics008_wasm_client::StorageState::Empty => near_verifier::verify_state(
                proof,
                &consensus_state.data.chunk_prev_state_root,
                &client_state.data.ibc_account_id,
                key.as_bytes(),
                None,
            ),
        }
        .map_err(Into::<Error>::into)?;

        Ok(())
    }

    fn verify_header(
        deps: Deps<Self::CustomQuery>,
        _env: cosmwasm_std::Env,
        header: Self::Header,
    ) -> Result<(), ics008_wasm_client::IbcClientError<Self>> {
        let wasm_consensus_state = read_consensus_state(deps, &height(header.trusted_height))?
            .ok_or(Error::ConsensusStateNotFound(header.trusted_height))?;

        near_verifier::verify_header(
            &NearVerifierCtx { deps },
            wasm_consensus_state.data.state.clone(),
            header.new_state.clone(),
        )
        .map_err(Into::<Error>::into)?;

        // verify the `prev_state_root` of the chunk that contains the light client against the merkle root of the `prev_state_root`s of all chunks
        near_verifier::verify_path(
            header.new_state.inner_lite.prev_state_root,
            &header.prev_state_root_proof,
            header.prev_state_root,
        )
        .map_err(Into::<Error>::into)?;

        Ok(())
    }

    fn verify_misbehaviour(
        deps: Deps<Self::CustomQuery>,
        env: cosmwasm_std::Env,
        misbehaviour: Self::Misbehaviour,
    ) -> Result<(), ics008_wasm_client::IbcClientError<Self>> {
        unimplemented!()
    }

    fn update_state(
        deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
        env: cosmwasm_std::Env,
        header: Self::Header,
    ) -> Result<Vec<Height>, ics008_wasm_client::IbcClientError<Self>> {
        todo!()
    }

    fn update_state_on_misbehaviour(
        deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
        env: cosmwasm_std::Env,
        client_message: Vec<u8>,
    ) -> Result<(), ics008_wasm_client::IbcClientError<Self>> {
        todo!()
    }

    fn check_for_misbehaviour_on_header(
        deps: Deps<Self::CustomQuery>,
        header: Self::Header,
    ) -> Result<bool, ics008_wasm_client::IbcClientError<Self>> {
        todo!()
    }

    fn check_for_misbehaviour_on_misbehaviour(
        deps: Deps<Self::CustomQuery>,
        misbehaviour: Self::Misbehaviour,
    ) -> Result<bool, ics008_wasm_client::IbcClientError<Self>> {
        todo!()
    }

    fn verify_upgrade_and_update_state(
        deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
        upgrade_client_state: Self::ClientState,
        upgrade_consensus_state: Self::ConsensusState,
        proof_upgrade_client: Vec<u8>,
        proof_upgrade_consensus_state: Vec<u8>,
    ) -> Result<(), ics008_wasm_client::IbcClientError<Self>> {
        todo!()
    }

    fn migrate_client_store(
        deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
    ) -> Result<(), ics008_wasm_client::IbcClientError<Self>> {
        todo!()
    }

    fn status(
        deps: Deps<Self::CustomQuery>,
        env: &cosmwasm_std::Env,
    ) -> Result<ics008_wasm_client::Status, ics008_wasm_client::IbcClientError<Self>> {
        todo!()
    }

    fn export_metadata(
        deps: Deps<Self::CustomQuery>,
        env: &cosmwasm_std::Env,
    ) -> Result<
        Vec<unionlabs::ibc::core::client::genesis_metadata::GenesisMetadata>,
        ics008_wasm_client::IbcClientError<Self>,
    > {
        todo!()
    }

    fn timestamp_at_height(
        deps: Deps<Self::CustomQuery>,
        height: Height,
    ) -> Result<u64, ics008_wasm_client::IbcClientError<Self>> {
        todo!()
    }
}

pub struct NearVerifierCtx<'a> {
    deps: Deps<'a>,
}

impl<'a> near_verifier::NearVerifierCtx for NearVerifierCtx<'a> {
    fn get_epoch_block_producers(&self, epoch_id: CryptoHash) -> Option<Vec<ValidatorStakeView>> {
        match EPOCH_BLOCK_PRODUCERS_MAP.load(self.deps.storage, epoch_id.0) {
            Ok(bp) => Some(bp),
            Err(_) => None,
        }
    }

    fn ed25519_verify(
        &self,
        public_key: &[u8],
        signature: &[u8],
        message: &[u8],
    ) -> Result<(), near_verifier::error::Error> {
        match self.deps.api.ed25519_verify(message, signature, public_key) {
            Ok(true) => Ok(()),
            _ => Err(near_verifier::error::Error::VerificationFailure(
                public_key.into(),
                signature.into(),
                message.into(),
            )),
        }
    }
}

fn height(height: u64) -> Height {
    Height {
        revision_number: 0,
        revision_height: height,
    }
}
