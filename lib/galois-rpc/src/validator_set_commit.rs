use cometbft_types::types::simple_validator::SimpleValidator;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorSetCommit {
    pub validators: Vec<SimpleValidator>,
    // REVIEW: Is this arbitrary bytes or strongly typed? (i.e. H512)
    #[serde(with = "::serde_utils::hex_string_list")]
    pub signatures: Vec<Vec<u8>>,
    pub bitmap: Vec<u8>,
}

impl From<ValidatorSetCommit> for protos::union::galois::api::v3::ValidatorSetCommit {
    fn from(value: ValidatorSetCommit) -> Self {
        Self {
            validators: value.validators.into_iter().map(Into::into).collect(),
            signatures: value.signatures,
            bitmap: value.bitmap,
        }
    }
}
