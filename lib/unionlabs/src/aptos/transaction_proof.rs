use macros::model;

use crate::{
    aptos::transaction_info::TransactionInfo,
    hash::{
        hash_v2::{Hash, HexUnprefixed},
        H256,
    },
};

/// `TransactionInfo` and a `TransactionAccumulatorProof` connecting it to the ledger root.
#[model(proto(
    raw(protos::union::ibc::lightclients::movement::v1::TransactionInfoWithProof),
    into,
    from
))]
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

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        aptos::{
            transaction_info::proto::TryFromTransactionInfoError,
            transaction_proof::{Null, TransactionAccumulatorProof, TransactionInfoWithProof},
        },
        errors::{required, InvalidLength, MissingField},
    };

    impl From<TransactionInfoWithProof>
        for protos::union::ibc::lightclients::movement::v1::TransactionInfoWithProof
    {
        fn from(value: TransactionInfoWithProof) -> Self {
            Self {
                ledger_info_to_transaction_info_proof: value
                    .ledger_info_to_transaction_info_proof
                    .siblings
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                transaction_info: Some(value.transaction_info.into()),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum TryFromTransactionInfoWithProofError {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid transaction proof sibling")]
        InvalidTxProofSibling(#[from] InvalidLength),
        #[error("invalid transaction info")]
        InvalidTxInfo(#[from] TryFromTransactionInfoError),
    }

    impl TryFrom<protos::union::ibc::lightclients::movement::v1::TransactionInfoWithProof>
        for TransactionInfoWithProof
    {
        type Error = TryFromTransactionInfoWithProofError;

        fn try_from(
            value: protos::union::ibc::lightclients::movement::v1::TransactionInfoWithProof,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                ledger_info_to_transaction_info_proof: TransactionAccumulatorProof {
                    siblings: value
                        .ledger_info_to_transaction_info_proof
                        .into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<Vec<_>, _>>()?,
                    phantom: Null,
                },
                transaction_info: required!(value.transaction_info)?.try_into()?,
            })
        }
    }
}
