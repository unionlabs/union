use macros::model;

use crate::{errors::InvalidLength, hash::H256};

#[model(proto(
    raw(protos::union::ibc::lightclients::scroll::v1::ScrollFinalizedProof),
    into,
    from
))]
pub struct ScrollFinalizedProof {
    pub batch_index: u64,
    pub finalized_state_root: H256,
    #[serde(with = "::serde_utils::hex_string_list")]
    #[debug(wrap = ::serde_utils::fmt::DebugListAsHex)]
    pub proof: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub enum TryFromScrollFinalizedProofError {
    Value(InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::scroll::v1::ScrollFinalizedProof>
    for ScrollFinalizedProof
{
    type Error = TryFromScrollFinalizedProofError;

    fn try_from(
        value: protos::union::ibc::lightclients::scroll::v1::ScrollFinalizedProof,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            batch_index: value.batch_index,
            finalized_state_root: value
                .finalized_state_root
                .try_into()
                .map_err(TryFromScrollFinalizedProofError::Value)?,
            proof: value.proof,
        })
    }
}

impl From<ScrollFinalizedProof>
    for protos::union::ibc::lightclients::scroll::v1::ScrollFinalizedProof
{
    fn from(value: ScrollFinalizedProof) -> Self {
        Self {
            batch_index: value.batch_index,
            finalized_state_root: value.finalized_state_root.into(),
            proof: value.proof,
        }
    }
}
