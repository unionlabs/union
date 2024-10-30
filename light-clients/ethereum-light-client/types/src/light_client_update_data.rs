use beacon_api_types::{
    light_client_update::{FinalityBranch, NextSyncCommitteeBranch},
    LightClientHeader, SyncAggregate, SyncCommittee,
};

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

impl LightClientUpdateData {
    pub fn new_beacon_light_client_update(
        self,
        next_sync_committee: Option<SyncCommittee>,
        next_sync_committee_branch: Option<NextSyncCommitteeBranch>,
    ) -> beacon_api_types::LightClientUpdate {
        beacon_api_types::LightClientUpdate {
            attested_header: self.attested_header,
            next_sync_committee,
            next_sync_committee_branch,
            finalized_header: self.finalized_header,
            finality_branch: self.finality_branch,
            sync_aggregate: self.sync_aggregate,
            signature_slot: self.signature_slot,
        }
    }
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{errors::MissingField, required};

    use crate::{light_client_header_proto, sync_aggregate_proto, LightClientUpdateData};

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
                sync_aggregate: Some(sync_aggregate_proto::into_proto(value.sync_aggregate)),
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
