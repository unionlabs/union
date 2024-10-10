use macros::model;

use crate::{
    errors::{required, MissingField},
    tendermint::types::{
        commit::{Commit, TryFromCommitError},
        header::{Header, TryFromHeaderError},
    },
};

#[model(proto(raw(protos::tendermint::types::SignedHeader), into, from))]
pub struct SignedHeader {
    pub header: Header,
    pub commit: Commit,
}

impl From<SignedHeader> for protos::tendermint::types::SignedHeader {
    fn from(value: SignedHeader) -> Self {
        Self {
            header: Some(value.header.into()),
            commit: Some(value.commit.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromSignedHeaderError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid header")]
    Header(#[source] TryFromHeaderError),
    #[error("invalid commit")]
    Commit(#[source] TryFromCommitError),
}

impl TryFrom<protos::tendermint::types::SignedHeader> for SignedHeader {
    type Error = TryFromSignedHeaderError;

    fn try_from(value: protos::tendermint::types::SignedHeader) -> Result<Self, Self::Error> {
        Ok(Self {
            header: required!(value.header)?
                .try_into()
                .map_err(TryFromSignedHeaderError::Header)?,
            commit: required!(value.commit)?
                .try_into()
                .map_err(TryFromSignedHeaderError::Commit)?,
        })
    }
}
