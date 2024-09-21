use macros::model;
use serde::{Deserialize, Serialize};

use crate::{
    errors::InvalidLength,
    hash::hash_v2::{Hash, HexUnprefixed},
};

/// `TransactionInfo` is the object we store in the transaction accumulator. It consists of the
/// transaction as well as the execution result of this transaction.
#[model(
    proto(
        raw(protos::union::ibc::lightclients::movement::v1::TransactionInfo),
        into,
        from
    ),
    no_serde
)]
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
    pub transaction_hash: Hash<32, HexUnprefixed>,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    pub event_root_hash: Hash<32, HexUnprefixed>,

    /// The hash value summarizing all changes caused to the world state by this transaction.
    /// i.e. hash of the output write set.
    pub state_change_hash: Hash<32, HexUnprefixed>,

    /// The root hash of the Sparse Merkle Tree describing the world state at the end of this
    /// transaction. Depending on the protocol configuration, this can be generated periodical
    /// only, like per block.
    pub state_checkpoint_hash: Option<Hash<32, HexUnprefixed>>,

    /// Potentially summarizes all evicted items from state. Always `None` for now.
    pub state_cemetery_hash: Option<Hash<32, HexUnprefixed>>,
}

// impl TransactionInfoV0 {
//     pub fn hash(&self) -> Hash<32, HexUnprefixed> {
//         let mut state = Sha3_256::new();
//         state.update(
//             Sha3_256::new()
//                 .chain_update("APTOS::TransactionInfoV0")
//                 .finalize(),
//         );
//         bcs::serialize_into(&mut state, &self).expect("expected to be able to serialize");
//         Hash<32, HexUnprefixed>(state.finalize().into())
//     }
// }

#[model(no_serde)]
#[derive(Serialize, Deserialize)]
pub enum ExecutionStatus {
    #[serde(rename = "Success")]
    Success,
}

impl From<TransactionInfo> for protos::union::ibc::lightclients::movement::v1::TransactionInfo {
    fn from(value: TransactionInfo) -> Self {
        let TransactionInfo::V0(value) = value;
        Self {
            gas_used: value.gas_used,
            transaction_hash: value.transaction_hash.into(),
            event_root_hash: value.event_root_hash.into(),
            state_change_hash: value.state_change_hash.into(),
            state_checkpoint_hash: match value.state_checkpoint_hash {
                Some(val) => val.into(),
                None => Vec::default(),
            },
            state_cemetery_hash: match value.state_cemetery_hash {
                Some(val) => val.into(),
                None => Vec::default(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromTransactionInfoError {
    #[error("invalid transaction hash")]
    TransactionHash(#[source] InvalidLength),
    #[error("invalid event root hash")]
    EventRootHash(#[source] InvalidLength),
    #[error("invalid state change hash")]
    StateChangeHash(#[source] InvalidLength),
    #[error("invalid state checkpoint hash")]
    StateCheckpointHash(#[source] InvalidLength),
    #[error("invalid state cemetery hash")]
    StateCemeteryHash(#[source] InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::movement::v1::TransactionInfo> for TransactionInfo {
    type Error = TryFromTransactionInfoError;

    fn try_from(
        value: protos::union::ibc::lightclients::movement::v1::TransactionInfo,
    ) -> Result<Self, Self::Error> {
        Ok(Self::V0(TransactionInfoV0 {
            gas_used: value.gas_used,
            status: ExecutionStatus::Success,
            transaction_hash: value
                .transaction_hash
                .try_into()
                .map_err(TryFromTransactionInfoError::TransactionHash)?,
            event_root_hash: value
                .event_root_hash
                .try_into()
                .map_err(TryFromTransactionInfoError::EventRootHash)?,
            state_change_hash: value
                .state_change_hash
                .try_into()
                .map_err(TryFromTransactionInfoError::StateChangeHash)?,
            state_checkpoint_hash: if value.state_checkpoint_hash.is_empty() {
                None
            } else {
                Some(
                    value
                        .state_checkpoint_hash
                        .try_into()
                        .map_err(TryFromTransactionInfoError::StateCheckpointHash)?,
                )
            },
            state_cemetery_hash: if value.state_cemetery_hash.is_empty() {
                None
            } else {
                Some(
                    value
                        .state_cemetery_hash
                        .try_into()
                        .map_err(TryFromTransactionInfoError::StateCemeteryHash)?,
                )
            },
        }))
    }
}
