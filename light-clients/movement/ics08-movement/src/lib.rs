mod block_info;
mod epoch_state;
mod hash_value;
mod sparse_merkle_proof;
mod state_proof;
mod types;
mod validator_verifier;

use aptos_types::{proof::TransactionInfoWithProof, state_proof::StateProof};
use hash_value::HashValue;
// use aptos_crypto::{hash::CryptoHash, HashValue};
// use aptos_types::{
//     account_address::AccountAddress,
//     proof::{
//         SparseMerkleInternalNode, SparseMerkleLeafNode, SparseMerkleProof, TransactionInfoWithProof,
//     },
//     state_proof::StateProof,
//     state_store::{state_key::StateKey, table::TableHandle},
// };
use hex_literal::hex;
use sparse_merkle_proof::SparseMerkleProof;
// use sparse_merkle_proof::*;

#[test]
fn merkle_proof() {
    // let internal = SparseMerkleInternalNode::new(HashValue::zero(), HashValue::zero()).hash();
    // let mine = sparse_merkle_proof::SparseMerkleInternalNode::new(
    //     sparse_merkle_proof::HashValue::zero(),
    //     sparse_merkle_proof::HashValue::zero(),
    // )
    // .hash();

    // let internal = SparseMerkleLeafNode::new(HashValue::zero(), HashValue::zero()).hash();
    // let mine = sparse_merkle_proof::SparseMerkleLeafNode::new(
    //     sparse_merkle_proof::HashValue::zero(),
    //     sparse_merkle_proof::HashValue::zero(),
    // )
    // .hash();

    // println!("internal == mine: {}", internal.as_ref() == mine.as_ref());

    let proof: SparseMerkleProof = serde_json::from_str(
        r#"{"leaf":{"key":"f2d067d8ef7e97deb231d46f40f9f30200e6f1dad495d33e2a7911825a97ad14","value_hash":"40414333f8109f8cb971c67c9eca3c0049e21e6c5e28551f1a4975c96ab15212"},"siblings":["fafdceaec25fd64517ce3745992467dfac306a5ce59e63255da5b9f58d1417ea","4480c449082954642653a4570c7cb2ea2114d79b61621b94f095f25d640b6e27","0fc055434d70262945d428a5eda3d8396aa960c65ee8c4e79bd20638a95e7a31","731e28eb6655e01b8714aa72f76a0f468c330b46eb9c21816a88e56840896f24","b120265e60289e6e44216efd4f3fba86a8de645d3eb7912ff09812024c639b2f","d90e0a63c7c3cf7ed000841a85f981d8c6bec4c23353822204c7c9e9c5dee4db","30b21a8a3bf202b5fe18e415c299fd3b9985462a6292fffdabd5b32fbc27ba30","5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000","5b9096922002407577b4e46e6466aadb15cbb9521fd0e9847d474398ea3736e2","5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000","884f8b72a832aa718c6590d0bfeb1ec85b546611b6c07f8df563827974ad8134"]}"#,
    ).unwrap();

    let txs_with_proof: TransactionInfoWithProof = serde_json::from_str(
        r#"{"ledger_info_to_transaction_info_proof":{"siblings":["38ebb945a351a6701658fe7f5398133ad62777754e3ea44834da0d1e75a87e10","cdb6d7b047d18fdf13f27f13cf1077bc03b1e93b283feeecec55e31176a1f328","835449fd22e856b1f0fdb76d1ff3e493b7c0f8f43b9f66690b4d4d90a0c424ac","af625d6b7281a633d6bdf5cc144186e1ff094e9953165fd1516ab16544776631","d00d20a6fb6874e4c36e5690a4069b94a41f1ae197ae8769d2207f668f016c1c","3c929e62e334cb0ca8dbfd955899aa2bb09e6cc2ce053261689bb69d31c133f4","819a3f1ed1827d33e60b91da7f44736b4c583faec52b8867f45a97c1803b2b66","ea3756c694f6ed5782c91640e5e821604fa39cc55ff85691949d5c93f5c9fb95"],"phantom":null},"transaction_info":{"V0":{"gas_used":0,"status":"Success","transaction_hash":"e77d9016e431a2d367c513ebaf1bc39e291dc9589728e9bf1495fc573cb085ca","event_root_hash":"414343554d554c41544f525f504c414345484f4c4445525f4841534800000000","state_change_hash":"afb6e14fe47d850fd0a7395bcfb997ffacf4715e0f895cc162c218e4a7564bc6","state_checkpoint_hash":"02388da3aee85236d64e272fec0b1a6fcd4962986327971faef9ee2951a4ad6a","state_cemetery_hash":null}}}"#,
    ).unwrap();

    let state_proof: StateProof = serde_json::from_str(
        r#"{"latest_li_w_sigs":{"V0":{"ledger_info":{"commit_info":{"epoch":1,"round":0,"id":"0bb023a37e8dbe213f277bab59579e8a75ff6c646d6a5e5384789526128043b3","executed_state_id":"b4f2928670ff96185bc02ed57168c18443b6735524b580114aff5dc262f6ff3c","version":181,"timestamp_usecs":1723755828375426,"next_epoch_state":{"epoch":1,"verifier":{"validator_infos":[{"address":"d1126ce48bd65fb72190dbd9a6eaa65ba973f1e1664ac0cfba4db1d071fd0c36","public_key":"0x86fb211f41a07c6399ccc6ab3a8fe568fb0f574ce1b811896c44c6da4f267d543c6cac9fb8f4e9b92a3b809eefb91cbd","voting_power":100000000}]}}},"consensus_data_hash":"0000000000000000000000000000000000000000000000000000000000000000"},"signatures":{"validator_bitmask":{"inner":[]},"sig":null}}},"epoch_changes":{"ledger_info_with_sigs":[],"more":false}}"#,
    ).unwrap();

    // for i in 0..20 {
    //     let res = txs_with_proof.ledger_info_to_transaction_info_proof.verify(
    //         state_proof
    //             .latest_ledger_info()
    //             .commit_info()
    //             .executed_state_id(),
    //         txs_with_proof.transaction_info.hash(),
    //         i,
    //     );

    //     println!("Res: {res:?}");
    // }

    // let key = StateKey::table_item(
    //     &TableHandle(AccountAddress::new(hex!(
    //         "be769b7536776eb353a61aa4d26de32ee16844d89d8cc2ede29732e3d19407ea"
    //     ))),
    //     &vec![04, 0x41, 0x42, 0x43, 0x44],
    // );

    // println!("{}", key.hash());

    proof.verify_by_hash(
        HashValue(hex!(
            "02388da3aee85236d64e272fec0b1a6fcd4962986327971faef9ee2951a4ad6a"
        )),
        HashValue(hex!(
            "f2d067d8ef7e97deb231d46f40f9f30200e6f1dad495d33e2a7911825a97ad14"
        )),
        Some(HashValue(hex!(
            "40414333f8109f8cb971c67c9eca3c0049e21e6c5e28551f1a4975c96ab15212"
        ))),
    );

    // println!("Res: {res:?}");
}
