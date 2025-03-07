use cometbls_light_client::client::CometblsLightClient;
use cosmwasm_std::Empty;
use ethereum_light_client_types::StorageProof;
use ibc_union_light_client::{IbcClient, IbcClientCtx, IbcClientError};
use ibc_union_msg::lightclient::{Status, VerifyCreationResponseEvent};
use ibc_union_spec::path::ConsensusStatePath;
use state_lens_ics23_mpt_light_client_types::{ClientState, ConsensusState};
use state_lens_light_client_types::Header;
use unionlabs::{
    encoding::{Bincode, DecodeAs},
    ethereum::{ibc_commitment_key, keccak256},
    ibc::core::commitment::merkle_proof::MerkleProof,
    primitives::{H256, U256},
};

use crate::errors::Error;

pub struct StateLensIcs23MptLightClient;

impl IbcClient for StateLensIcs23MptLightClient {
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

        verify_membership(key, consensus_state.storage_root, storage_proof, value)?;

        Ok(())
    }

    fn verify_non_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;

        verify_non_membership(key, consensus_state.storage_root, storage_proof)?;

        Ok(())
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64 {
        consensus_state.timestamp
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        client_state.l2_latest_height
    }

    fn get_counterparty_chain_id(client_state: &Self::ClientState) -> String {
        client_state.l2_chain_id.clone()
    }

    fn status(ctx: IbcClientCtx<Self>, client_state: &Self::ClientState) -> Status {
        let _ = ctx;
        let _ = client_state;

        // FIXME: expose the ctx to this call to allow threading this call to L1
        // client. generally, we want to thread if a client is an L2 so always
        // provide the ctx?
        // let client_state: WasmClientState = read_client_state(deps)?;
        // let l1_client_state = query_client_state::<WasmL1ClientState>(
        //     deps,
        //     env,
        //     client_state.data.l1_client_id.clone(),
        // )
        // .map_err(Error::CustomQuery)?;

        // if l1_client_state.data.frozen_height != Height::default() {
        //     return Ok(Status::Frozen);
        // }

        // let Some(_) = read_consensus_state::<Self>(deps, &client_state.latest_height)? else {
        //     return Ok(Status::Expired);
        // };

        // Ok(Status::Active)
        Status::Active
    }

    fn verify_creation(
        client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
    ) -> Result<
        Option<Vec<VerifyCreationResponseEvent>>,
        IbcClientError<StateLensIcs23MptLightClient>,
    > {
        Ok(Some(vec![VerifyCreationResponseEvent::CreateLensClient {
            l1_client_id: client_state.l1_client_id,
            l2_client_id: client_state.l2_client_id,
            l2_chain_id: client_state.l2_chain_id.clone(),
        }]))
    }

    fn verify_header(
        ctx: IbcClientCtx<Self>,
        header: Self::Header,
        _caller: cosmwasm_std::Addr,
    ) -> Result<(u64, Self::ClientState, Self::ConsensusState), IbcClientError<Self>> {
        let mut client_state = ctx.read_self_client_state()?;

        let storage_proof = MerkleProof::decode_as::<Bincode>(&header.l2_consensus_state_proof)
            .map_err(|_| Error::ProofDecode(header.l2_consensus_state_proof))?;

        ctx.verify_membership::<CometblsLightClient>(
            client_state.l1_client_id,
            header.l1_height.height(),
            ConsensusStatePath {
                client_id: client_state.l2_client_id,
                height: header.l2_height.height(),
            }
            .key()
            .into_bytes(),
            storage_proof,
            keccak256(&header.l2_consensus_state).into(),
        )
        .map_err(Error::L1Error)?;

        let l2_timestamp = extract_uint64(
            &header.l2_consensus_state,
            client_state.extra.timestamp_offset as usize,
        );

        let l2_state_root = extract_bytes32(
            &header.l2_consensus_state,
            client_state.extra.state_root_offset as usize,
        );

        let l2_storage_root = extract_bytes32(
            &header.l2_consensus_state,
            client_state.extra.storage_root_offset as usize,
        );

        if client_state.l2_latest_height < header.l2_height.height() {
            client_state.l2_latest_height = header.l2_height.height();
        }

        let consensus_state = ConsensusState {
            timestamp: l2_timestamp,
            state_root: l2_state_root,
            storage_root: l2_storage_root,
        };

        Ok((header.l2_height.height(), client_state, consensus_state))
    }

    fn misbehaviour(
        _ctx: IbcClientCtx<Self>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<Self::ClientState, IbcClientError<Self>> {
        unimplemented!()
    }
}

fn extract_uint64(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .expect("impossible; qed"),
    )
}

fn extract_bytes32(data: &[u8], offset: usize) -> H256 {
    H256::new(
        data[offset..offset + 32]
            .try_into()
            .expect("impossible; qed"),
    )
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
        &storage_proof.proof,
    )
    .map_err(Error::VerifyStorageProof)
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
