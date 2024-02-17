use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{errors::InvalidLength, hash::H256, Proto, TypeUrl};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct ScrollFinalizedProof {
    pub batch_index: u64,
    pub finalized_state_root: H256,
    #[serde(with = "::serde_utils::hex_string_list")]
    pub proof: Vec<Vec<u8>>,
}

impl Debug for ScrollFinalizedProof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScrollRollupProof")
            .field("batch_index", &self.batch_index)
            .field("finalized_state_root", &self.finalized_state_root)
            .field(
                "proof",
                &self
                    .proof
                    .iter()
                    .map(serde_utils::to_hex)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl TypeUrl for protos::union::ibc::lightclients::scroll::v1::ScrollFinalizedProof {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.scroll.v1.ScrollFinalizedProof";
}

impl Proto for ScrollFinalizedProof {
    type Proto = protos::union::ibc::lightclients::scroll::v1::ScrollFinalizedProof;
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
            finalized_state_root: H256::try_from(value.finalized_state_root.as_ref())
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
