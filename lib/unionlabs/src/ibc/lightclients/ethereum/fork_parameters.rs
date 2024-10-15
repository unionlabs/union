use macros::model;

use crate::{ethereum::Version, ibc::lightclients::ethereum::fork::Fork};

#[cfg_attr(feature = "ssz", derive(::ssz::Ssz))]
#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::ForkParameters),
    into,
    from
))]
pub struct ForkParameters {
    pub genesis_fork_version: Version,
    pub genesis_slot: u64,
    pub altair: Fork,
    pub bellatrix: Fork,
    pub capella: Fork,
    pub deneb: Fork,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        errors::{required, InvalidLength, MissingField},
        ibc::lightclients::ethereum::{
            fork::proto::TryFromForkError, fork_parameters::ForkParameters,
        },
    };

    impl From<ForkParameters> for protos::union::ibc::lightclients::ethereum::v1::ForkParameters {
        fn from(value: ForkParameters) -> Self {
            Self {
                genesis_fork_version: value.genesis_fork_version.into(),
                genesis_slot: value.genesis_slot,
                altair: Some(value.altair.into()),
                bellatrix: Some(value.bellatrix.into()),
                capella: Some(value.capella.into()),
                deneb: Some(value.deneb.into()),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromForkParametersError {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid genesis fork version")]
        GenesisForkVersion(#[source] InvalidLength),
        #[error("invalid altair")]
        Altair(#[source] TryFromForkError),
        #[error("invalid bellatrix")]
        Bellatrix(#[source] TryFromForkError),
        #[error("invalid capella")]
        Capella(#[source] TryFromForkError),
        #[error("invalid deneb")]
        Deneb(#[source] TryFromForkError),
    }

    impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::ForkParameters> for ForkParameters {
        type Error = TryFromForkParametersError;

        fn try_from(
            proto: protos::union::ibc::lightclients::ethereum::v1::ForkParameters,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                genesis_fork_version: proto
                    .genesis_fork_version
                    .try_into()
                    .map_err(TryFromForkParametersError::GenesisForkVersion)?,
                genesis_slot: proto.genesis_slot,
                altair: required!(proto.altair)?
                    .try_into()
                    .map_err(TryFromForkParametersError::Altair)?,
                bellatrix: required!(proto.bellatrix)?
                    .try_into()
                    .map_err(TryFromForkParametersError::Bellatrix)?,
                capella: required!(proto.capella)?
                    .try_into()
                    .map_err(TryFromForkParametersError::Capella)?,
                deneb: required!(proto.deneb)?
                    .try_into()
                    .map_err(TryFromForkParametersError::Deneb)?,
            })
        }
    }
}
