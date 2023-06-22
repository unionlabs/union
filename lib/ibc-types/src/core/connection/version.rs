use contracts::ibc_handler::IbcCoreConnectionV1VersionData;

use crate::{core::channel::order::Order, errors::UnknownEnumVariant};

#[derive(Debug, Clone)]
pub struct Version {
    // TODO(benluelo): "The identifier field specifies a unique version identifier. A value of "1" specifies IBC 1.0.0."
    pub identifier: String,
    pub features: Vec<Order>,
}

impl TryFrom<protos::ibc::core::connection::v1::Version> for Version {
    type Error = UnknownEnumVariant<String>;

    fn try_from(proto: protos::ibc::core::connection::v1::Version) -> Result<Self, Self::Error> {
        Ok(Self {
            identifier: proto.identifier,
            features: proto
                .features
                .into_iter()
                .map(Order::try_from_str)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl From<Version> for protos::ibc::core::connection::v1::Version {
    fn from(value: Version) -> Self {
        Self {
            identifier: value.identifier,
            features: value
                .features
                .into_iter()
                .map(|feature| <&'static str>::from(feature).to_string())
                .collect(),
        }
    }
}

impl From<Version> for IbcCoreConnectionV1VersionData {
    fn from(value: Version) -> Self {
        Self {
            identifier: value.identifier,
            features: value
                .features
                .into_iter()
                .map(|order| <&'static str>::from(order).to_string())
                .collect(),
        }
    }
}

impl TryFrom<IbcCoreConnectionV1VersionData> for Version {
    type Error = UnknownEnumVariant<String>;

    fn try_from(value: IbcCoreConnectionV1VersionData) -> Result<Self, Self::Error> {
        Ok(Self {
            identifier: value.identifier,
            features: value
                .features
                .into_iter()
                .map(Order::try_from_str)
                .collect::<Result<_, _>>()?,
        })
    }
}
