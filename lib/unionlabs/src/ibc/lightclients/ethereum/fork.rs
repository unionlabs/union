use macros::model;

use crate::ethereum::Version;

#[cfg_attr(feature = "ssz", derive(::ssz::Ssz))]
#[model(proto(raw(protos::union::ibc::lightclients::ethereum::v1::Fork), into, from))]
pub struct Fork {
    pub version: Version,
    pub epoch: u64,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{errors::InvalidLength, ibc::lightclients::ethereum::fork::Fork};

    impl From<Fork> for protos::union::ibc::lightclients::ethereum::v1::Fork {
        fn from(value: Fork) -> Self {
            Self {
                version: value.version.into(),
                epoch: value.epoch,
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromForkError {
        #[error("invalid version")]
        Version(#[source] InvalidLength),
    }

    impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::Fork> for Fork {
        type Error = TryFromForkError;

        fn try_from(
            value: protos::union::ibc::lightclients::ethereum::v1::Fork,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                version: value
                    .version
                    .try_into()
                    .map_err(TryFromForkError::Version)?,
                epoch: value.epoch,
            })
        }
    }
}
