use unionlabs::{
    bounded::BoundedI64,
    google::protobuf::timestamp::Timestamp,
    primitives::{H256, encoding::HexUnprefixed},
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct LightHeader {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub height: BoundedI64<0, { i64::MAX }>,
    pub time: Timestamp,
    pub validators_hash: H256<HexUnprefixed>,
    pub next_validators_hash: H256<HexUnprefixed>,
    pub app_hash: H256<HexUnprefixed>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{
        bounded::BoundedIntError, errors::MissingField,
        google::protobuf::timestamp::TryFromTimestampError, impl_proto_via_try_from_into,
        primitives::FixedBytesError, required,
    };

    use crate::light_header::LightHeader;

    impl_proto_via_try_from_into!(LightHeader => protos::union::ibc::lightclients::cometbls::v1::LightHeader);

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

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid height")]
        Height(#[source] BoundedIntError<i64>),
        #[error("invalid timestamp")]
        Timestamp(#[from] TryFromTimestampError),
        #[error("invalid validators hash")]
        ValidatorsHash(#[source] FixedBytesError),
        #[error("invalid next validators hash")]
        NextValidatorsHash(#[source] FixedBytesError),
        #[error("invalid app hash")]
        AppHash(#[source] FixedBytesError),
    }

    impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::LightHeader> for LightHeader {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::cometbls::v1::LightHeader,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                height: value.height.try_into().map_err(Error::Height)?,
                time: required!(value.time)?.try_into()?,
                validators_hash: value
                    .validators_hash
                    .try_into()
                    .map_err(Error::ValidatorsHash)?,
                next_validators_hash: value
                    .next_validators_hash
                    .try_into()
                    .map_err(Error::NextValidatorsHash)?,
                app_hash: value.app_hash.try_into().map_err(Error::AppHash)?,
            })
        }
    }
}
