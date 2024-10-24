use cometbft_types::{
    crypto::public_key::PublicKey,
    types::{
        block_id::BlockId, canonical_block_header::CanonicalPartSetHeader,
        canonical_block_id::CanonicalBlockId, canonical_vote::CanonicalVote, commit::Commit,
        commit_sig::CommitSig, signed_header::SignedHeader, signed_msg_type::SignedMsgType,
        simple_validator::SimpleValidator, validator::Validator, validator_set::ValidatorSet,
    },
};
use prost::Message;
use unionlabs::{
    encoding::{EncodeAs, Proto},
    google::protobuf::{duration::Duration, timestamp::Timestamp},
    hash::{H160, H256},
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
        Into::<protos::tendermint::types::LegacyCanonicalVote>::into(CanonicalVote {
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

#[must_use]
pub fn validators_hash(validator_set: &ValidatorSet) -> H256 {
    let raw_validators = validator_set
        .validators
        .iter()
        .map(|validator| {
            SimpleValidator {
                pub_key: match &validator.pub_key {
                    // hackerman
                    // https://github.com/unionlabs/cometbls/issues/86
                    PublicKey::Bls12_381(key) => PublicKey::Bn254(key.clone()),
                    key => key.clone(),
                },
                voting_power: validator.voting_power.inner(),
            }
            .encode_as::<Proto>()
        })
        .collect::<Vec<_>>();

    calculate_merkle_root(&raw_validators)
}

#[must_use]
pub fn header_expired(_h: &SignedHeader, _trusting_period: Duration, _now: Timestamp) -> bool {
    // TODO: Re-enable
    // let Some(expiration_time) = h.header.time.checked_add(trusting_period) else {
    //     return false;
    // };

    // expiration_time <= now

    false
}
