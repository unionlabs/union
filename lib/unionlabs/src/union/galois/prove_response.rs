use serde::{Deserialize, Serialize};

use crate::{
    errors::{required, InvalidLength, MissingField},
    hash::H256,
    union::galois::zero_knowledge_proof::ZeroKnowledgeProof,
    Proto, TypeUrl,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ProveResponse {
    pub proof: ZeroKnowledgeProof,
    pub trusted_validator_set_root: H256,
    pub untrusted_validator_set_root: H256,
}

impl Proto for ProveResponse {
    type Proto = protos::union::galois::api::v1::ProveResponse;
}

impl TypeUrl for protos::union::galois::api::v1::ProveResponse {
    const TYPE_URL: &'static str = "/union.galois.api.v1.ProveResponse";
}

impl From<ProveResponse> for protos::union::galois::api::v1::ProveResponse {
    fn from(value: ProveResponse) -> Self {
        Self {
            proof: Some(value.proof.into()),
            trusted_validator_set_root: value.trusted_validator_set_root.into(),
            untrusted_validator_set_root: value.untrusted_validator_set_root.into(),
        }
    }
}

#[derive(Debug)]
pub enum TryFromProveResponseError {
    MissingField(MissingField),
    TrustedValidatorSetRoot(InvalidLength),
    UntrustedValidatorSetRoot(InvalidLength),
}

impl TryFrom<protos::union::galois::api::v1::ProveResponse> for ProveResponse {
    type Error = TryFromProveResponseError;

    fn try_from(value: protos::union::galois::api::v1::ProveResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            proof: required!(value.proof)?.into(),
            trusted_validator_set_root: value
                .trusted_validator_set_root
                .try_into()
                .map_err(TryFromProveResponseError::TrustedValidatorSetRoot)?,
            untrusted_validator_set_root: value
                .untrusted_validator_set_root
                .try_into()
                .map_err(TryFromProveResponseError::UntrustedValidatorSetRoot)?,
        })
    }
}
