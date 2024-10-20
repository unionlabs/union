use macros::model;

use crate::{
    errors::{required, MissingField},
    ethereum::config::{BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE},
    ibc::lightclients::ethereum::{
        account_update::{AccountUpdate, TryFromAccountUpdateError},
        light_client_update::{
            LightClientUpdate, TryFromLightClientUpdateError, UnboundedLightClientUpdate,
        },
        trusted_sync_committee::{
            TrustedSyncCommittee, TryFromTrustedSyncCommitteeError, UnboundedTrustedSyncCommittee,
        },
    },
};

#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::Header),
    into,
    from
))]
#[serde(bound(serialize = "", deserialize = ""))]
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

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromHeaderError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid `trusted_sync_committee`")]
    TrustedSyncCommittee(#[from] TryFromTrustedSyncCommitteeError),
    #[error("invalid `consensus_update`")]
    ConsensusUpdate(#[from] TryFromLightClientUpdateError),
    #[error("invalid `account_update`")]
    AccountUpdate(#[from] TryFromAccountUpdateError),
}

impl<C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES>
    TryFrom<protos::union::ibc::lightclients::ethereum::v1::Header> for Header<C>
{
    type Error = TryFromHeaderError;

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

#[model(proto(raw(protos::union::ibc::lightclients::ethereum::v1::Header), from))]
pub struct UnboundedHeader {
    pub trusted_sync_committee: UnboundedTrustedSyncCommittee,
    pub consensus_update: UnboundedLightClientUpdate,
    pub account_update: AccountUpdate,
}

impl From<UnboundedHeader> for protos::union::ibc::lightclients::ethereum::v1::Header {
    fn from(value: UnboundedHeader) -> Self {
        Self {
            trusted_sync_committee: Some(value.trusted_sync_committee.into()),
            consensus_update: Some(value.consensus_update.into()),
            account_update: Some(value.account_update.into()),
        }
    }
}
