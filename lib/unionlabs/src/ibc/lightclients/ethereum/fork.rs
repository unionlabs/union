use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};

use crate::{errors::InvalidLength, ethereum::Version, IntoProto, TryFromProto, TypeUrl};

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
pub struct Fork {
    pub version: Version,
    pub epoch: u64,
}

impl From<Fork> for protos::union::ibc::lightclients::ethereum::v1::Fork {
    fn from(value: Fork) -> Self {
        Self {
            version: value.version.into(),
            epoch: value.epoch,
        }
    }
}

impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::Fork> for Fork {
    type Error = InvalidLength;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::Fork,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            version: value.version.try_into()?,
            epoch: value.epoch,
        })
    }
}

impl TypeUrl for protos::union::ibc::lightclients::ethereum::v1::Fork {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.ethereum.v1.Fork";
}

impl IntoProto for Fork {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::Fork;
}

impl TryFromProto for Fork {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::Fork;
}
