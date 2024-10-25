use serde::{Deserialize, Serialize};
use ssz::Ssz;
use typenum::Unsigned;
use unionlabs::{
    errors::{ExpectedLength, InvalidLength, MissingField},
    ethereum::config::{
        consts::{floorlog2, EXECUTION_PAYLOAD_INDEX},
        BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES,
    },
    hash::H256,
};

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::required;

    use crate::LightClientHeader;

    impl From<UnboundedLightClientHeader>
        for protos::union::ibc::lightclients::ethereum::v1::LightClientHeader
    {
        fn from(value: UnboundedLightClientHeader) -> Self {
            Self {
                beacon: Some(value.beacon.into()),
                execution: Some(value.execution.into()),
                execution_branch: Vec::from(value.execution_branch)
                    .into_iter()
                    .map(H256::into_bytes)
                    .collect(),
            }
        }
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

    impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::LightClientHeader>
        for LightClientHeader<C>
    {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::ethereum::v1::LightClientHeader,
        ) -> Result<Self, Self::Error> {
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
    }
}
