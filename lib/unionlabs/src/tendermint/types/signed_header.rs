use serde::{Deserialize, Serialize};

use crate::{
    tendermint::types::{commit::Commit, header::Header},
    Proto, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl Proto for SignedHeader {
    type Proto = protos::tendermint::types::SignedHeader;
}

impl TypeUrl for protos::tendermint::types::SignedHeader {
    const TYPE_URL: &'static str = "/tendermint.types.SignedHeader";
}
