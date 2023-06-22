use crate::{core::client::height::Height, lightclients::ethereum::sync_committee::SyncCommittee};

#[derive(Debug)]
pub struct TrustedSyncCommittee {
    pub trusted_height: Height,
    pub sync_committee: SyncCommittee,
    pub is_next: bool,
}

impl From<TrustedSyncCommittee>
    for protos::union::ibc::lightclients::ethereum::v1::TrustedSyncCommittee
{
    fn from(value: TrustedSyncCommittee) -> Self {
        Self {
            trusted_height: Some(value.trusted_height.into()),
            sync_committee: Some(value.sync_committee.into()),
            is_next: value.is_next,
        }
    }
}
