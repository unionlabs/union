use crate::{
    lightclients::ethereum::{
        account_update::AccountUpdate, light_client_update::LightClientUpdate,
        trusted_sync_committee::TrustedSyncCommittee,
    },
    IntoProto, TypeUrl,
};

#[derive(Debug)]
pub struct Header {
    pub trusted_sync_committee: TrustedSyncCommittee,
    pub consensus_update: LightClientUpdate,
    pub account_update: AccountUpdate,
    pub timestamp: u64,
}

impl From<Header> for protos::union::ibc::lightclients::ethereum::v1::Header {
    fn from(value: Header) -> Self {
        Self {
            trusted_sync_committee: Some(value.trusted_sync_committee.into()),
            consensus_update: Some(value.consensus_update.into()),
            account_update: Some(value.account_update.into()),
            timestamp: value.timestamp,
        }
    }
}

impl IntoProto for Header {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::Header;
}

impl TypeUrl for protos::union::ibc::lightclients::ethereum::v1::Header {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.ethereum.v1.Header";
}
