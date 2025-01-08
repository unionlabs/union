use unionlabs::bls::BlsPublicKey;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SyncCommittee {
    pub pubkeys: Vec<BlsPublicKey>,
    pub aggregate_pubkey: BlsPublicKey,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, ::ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct SyncCommitteeSsz<C: crate::SYNC_COMMITTEE_SIZE> {
    pub pubkeys: ::ssz::types::Vector<BlsPublicKey, C::SYNC_COMMITTEE_SIZE>,
    pub aggregate_pubkey: BlsPublicKey,
}

#[cfg(feature = "ssz")]
pub mod ssz {
    use typenum::Unsigned;
    use unionlabs::errors::{ExpectedLength, InvalidLength};

    use super::*;
    use crate::SYNC_COMMITTEE_SIZE;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("invalid pubkeys")]
        Pubkeys(#[from] InvalidLength),
    }

    impl<C: SYNC_COMMITTEE_SIZE> TryFrom<SyncCommittee> for SyncCommitteeSsz<C> {
        type Error = Error;

        fn try_from(value: SyncCommittee) -> Result<Self, Self::Error> {
            Ok(Self {
                pubkeys: value.pubkeys.try_into().map_err(|v: Vec<_>| {
                    Error::Pubkeys(InvalidLength {
                        expected: ExpectedLength::Exact(C::SYNC_COMMITTEE_SIZE::USIZE),
                        found: v.len(),
                    })
                })?,
                aggregate_pubkey: value.aggregate_pubkey,
            })
        }
    }
}
