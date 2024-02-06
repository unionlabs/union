use std::collections::BTreeMap;

use merkle::calculate_merkle_root;
use prost::Message;
use unionlabs::{
    cosmos::crypto::AnyPubKey,
    google::protobuf::{duration::Duration, timestamp::Timestamp},
    hash::{H160, H256, H512},
    ibc::lightclients::tendermint::fraction::Fraction,
    tendermint::{
        crypto::public_key::PublicKey,
        types::{
            block_id::BlockId, canonical_block_header::CanonicalPartSetHeader,
            canonical_block_id::CanonicalBlockId, commit::Commit, commit_sig::CommitSig,
            legacy_canonical_vote::LegacyCanonicalVote as CanonicalVote,
            signed_header::SignedHeader, signed_msg_type::SignedMsgType,
            simple_validator::SimpleValidator, validator::Validator, validator_set::ValidatorSet,
            vote::Vote,
        },
    },
    IntoProto,
};

pub const BATCH_VERIFY_THRESHOLD: usize = 2;

pub mod merkle;

pub trait SignatureVerifier {
    fn verify_signature(pubkey: &PublicKey, msg: &[u8], sig: &[u8]) -> bool;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("integer overflow")]
    IntegerOverflow,
    #[error("divide by 0")]
    DivideByZero,
    #[error("headers must be adjacent")]
    HeadersMustBeAdjacent,
    #[error("header with the timestamp ({header_timestamp}) is expired (trusting period {trusting_period})")]
    HeaderExpired {
        trusting_period: Duration,
        header_timestamp: Timestamp,
    },
    #[error("untrusted ({untrusted_header_chain_id}) and trusted header ({trusted_header_chain_id}) chain id mismatch")]
    ChainIdMismatch {
        untrusted_header_chain_id: String,
        trusted_header_chain_id: String,
    },
    #[error("trusted header height ({untrusted_header_height}) cannot be greater than or equal to the untrusted height ({untrusted_header_height})")]
    UntrustedHeaderHeightIsSmaller {
        untrusted_header_height: i64,
        trusted_header_height: i64,
    },
    #[error("trusted header timestamp ({untrusted_header_timestamp}) cannot be greater than or equal to the untrusted timestamp ({untrusted_header_timestamp})")]
    UntrustedHeaderTimestampIsSmaller {
        untrusted_header_timestamp: Timestamp,
        trusted_header_timestamp: Timestamp,
    },
    #[error("expected the untrusted validator set to match the validators hash")]
    UntrustedValidatorSetMismatch,
    #[error("invalid index ({index}) while getting a validator with len ({val_len})")]
    InvalidIndexInValidatorSet { index: usize, val_len: usize },
    #[error("double vote from ({0})")]
    DoubleVote(H160),
    #[error("not enough voting power, have ({have}), need ({need})")]
    NotEnoughVotingPower { have: u64, need: u64 },
    #[error("signature cannot be verified")]
    SignatureVerification,
}

pub fn verify<V: SignatureVerifier>(
    trusted_header: &SignedHeader,
    trusted_vals: &ValidatorSet,     // height=X or height=X+1
    untrusted_header: &SignedHeader, // height=Y
    untrusted_vals: &ValidatorSet,   // height=Y
    // TODO(aeryz): Is this duration supposed to be proto?
    trusting_period: Duration,
    // TODO(aeryz): Until we define a Time type
    now: Timestamp,
    // TODO(aeryz): Is this duration supposed to be proto?
    max_clock_drift: Duration,
    trust_level: Fraction,
) -> Result<(), Error> {
    if untrusted_header.header.height.inner()
        != trusted_header
            .header
            .height
            .inner()
            .checked_add(1)
            .ok_or(Error::IntegerOverflow)?
    {
        verify_non_adjacent::<V>(
            trusted_header,
            trusted_vals,
            untrusted_header,
            untrusted_vals,
            trusting_period,
            now,
            max_clock_drift,
            trust_level,
        )
    } else {
        verify_adjacent(
            trusted_header,
            untrusted_header,
            untrusted_vals,
            trusting_period,
            now,
            max_clock_drift,
        )
    }
}

