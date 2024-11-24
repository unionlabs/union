use macros::model;

use crate::{errors::UnknownEnumVariant, ibc::core::channel::order::Order};

#[model(proto(raw(protos::ibc::core::connection::v1::Version), into, from))]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
pub struct Version {
    // TODO(benluelo): "The identifier field specifies a unique version identifier. A value of "1" specifies IBC 1.0.0."
    // TODO: Cow
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
                .map(|order| order.parse())
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
