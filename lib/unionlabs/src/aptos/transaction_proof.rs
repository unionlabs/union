use macros::model;

use crate::{
    aptos::transaction_info::{TransactionInfo, TryFromTransactionInfoError},
    errors::{required, InvalidLength, MissingField},
    hash::H256,
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

#[model]
pub struct TransactionAccumulatorProof {
    #[serde(with = "::serde_utils::hex_allow_unprefixed_list")]
    pub siblings: Vec<H256>,
    pub phantom: Null,
}

// idk man, it's in the json
#[model]
pub struct Null;

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
