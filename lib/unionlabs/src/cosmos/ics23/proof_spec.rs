use serde::{Deserialize, Serialize};

use super::{inner_spec::InnerSpec, leaf_op::LeafOp};
use crate::{
    errors::{required, MissingField},
    Proto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofSpec {
    pub leaf_spec: LeafOp,
    pub inner_spec: InnerSpec,
    pub max_depth: i32,
    pub min_depth: i32,
    // TODO(aeryz): ibc-go seems to have this field, is this wrong or am I looking at an older version
    // pub prehash_key_before_comparison: bool,
}

impl TypeUrl for protos::cosmos::ics23::v1::ProofSpec {
    // TODO(aeryz): Check type url
    const TYPE_URL: &'static str = "/cosmos.ics23.v1.ProofSpec";
}

impl Proto for ProofSpec {
    type Proto = protos::cosmos::ics23::v1::ProofSpec;
}

impl From<ProofSpec> for protos::cosmos::ics23::v1::ProofSpec {
    fn from(value: ProofSpec) -> Self {
        Self {
            leaf_spec: Some(value.leaf_spec.into()),
            inner_spec: Some(value.inner_spec.into()),
            max_depth: value.max_depth,
            min_depth: value.min_depth,
        }
    }
}

#[derive(Debug)]
pub enum TryFromProofSpecError {
    MissingField(MissingField),
    LeafSpec(TryFromProtoErrorOf<LeafOp>),
    InnerSpec(TryFromProtoErrorOf<InnerSpec>),
}

impl TryFrom<protos::cosmos::ics23::v1::ProofSpec> for ProofSpec {
    type Error = TryFromProofSpecError;

    fn try_from(value: protos::cosmos::ics23::v1::ProofSpec) -> Result<Self, Self::Error> {
        Ok(Self {
            leaf_spec: required!(value.leaf_spec)?
                .try_into()
                .map_err(TryFromProofSpecError::LeafSpec)?,
            inner_spec: required!(value.inner_spec)?
                .try_into()
                .map_err(TryFromProofSpecError::InnerSpec)?,
            max_depth: value.max_depth,
            min_depth: value.min_depth,
        })
    }
}
