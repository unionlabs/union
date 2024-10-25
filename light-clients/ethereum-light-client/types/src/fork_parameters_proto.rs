use beacon_api_types::{fork::Fork, ForkParameters, Version};
use unionlabs::{
    errors::{InvalidLength, MissingField},
    required,
};

pub fn into_proto(
    value: ForkParameters,
) -> protos::union::ibc::lightclients::ethereum::v1::ForkParameters {
    protos::union::ibc::lightclients::ethereum::v1::ForkParameters {
        genesis_fork_version: value.genesis_fork_version.0.into(),
        genesis_slot: value.genesis_slot,
        altair: Some(fork_into_proto(value.altair)),
        bellatrix: Some(fork_into_proto(value.bellatrix)),
        capella: Some(fork_into_proto(value.capella)),
        deneb: Some(fork_into_proto(value.deneb)),
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid genesis fork version")]
    GenesisForkVersion(#[source] InvalidLength),
    #[error("invalid altair")]
    Altair(#[source] ForkError),
    #[error("invalid bellatrix")]
    Bellatrix(#[source] ForkError),
    #[error("invalid capella")]
    Capella(#[source] ForkError),
    #[error("invalid deneb")]
    Deneb(#[source] ForkError),
}

pub fn try_from_proto(
    proto: protos::union::ibc::lightclients::ethereum::v1::ForkParameters,
) -> Result<ForkParameters, Error> {
    Ok(ForkParameters {
        genesis_fork_version: proto
            .genesis_fork_version
            .try_into()
            .map(Version)
            .map_err(Error::GenesisForkVersion)?,
        genesis_slot: proto.genesis_slot,
        altair: required!(proto.altair)
            .map(fork_try_from_proto)?
            .map_err(Error::Altair)?,
        bellatrix: required!(proto.bellatrix)
            .map(fork_try_from_proto)?
            .map_err(Error::Bellatrix)?,
        capella: required!(proto.capella)
            .map(fork_try_from_proto)?
            .map_err(Error::Capella)?,
        deneb: required!(proto.deneb)
            .map(fork_try_from_proto)?
            .map_err(Error::Deneb)?,
    })
}

pub fn fork_into_proto(value: Fork) -> protos::union::ibc::lightclients::ethereum::v1::Fork {
    protos::union::ibc::lightclients::ethereum::v1::Fork {
        version: value.version.0.into(),
        epoch: value.epoch,
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum ForkError {
    #[error("invalid version")]
    Version(#[source] InvalidLength),
}

fn fork_try_from_proto(
    value: protos::union::ibc::lightclients::ethereum::v1::Fork,
) -> Result<Fork, ForkError> {
    Ok(Fork {
        version: value
            .version
            .try_into()
            .map(Version)
            .map_err(ForkError::Version)?,
        epoch: value.epoch,
    })
}
