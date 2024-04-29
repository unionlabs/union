use macros::model;

#[cfg(feature = "ethabi")]
use crate::ibc::lightclients::cometbls::light_header::TryFromEthAbiLightHeaderError;
use crate::{
    errors::{required, MissingField},
    ibc::{
        core::client::height::Height,
        lightclients::cometbls::light_header::{LightHeader, TryFromLightHeaderError},
    },
};

#[model(
    proto(
        raw(protos::union::ibc::lightclients::cometbls::v1::Header),
        into,
        from
    ),
    ethabi(
        raw(contracts::glue::UnionIbcLightclientsCometblsV1HeaderData),
        into,
        from
    )
)]
pub struct Header {
    pub signed_header: LightHeader,
    pub trusted_height: Height,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug("{}", ::serde_utils::to_hex(&zero_knowledge_proof))]
    pub zero_knowledge_proof: Vec<u8>,
}

impl From<Header> for protos::union::ibc::lightclients::cometbls::v1::Header {
    fn from(value: Header) -> Self {
        Self {
            signed_header: Some(value.signed_header.into()),
            trusted_height: Some(value.trusted_height.into()),
            zero_knowledge_proof: value.zero_knowledge_proof,
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<Header> for contracts::glue::UnionIbcLightclientsCometblsV1HeaderData {
    fn from(value: Header) -> Self {
        Self {
            signed_header: value.signed_header.into(),
            trusted_height: value.trusted_height.into(),
            zero_knowledge_proof: value.zero_knowledge_proof.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromHeaderError {
    #[error(transparent)]
    MissingField(MissingField),
    #[error("invalid signed header")]
    SignedHeader(#[from] TryFromLightHeaderError),
}

impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::Header> for Header {
    type Error = TryFromHeaderError;

    fn try_from(
        value: protos::union::ibc::lightclients::cometbls::v1::Header,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            signed_header: required!(value.signed_header)?.try_into()?,
            trusted_height: required!(value.trusted_height)?.into(),
            zero_knowledge_proof: value.zero_knowledge_proof,
        })
    }
}

#[cfg(feature = "ethabi")]
#[derive(Debug, Clone, PartialEq)]
pub enum TryFromEthAbiHeaderError {
    SignedHeader(TryFromEthAbiLightHeaderError),
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::UnionIbcLightclientsCometblsV1HeaderData> for Header {
    type Error = TryFromEthAbiHeaderError;

    fn try_from(
        value: contracts::glue::UnionIbcLightclientsCometblsV1HeaderData,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            signed_header: value
                .signed_header
                .try_into()
                .map_err(TryFromEthAbiHeaderError::SignedHeader)?,
            trusted_height: value.trusted_height.into(),
            zero_knowledge_proof: value.zero_knowledge_proof.to_vec(),
        })
    }
}
