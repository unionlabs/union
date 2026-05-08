use cometbft_types::types::{
    block_id::BlockId, canonical_block_id::CanonicalBlockId,
    canonical_part_set_header::CanonicalPartSetHeader, canonical_vote::CanonicalVote,
    commit::Commit, commit_sig::CommitSig, signed_header::SignedHeader,
    signed_msg_type::SignedMsgType, simple_validator::SimpleValidator, validator::Validator,
    validator_set::ValidatorSet,
};
use gno_types::Validator;
use prost::Message;
use unionlabs::{
    encoding::{EncodeAs, Proto},
    google::protobuf::{duration::Duration, timestamp::Timestamp},
    primitives::{H160, H256},
};

use crate::{error::Error, merkle::calculate_merkle_root};

pub(crate) fn canonical_vote(
    commit: &Commit,
    commit_sig: &CommitSig,
    timestamp: &Timestamp,
    chain_id: &str,
) -> Result<Vec<u8>, Error> {
    let block_id = match commit_sig {
        CommitSig::Absent => BlockId::default(),
        CommitSig::Commit { .. } => commit.block_id.clone(),
        CommitSig::Nil { .. } => BlockId::default(),
    };

    Ok(
        Into::<protos::tendermint::types::CanonicalVote>::into(CanonicalVote {
            ty: SignedMsgType::Precommit,
            height: commit.height,
            // roundabout way to go from i32 >= 0 to i64 >= 0
            round: i64::from(commit.round.inner())
                .try_into()
                .expect("value is bounded >= 0; qed;"), // SAFE because within the bounds
            block_id: CanonicalBlockId {
                hash: block_id.hash.ok_or(Error::MissingBlockIdHash)?,
                part_set_header: CanonicalPartSetHeader {
                    total: block_id.part_set_header.total,
                    hash: block_id
                        .part_set_header
                        .hash
                        .ok_or(Error::MissingBlockIdHash)?,
                },
            },
            chain_id: chain_id.to_string(),
            timestamp: *timestamp,
        })
        .encode_length_delimited_to_vec(),
    )
}

#[must_use]
pub(crate) fn get_validator_by_address<'a>(
    vals: &'a ValidatorSet,
    address: &H160,
) -> Option<(usize, &'a Validator)> {
    vals.validators
        .iter()
        .enumerate()
        .find(|(_, val)| &val.address == address)
}

/// Hash returns the Merkle root hash build using validators (as leaves) in the set.
///
/// Source: <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/bft/types/validator_set.go#L309>
#[must_use]
pub fn validators_hash(validator_set: &ValidatorSet) -> H256 {
    let raw_validators = validator_set
        .validators
        .iter()
        .map(validator_bytes)
        .collect::<Vec<_>>();

    calculate_merkle_root(&raw_validators)
}

/// Bytes computes the unique encoding of a validator with a given voting power.
/// These are the bytes that gets hashed in consensus. It excludes address
/// as its redundant with the pubkey. This also excludes ProposerPriority
/// which changes every round.
///
/// Source: <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/bft/types/validator.go#L88>
pub fn validator_bytes(validator: &Validator) -> Bytes {
    todo!()
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
