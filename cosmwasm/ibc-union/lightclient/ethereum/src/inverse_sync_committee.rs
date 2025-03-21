use ark_bls12_381::G1Affine;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use beacon_api_types::altair::SyncCommittee;
use unionlabs::primitives::H384;

/// Sync committee where all the public keys are changed to additive inverses.
#[derive(Debug, Clone, PartialEq, Eq, Default, bincode::Encode, bincode::Decode)]
pub struct InverseSyncCommittee {
    pub pubkeys: Vec<H384>,
    pub aggregate_pubkey: H384,
}

impl InverseSyncCommittee {
    /// Additive inverse the public keys of the sync committee to do inverse aggregation
    pub fn take_inverse(sync_committee: &SyncCommittee) -> InverseSyncCommittee {
        InverseSyncCommittee {
            pubkeys: sync_committee
                .pubkeys
                .iter()
                .map(|x| {
                    -G1Affine::deserialize_compressed(x.as_ref())
                        .expect("pubkey that is validated by the sync protocol is valid")
                })
                .map(|x| {
                    let mut buf = vec![];
                    x.serialize_compressed(&mut buf)
                        .expect("serializing into a buffer will always work");
                    buf.try_into().expect("compressed data first H384")
                })
                .collect(),
            aggregate_pubkey: sync_committee.aggregate_pubkey,
        }
    }

    /// Do a basic cast from `InverseSyncCommittee` to `SyncCommittee`
    /// Note that this function does not inverse the inverse public keys, just does a cast.
    pub fn as_sync_committee(self) -> SyncCommittee {
        SyncCommittee {
            pubkeys: self.pubkeys,
            aggregate_pubkey: self.aggregate_pubkey,
        }
    }
}
