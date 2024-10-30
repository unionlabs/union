use core::fmt::Debug;

use arbitrum_light_client_types::{ClientState, Header};
use ethereum_verifier::{verify_account_storage_root, verify_storage_proof};
use sha3::{Digest, Keccak256};
use unionlabs::{
    ethereum::slot::{MappingKey, Slot},
    hash::H256,
    uint::U256,
};

#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum Error {
    #[error("invalid contract address proof")]
    InvalidContractAddressProof(#[source] ethereum_verifier::error::Error),
    #[error("invalid _latestConfirmed proof")]
    InvalidNextNodeNumSlotProof(#[source] ethereum_verifier::error::Error),
    #[error("invalid _nodes[_latestConfirmed].confirmData proof")]
    InvalidNodeConfirmDataProof(#[source] ethereum_verifier::error::Error),
    #[error("invalid L2 proof")]
    InvalidL2Proof(#[source] ethereum_verifier::error::Error),
}

pub fn verify_header(
    client_state: ClientState,
    header: Header,
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

    // Verify that the L1 `ClientState.l1_next_node_num_slot` is part of the L1 account root
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

    // Verify that the L1 `_nodes[ClientState.l1_nodes_slot].confirmData` is part of the L1 account root
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

    // Verify that the ibc account root is part of the L1 root
    verify_account_storage_root(
        header.l2_header.state_root,
        &client_state.l2_ibc_contract_address,
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

// #[cfg(test)]
// mod tests {
//     use std::path::Path;

//     use hex_literal::hex;
//     use serde::de::DeserializeOwned;
//     use unionlabs::{
//         encoding::{DecodeAs, EncodeAs, Proto},
//         ibc::lightclients::arbitrum::header::Header,
//     };

//     use crate::verify_header;

//     fn read_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> T {
//         serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
//     }

//     // #[test]
//     // fn test_update_header() {
//     //     let arbitrum_client_state =
//     //         read_json("/home/ben/projects/union/union/arb-client-state.json");
//     //     let arbitrum_header: Header = read_json("/home/ben/projects/union/union/arb-header.json");

//     //     let proto_header = arbitrum_header.clone().encode_as::<Proto>();
//     //     let rt_header = Header::decode_as::<Proto>(&proto_header).unwrap();

//     //     std::fs::write(
//     //         "/home/ben/projects/union/union/arb-header-rt.json",
//     //         serde_json::to_string_pretty(&rt_header).unwrap(),
//     //     )
//     //     .unwrap();

//     //     assert_eq!(arbitrum_header, rt_header);

//     //     let res = verify_header(
//     //         arbitrum_client_state,
//     //         arbitrum_header,
//     //         hex!("2b06d9a1b1e74dc203face3a78f8b0fbaf2c07aca9d9520cf75ea3b6682bff93").into(),
//     //     );

//     //     let () = res.map_err(error_reporter::Report::new).unwrap();

//     //     // assert!(matches!(res, Ok(())));
//     // }

//     // #[test]
//     // fn test_l2_contract_slot_exist() {
//     //     let proof: Proof =
//     //         serde_json::from_str(&std::fs::read_to_string("tests/arbitrum_proof.json").unwrap())
//     //             .unwrap();
//     //     assert!(matches!(
//     //         verify_zktrie_storage_proof(
//     //             H256(hex!(
//     //                 "1b52888cae05bdba27f8470293a7d2bc3b9a9c822d96affe05ef243e0dfd44a0"
//     //             )),
//     //             proof.key.to_be_bytes().into(),
//     //             &proof.value.to_be_bytes(),
//     //             &proof.proof
//     //         ),
//     //         Ok(())
//     //     ))
//     // }

//     // #[test]
//     // fn test_l2_contract_slot_absent() {
//     //     let proof: Proof =
//     //         serde_json::from_str(&std::fs::read_to_string("tests/arbitrum_absent.json").unwrap())
//     //             .unwrap();
//     //     assert!(matches!(
//     //         verify_zktrie_storage_absence(
//     //             H256(hex!(
//     //                 "1b52888cae05bdba27f8470293a7d2bc3b9a9c822d96affe05ef243e0dfd44a0"
//     //             )),
//     //             proof.key.to_be_bytes().into(),
//     //             &proof.proof
//     //         ),
//     //         Ok(())
//     //     ))
//     // }
// }
