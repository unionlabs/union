use serde::{Deserialize, Serialize};

use crate::{
    cosmos::ics23::{inner_spec::InnerSpec, leaf_op::LeafOp},
    errors::{required, MissingField},
    Proto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProofSpec {
    pub leaf_spec: LeafOp,
    pub inner_spec: InnerSpec,
    // REVIEW: > 0?
    pub max_depth: i32,
    // REVIEW: > 0?
    pub min_depth: i32,
}

impl ProofSpec {
    // TODO(aeryz): what the hell is this
    pub fn compatible(&self, spec: &ProofSpec) -> bool {
        self.leaf_spec.hash == spec.leaf_spec.hash
            && self.leaf_spec.prehash_key == spec.leaf_spec.prehash_key
            && self.leaf_spec.prehash_value == spec.leaf_spec.prehash_value
            && self.leaf_spec.length == spec.leaf_spec.length
            && self.inner_spec.hash == spec.inner_spec.hash
            && self.inner_spec.min_prefix_length == spec.inner_spec.min_prefix_length
            && self.inner_spec.max_prefix_length == spec.inner_spec.max_prefix_length
            && self.inner_spec.child_size == spec.inner_spec.child_size
            && self.inner_spec.child_order.len() == spec.inner_spec.child_order.len()
    }
}

impl TypeUrl for protos::cosmos::ics23::v1::ProofSpec {
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
