use crate::{tendermint::types::simple_validator::SimpleValidator, Proto, TypeUrl};

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatorSetCommit {
    pub validators: Vec<SimpleValidator>,
    // REVIEW: Is this arbitrary bytes or strongly typed?
    pub signatures: Vec<Vec<u8>>,
    pub bitmap: Vec<u8>,
}

impl Proto for ValidatorSetCommit {
    type Proto = protos::union::galois::api::v1::ValidatorSetCommit;
}

impl TypeUrl for protos::union::galois::api::v1::ValidatorSetCommit {
    const TYPE_URL: &'static str = "/union.galois.api.v1.ValidatorSetCommit";
}

impl From<ValidatorSetCommit> for protos::union::galois::api::v1::ValidatorSetCommit {
    fn from(value: ValidatorSetCommit) -> Self {
        Self {
            validators: value.validators.into_iter().map(Into::into).collect(),
            signatures: value.signatures,
            bitmap: value.bitmap,
        }
    }
}
