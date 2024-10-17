use macros::model;

use crate::cometbft::types::evidence::{Evidence, TryFromEvidenceError};

#[model(proto(raw(protos::cometbft::types::v1::EvidenceList), into, from))]
pub struct EvidenceList {
    pub evidence: Vec<Evidence>,
}

impl From<EvidenceList> for protos::cometbft::types::v1::EvidenceList {
    fn from(value: EvidenceList) -> Self {
        Self {
            evidence: value.evidence.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromEvidenceListError {
    #[error("invalid evidence")]
    Evidence(#[from] TryFromEvidenceError),
}

impl TryFrom<protos::cometbft::types::v1::EvidenceList> for EvidenceList {
    type Error = TryFromEvidenceListError;

    fn try_from(value: protos::cometbft::types::v1::EvidenceList) -> Result<Self, Self::Error> {
        Ok(Self {
            evidence: value
                .evidence
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}
