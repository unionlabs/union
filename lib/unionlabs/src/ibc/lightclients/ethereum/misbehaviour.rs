use serde::{Deserialize, Serialize};

use super::{
    light_client_update::{LightClientUpdate, TryFromLightClientUpdateError},
    trusted_sync_committee::TrustedSyncCommittee,
};
use crate::{
    errors::{required, MissingField},
    ethereum::config::{BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE},
    Proto, TryFromProtoErrorOf, TypeUrl,
};

// trait alias would be nice
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Misbehaviour<C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> {
    pub trusted_sync_committee: TrustedSyncCommittee<C>,
    pub update_1: LightClientUpdate<C>,
    pub update_2: LightClientUpdate<C>,
}

impl<C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> From<Misbehaviour<C>>
    for protos::union::ibc::lightclients::ethereum::v1::Misbehaviour
{
    fn from(value: Misbehaviour<C>) -> Self {
        Self {
            trusted_sync_committee: Some(value.trusted_sync_committee.into()),
            update_1: Some(value.update_1.into()),
            update_2: Some(value.update_2.into()),
        }
    }
}

#[derive(Debug)]
pub enum TryFromMisbehaviourError<
    C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES,
> {
    MissingField(MissingField),
    TrustedSyncCommittee(TryFromProtoErrorOf<TrustedSyncCommittee<C>>),
    Update1(TryFromLightClientUpdateError<C>),
    Update2(TryFromLightClientUpdateError<C>),
}

impl<C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES>
    TryFrom<protos::union::ibc::lightclients::ethereum::v1::Misbehaviour> for Misbehaviour<C>
{
    type Error = TryFromMisbehaviourError<C>;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::Misbehaviour,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            trusted_sync_committee: required!(value.trusted_sync_committee)?
                .try_into()
                .map_err(TryFromMisbehaviourError::TrustedSyncCommittee)?,
            update_1: required!(value.update_1)?
                .try_into()
                .map_err(TryFromMisbehaviourError::Update1)?,
            update_2: required!(value.update_2)?
                .try_into()
                .map_err(TryFromMisbehaviourError::Update2)?,
        })
    }
}

impl<C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> Proto
    for Misbehaviour<C>
{
    type Proto = protos::union::ibc::lightclients::ethereum::v1::Misbehaviour;
}

impl TypeUrl for protos::union::ibc::lightclients::ethereum::v1::Misbehaviour {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.ethereum.v1.Misbehaviour";
}
