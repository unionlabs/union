use beacon_api_types::BeaconBlockHeader;
use unionlabs::errors::InvalidLength;

pub fn into_proto(
    value: BeaconBlockHeader,
) -> protos::union::ibc::lightclients::ethereum::v1::BeaconBlockHeader {
    protos::union::ibc::lightclients::ethereum::v1::BeaconBlockHeader {
        slot: value.slot,
        proposer_index: value.proposer_index,
        parent_root: value.parent_root.get().into(),
        state_root: value.state_root.get().into(),
        body_root: value.body_root.get().into(),
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum Error {
    #[error("invalid `parent_root`")]
    ParentRoot(#[source] InvalidLength),
    #[error("invalid `state_root`")]
    StateRoot(#[source] InvalidLength),
    #[error("invalid `body_root`")]
    BodyRoot(#[source] InvalidLength),
}

fn try_from_proto(
    value: protos::union::ibc::lightclients::ethereum::v1::BeaconBlockHeader,
) -> Result<BeaconBlockHeader, Error> {
    Ok(BeaconBlockHeader {
        slot: value.slot,
        proposer_index: value.proposer_index,
        parent_root: value.parent_root.try_into().map_err(Error::ParentRoot)?,
        state_root: value.state_root.try_into().map_err(Error::StateRoot)?,
        body_root: value.body_root.try_into().map_err(Error::BodyRoot)?,
    })
}
