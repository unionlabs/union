use beacon_api_types::{light_client_update::NextSyncCommitteeBranch, SyncCommittee};

use crate::LightClientUpdateData;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EpochChangeUpdate {
    /// The trusted sync committee for the *next* epoch that the client is being updated to.
    ///
    /// If the current epoch is 10, this will be the sync committee for epoch 11.
    pub sync_committee: SyncCommittee,

    /// The next sync committee of the epoch that the client is being updated to, corresponding to `update_data.attested_header.state_root`.
    ///
    /// If the current epoch is 10, this will be the *next* sync committee for epoch 11 (i.e. the sync committee for epoch 12).
    pub next_sync_committee: SyncCommittee,
    /// The path of the next sync committee in the beacon chain SSZ state root.
    pub next_sync_committee_branch: NextSyncCommitteeBranch,

    pub update_data: LightClientUpdateData,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WithinEpochUpdate {
    /// The trusted sync committee for the *current* epoch.
    ///
    /// If the current epoch is 10, this will be the sync committee for epoch 10.
    pub sync_committee: SyncCommittee,

    pub update_data: LightClientUpdateData,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// boxed for size
pub enum LightClientUpdate {
    EpochChange(Box<EpochChangeUpdate>),
    WithinEpoch(Box<WithinEpochUpdate>),
}

impl LightClientUpdate {
    pub fn update_data(&self) -> &LightClientUpdateData {
        match self {
            LightClientUpdate::EpochChange(update) => &update.update_data,
            LightClientUpdate::WithinEpoch(update) => &update.update_data,
        }
    }

    pub fn trusted_sync_committee(&self) -> &SyncCommittee {
        match self {
            LightClientUpdate::EpochChange(update) => &update.sync_committee,
            LightClientUpdate::WithinEpoch(update) => &update.sync_committee,
        }
    }
}

impl From<LightClientUpdate> for beacon_api_types::LightClientUpdate {
    fn from(value: LightClientUpdate) -> Self {
        match value {
            LightClientUpdate::EpochChange(update) => {
                update.update_data.new_beacon_light_client_update(
                    Some(update.next_sync_committee),
                    Some(update.next_sync_committee_branch),
                )
            }
            LightClientUpdate::WithinEpoch(update) => update
                .update_data
                .new_beacon_light_client_update(None, None),
        }
    }
}

#[cfg(feature = "proto")]
pub mod proto {
    use protos::union::ibc::lightclients::ethereum::v1::light_client_update;
    use unionlabs::{errors::MissingField, required};

    use crate::{sync_committee_proto, EpochChangeUpdate, LightClientUpdate, WithinEpochUpdate};

    impl From<LightClientUpdate> for protos::union::ibc::lightclients::ethereum::v1::LightClientUpdate {
        fn from(value: LightClientUpdate) -> Self {
            Self {
                update: Some(match value {
                    LightClientUpdate::EpochChange(epoch_change_update) => {
                        light_client_update::Update::EpochChangeUpdate(
                            protos::union::ibc::lightclients::ethereum::v1::EpochChangeUpdate {
                                sync_committee: Some(sync_committee_proto::into_proto(
                                    epoch_change_update.sync_committee,
                                )),
                                next_sync_committee: Some(sync_committee_proto::into_proto(
                                    epoch_change_update.next_sync_committee,
                                )),
                                next_sync_committee_branch: epoch_change_update
                                    .next_sync_committee_branch
                                    .into_iter()
                                    .map(Into::into)
                                    .collect(),
                                update_data: Some(epoch_change_update.update_data.into()),
                            },
                        )
                    }
                    LightClientUpdate::WithinEpoch(within_epoch_update) => {
                        light_client_update::Update::WithinEpochUpdate(
                            protos::union::ibc::lightclients::ethereum::v1::WithinEpochUpdate {
                                sync_committee: Some(sync_committee_proto::into_proto(
                                    within_epoch_update.sync_committee,
                                )),
                                update_data: Some(within_epoch_update.update_data.into()),
                            },
                        )
                    }
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
                ) => Self::EpochChange(Box::new(EpochChangeUpdate {
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
                })),
                light_client_update::Update::WithinEpochUpdate(
                    protos::union::ibc::lightclients::ethereum::v1::WithinEpochUpdate {
                        sync_committee,
                        update_data,
                    },
                ) => Self::WithinEpoch(Box::new(WithinEpochUpdate {
                    sync_committee: sync_committee_proto::try_from_proto(required!(
                        sync_committee
                    )?)
                    .unwrap(),
                    update_data: required!(update_data)?.try_into().unwrap(),
                })),
            })
        }
    }
}
