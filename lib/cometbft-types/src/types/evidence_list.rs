use serde::{Deserialize, Serialize};

use crate::types::{
    commit_sig::{CommitSig, CommitSigRaw},
    evidence::Evidence,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceList {
    pub evidence: Vec<Evidence<CommitSigRaw>>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::types::{evidence, evidence_list::EvidenceList};

    impl From<EvidenceList> for protos::cometbft::types::v1::EvidenceList {
        fn from(value: EvidenceList) -> Self {
            Self {
                evidence: value.evidence.into_iter().map(Into::into).collect(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid evidence")]
        Evidence(#[from] evidence::proto::Error),
    }

    impl TryFrom<protos::cometbft::types::v1::EvidenceList> for EvidenceList {
        type Error = Error;

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
}
