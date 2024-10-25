use beacon_api_types::LightClientHeader;
use unionlabs::{
    errors::{InvalidLength, MissingField},
    hash::H256,
    required,
};

use crate::{beacon_block_header_proto, execution_payload_header_proto};

pub fn try_from_proto(
    value: protos::union::ibc::lightclients::ethereum::v1::LightClientHeader,
) -> Result<LightClientHeader, Error> {
    Ok(LightClientHeader {
        beacon: required!(value.beacon)?
            .try_into()
            .map_err(Error::BeaconBlockHeader)?,
        execution: required!(value.execution)?
            .try_into()
            .map_err(Error::ExecutionPayloadHeader)?,
        execution_branch: value
            .execution_branch
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::ExecutionBranchNode)?,
    })
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid `beacon_block_header`")]
    BeaconBlockHeader(#[from] beacon_block_header_proto::Error),
    #[error("invalid `execution_payload_header`")]
    ExecutionPayloadHeader(#[from] execution_payload_header_proto::Error),
    #[error("invalid `execution_branch_node`")]
    ExecutionBranchNode(#[source] InvalidLength),
}

pub fn into_proto(
    value: LightClientHeader,
) -> protos::union::ibc::lightclients::ethereum::v1::LightClientHeader {
    protos::union::ibc::lightclients::ethereum::v1::LightClientHeader {
        beacon: Some(value.beacon.into()),
        execution: Some(value.execution.into()),
        execution_branch: Vec::from(value.execution_branch)
            .into_iter()
            .map(H256::into_bytes)
            .collect(),
    }
}
