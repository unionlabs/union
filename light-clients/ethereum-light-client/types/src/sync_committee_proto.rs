use unionlabs::errors::InvalidLength;

pub fn try_from_proto(
    value: protos::union::ibc::lightclients::ethereum::v1::SyncCommittee,
) -> Result<SyncCommittee, Error> {
    Ok(Self {
        pubkeys: value
            .pubkeys
            .iter()
            .cloned()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::PubKey)?,
        aggregate_pubkey: value
            .aggregate_pubkey
            .try_into()
            .map_err(Error::AggregatePubKey)?,
    })
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum Error {
    #[error("invalid `pubkeys`")]
    PubKey(#[source] InvalidLength),
    #[error("invalid `aggregate_pubkey`")]
    AggregatePubKey(#[source] InvalidLength),
}

pub fn into_proto(
    value: SyncCommittee,
) -> protos::union::ibc::lightclients::ethereum::v1::SyncCommittee {
    protos::union::ibc::lightclients::ethereum::v1::SyncCommittee {
        pubkeys: value.pubkeys.iter().copied().map(Into::into).collect(),
        aggregate_pubkey: value.aggregate_pubkey.into(),
    }
}
