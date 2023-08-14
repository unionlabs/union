use serde::{Deserialize, Serialize};

use crate::tendermint::types::{commit::Commit, header::Header};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignedHeader {
    pub header: Header,
    pub commit: Commit,
}

#[cfg(feature = "ethabi")]
#[derive(Debug)]
pub enum TryFromEthAbiSignedHeaderError {
    Header(crate::TryFromEthAbiErrorOf<Header>),
    Commit(crate::TryFromEthAbiErrorOf<Commit>),
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::TendermintTypesSignedHeaderData> for SignedHeader {
    type Error = TryFromEthAbiSignedHeaderError;

    fn try_from(
        value: contracts::glue::TendermintTypesSignedHeaderData,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            header: value
                .header
                .try_into()
                .map_err(TryFromEthAbiSignedHeaderError::Header)?,
            commit: value
                .commit
                .try_into()
                .map_err(TryFromEthAbiSignedHeaderError::Commit)?,
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<SignedHeader> for contracts::glue::TendermintTypesSignedHeaderData {
    fn from(value: SignedHeader) -> Self {
        Self {
            header: value.header.into(),
            commit: value.commit.into(),
        }
    }
}
