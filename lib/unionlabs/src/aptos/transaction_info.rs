use macros::model;
use serde::{Deserialize, Serialize};

use crate::primitives::{encoding::HexUnprefixed, H256};

/// `TransactionInfo` is the object we store in the transaction accumulator. It consists of the
/// transaction as well as the execution result of this transaction.
#[model(no_serde)]
#[derive(Serialize, Deserialize)]
pub enum TransactionInfo {
    V0(TransactionInfoV0),
}

#[model]
pub struct TransactionInfoV0 {
    /// The amount of gas used.
    pub gas_used: u64,

    /// The vm status. If it is not `Executed`, this will provide the general error class. Execution
    /// failures and Move abort's receive more detailed information. But other errors are generally
    /// categorized with no status code or other information
    pub status: ExecutionStatus,

    /// The hash of this transaction.
    pub transaction_hash: H256<HexUnprefixed>,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    pub event_root_hash: H256<HexUnprefixed>,

    /// The hash value summarizing all changes caused to the world state by this transaction.
    /// i.e. hash of the output write set.
    pub state_change_hash: H256<HexUnprefixed>,

    /// The root hash of the Sparse Merkle Tree describing the world state at the end of this
    /// transaction. Depending on the protocol configuration, this can be generated periodical
    /// only, like per block.
    pub state_checkpoint_hash: Option<H256<HexUnprefixed>>,

    /// Potentially summarizes all evicted items from state. Always `None` for now.
    pub state_cemetery_hash: Option<H256<HexUnprefixed>>,
}

// impl TransactionInfoV0 {
//     pub fn hash(&self) -> H256<HexUnprefixed> {
//         let mut state = Sha3_256::new();
//         state.update(
//             Sha3_256::new()
//                 .chain_update("APTOS::TransactionInfoV0")
//                 .finalize(),
//         );
//         bcs::serialize_into(&mut state, &self).expect("expected to be able to serialize");
//         H256<HexUnprefixed>(state.finalize().into())
//     }
// }

#[model(no_serde)]
#[derive(Serialize, Deserialize)]
pub enum ExecutionStatus {
    #[serde(rename = "Success")]
    Success,
}
