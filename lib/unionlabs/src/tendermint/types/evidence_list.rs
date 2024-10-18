use macros::model;

use crate::tendermint::types::evidence::Evidence;

#[model(proto(raw(protos::tendermint::types::EvidenceList), into, from))]
pub struct EvidenceList {
    pub evidence: Vec<Evidence>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::tendermint::types::{
        evidence::proto::TryFromEvidenceError, evidence_list::EvidenceList,
    };

    impl From<EvidenceList> for protos::tendermint::types::EvidenceList {
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

    impl TryFrom<protos::tendermint::types::EvidenceList> for EvidenceList {
        type Error = TryFromEvidenceListError;

        fn try_from(value: protos::tendermint::types::EvidenceList) -> Result<Self, Self::Error> {
            Ok(Self {
                evidence: value
                    .evidence
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
            })
        }
    }
}
