use prost::Message;
use unionlabs::{
    google::protobuf::{duration::Duration, timestamp::Timestamp},
    hash::{H160, H256},
    tendermint::types::{
        block_id::BlockId, canonical_block_header::CanonicalPartSetHeader,
        canonical_block_id::CanonicalBlockId, commit::Commit, commit_sig::CommitSig,
        legacy_canonical_vote::LegacyCanonicalVote as CanonicalVote, signed_header::SignedHeader,
        signed_msg_type::SignedMsgType, simple_validator::SimpleValidator, validator::Validator,
        validator_set::ValidatorSet,
    },
    IntoProto,
};

use crate::merkle::calculate_merkle_root;

#[must_use]
pub(crate) fn canonical_vote(
    commit: &Commit,
    commit_sig: &CommitSig,
    timestamp: &Timestamp,
    chain_id: &str,
) -> Vec<u8> {
    let block_id = match commit_sig {
        CommitSig::Absent => BlockId::default(),
        CommitSig::Commit { .. } => commit.block_id.clone(),
        CommitSig::Nil { .. } => BlockId::default(),
    };

    Into::<protos::tendermint::types::LegacyCanonicalVote>::into(CanonicalVote {
        ty: SignedMsgType::Precommit,
        height: commit.height,
        round: (commit.round.inner() as i64)
            .try_into()
            .expect("impossible"), // SAFE because within the bounds
        block_id: CanonicalBlockId {
            hash: block_id.hash.clone(),
            part_set_header: CanonicalPartSetHeader {
                total: block_id.part_set_header.total,
                hash: block_id.part_set_header.hash.clone(),
            },
        },
        chain_id: chain_id.to_string(),
        timestamp: timestamp.clone(),
    })
    .encode_length_delimited_to_vec()
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

#[must_use]
pub fn validators_hash(vals: &ValidatorSet) -> H256 {
    let raw_validators: Vec<Vec<u8>> = vals
        .validators
        .iter()
        .map(|validator| SimpleValidator::from(validator.clone()).into_proto_bytes())
        .collect();

    calculate_merkle_root(&raw_validators)
}

#[must_use]
pub fn header_expired(h: &SignedHeader, trusting_period: Duration, now: Timestamp) -> bool {
    let Some(expiration_time) = h.header.time.checked_add(trusting_period) else {
        return false;
    };

    expiration_time <= now
}
