use beacon_api_types::SyncAggregate;
use unionlabs::errors::InvalidLength;

fn from(value: SyncAggregate) -> Self {
    Self {
        sync_committee_bits: value.sync_committee_bits.into_bytes().into_vec(),
        sync_committee_signature: value.sync_committee_signature.into_bytes().into(),
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum Error {
    #[error("invalid `sync_committee_signature`")]
    SyncCommitteeSignature(#[from] InvalidLength),
}

pub fn try_from_proto(
    value: protos::union::ibc::lightclients::ethereum::v1::SyncAggregate,
) -> Result<SyncAggregate, Error> {
    Ok(
        protos::union::ibc::lightclients::ethereum::v1::SyncAggregate {
            sync_committee_bits: value.sync_committee_bits,
            sync_committee_signature: value
                .sync_committee_signature
                .try_into()
                .map_err(Error::SyncCommitteeSignature)?,
        },
    )
}
