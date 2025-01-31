use cometbls_light_client::client::CometblsLightClient;
use cosmwasm_std::Empty;
use ibc_union_light_client::IbcClient;
use ibc_union_msg::lightclient::Status;
use ibc_union_spec::ConsensusStatePath;
use movement_light_client_types::ConsensusState as L2ConsensusState;
use state_lens_ics23_smt_light_client_types::{ClientState, ConsensusState};
use state_lens_light_client_types::Header;
use unionlabs::{
    aptos::{account::AccountAddress, storage_proof::StorageProof},
    encoding::{Bincode, DecodeAs, EthAbi},
    ethereum::{ibc_commitment_key, keccak256},
    ibc::core::commitment::merkle_proof::MerkleProof,
    primitives::{H256, U256},
};

use crate::errors::Error;

pub struct StateLensIcs23SmtLightClient;

impl IbcClient for StateLensIcs23SmtLightClient {
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
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(height)?;
        verify_membership(
            &key,
            consensus_state.state_root,
            client_state.table_handle,
            storage_proof,
            &value,
        )
        .map_err(Into::into)
    }

    fn verify_non_membership(
        _ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _height: u64,
        _key: Vec<u8>,
        _storage_proof: Self::StorageProof,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        unimplemented!()
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

    fn status(_client_state: &Self::ClientState) -> Status {
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
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        Ok(())
    }

    fn verify_header(
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        header: Self::Header,
        _caller: cosmwasm_std::Addr,
    ) -> Result<
        (u64, Self::ClientState, Self::ConsensusState),
        ibc_union_light_client::IbcClientError<Self>,
    > {
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

        let l2_consensus_state = L2ConsensusState::decode_as::<EthAbi>(&header.l2_consensus_state)
            .map_err(|_| Error::L2ConsensusStateDecode(header.l2_consensus_state))?;

        if client_state.l2_latest_height < header.l2_height.height() {
            client_state.l2_latest_height = header.l2_height.height();
        }

        let consensus_state = ConsensusState {
            timestamp: l2_consensus_state.timestamp,
            state_root: l2_consensus_state.state_root,
        };

        Ok((header.l2_height.height(), client_state, consensus_state))
    }

    fn misbehaviour(
        _ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<Self::ClientState, ibc_union_light_client::IbcClientError<Self>> {
        unimplemented!()
    }
}

pub fn verify_membership(
    path: &[u8],
    state_root: H256,
    table_handle: AccountAddress,
    proof: StorageProof,
    value: &[u8],
) -> Result<(), Error> {
    let Some(proof_value) = &proof.state_value else {
        return Err(Error::MembershipProofWithoutValue);
    };

    // `aptos_std::table` stores the value as bcs encoded
    let given_value = bcs::to_bytes(&value).expect("cannot fail");
    if proof_value.data() != given_value {
        return Err(Error::ProofValueMismatch(
            proof_value.data().to_vec(),
            given_value,
        ));
    }

    let Some(proof_leaf) = proof.proof.leaf.as_ref() else {
        return Err(Error::MembershipProofWithoutValue);
    };

    if aptos_verifier::hash_state_value(proof_value) != *proof_leaf.value_hash.get() {
        return Err(Error::ProofValueHashMismatch);
    }

    let key =
        aptos_verifier::hash_table_key(&bcs::to_bytes(path).expect("cannot fail"), &table_handle);

    if key != *proof_leaf.key.get() {
        return Err(Error::ProofKeyMismatch);
    }

    Ok(aptos_verifier::verify_membership(
        proof.proof,
        state_root.into(),
    )?)
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
