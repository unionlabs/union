use beacon_api_types::LightClientHeader;
use unionlabs::required;

pub fn try_from_proto(
    value: protos::union::ibc::lightclients::ethereum::v1::LightClientHeader,
) -> Result<LightClientHeader, Error> {
    Ok(Self {
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
            .map_err(Error::ExecutionBranchNode)?
            .try_into()
            .map_err(|vec: Vec<_>| {
                Error::ExecutionBranch(InvalidLength {
                    expected: ExpectedLength::Exact(C::MAX_EXTRA_DATA_BYTES::USIZE),
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
    BeaconBlockHeader(#[from] TryFromBeaconBlockHeaderError),
    #[error("invalid `execution_payload_header`")]
    ExecutionPayloadHeader(#[from] TryFromExecutionPayloadHeaderError),
    #[error("invalid `execution_branch`")]
    ExecutionBranch(#[source] InvalidLength),
    #[error("invalid `execution_branch_node`")]
    ExecutionBranchNode(#[source] InvalidLength),
}

pub fn into_proto(
    value: LightClientHeader,
) -> protos::union::ibc::lightclients::ethereum::v1::LightClientHeader {
    Self {
        beacon: Some(value.beacon.into()),
        execution: Some(value.execution.into()),
        execution_branch: Vec::from(value.execution_branch)
            .into_iter()
            .map(H256::into_bytes)
            .collect(),
    }
}
