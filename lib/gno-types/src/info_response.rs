use serde::{Deserialize, Serialize};
use unionlabs::{
    bounded::BoundedI64,
    primitives::{H256, encoding::Base64},
};

use crate::ResponseBase;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoResponse {
    #[serde(rename = "ResponseBase")]
    pub response_base: ResponseBase,
    #[serde(rename = "ABCIVersion")]
    pub abci_version: String,
    #[serde(rename = "AppVersion")]
    pub app_version: String,
    #[serde(rename = "LastBlockHeight", with = "::serde_utils::string")]
    pub last_block_height: BoundedI64<0, { i64::MAX }>,
    #[serde(rename = "LastBlockAppHash")]
    pub last_block_app_hash: H256<Base64>,
}