pub fn verify_non_adjacent<V: SignatureVerifier>(
    trusted_header: &SignedHeader,
    trusted_vals: &ValidatorSet,     // height=X or height=X+1
    untrusted_header: &SignedHeader, // height=Y
    untrusted_vals: &ValidatorSet,   // height=Y
    // TODO(aeryz): Is this duration supposed to be proto?
    trusting_period: Duration,
    // TODO(aeryz): Until we define a Time type
    now: Timestamp,
    // TODO(aeryz): Is this duration supposed to be proto?
    max_clock_drift: Duration,
    trust_level: Fraction,
) -> Result<(), Error> {
    if untrusted_header.header.height.inner()
        == trusted_header
            .header
            .height
            .inner()
            .checked_add(1)
            .ok_or(Error::IntegerOverflow)?
    {
        return Err(Error::HeadersMustBeAdjacent);
    }

    if header_expired(trusted_header, trusting_period, now) {
        return Err(Error::HeaderExpired {
            trusting_period,
            header_timestamp: trusted_header.header.time,
        });
    }

    verify_new_headers_and_vals(
        untrusted_header,
        untrusted_vals,
        trusted_header,
        now,
        max_clock_drift,
    )?;

    verify_commit_light_trusting::<V>(
        &trusted_header.header.chain_id,
        trusted_vals,
        &untrusted_header.commit,
        trust_level,
        // TODO(aeryz): make this internal
        false,
    )?;

    verify_commit_light(
        &trusted_header.header.chain_id,
        &untrusted_header.commit.block_id,
        untrusted_header.header.height.inner(),
        &untrusted_header.commit,
    )?;

    Ok(())
}

pub fn verify_commit_light(
    chain_id: &str,
    block_id: &BlockId,
    height: i64,
    commit: &Commit,
) -> Result<(), Error> {
    Ok(())
}

pub fn verify_commit_light_trusting<V: SignatureVerifier>(
    chain_id: &str,
    vals: &ValidatorSet,
    commit: &Commit,
    trust_level: Fraction,
    count_all_signatures: bool,
) -> Result<(), Error> {
    // TODO(aeryz): cometbft recalculates it if this is 0, why?
    // SAFETY: as u64 is safe here since we do `abs` which makes it always positive
    let total_voting_power_mul_by_numerator = (vals.total_voting_power.abs() as u64)
        .checked_mul(trust_level.numerator)
        .ok_or(Error::IntegerOverflow)?;
    let voting_power_needed = total_voting_power_mul_by_numerator
        .checked_div(trust_level.denominator)
        .ok_or(Error::DivideByZero)?;

    // ignore all commit signatures that are not for the block
    let ignore =
        |commit_sig: &CommitSig| -> bool { matches!(commit_sig, CommitSig::Commit { .. }) };

    let filter_commit = |commit_sig: &CommitSig| -> Option<(H160, Timestamp, H512)> {
        match commit_sig {
            CommitSig::Commit {
                validator_address,
                timestamp,
                signature,
            } => Some((
                validator_address.clone(),
                timestamp.clone(),
                signature.clone(),
            )),
            _ => None,
        }
    };

    // attempt to batch verify commit. As the validator set doesn't necessarily
    // correspond with the validator set that signed the block we need to look
    // up by address rather than index.
    if should_batch_verify(vals, commit) {
        println!("asdasd");
        verify_commit_batch(
            chain_id,
            vals,
            commit,
            voting_power_needed,
            filter_commit,
            |_| true,
            count_all_signatures,
            false,
        )
    } else {
        verify_commit_single::<V>(
            chain_id,
            vals,
            commit,
            voting_power_needed,
            filter_commit,
            |_| true,
            count_all_signatures,
            false,
        )
    }
}

fn verify_commit_batch(
    _chain_id: &str,
    _vals: &ValidatorSet,
    _commit: &Commit,
    _voting_power_needed: u64,
    _filter_commit: fn(&CommitSig) -> Option<(H160, Timestamp, H512)>,
    _count_sig: fn(&CommitSig) -> bool,
    _count_all_signatures: bool,
    _lookup_by_index: bool,
) -> Result<(), Error> {
    Ok(())
}

