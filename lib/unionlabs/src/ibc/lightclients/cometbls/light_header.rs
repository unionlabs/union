use macros::model;

#[cfg(feature = "ethabi")]
use crate::google::protobuf::timestamp::TryFromEthAbiTimestampError;
use crate::{
    bounded::{BoundedI64, BoundedIntError},
    errors::{required, InvalidLength, MissingField},
    google::protobuf::timestamp::{Timestamp, TryFromTimestampError},
    hash::H256,
};

#[model(
    proto(
        raw(protos::union::ibc::lightclients::cometbls::v1::LightHeader),
        into,
        from
    ),
    ethabi(
        raw(contracts::glue::UnionIbcLightclientsCometblsV1LightHeaderData),
        into,
        from
    )
)]
pub struct LightHeader {
    #[serde(with = "::serde_utils::string")]
    pub height: BoundedI64<0, { i64::MAX }>,
    pub time: Timestamp,
    pub validators_hash: H256,
    pub next_validators_hash: H256,
    pub app_hash: H256,
}

impl From<LightHeader> for protos::union::ibc::lightclients::cometbls::v1::LightHeader {
    fn from(value: LightHeader) -> Self {
        Self {
            height: value.height.into(),
            time: Some(value.time.into()),
            validators_hash: value.validators_hash.into(),
            next_validators_hash: value.next_validators_hash.into(),
            app_hash: value.app_hash.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TryFromLightHeaderError {
    MissingField(MissingField),
    Height(BoundedIntError<i64>),
    Timestamp(TryFromTimestampError),
    ValidatorsHash(InvalidLength),
    NextValidatorsHash(InvalidLength),
    AppHash(InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::LightHeader> for LightHeader {
    type Error = TryFromLightHeaderError;

    fn try_from(
        value: protos::union::ibc::lightclients::cometbls::v1::LightHeader,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            height: value
                .height
                .try_into()
                .map_err(TryFromLightHeaderError::Height)?,
            time: required!(value.time)?
                .try_into()
                .map_err(TryFromLightHeaderError::Timestamp)?,
            validators_hash: value
                .validators_hash
                .try_into()
                .map_err(TryFromLightHeaderError::ValidatorsHash)?,
            next_validators_hash: value
                .next_validators_hash
                .try_into()
                .map_err(TryFromLightHeaderError::NextValidatorsHash)?,
            app_hash: value
                .app_hash
                .try_into()
                .map_err(TryFromLightHeaderError::AppHash)?,
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<LightHeader> for contracts::glue::UnionIbcLightclientsCometblsV1LightHeaderData {
    fn from(value: LightHeader) -> Self {
        Self {
            height: value.height.into(),
            time: value.time.into(),
            validators_hash: value.validators_hash.into(),
            next_validators_hash: value.next_validators_hash.into(),
            app_hash: value.app_hash.into(),
        }
    }
}

#[cfg(feature = "ethabi")]
#[derive(Debug, Clone, PartialEq)]
pub enum TryFromEthAbiLightHeaderError {
    Height(BoundedIntError<i64>),
    Timestamp(TryFromEthAbiTimestampError),
    ValidatorsHash(InvalidLength),
    NextValidatorsHash(InvalidLength),
    AppHash(InvalidLength),
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::UnionIbcLightclientsCometblsV1LightHeaderData> for LightHeader {
    type Error = TryFromEthAbiLightHeaderError;

    fn try_from(
        value: contracts::glue::UnionIbcLightclientsCometblsV1LightHeaderData,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            height: value
                .height
                .try_into()
                .map_err(TryFromEthAbiLightHeaderError::Height)?,
            time: value
                .time
                .try_into()
                .map_err(TryFromEthAbiLightHeaderError::Timestamp)?,
            validators_hash: value
                .validators_hash
                .to_vec()
                .try_into()
                .map_err(TryFromEthAbiLightHeaderError::ValidatorsHash)?,
            next_validators_hash: value
                .next_validators_hash
                .to_vec()
                .try_into()
                .map_err(TryFromEthAbiLightHeaderError::NextValidatorsHash)?,
            app_hash: value
                .app_hash
                .to_vec()
                .try_into()
                .map_err(TryFromEthAbiLightHeaderError::AppHash)?,
        })
    }
}
