#[derive(Debug, Clone, PartialEq)]
pub struct SyncAggregate {
    // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub sync_committee_bits: Vec<u8>,
    // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub sync_committee_signature: Vec<u8>,
}

impl From<SyncAggregate> for protos::union::ibc::lightclients::ethereum::v1::SyncAggregate {
    fn from(value: SyncAggregate) -> Self {
        Self {
            sync_committee_bits: value.sync_committee_bits,
            sync_committee_signature: value.sync_committee_signature,
        }
    }
}
