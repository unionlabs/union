use beacon_api_types::SyncCommittee;
use unionlabs::ibc::core::client::height::Height;

use crate::LightClientUpdate;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Misbehaviour {
    pub sync_committee: SyncCommittee,
    pub trusted_height: Height,
    pub update_1: LightClientUpdate,
    pub update_2: LightClientUpdate,
}

#[cfg(feature = "proto")]
mod proto {
    use unionlabs::{errors::MissingField, impl_proto_via_try_from_into, required};

    use crate::{light_client_update, sync_committee_proto, Misbehaviour};

    impl_proto_via_try_from_into!(Misbehaviour => protos::union::ibc::lightclients::ethereum::v1::Misbehaviour);

    impl From<Misbehaviour> for protos::union::ibc::lightclients::ethereum::v1::Misbehaviour {
        fn from(value: Misbehaviour) -> Self {
            Self {
                current_sync_committee: Some(sync_committee_proto::into_proto(
                    value.sync_committee,
                )),
                trusted_height: Some(value.trusted_height.into()),
                update_1: Some(value.update_1.into()),
                update_2: Some(value.update_2.into()),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromMisbehaviourError {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid sync committee")]
        SyncCommittee(#[source] sync_committee_proto::Error),
        #[error("invalid update1")]
        Update1(#[source] light_client_update::proto::Error),
        #[error("invalid update2")]
        Update2(#[source] light_client_update::proto::Error),
    }

    impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::Misbehaviour> for Misbehaviour {
        type Error = TryFromMisbehaviourError;

        fn try_from(
            value: protos::union::ibc::lightclients::ethereum::v1::Misbehaviour,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                sync_committee: sync_committee_proto::try_from_proto(required!(
                    value.current_sync_committee
                )?)
                .map_err(TryFromMisbehaviourError::SyncCommittee)?,
                trusted_height: required!(value.trusted_height)?.into(),
                update_1: required!(value.update_1)?
                    .try_into()
                    .map_err(TryFromMisbehaviourError::Update1)?,
                update_2: required!(value.update_2)?
                    .try_into()
                    .map_err(TryFromMisbehaviourError::Update2)?,
            })
        }
    }
}
