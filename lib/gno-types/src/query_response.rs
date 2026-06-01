use serde::{Deserialize, Serialize};
use unionlabs::{
    bounded::BoundedI64,
    primitives::{Bytes, encoding::Base64},
};

use crate::{Proof, ResponseBase};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryResponse {
    #[serde(rename = "ResponseBase")]
    pub response_base: ResponseBase,
    #[serde(rename = "Key")]
    pub key: Option<Bytes<Base64>>,
    #[serde(rename = "Value")]
    pub value: Option<Bytes<Base64>>,
    #[serde(rename = "Proof")]
    pub proof: Option<Proof>,
    #[serde(rename = "Height", with = "::serde_utils::string")]
    pub height: BoundedI64<0, { i64::MAX }>,
}
