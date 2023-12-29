use serde::{Deserialize, Serialize};

use super::hash_op::HashOp;
use crate::{errors::UnknownEnumVariant, Proto, TypeUrl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InnerSpec {
    pub child_order: Vec<i32>,
    pub child_size: i32,
    pub min_prefix_length: i32,
    pub max_prefix_length: i32,
    #[serde(with = "::serde_utils::hex_string")]
    pub empty_child: Vec<u8>,
    pub hash: HashOp,
}

impl TypeUrl for protos::cosmos::ics23::v1::InnerSpec {
    const TYPE_URL: &'static str = "/cosmos.ics23.v1.InnerSpec";
}

impl Proto for InnerSpec {
    type Proto = protos::cosmos::ics23::v1::InnerSpec;
}

impl From<InnerSpec> for protos::cosmos::ics23::v1::InnerSpec {
    fn from(value: InnerSpec) -> Self {
        Self {
            child_order: value.child_order,
            child_size: value.child_size,
            min_prefix_length: value.min_prefix_length,
            max_prefix_length: value.max_prefix_length,
            empty_child: value.empty_child,
            hash: value.hash.into(),
        }
    }
}

#[derive(Debug)]
pub enum TryFromInnerSpecError {
    Hash(UnknownEnumVariant<i32>),
}

impl TryFrom<protos::cosmos::ics23::v1::InnerSpec> for InnerSpec {
    type Error = TryFromInnerSpecError;

    fn try_from(value: protos::cosmos::ics23::v1::InnerSpec) -> Result<Self, Self::Error> {
        Ok(Self {
            child_order: value.child_order,
            child_size: value.child_size,
            min_prefix_length: value.min_prefix_length,
            max_prefix_length: value.max_prefix_length,
            empty_child: value.empty_child,
            hash: value.hash.try_into().map_err(TryFromInnerSpecError::Hash)?,
        })
    }
}
