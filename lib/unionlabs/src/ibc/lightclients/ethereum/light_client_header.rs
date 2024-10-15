use macros::model;

use crate::{
    ethereum::config::consts::{floorlog2, EXECUTION_PAYLOAD_INDEX},
    hash::H256,
    ibc::lightclients::ethereum::{
        beacon_block_header::BeaconBlockHeader,
        execution_payload_header::UnboundedExecutionPayloadHeader,
    },
};
#[cfg(feature = "ssz")]
use crate::{
    ethereum::config::{BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES},
    ibc::lightclients::ethereum::execution_payload_header::ExecutionPayloadHeader,
};

#[cfg(feature = "ssz")]
#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::LightClientHeader),
    into,
    from
))]
#[derive(::ssz::Ssz)]
#[cfg_attr(feature = "serde", serde(bound(serialize = "", deserialize = "")))]
pub struct LightClientHeader<C: BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> {
    pub beacon: BeaconBlockHeader,
    pub execution: ExecutionPayloadHeader<C>,
    pub execution_branch: [H256; floorlog2(EXECUTION_PAYLOAD_INDEX)],
}

#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::LightClientHeader),
    from
))]
pub struct UnboundedLightClientHeader {
    pub beacon: BeaconBlockHeader,
    pub execution: UnboundedExecutionPayloadHeader,
    pub execution_branch: [H256; floorlog2(EXECUTION_PAYLOAD_INDEX)],
}

#[cfg(feature = "proto")]
pub mod proto {
    #[cfg(feature = "ssz")]
    use {
        crate::{
            errors::{required, ExpectedLength, InvalidLength, MissingField},
            ethereum::config::{BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES},
            ibc::lightclients::ethereum::{
                beacon_block_header::proto::TryFromBeaconBlockHeaderError,
                execution_payload_header::proto::TryFromExecutionPayloadHeaderError,
                light_client_header::LightClientHeader,
            },
        },
        typenum::Unsigned,
    };

    use crate::{
        hash::H256, ibc::lightclients::ethereum::light_client_header::UnboundedLightClientHeader,
    };

    #[cfg(feature = "ssz")]
    impl<C: BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> From<LightClientHeader<C>>
        for protos::union::ibc::lightclients::ethereum::v1::LightClientHeader
    {
        fn from(value: LightClientHeader<C>) -> Self {
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

    #[cfg(feature = "ssz")]
    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromLightClientHeaderError {
        #[error(transparent)]
        MissingField(MissingField),
        #[error("invalid `beacon_block_header`")]
        BeaconBlockHeader(#[from] TryFromBeaconBlockHeaderError),
        #[error("invalid `execution_payload_header`")]
        ExecutionPayloadHeader(#[from] TryFromExecutionPayloadHeaderError),
        #[error("invalid `execution_branch`")]
        ExecutionBranch(#[source] InvalidLength),
        #[error("invalid `execution_branch_node`")]
        ExecutionBranchNode(#[source] InvalidLength),
    }

    #[cfg(feature = "ssz")]
    impl<C: BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES>
        TryFrom<protos::union::ibc::lightclients::ethereum::v1::LightClientHeader>
        for LightClientHeader<C>
    {
        type Error = TryFromLightClientHeaderError;

        fn try_from(
            value: protos::union::ibc::lightclients::ethereum::v1::LightClientHeader,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                beacon: required!(value.beacon)?
                    .try_into()
                    .map_err(TryFromLightClientHeaderError::BeaconBlockHeader)?,
                execution: required!(value.execution)?
                    .try_into()
                    .map_err(TryFromLightClientHeaderError::ExecutionPayloadHeader)?,
                execution_branch: value
                    .execution_branch
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(TryFromLightClientHeaderError::ExecutionBranchNode)?
                    .try_into()
                    .map_err(|vec: Vec<_>| {
                        TryFromLightClientHeaderError::ExecutionBranch(InvalidLength {
                            expected: ExpectedLength::Exact(C::MAX_EXTRA_DATA_BYTES::USIZE),
                            found: vec.len(),
                        })
                    })?,
            })
        }
    }

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
}
