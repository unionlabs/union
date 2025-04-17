use cometbft_types::types::{
    block_id::BlockId, canonical_block_id::CanonicalBlockId,
    canonical_part_set_header::CanonicalPartSetHeader, canonical_vote::CanonicalVote,
    commit::Commit, commit_sig::CommitSig, signed_header::SignedHeader,
    signed_msg_type::SignedMsgType, simple_validator::SimpleValidator, validator::Validator,
    validator_set::ValidatorSet,
};
use prost::Message;
use unionlabs::{
    encoding::{EncodeAs, Proto},
    google::protobuf::{duration::Duration, timestamp::Timestamp},
    primitives::{H160, H256},
};

use crate::{error::Error, merkle::calculate_merkle_root};

/// Calculate the [`CanonicalVote`] bytes for the provided CommitSig.
///
/// # Errors
///
/// This function assumes that `commit_sig` is [`CommitSig::Commit`], and will fail otherwise.
pub fn canonical_vote_bytes(
    commit: &Commit,
    commit_sig: &CommitSig,
    timestamp: &Timestamp,
    chain_id: &str,
) -> Result<Vec<u8>, Error> {
    // TODO: Instead of erroring below with `Error::MissingBlockIdHash`, we should instead return an error *here* that says only `CommitSig::Commit` will have a valid canonical vote.
    let block_id = match commit_sig {
        CommitSig::Absent => BlockId::default(),
        CommitSig::Commit { .. } => commit.block_id.clone(),
        CommitSig::Nil { .. } => BlockId::default(),
    };

    Ok(
        protos::tendermint::types::CanonicalVote::from(CanonicalVote {
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
pub fn get_validator_by_address<'a>(
    vals: &'a ValidatorSet,
    address: &H160,
) -> Option<(usize, &'a Validator)> {
    vals.validators
        .iter()
        .enumerate()
        .find(|(_, val)| &val.address == address)
}

#[must_use]
pub fn validators_hash(validator_set: &ValidatorSet) -> H256 {
    let raw_validators = validator_set
        .validators
        .iter()
        .map(|validator| {
            SimpleValidator {
                pub_key: validator.pub_key.clone(),
                voting_power: validator.voting_power.inner(),
            }
            .encode_as::<Proto>()
        })
        .collect::<Vec<_>>();

    calculate_merkle_root(&raw_validators)
}

#[must_use]
pub fn header_expired(h: &SignedHeader, trusting_period: Duration, now: Timestamp) -> bool {
    let Some(expiration_time) = h.header.time.checked_add(trusting_period) else {
        return false;
    };

    expiration_time <= now
}
