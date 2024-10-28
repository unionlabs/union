use beacon_api_types::{
    light_client_update::{FinalityBranch, NextSyncCommitteeBranch},
    LightClientHeader, SyncAggregate, SyncCommittee,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LightClientUpdate {
    EpochChange {
        /// The trusted sync committee for the *next* epoch that the client is being updated to.
        ///
        /// If the current epoch is 10, this will be the sync committee for epoch 11.
        sync_committee: SyncCommittee,

        /// The next sync committee of the epoch that the client is being updated to, corresponding to `update_data.attested_header.state_root`.
        ///
        /// If the current epoch is 10, this will be the next sync committee for epoch 11 (i.e. the sync committee for epoch 12).
        next_sync_committee: SyncCommittee,
        /// The path of the next sync committee in the beacon SSZ state root.
        next_sync_committee_branch: NextSyncCommitteeBranch,

        update_data: LightClientUpdateData,
    },
    WithinEpoch {
        /// The trusted sync committee for the *current* epoch.
        ///
        /// If the current epoch is 10, this will be the sync committee for epoch 10.
        sync_committee: SyncCommittee,

        update_data: LightClientUpdateData,
    },
}

/// Common data required for all light client updates.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LightClientUpdateData {
    /// Header attested to by the sync committee
    pub attested_header: LightClientHeader,
    /// Finalized header corresponding to `attested_header.state_root`
    pub finalized_header: LightClientHeader,
    pub finality_branch: FinalityBranch,
    /// Sync committee aggregate signature
    pub sync_aggregate: SyncAggregate,
    /// Slot at which the aggregate signature was created (untrusted)
    pub signature_slot: u64,
}

#[cfg(feature = "proto")]
pub mod proto {
    use protos::union::ibc::lightclients::ethereum::v1::light_client_update;
    use unionlabs::{errors::MissingField, required};

    use super::{LightClientUpdate, LightClientUpdateData};
    use crate::{light_client_header_proto, sync_aggregate_proto, sync_committee_proto};

    impl From<LightClientUpdate> for protos::union::ibc::lightclients::ethereum::v1::LightClientUpdate {
        fn from(value: LightClientUpdate) -> Self {
            Self {
                update: Some(match value {
                    LightClientUpdate::EpochChange {
                        sync_committee,
                        next_sync_committee,
                        next_sync_committee_branch,
                        update_data,
                    } => light_client_update::Update::EpochChangeUpdate(
                        protos::union::ibc::lightclients::ethereum::v1::EpochChangeUpdate {
                            sync_committee: Some(sync_committee_proto::into_proto(sync_committee)),
                            next_sync_committee: Some(sync_committee_proto::into_proto(
                                next_sync_committee,
                            )),
                            next_sync_committee_branch: next_sync_committee_branch
                                .into_iter()
                                .map(Into::into)
                                .collect(),
                            update_data: Some(update_data.into()),
                        },
                    ),
                    LightClientUpdate::WithinEpoch {
                        sync_committee,
                        update_data,
                    } => light_client_update::Update::WithinEpochUpdate(
                        protos::union::ibc::lightclients::ethereum::v1::WithinEpochUpdate {
                            sync_committee: Some(sync_committee_proto::into_proto(sync_committee)),
                            update_data: Some(update_data.into()),
                        },
                    ),
                }),
            }
        }
    }

    #[derive(Clone, PartialEq, Debug, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
    }

    impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::LightClientUpdate>
        for LightClientUpdate
    {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::ethereum::v1::LightClientUpdate,
        ) -> Result<Self, Self::Error> {
            Ok(match required!(value.update)? {
                light_client_update::Update::EpochChangeUpdate(
                    protos::union::ibc::lightclients::ethereum::v1::EpochChangeUpdate {
                        sync_committee,
                        next_sync_committee,
                        next_sync_committee_branch,
                        update_data,
                    },
                ) => Self::EpochChange {
                    sync_committee: sync_committee_proto::try_from_proto(required!(
                        sync_committee
                    )?)
                    .unwrap(),
                    next_sync_committee: sync_committee_proto::try_from_proto(required!(
                        next_sync_committee
                    )?)
                    .unwrap(),
                    next_sync_committee_branch: next_sync_committee_branch
                        .iter()
                        .map(TryInto::try_into)
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap()
                        .try_into()
                        .unwrap(),
                    update_data: required!(update_data)?.try_into().unwrap(),
                },
                light_client_update::Update::WithinEpochUpdate(
                    protos::union::ibc::lightclients::ethereum::v1::WithinEpochUpdate {
                        sync_committee,
                        update_data,
                    },
                ) => Self::WithinEpoch {
                    sync_committee: sync_committee_proto::try_from_proto(required!(
                        sync_committee
                    )?)
                    .unwrap(),
                    update_data: required!(update_data)?.try_into().unwrap(),
                },
            })
        }
    }

    impl From<LightClientUpdateData>
        for protos::union::ibc::lightclients::ethereum::v1::LightClientUpdateData
    {
        fn from(value: LightClientUpdateData) -> Self {
            Self {
                attested_header: Some(light_client_header_proto::into_proto(value.attested_header)),
                finalized_header: Some(light_client_header_proto::into_proto(
                    value.finalized_header,
                )),
                finality_branch: value.finality_branch.into_iter().map(Into::into).collect(),
                sync_aggregate: Some(sync_aggregate_proto::into_proto(
                    value.sync_aggregate.into(),
                )),
                signature_slot: value.signature_slot,
            }
        }
    }

    #[derive(Debug, thiserror::Error)]
    pub enum TryFromLightClientUpdateDataError {
        #[error(transparent)]
        MissingField(#[from] MissingField),
    }

    impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::LightClientUpdateData>
        for LightClientUpdateData
    {
        type Error = TryFromLightClientUpdateDataError;

        fn try_from(
            value: protos::union::ibc::lightclients::ethereum::v1::LightClientUpdateData,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                attested_header: light_client_header_proto::try_from_proto(required!(
                    value.attested_header
                )?)
                .unwrap(),
                finalized_header: light_client_header_proto::try_from_proto(required!(
                    value.finalized_header
                )?)
                .unwrap(),
                finality_branch: value
                    .finality_branch
                    .iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap()
                    .try_into()
                    .unwrap(),
                sync_aggregate: sync_aggregate_proto::try_from_proto(required!(
                    value.sync_aggregate
                )?)
                .unwrap(),
                signature_slot: value.signature_slot,
            })
        }
    }
}
