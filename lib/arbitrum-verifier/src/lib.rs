use core::fmt::Debug;

use arbitrum_light_client_types::{ClientStateV1, Header};
use evm_storage_verifier::{verify_account_storage_root, verify_storage_proof};
use sha3::{Digest, Keccak256};
use unionlabs::{
    ethereum::slot::{MappingKey, Slot},
    primitives::{H256, U256},
};

#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum Error {
    #[error("invalid contract address proof")]
    InvalidContractAddressProof(#[source] evm_storage_verifier::error::Error),
    #[error("invalid _latestConfirmed proof")]
    InvalidNextNodeNumSlotProof(#[source] evm_storage_verifier::error::Error),
    #[error("invalid _nodes[_latestConfirmed].confirmData proof")]
    InvalidNodeConfirmDataProof(#[source] evm_storage_verifier::error::Error),
    #[error("invalid L2 proof")]
    InvalidL2Proof(#[source] evm_storage_verifier::error::Error),
}

pub fn verify_header_v1(
    client_state: &ClientStateV1,
    header: &Header,
    l1_state_root: H256,
) -> Result<(), Error> {
    // Verify that the L1 account root is part of the L1 root
    verify_account_storage_root(
        l1_state_root,
        &client_state.l1_contract_address,
        &header.l1_account_proof.proof,
        &header.l1_account_proof.storage_root,
    )
    .map_err(Error::InvalidContractAddressProof)?;

    // Verify that the L1 `ClientState.l1_next_node_num_slot` is part of the L1 Rollup account root
    verify_storage_proof(
        header.l1_account_proof.storage_root,
        client_state.l1_next_node_num_slot,
        &rlp::encode(&header.l1_next_node_num_slot_proof.value),
        &header.l1_next_node_num_slot_proof.proof,
    )
    .map_err(Error::InvalidNextNodeNumSlotProof)?;

    let node_num = {
        let slot_offset_bytes = client_state.l1_next_node_num_slot_offset_bytes.inner() as usize;
        // the .value is verified by the proof above
        u64::from_be_bytes(
            header.l1_next_node_num_slot_proof.value.to_be_bytes()
                [slot_offset_bytes..slot_offset_bytes + 8]
                .try_into()
                .expect("size is correct; qed;"),
        )
    };

    // Verify that the L1 `_nodes[ClientState.l1_nodes_slot].confirmData` is part of the L1 Rollup account root
    let node_confirm_data_slot = nodes_confirm_data_mapping_key(
        client_state.l1_nodes_slot,
        node_num,
        client_state.l1_nodes_confirm_data_offset,
    );

    let expected_confirm_data = Keccak256::new()
        .chain_update(header.l2_header.hash())
        .chain_update(header.l2_header.extra_data)
        .finalize();

    // Verify that the node's `confirmData` is correct
    verify_storage_proof(
        header.l1_account_proof.storage_root,
        node_confirm_data_slot,
        &rlp::encode(&U256::from_be_bytes(expected_confirm_data.into())),
        &header.l1_nodes_slot_proof.proof,
    )
    .map_err(Error::InvalidNodeConfirmDataProof)?;

    // Verify that the ibc account root is part of the L2 root
    verify_account_storage_root(
        header.l2_header.state_root,
        &client_state.ibc_contract_address,
        &header.l2_ibc_account_proof.proof,
        &header.l2_ibc_account_proof.storage_root,
    )
    .map_err(Error::InvalidL2Proof)?;

    Ok(())
}

/// Storage slot of a `mapping(uint64 => Node)` mapping, where the mapping is at slot `slot` and the `uint64` is the `nodeNum`, accessing the storage at the offset of confirm_data_offset.
pub fn nodes_confirm_data_mapping_key(
    slot: U256,
    node_num: u64,
    confirm_data_offset: U256,
) -> U256 {
    Slot::Mapping(&Slot::Offset(slot), MappingKey::Uint64(node_num)).slot() + confirm_data_offset
}