fn verify_commit_single<V: SignatureVerifier>(
    chain_id: &str,
    vals: &ValidatorSet,
    commit: &Commit,
    voting_power_needed: u64,
    filter_commit: fn(&CommitSig) -> Option<(H160, Timestamp, H512)>,
    count_sig: fn(&CommitSig) -> bool,
    count_all_signatures: bool,
    lookup_by_index: bool,
) -> Result<(), Error> {
    let mut seen_vals: BTreeMap<usize, usize> = BTreeMap::new();
    let mut tallied_voting_power: u64 = 0;
    for (i, commit_sig) in commit.signatures.iter().enumerate() {
        let Some((validator_address, timestamp, signature)) = filter_commit(commit_sig) else {
            continue;
        };

        // TODO(aeryz): commit_sig.ValidateBasic()
        let val = if lookup_by_index {
            vals.validators
                .get(i)
                .ok_or(Error::InvalidIndexInValidatorSet {
                    index: i,
                    val_len: vals.validators.len(),
                })?
        } else {
            let Some((val_idx, val)) = get_validator_by_address(vals, &validator_address) else {
                continue;
            };

            if seen_vals.get(&val_idx).is_some() {
                return Err(Error::DoubleVote(validator_address));
            }

            seen_vals.insert(val_idx, i);

            val
        };

        let vote_sign_bytes = canonical_vote(commit, commit_sig, &timestamp, chain_id);

        if !V::verify_signature(&val.pub_key, &vote_sign_bytes, signature.as_ref()) {
            return Err(Error::SignatureVerification);
        }

        // If this signature counts then add the voting power of the validator
        // to the tally
        if count_sig(commit_sig) {
            tallied_voting_power += val.voting_power.inner() as u64; // SAFE because within the bounds
        }

        if !count_all_signatures && tallied_voting_power > voting_power_needed {
            return Ok(());
        }
    }

    if tallied_voting_power <= voting_power_needed {
        Err(Error::NotEnoughVotingPower {
            have: tallied_voting_power,
            need: voting_power_needed,
        })
    } else {
        Ok(())
    }
}

fn canonical_vote(
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

fn get_validator_by_address<'a>(
    vals: &'a ValidatorSet,
    address: &H160,
) -> Option<(usize, &'a Validator)> {
    vals.validators
        .iter()
        .enumerate()
        .find(|(_, val)| &val.address == address)
}

// TODO(aeryz): check if we need to implement `supportsBatchVerify`
fn should_batch_verify(vals: &ValidatorSet, commit: &Commit) -> bool {
    // TODO(aeryz): remove
    // commit.signatures.len() >= BATCH_VERIFY_THRESHOLD
    false
}

fn verify_new_headers_and_vals(
    untrusted_header: &SignedHeader, // height=Y
    untrusted_vals: &ValidatorSet,   // height=Y
    trusted_header: &SignedHeader,
    // TODO(aeryz): Until we define a Time type
    now: Timestamp,
    // TODO(aeryz): Is this duration supposed to be proto?
    max_clock_drift: Duration,
) -> Result<(), Error> {
    if untrusted_header.header.chain_id != trusted_header.header.chain_id {
        return Err(Error::ChainIdMismatch {
            untrusted_header_chain_id: untrusted_header.header.chain_id.clone(),
            trusted_header_chain_id: trusted_header.header.chain_id.clone(),
        });
    }

    // TODO(aeryz): the original implementation checks whether `sh.Commit.Height == sh.Header.Height` here
    // should we do it in `unionlabs`?
    // it also does hash check. We should check out when we want to do `ValidateBasic`

    if untrusted_header.header.height <= trusted_header.header.height {
        return Err(Error::UntrustedHeaderHeightIsSmaller {
            untrusted_header_height: untrusted_header.header.height.inner(),
            trusted_header_height: trusted_header.header.height.inner(),
        });
    }

    if untrusted_header.header.time <= trusted_header.header.time {
        return Err(Error::UntrustedHeaderTimestampIsSmaller {
            untrusted_header_timestamp: untrusted_header.header.time,
            trusted_header_timestamp: trusted_header.header.time,
        });
    }

    // TODO(aeryz): time + duration math
    // if !untrustedHeader.Time.Before(now.Add(maxClockDrift)) {
    // 	return fmt.Errorf("new header has a time from the future %v (now: %v; max clock drift: %v)",
    // 		untrustedHeader.Time,
    // 		now,
    // 		maxClockDrift)
    // }

    if untrusted_header.header.validators_hash != validators_hash(untrusted_vals) {
        return Err(Error::UntrustedValidatorSetMismatch);
    }

    Ok(())
}

