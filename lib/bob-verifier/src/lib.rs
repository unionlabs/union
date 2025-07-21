use core::fmt::Debug;

use alloy_sol_types::SolValue;
use bob_light_client_types::{
    header::{L2Header, OutputRootProof},
    ClientStateV2, Header,
};
use evm_storage_verifier::{
    verify_account_code_hash, verify_account_storage_root, verify_storage_proof,
};
use unionlabs::{
    ethereum::{keccak256, slot::Slot},
    primitives::{encoding::HexPrefixed, ByteArrayExt, H160, H256, U256},
};

#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum Error {
    #[error("invalid l2 oracle account proof")]
    InvalidL2OracleAccountProof(#[source] evm_storage_verifier::error::Error),
    #[error("invalid output proposal storage proof")]
    InvalidOutputProposalStorageProof(#[source] evm_storage_verifier::error::Error),
    #[error("output root proof hash mismatch: actual={actual}, expected={expected}")]
    OutputRootHashMismatch { actual: H256, expected: H256 },
    #[error("invalid ibc contract account proof")]
    InvalidIbcContractProof(#[source] evm_storage_verifier::error::Error),
    #[error("invalid game code proof")]
    InvalidGameCodeProof(#[source] evm_storage_verifier::error::Error),
    #[error("invalid code size, expected a minimum of {expected_minimum}, got {actual}")]
    InvalidCodeSize {
        expected_minimum: usize,
        actual: usize,
    },
    #[error("invalid ibc contract account proof")]
    InvalidIbcContractStorageRoot,
    #[error("the l2 header is not finalized")]
    HeaderNotFinalized,
}

pub fn verify_header(
    client_state: &ClientStateV2,
    header: &Header,
    l1_state_root: H256,
) -> Result<(), Error> {
    // 1. Verify that the DisputeGameFactory is member of the L1 root.
    verify_account_storage_root(
        l1_state_root,
        &client_state.dispute_game_factory_address,
        &header.dispute_game_factory_account_proof.proof,
        &header.dispute_game_factory_account_proof.storage_root,
    )
    .map_err(Error::InvalidL2OracleAccountProof)?;

    let game_slot = compute_game_slot(
        client_state.dispute_game_factory_dispute_game_list_slot,
        header.game_index,
    );

    // 2. Verify game is included in the DisputeGameFactory.
    verify_storage_proof(
        header.dispute_game_factory_account_proof.storage_root,
        game_slot,
        &rlp::encode(&header.game_proof.value),
        &header.game_proof.proof,
    )
    .map_err(Error::InvalidOutputProposalStorageProof)?;

    let game_id = header.game_proof.value.to_be_bytes();
    // See https://github.com/ethereum-optimism/optimism/blob/4a7cb8a198a1f027e739d2e51dc170faf02b5d28/packages/contracts-bedrock/src/dispute/lib/LibUDT.sol#L70-L79
    let game_account_address = H160::<HexPrefixed>::new(game_id.array_slice::<12, 20>());
    let game_account_code_hash = keccak256(&header.game_account_code);

    // 3. Verify that the provided code is what's currently backing the game account.
    verify_account_code_hash(
        l1_state_root,
        &game_account_address,
        &header.game_account_proof.proof,
        &game_account_code_hash,
    )
    .map_err(Error::InvalidGameCodeProof)?;

    // 4. Verify that the provided l2 header hash matches the block hash within
    // the output root proof.
    verify_l2_header_is_related_to_output_root_proof(&header.output_root_proof, &header.l2_header)?;

    let minimum_code_size = client_state.fault_dispute_game_code_root_claim_index as usize
        + H256::<HexPrefixed>::BYTES_LEN;
    if header.game_account_code.len() < minimum_code_size {
        return Err(Error::InvalidCodeSize {
            expected_minimum: minimum_code_size,
            actual: header.game_account_code.len(),
        });
    }
    let output_root_proof_hash = compute_output_root_proof_hash(&header.output_root_proof);

    let root_claim_index = client_state.fault_dispute_game_code_root_claim_index;
    let root_claim = H256::<HexPrefixed>::new(
        header.game_account_code
            [root_claim_index as _..root_claim_index as usize + H256::<HexPrefixed>::BYTES_LEN]
            .try_into()
            .expect("impossible"),
    );

    // 5. Verify that the provided output root proof hash matches the stored root claim.
    if root_claim != output_root_proof_hash {
        return Err(Error::OutputRootHashMismatch {
            actual: output_root_proof_hash,
            expected: root_claim,
        });
    }

    // 6. Verify that the ibc account root is part of the L2 root.
    verify_account_storage_root(
        header.l2_header.state_root,
        &client_state.ibc_contract_address,
        &header.l2_ibc_account_proof.proof,
        &header.l2_ibc_account_proof.storage_root,
    )
    .map_err(Error::InvalidIbcContractProof)?;

    Ok(())
}

pub fn compute_game_slot(dispute_game_list_slot: U256, index: U256) -> U256 {
    let offset = Slot::Offset(dispute_game_list_slot);
    Slot::Array(&offset, index).slot()
}

// https://github.com/ethereum-optimism/optimism/blob/99a53381019d3571359d989671ccf70f8d69dfd9/packages/contracts-bedrock/src/libraries/Hashing.sol#L114
pub fn compute_output_root_proof_hash(output_root_proof: &OutputRootProof) -> H256 {
    keccak256(
        (
            output_root_proof.version,
            output_root_proof.state_root,
            output_root_proof.message_passer_storage_root,
            output_root_proof.latest_block_hash,
        )
            .abi_encode_params(),
    )
}

pub fn verify_l2_header_is_related_to_output_root_proof(
    output_root_proof: &OutputRootProof,
    l2_header: &L2Header,
) -> Result<(), Error> {
    let block_hash = l2_header.hash();
    if block_hash == output_root_proof.latest_block_hash {
        Ok(())
    } else {
        Err(Error::OutputRootHashMismatch {
            actual: block_hash,
            expected: output_root_proof.latest_block_hash,
        })
    }
}
