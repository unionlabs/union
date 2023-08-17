use core::num::TryFromIntError;

use serde::{Deserialize, Serialize};

use crate::{
    bounded_int::{BoundedI32, BoundedI64, BoundedIntError},
    Proto, TypeUrl,
};

/// See <https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=2a4088bba6218db02520968c4a4aee87>
const TS_SECONDS_MAX: i64 = 253_402_300_799;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Timestamp {
    /// As per the proto docs: "Must be from 0001-01-01T00:00:00Z to
    /// 9999-12-31T23:59:59Z inclusive."
    pub seconds: BoundedI64<0, TS_SECONDS_MAX>,
    // As per the proto docs: "Must be from 0 to 999,999,999 inclusive."
    pub nanos: BoundedI32<0, 999_999_999>,
}

impl Proto for Timestamp {
    type Proto = protos::google::protobuf::Timestamp;
}

impl TypeUrl for protos::google::protobuf::Timestamp {
    const TYPE_URL: &'static str = "/google.protobuf.Timestamp";
}

impl TryFrom<cosmwasm_std::Timestamp> for Timestamp {
    type Error = TryFromTimestampError;

    fn try_from(value: cosmwasm_std::Timestamp) -> Result<Self, Self::Error> {
        Ok(Self {
            seconds: TryInto::<i64>::try_into(value.seconds())
                .map_err(TryFromTimestampError::IntCast)?
                .try_into()
                .map_err(TryFromTimestampError::Seconds)?,
            nanos: TryInto::<i32>::try_into(value.nanos())
                .map_err(TryFromTimestampError::IntCast)?
                .try_into()
                .map_err(TryFromTimestampError::Nanos)?,
        })
    }
}

#[allow(clippy::cast_sign_loss)]
impl From<Timestamp> for cosmwasm_std::Timestamp {
    fn from(value: Timestamp) -> Self {
        cosmwasm_std::Timestamp::from_seconds(value.seconds.inner() as u64)
            .plus_nanos(value.nanos.inner() as u64)
    }
}

impl From<Timestamp> for protos::google::protobuf::Timestamp {
    fn from(value: Timestamp) -> Self {
        Self {
            seconds: value.seconds.into(),
            nanos: value.nanos.into(),
        }
    }
}

#[derive(Debug)]
pub enum TryFromTimestampError {
    Seconds(BoundedIntError<i64>),
    Nanos(BoundedIntError<i32>),
    IntCast(TryFromIntError),
}

impl TryFrom<protos::google::protobuf::Timestamp> for Timestamp {
    type Error = TryFromTimestampError;

    fn try_from(value: protos::google::protobuf::Timestamp) -> Result<Self, Self::Error> {
        Ok(Self {
            seconds: value
                .seconds
                .try_into()
                .map_err(TryFromTimestampError::Seconds)?,
            nanos: value
                .nanos
                .try_into()
                .map_err(TryFromTimestampError::Nanos)?,
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<Timestamp> for contracts::glue::GoogleProtobufTimestampData {
    fn from(value: Timestamp) -> Self {
        Self {
            secs: value.seconds.into(),
            nanos: value.nanos.inner().into(),
        }
    }
}

#[cfg(feature = "ethabi")]
#[derive(Debug)]
pub enum TryFromEthAbiTimestampError {
    Seconds(BoundedIntError<i64>),
    Nanos(BoundedIntError<i32>),
    NanosTryFromI64(std::num::TryFromIntError),
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::GoogleProtobufTimestampData> for Timestamp {
    type Error = TryFromEthAbiTimestampError;

    fn try_from(value: contracts::glue::GoogleProtobufTimestampData) -> Result<Self, Self::Error> {
        Ok(Self {
            seconds: value
                .secs
                .try_into()
                .map_err(TryFromEthAbiTimestampError::Seconds)?,
            nanos: i32::try_from(value.nanos)
                .map_err(TryFromEthAbiTimestampError::NanosTryFromI64)?
                .try_into()
                .map_err(TryFromEthAbiTimestampError::Nanos)?,
        })
    }
}

#[cfg(feature = "ethabi")]
impl crate::EthAbi for Timestamp {
    type EthAbi = contracts::glue::GoogleProtobufTimestampData;
}
