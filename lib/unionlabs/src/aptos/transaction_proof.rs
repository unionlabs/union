use macros::model;

use crate::{
    aptos::transaction_info::TransactionInfo,
    primitives::{encoding::HexUnprefixed, Hash, H256},
};

/// `TransactionInfo` and a `TransactionAccumulatorProof` connecting it to the ledger root.
#[model]
pub struct TransactionInfoWithProof {
    /// The accumulator proof from ledger info root to leaf that authenticates the hash of the
    /// `TransactionInfo` object.
    pub ledger_info_to_transaction_info_proof: TransactionAccumulatorProof,

    /// The `TransactionInfo` object at the leaf of the accumulator.
    pub transaction_info: TransactionInfo,
}

// TODO(aeryz): only for testing purposes until we have proper proofs
impl Default for TransactionInfoWithProof {
    fn default() -> Self {
        Self {
            ledger_info_to_transaction_info_proof: TransactionAccumulatorProof {
                siblings: vec![],
                phantom: Null,
            },
            transaction_info: TransactionInfo::V0(super::transaction_info::TransactionInfoV0 {
                gas_used: 0,
                status: super::transaction_info::ExecutionStatus::Success,
                transaction_hash: Hash::default(),
                event_root_hash: Hash::default(),
                state_change_hash: Hash::default(),
                state_checkpoint_hash: Some(Hash::default()),
                state_cemetery_hash: None,
            }),
        }
    }
}

#[model]
pub struct TransactionAccumulatorProof {
    pub siblings: Vec<H256<HexUnprefixed>>,
    pub phantom: Null,
}

// idk man, it's in the json
#[model]
pub struct Null;
