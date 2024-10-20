use macros::model;

use crate::{
    errors::{required, InvalidLength, MissingField},
    hash::H256,
    union::galois::zero_knowledge_proof::ZeroKnowledgeProof,
};

#[model(proto(raw(protos::union::galois::api::v3::ProveResponse), into, from))]
pub struct ProveResponse {
    pub proof: ZeroKnowledgeProof,
    pub trusted_validator_set_root: H256,
}

impl From<ProveResponse> for protos::union::galois::api::v3::ProveResponse {
    fn from(value: ProveResponse) -> Self {
        Self {
            proof: Some(value.proof.into()),
            trusted_validator_set_root: value.trusted_validator_set_root.into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TryFromProveResponseError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid trusted_validator_set_root")]
    TrustedValidatorSetRoot(#[source] InvalidLength),
    #[error("invalid untrusted_validator_set_root")]
    UntrustedValidatorSetRoot(#[source] InvalidLength),
}

impl TryFrom<protos::union::galois::api::v3::ProveResponse> for ProveResponse {
    type Error = TryFromProveResponseError;

    fn try_from(value: protos::union::galois::api::v3::ProveResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            proof: required!(value.proof)?.into(),
            trusted_validator_set_root: value
                .trusted_validator_set_root
                .try_into()
                .map_err(TryFromProveResponseError::TrustedValidatorSetRoot)?,
        })
    }
}
