use gno_types::{SignedHeader, Validator, ValidatorSet};
use unionlabs::{
    google::protobuf::{duration::Duration, timestamp::Timestamp},
    primitives::H256,
};

use crate::merkle::calculate_merkle_root;

/// Hash returns the Merkle root hash build using validators (as leaves) in the set.
///
/// Source: <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/bft/types/validator_set.go#L309>
#[must_use]
pub fn validators_hash(validator_set: &ValidatorSet) -> H256 {
    let raw_validators = validator_set
        .validators
        .iter()
        .map(Validator::bytes)
        .collect::<Vec<_>>();

    calculate_merkle_root(&raw_validators)
}

/// HeaderExpired return true if the given header expired.
///
/// Source: <https://github.com/atomone-hub/atomone/blob/5e3a5d733d818c1fd3d8b08aac9baf329737d27d/modules/10-gno/verifier.go#L268>
pub fn header_expired(h: &SignedHeader, trusting_period: Duration, now: Timestamp) -> bool {
    let Some(expiration_time) = h.header.time.checked_add(trusting_period) else {
        return false;
    };

    expiration_time <= now
}
