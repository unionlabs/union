use cosmwasm_std::{Addr, Empty};
use ethereum_light_client_types::StorageProof;
use ibc_union_light_client::{
    ClientCreationResult, IbcClient, IbcClientCtx, IbcClientError, StateUpdate,
};
use ibc_union_msg::lightclient::Status;
use trusted_mpt_light_client_types::{ClientState, ConsensusState, Header};
use unionlabs::{
    encoding::Bincode,
    ethereum::ibc_commitment_key,
    primitives::{H256, U256},
};

use crate::errors::Error;

pub enum MptTrustedLightClient {}

impl IbcClient for MptTrustedLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type StorageProof = StorageProof;

    type Encoding = Bincode;

    fn verify_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;

        Ok(verify_membership(
            key,
            consensus_state.storage_root,
            storage_proof,
            value,
        )?)
    }

    fn verify_non_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;
        Ok(verify_non_membership(
            key,
            consensus_state.storage_root,
            storage_proof,
        )?)
    }

    fn verify_header(
        ctx: IbcClientCtx<Self>,
        caller: Addr,
        header: Self::Header,
        _relayer: Addr,
    ) -> Result<StateUpdate<Self>, IbcClientError<Self>> {
        let ClientState::V1(mut client_state) = ctx.read_self_client_state()?;
        if !client_state
            .whitelisted_relayers
            .contains(&caller.to_string())
        {
            return Err(Error::Unauthorized(caller).into());
        }

        // We still verify the account storage root since we only trust `state_root`
        evm_storage_verifier::verify_account_storage_root(
            header.state_root,
            &client_state.ibc_contract_address,
            &header.ibc_account_proof.proof,
            &header.ibc_account_proof.storage_root,
        )
        .map_err(Error::InvalidContractAddressProof)?;

        let mut update = StateUpdate::new(
            header.height,
            ConsensusState {
                state_root: header.state_root,
                storage_root: header.ibc_account_proof.storage_root,
                timestamp: header.timestamp,
            },
        );

        if header.height > client_state.latest_height {
            client_state.latest_height = header.height;
            update = update.overwrite_client_state(ClientState::V1(client_state));
        }

        Ok(update)
    }

    fn misbehaviour(
        _ctx: IbcClientCtx<Self>,
        _caller: Addr,
        _misbehaviour: Self::Misbehaviour,
        _relayer: Addr,
    ) -> Result<Self::ClientState, IbcClientError<Self>> {
        Err(Error::NoMisbehaviourInTrustedClient.into())
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
    ) -> Result<ClientCreationResult<Self>, IbcClientError<MptTrustedLightClient>> {
        Ok(ClientCreationResult::new())
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64 {
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

pub fn verify_membership(
    key: Vec<u8>,
    storage_root: H256,
    storage_proof: StorageProof,
    value: Vec<u8>,
) -> Result<(), Error> {
    check_commitment_key(
        H256::try_from(&key).map_err(|_| Error::InvalidCommitmentKeyLength(key))?,
        storage_proof.key,
    )?;

    let value = H256::try_from(&value).map_err(|_| Error::InvalidCommitmentValueLength(value))?;

    let proof_value = H256::from(storage_proof.value.to_be_bytes());

    if value != proof_value {
        return Err(Error::StoredValueMismatch {
            expected: value,
            stored: proof_value,
        });
    }

    evm_storage_verifier::verify_storage_proof(
        storage_root,
        storage_proof.key,
        &rlp::encode(&storage_proof.value),
        storage_proof.proof,
    )
    .map_err(Error::VerifyStorageProof)
}

/// Verifies that no value is committed at `path` in the counterparty light client's storage.
pub fn verify_non_membership(
    key: Vec<u8>,
    storage_root: H256,
    storage_proof: StorageProof,
) -> Result<(), Error> {
    check_commitment_key(
        H256::try_from(&key).map_err(|_| Error::InvalidCommitmentKeyLength(key))?,
        storage_proof.key,
    )?;

    if evm_storage_verifier::verify_storage_absence(
        storage_root,
        storage_proof.key,
        &storage_proof.proof,
    )
    .map_err(Error::VerifyStorageAbsence)?
    {
        Ok(())
    } else {
        Err(Error::CounterpartyStorageNotNil)
    }
}
pub fn check_commitment_key(path: H256, key: U256) -> Result<(), Error> {
    let expected_commitment_key = ibc_commitment_key(path);

    if expected_commitment_key != key {
        Err(Error::InvalidCommitmentKey {
            expected: expected_commitment_key,
            found: key,
        })
    } else {
        Ok(())
    }
}
