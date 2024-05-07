use core::fmt::Debug;

use ethereum_verifier::{verify_account_storage_root, verify_storage_proof};
use gnark_mimc::new_mimc_constants_bls12_377;
use sha3::Digest;
use unionlabs::{
    hash::H256,
    ibc::lightclients::linea::{client_state::ClientState, header::Header},
    linea::account::ZkAccount,
    uint::U256,
};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("invalid rollup contract proof {0}")]
    InvalidRollupContractProof(ethereum_verifier::Error),
    #[error("invalid l2 block number proof {0}")]
    InvalidL2BlockNumberProof(ethereum_verifier::Error),
    #[error("invalid l2 timestamp proof {0}")]
    InvalidL2TimestampProof(ethereum_verifier::Error),
    #[error("invalid l2 state root {0}")]
    InvalidL2StateRootProof(ethereum_verifier::Error),
    #[error("invalid l2 ibc contract proof {0}")]
    InvalidL2IbcContractProof(linea_zktrie::verify::Error),
    #[error("the l2 ibc contract proof must be an inclusion proof, verify the address?")]
    L2IbcContractProofIsNotInclusion,
    #[error("node value mismatch")]
    ValueMismatch,
}

/*
1. assert rootHash(rollup) in l1StateRoot
2. assert rollup.currentL2BlockNumber = l2BlockNumber
3. assert rollup.currentL2Timestamp = l2Timestamp
4. assert rollup.stateRootHashes[l2BlockNumber] = l2StateRoot
5. assert rootHash(l2IbcContract) in l2StateRoot
 */
pub fn verify_header(
    client_state: ClientState,
    header: Header,
    l1_state_root: H256,
) -> Result<(), Error> {
    // 1.
    verify_account_storage_root(
        l1_state_root,
        &client_state.l1_rollup_contract_address,
        &header.l1_rollup_contract_proof.proof,
        &header.l1_rollup_contract_proof.storage_root,
    )
    .map_err(Error::InvalidRollupContractProof)?;

    // 2.
    verify_storage_proof(
        header.l1_rollup_contract_proof.storage_root,
        client_state.l1_rollup_current_l2_block_number_slot,
        &rlp::encode(&header.l2_block_number),
        &header.l2_block_number_proof.proofs[0].proof,
    )
    .map_err(Error::InvalidL2BlockNumberProof)?;

    // 3.
    verify_storage_proof(
        header.l1_rollup_contract_proof.storage_root,
        client_state.l1_rollup_current_l2_timestamp_slot,
        &rlp::encode(&header.l2_timestamp),
        &header.l2_timestamp_proof.proofs[0].proof,
    )
    .map_err(Error::InvalidL2TimestampProof)?;

    // 4.
    verify_storage_proof(
        header.l1_rollup_contract_proof.storage_root,
        state_root_hashes_mapping_key(
            client_state.l1_rollup_l2_state_root_hashes_slot,
            header.l2_block_number.into(),
        ),
        &rlp::encode(&U256::from_be_bytes(header.l2_state_root.into())),
        &header.l2_state_root_proof.proofs[0].proof,
    )
    .map_err(Error::InvalidL2StateRootProof)?;

    // 5.
    // TODO: perhaps force the proof to be an actual inclusion proof off-chain
    let account = linea_zktrie::verify::verify::<ZkAccount>(
        &new_mimc_constants_bls12_377(),
        &header.l2_ibc_contract_proof,
        header.l2_state_root,
        client_state.l2_ibc_contract_address,
    )
    .map_err(Error::InvalidL2IbcContractProof)?;

    match account {
        Some(_) => Ok(()),
        None => Err(Error::L2IbcContractProofIsNotInclusion),
    }
}

pub fn state_root_hashes_mapping_key(slot: U256, l2_block_number: U256) -> U256 {
    U256::from_be_bytes(
        sha3::Keccak256::new()
            .chain_update(l2_block_number.to_be_bytes())
            .chain_update(slot.to_be_bytes())
            .finalize()
            .into(),
    )
}
