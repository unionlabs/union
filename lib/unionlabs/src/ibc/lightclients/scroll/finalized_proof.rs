use custom_debug_derive::Debug;
use serde::{Deserialize, Serialize};

use crate::{errors::InvalidLength, hash::H256, Proto, TypeUrl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct FinalizedProof {
    pub batch_index: u64,
    #[serde(with = "::serde_utils::hex_string_list")]
    #[debug(with = "::serde_utils::fmt::hex_list")]
    pub proof: Vec<Vec<u8>>,
}

impl TypeUrl for protos::union::ibc::lightclients::scroll::v1::ScrollFinalizedProof {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.scroll.v1.ScrollFinalizedProof";
}

impl Proto for FinalizedProof {
    type Proto = protos::union::ibc::lightclients::scroll::v1::ScrollFinalizedProof;
}

#[derive(Debug)]
pub enum TryFromScrollFinalizedProofError {
    Value(InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::scroll::v1::ScrollFinalizedProof>
    for FinalizedProof
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

impl From<FinalizedProof> for protos::union::ibc::lightclients::scroll::v1::ScrollFinalizedProof {
    fn from(value: FinalizedProof) -> Self {
        Self {
            batch_index: value.batch_index,
            finalized_state_root: value.finalized_state_root.into(),
            proof: value.proof,
        }
    }
}
