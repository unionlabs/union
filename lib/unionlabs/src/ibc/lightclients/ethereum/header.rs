use serde::{Deserialize, Serialize};

use crate::{
    errors::{required, MissingField},
    ethereum::config::{BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE},
    ibc::lightclients::ethereum::{
        account_update::AccountUpdate, light_client_update::LightClientUpdate,
        trusted_sync_committee::TrustedSyncCommittee,
    },
    Proto, TryFromProtoErrorOf, TypeUrl,
};

// trait alias would be nice
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct Header<C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> {
    pub trusted_sync_committee: TrustedSyncCommittee<C>,
    pub consensus_update: LightClientUpdate<C>,
    pub account_update: AccountUpdate,
}

impl<C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> From<Header<C>>
    for protos::union::ibc::lightclients::ethereum::v1::Header
{
    fn from(value: Header<C>) -> Self {
        Self {
            trusted_sync_committee: Some(value.trusted_sync_committee.into()),
            consensus_update: Some(value.consensus_update.into()),
            account_update: Some(value.account_update.into()),
        }
    }
}

#[derive(Debug)]
pub enum TryFromHeaderError<C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> {
    MissingField(MissingField),
    TrustedSyncCommittee(TryFromProtoErrorOf<TrustedSyncCommittee<C>>),
    ConsensusUpdate(TryFromProtoErrorOf<LightClientUpdate<C>>),
    AccountUpdate(TryFromProtoErrorOf<AccountUpdate>),
}

impl<C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES>
    TryFrom<protos::union::ibc::lightclients::ethereum::v1::Header> for Header<C>
{
    type Error = TryFromHeaderError<C>;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::Header,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            trusted_sync_committee: required!(value.trusted_sync_committee)?
                .try_into()
                .map_err(TryFromHeaderError::TrustedSyncCommittee)?,
            consensus_update: required!(value.consensus_update)?
                .try_into()
                .map_err(TryFromHeaderError::ConsensusUpdate)?,
            account_update: required!(value.account_update)?
                .try_into()
                .map_err(TryFromHeaderError::AccountUpdate)?,
        })
    }
}

impl<C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> Proto for Header<C> {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::Header;
}

impl TypeUrl for protos::union::ibc::lightclients::ethereum::v1::Header {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.ethereum.v1.Header";
}
