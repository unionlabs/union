use serde::{Deserialize, Serialize};

use crate::types::{commit::Commit, data::Data, evidence_list::EvidenceList, header::Header};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub header: Header,
    pub data: Data,
    pub evidence: EvidenceList,
    pub last_commit: Commit,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{errors::MissingField, required};

    use crate::types::{
        block::Block,
        commit, evidence_list,
        header::{self},
    };

    impl From<Block> for protos::cometbft::types::v1::Block {
        fn from(value: Block) -> Self {
            Self {
                header: Some(value.header.into()),
                data: Some(value.data.into()),
                evidence: Some(value.evidence.into()),
                last_commit: Some(value.last_commit.into()),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid header")]
        Header(#[from] header::proto::Error),
        #[error("invalid evidence list")]
        EvidenceList(#[from] evidence_list::proto::Error),
        #[error("invalid commit")]
        Commit(#[from] commit::proto::Error),
    }

    impl TryFrom<protos::cometbft::types::v1::Block> for Block {
        type Error = Error;

        fn try_from(value: protos::cometbft::types::v1::Block) -> Result<Self, Self::Error> {
            Ok(Self {
                header: required!(value.header)?.try_into()?,
                data: required!(value.data)?.into(),
                evidence: required!(value.evidence)?.try_into()?,
                last_commit: required!(value.last_commit)?.try_into()?,
            })
        }
    }
}
