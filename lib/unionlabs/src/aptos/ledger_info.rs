use macros::model;

use super::{
    block_info::{BlockInfo, TryFromBlockInfoError},
    hash_value::HashValue,
    signature::{AggregateSignature, TryFromAggregateSignatureError},
};
use crate::errors::{required, ExpectedLength, InvalidLength, MissingField};

/// Wrapper to support future upgrades, this is the data being persisted.
#[model(proto(
    raw(protos::union::ibc::lightclients::movement::v1::LedgerInfoWithSignatures),
    into,
    from
))]
pub enum LedgerInfoWithSignatures {
    V0(LedgerInfoWithV0),
}

/// The validator node returns this structure which includes signatures
/// from validators that confirm the state.  The client needs to only pass back
/// the `LedgerInfo` element since the validator node doesn't need to know the signatures
/// again when the client performs a query, those are only there for the client
/// to be able to verify the state
#[model]
pub struct LedgerInfoWithV0 {
    pub ledger_info: LedgerInfo,
    /// Aggregated BLS signature of all the validators that signed the message. The bitmask in the
    /// aggregated signature can be used to find out the individual validators signing the message
    pub signatures: AggregateSignature,
}

#[model(proto(
    raw(protos::union::ibc::lightclients::movement::v1::LedgerInfo),
    into,
    from
))]
pub struct LedgerInfo {
    pub commit_info: BlockInfo,

    /// Hash of consensus specific data that is opaque to all parts of the system other than
    /// consensus.
    pub consensus_data_hash: HashValue,
}

impl From<LedgerInfoWithSignatures>
    for protos::union::ibc::lightclients::movement::v1::LedgerInfoWithSignatures
{
    fn from(value: LedgerInfoWithSignatures) -> Self {
        let LedgerInfoWithSignatures::V0(value) = value;
        Self {
            ledger_info: Some(value.ledger_info.into()),
            signatures: Some(value.signatures.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromLedgerInfoWithSignatures {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid ledger info")]
    LedgerInfo(#[from] TryFromLedgerInfo),
    #[error("invalid signatures")]
    Signatures(#[from] TryFromAggregateSignatureError),
}

impl TryFrom<protos::union::ibc::lightclients::movement::v1::LedgerInfoWithSignatures>
    for LedgerInfoWithSignatures
{
    type Error = TryFromLedgerInfoWithSignatures;

    fn try_from(
        value: protos::union::ibc::lightclients::movement::v1::LedgerInfoWithSignatures,
    ) -> Result<Self, Self::Error> {
        Ok(Self::V0(LedgerInfoWithV0 {
            ledger_info: required!(value.ledger_info)?.try_into()?,
            signatures: required!(value.signatures)?.try_into()?,
        }))
    }
}

impl From<LedgerInfo> for protos::union::ibc::lightclients::movement::v1::LedgerInfo {
    fn from(value: LedgerInfo) -> Self {
        Self {
            commit_info: Some(value.commit_info.into()),
            consensus_data_hash: value.consensus_data_hash.0.to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromLedgerInfo {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid commit info")]
    CommitInfo(#[from] TryFromBlockInfoError),
    #[error("invalid consensus data hash")]
    ConsensusDataHash(#[source] InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::movement::v1::LedgerInfo> for LedgerInfo {
    type Error = TryFromLedgerInfo;

    fn try_from(
        value: protos::union::ibc::lightclients::movement::v1::LedgerInfo,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            commit_info: required!(value.commit_info)?.try_into()?,
            consensus_data_hash: HashValue::new(
                value
                    .consensus_data_hash
                    .as_slice()
                    .try_into()
                    .map_err(|_| {
                        TryFromLedgerInfo::ConsensusDataHash(InvalidLength {
                            expected: ExpectedLength::Exact(HashValue::LENGTH),
                            found: value.consensus_data_hash.len(),
                        })
                    })?,
            ),
        })
    }
}