fn validators_hash(vals: &ValidatorSet) -> H256 {
    let raw_validators: Vec<Vec<u8>> = vals
        .validators
        .iter()
        .map(|validator| SimpleValidator::from(validator.clone()).into_proto_bytes())
        .collect();

    calculate_merkle_root(&raw_validators)
}

pub fn header_expired(_h: &SignedHeader, _trusting_period: Duration, _now: Timestamp) -> bool {
    // TODO(aeryz): Implement
    false
}

pub fn verify_adjacent(
    _trusted_header: &SignedHeader,
    _untrusted_header: &SignedHeader, // height=Y
    _untrusted_vals: &ValidatorSet,   // height=Y
    // TODO(aeryz): Is this duration supposed to be proto?
    _trusting_period: Duration,
    // TODO(aeryz): Until we define a Time type
    _now: Timestamp,
    // TODO(aeryz): Is this duration supposed to be proto?
    _max_clock_drift: Duration,
) -> Result<(), Error> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use unionlabs::{
        ibc::lightclients::tendermint::header::Header,
        tendermint::types::part_set_header::PartSetHeader,
    };

    use super::*;

    struct EdVerifier;
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    impl SignatureVerifier for EdVerifier {
        fn verify_signature(pubkey: &PublicKey, msg: &[u8], sig: &[u8]) -> bool {
            let PublicKey::Ed25519(pubkey) = pubkey else {
                panic!("invalid pubkey");
            };
            let key: VerifyingKey =
                VerifyingKey::from_bytes(pubkey.as_slice().try_into().unwrap()).unwrap();
            let signature: Signature = Signature::from_bytes(sig.try_into().unwrap());
            key.verify(msg, &signature).is_ok()
        }
    }

    #[test]
    fn verify_works() {
        let initial_header: Header = serde_json::from_str(include_str!("test/288.json")).unwrap();
        let update_header: Header = serde_json::from_str(include_str!("test/291.json")).unwrap();

        verify::<EdVerifier>(
            &initial_header.signed_header,
            &initial_header.validator_set,
            &update_header.signed_header,
            &update_header.validator_set,
            Duration::new(315576000000, 0).unwrap(),
            update_header.signed_header.header.time,
            Duration::new(315576000000, 0).unwrap(),
            Fraction {
                numerator: 1,
                denominator: 3,
            },
        )
        .unwrap();
    }

    #[test]
    fn canonical_vote() {
        let vote: protos::tendermint::types::LegacyCanonicalVote =
            protos::tendermint::types::LegacyCanonicalVote {
                r#type: 0,
                height: 0.try_into().unwrap(),
                round: 0.try_into().unwrap(),
                block_id: None,
                chain_id: "".to_string(),
                timestamp: None,
            };

        let delimited = vote.encode_length_delimited_to_vec();

        let canonical = protos::tendermint::types::LegacyCanonicalVote::decode_length_delimited(
            &[
                0xd_u8, 0x2a, 0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1,
            ][..],
        )
        .unwrap();

        println!("canonical: {:?}", canonical);

        println!("encoded: {:?}", canonical.encode_length_delimited_to_vec());

        // assert_eq!(
        //     delimited,
        //     vec![0xd, 0x2a, 0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1]
        // );
    }
}
