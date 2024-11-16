use beacon_api_types::{
    consts::{floorlog2, EXECUTION_PAYLOAD_INDEX},
    LightClientHeader,
};
use unionlabs::{
    errors::{ExpectedLength, InvalidLength, MissingField},
    hash::H256,
    required,
};

use crate::{beacon_block_header_proto, execution_payload_header_proto};

pub fn try_from_proto(
    value: protos::union::ibc::lightclients::ethereum::v1::LightClientHeader,
) -> Result<LightClientHeader, Error> {
    Ok(LightClientHeader {
        beacon: required!(value.beacon).map(beacon_block_header_proto::try_from_proto)??,
        execution: required!(value.execution)
            .map(execution_payload_header_proto::try_from_proto)??,
        execution_branch: value
            .execution_branch
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::ExecutionBranchNode)?
            .try_into()
            .map_err(|vec: Vec<_>| {
                Error::ExecutionBranchNode(InvalidLength {
                    expected: ExpectedLength::Exact(floorlog2(EXECUTION_PAYLOAD_INDEX)),
                    found: vec.len(),
                })
            })?,
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
    #[error("invalid `execution_branch` node")]
    ExecutionBranchNode(#[source] InvalidLength),
    #[error("invalid `execution_branch`")]
    ExecutionBranch(#[source] InvalidLength),
}

pub fn into_proto(
    value: LightClientHeader,
) -> protos::union::ibc::lightclients::ethereum::v1::LightClientHeader {
    protos::union::ibc::lightclients::ethereum::v1::LightClientHeader {
        beacon: Some(beacon_block_header_proto::into_proto(value.beacon)),
        execution: Some(execution_payload_header_proto::into_proto(value.execution)),
        execution_branch: Vec::from(value.execution_branch)
            .into_iter()
            .map(H256::into_bytes)
            .collect(),
    }
}
