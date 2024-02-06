use std::{collections::BTreeMap, fmt::Display};

use merkle::calculate_merkle_root;
use prost::Message;
use unionlabs::{
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
        },
    },
    IntoProto,
};

pub const BATCH_VERIFY_THRESHOLD: usize = 2;

pub mod merkle;

pub trait BatchSignatureVerifier {
    type Error: 'static + std::error::Error;
    /// Implementer should decide whether it's going to make sense to
    /// do batch verification based on how many signatures we have.
    fn should_batch_verify(signature_len: usize) -> bool;

    fn new() -> Self;

    fn add(
        &mut self,
        pubkey: &PublicKey,
        msg: Vec<u8>,
        signature: &[u8],
    ) -> Result<(), Self::Error>;

    fn verify_signature(&self) -> bool;
}

#[derive(Debug)]
pub struct BatchVerificationError;

impl Display for BatchVerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for BatchVerificationError {}

impl BatchSignatureVerifier for () {
    type Error = BatchVerificationError;

    fn should_batch_verify(_signature_len: usize) -> bool {
        false
    }

    fn new() -> Self {
        ()
    }

    fn add(
        &mut self,
        _pubkey: &PublicKey,
        _msg: Vec<u8>,
        _signature: &[u8],
    ) -> Result<(), Self::Error> {
        Err(BatchVerificationError)
    }

    fn verify_signature(&self) -> bool {
        false
    }
}

pub trait SignatureVerifier {
    fn verify_signature(pubkey: &PublicKey, msg: &[u8], sig: &[u8]) -> bool;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("integer overflow")]
    IntegerOverflow,
    #[error("divide by 0")]
    DivideByZero,
    #[error("headers must be non-adjacent")]
    HeadersMustBeNonAdjacent,
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
    #[error("max clock drift ({max_clock_drift:?}) check failed against ({timestamp:?})")]
    MaxClockDriftCheckFailed {
        max_clock_drift: Duration,
        timestamp: Timestamp,
    },
    #[error("next validators hash ({next_validators_hash}) of the trusted header does not match the adjacent header's validators hash ({validators_hash})", next_validators_hash = serde_utils::to_hex(next_validators_hash), validators_hash = serde_utils::to_hex(validators_hash))]
    NextValidatorsHashMismatch {
        next_validators_hash: H256,
        validators_hash: H256,
    },
    #[error("commit signatures length ({sig_len}) does not match the validators len ({val_len})")]
    InvalidCommitSignaturesLength { sig_len: usize, val_len: usize },
    #[error("commit height ({commit_height}) does not match the expected height ({height})")]
    InvalidCommitHeight { commit_height: i64, height: i64 },
    #[error(
        "commit block_id ({commit_block_id:?}) does not match the expected block id ({block_id:?})"
    )]
    InvalidCommitBlockId {
        commit_block_id: BlockId,
        block_id: BlockId,
    },
    #[error("voting power ({0}) cannot be negative")]
    NegativeVotingPower(i64),
    #[error("signature count ({count}) is below the batch verify threshold ({threshold})")]
    SignatureCountBelowBatchVerifyThreshold { threshold: usize, count: usize },

    #[error("batch verification ({0})")]
    BatchVerification(Box<dyn std::error::Error>),
}

pub fn verify<V: SignatureVerifier, B: BatchSignatureVerifier>(
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
        verify_non_adjacent::<V, B>(
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
        verify_adjacent::<V, B>(
            trusted_header,
            untrusted_header,
            untrusted_vals,
            trusting_period,
            now,
            max_clock_drift,
        )
    }
}

pub fn verify_non_adjacent<V: SignatureVerifier, B: BatchSignatureVerifier>(
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
        return Err(Error::HeadersMustBeNonAdjacent);
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

    verify_commit_light_trusting::<V, B>(
        &trusted_header.header.chain_id,
        trusted_vals,
        &untrusted_header.commit,
        trust_level,
        // TODO(aeryz): make this internal
        false,
    )?;

    verify_commit_light::<V, B>(
        untrusted_vals,
        &trusted_header.header.chain_id,
        &untrusted_header.commit.block_id,
        untrusted_header.header.height.inner(),
        &untrusted_header.commit,
    )?;

    Ok(())
}

pub fn verify_commit_light<V: SignatureVerifier, B: BatchSignatureVerifier>(
    vals: &ValidatorSet,
    chain_id: &str,
    block_id: &BlockId,
    height: i64,
    commit: &Commit,
) -> Result<(), Error> {
    verify_basic_vals_and_commit(vals, commit, height, block_id)?;

    let voting_power_needed = TryInto::<u64>::try_into(vals.total_voting_power)
        .map_err(|_| Error::NegativeVotingPower(vals.total_voting_power))?
        * 2
        / 3;

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

    if B::should_batch_verify(commit.signatures.len()) {
        verify_commit_batch::<B>(
            chain_id,
            vals,
            commit,
            voting_power_needed,
            filter_commit,
            |_| true,
            false,
            true,
        )
    } else {
        verify_commit_single::<V>(
            chain_id,
            vals,
            commit,
            voting_power_needed,
            filter_commit,
            |_| true,
            false,
            true,
        )
    }
}

fn verify_basic_vals_and_commit(
    vals: &ValidatorSet,
    commit: &Commit,
    height: i64,
    block_id: &BlockId,
) -> Result<(), Error> {
    if vals.validators.len() != commit.signatures.len() {
        return Err(Error::InvalidCommitSignaturesLength {
            sig_len: commit.signatures.len(),
            val_len: vals.validators.len(),
        });
    }

    if height != commit.height.inner() {
        return Err(Error::InvalidCommitHeight {
            commit_height: commit.height.inner(),
            height,
        });
    }

    if block_id != &commit.block_id {
        return Err(Error::InvalidCommitBlockId {
            commit_block_id: commit.block_id.clone(),
            block_id: block_id.clone(),
        });
    }

    Ok(())
}

pub fn verify_commit_light_trusting<V: SignatureVerifier, B: BatchSignatureVerifier>(
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

    // only use the commit signatures
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
    if B::should_batch_verify(commit.signatures.len()) {
        verify_commit_batch::<B>(
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

fn verify_commit_batch<V: BatchSignatureVerifier>(
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
    let mut batch_verifier = V::new();

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

        batch_verifier
            .add(&val.pub_key, vote_sign_bytes, signature.as_ref())
            .map_err(|e| Error::BatchVerification(Box::new(e)))?;

        // If this signature counts then add the voting power of the validator
        // to the tally
        if count_sig(commit_sig) {
            tallied_voting_power += val.voting_power.inner() as u64; // SAFE because within the bounds
        }

        if !count_all_signatures && tallied_voting_power > voting_power_needed {
            break;
        }
    }

    if tallied_voting_power <= voting_power_needed {
        return Err(Error::NotEnoughVotingPower {
            have: tallied_voting_power,
            need: voting_power_needed,
        });
    }

    if batch_verifier.verify_signature() {
        Ok(())
    } else {
        Err(Error::SignatureVerification)
    }
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

    let drift_timestamp =
        now.checked_add(max_clock_drift)
            .ok_or(Error::MaxClockDriftCheckFailed {
                max_clock_drift,
                timestamp: now,
            })?;

    if untrusted_header.header.time >= drift_timestamp {
        return Err(Error::MaxClockDriftCheckFailed {
            max_clock_drift,
            timestamp: untrusted_header.header.time,
        });
    }

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

pub fn header_expired(h: &SignedHeader, trusting_period: Duration, now: Timestamp) -> bool {
    let Some(expiration_time) = h.header.time.checked_add(trusting_period) else {
        return false;
    };

    expiration_time <= now
}

pub fn verify_adjacent<V: SignatureVerifier, B: BatchSignatureVerifier>(
    trusted_header: &SignedHeader,
    untrusted_header: &SignedHeader, // height=Y
    untrusted_vals: &ValidatorSet,   // height=Y
    // TODO(aeryz): Is this duration supposed to be proto?
    trusting_period: Duration,
    // TODO(aeryz): Until we define a Time type
    now: Timestamp,
    // TODO(aeryz): Is this duration supposed to be proto?
    max_clock_drift: Duration,
) -> Result<(), Error> {
    if untrusted_header.header.height.inner()
        != trusted_header
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

    if untrusted_header.header.validators_hash != trusted_header.header.next_validators_hash {
        return Err(Error::NextValidatorsHashMismatch {
            next_validators_hash: untrusted_header.header.next_validators_hash.clone(),
            validators_hash: trusted_header.header.next_validators_hash.clone(),
        });
    }

    verify_commit_light::<V, B>(
        untrusted_vals,
        &trusted_header.header.chain_id,
        &untrusted_header.commit.block_id,
        untrusted_header.header.height.inner(),
        &untrusted_header.commit,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    use unionlabs::ibc::lightclients::tendermint::header::Header;

    use super::*;

    struct EdVerifier;

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

    #[derive(Default)]
    struct BatchEdVerifier {
        signatures: Vec<Signature>,
        messages: Vec<Vec<u8>>,
        verifying_keys: Vec<VerifyingKey>,
    }

    impl BatchSignatureVerifier for BatchEdVerifier {
        type Error = BatchVerificationError;

        fn should_batch_verify(signature_len: usize) -> bool {
            signature_len >= 2
        }

        fn new() -> Self {
            BatchEdVerifier::default()
        }

        fn add(
            &mut self,
            pubkey: &PublicKey,
            msg: Vec<u8>,
            signature: &[u8],
        ) -> Result<(), Self::Error> {
            let PublicKey::Ed25519(pubkey) = pubkey else {
                panic!("invalid pubkey");
            };
            let key: VerifyingKey =
                VerifyingKey::from_bytes(pubkey.as_slice().try_into().unwrap()).unwrap();
            let signature: Signature = Signature::from_bytes(signature.try_into().unwrap());

            self.signatures.push(signature);
            self.verifying_keys.push(key);
            self.messages.push(msg.into());

            Ok(())
        }

        fn verify_signature(&self) -> bool {
            ed25519_dalek::verify_batch(
                self.messages
                    .iter()
                    .map(|v| v.as_slice())
                    .collect::<Vec<_>>()
                    .as_slice(),
                &self.signatures,
                &self.verifying_keys,
            )
            .is_ok()
        }
    }

    #[test]
    fn verify_works() {
        let initial_header: Header = serde_json::from_str(include_str!("test/288.json")).unwrap();
        let update_header: Header = serde_json::from_str(include_str!("test/291.json")).unwrap();

        verify::<EdVerifier, ()>(
            &initial_header.signed_header,
            &initial_header.validator_set,
            &update_header.signed_header,
            &update_header.validator_set,
            Duration::new(315576000000, 0).unwrap(),
            update_header.signed_header.header.time,
            Duration::new(100_000_000, 0).unwrap(),
            Fraction {
                numerator: 1,
                denominator: 3,
            },
        )
        .unwrap();
    }

    #[test]
    fn batch_verify_works() {
        let initial_header: Header = serde_json::from_str(include_str!("test/288.json")).unwrap();
        let update_header: Header = serde_json::from_str(include_str!("test/291.json")).unwrap();

        verify::<EdVerifier, BatchEdVerifier>(
            &initial_header.signed_header,
            &initial_header.validator_set,
            &update_header.signed_header,
            &update_header.validator_set,
            Duration::new(315576000000, 0).unwrap(),
            update_header.signed_header.header.time,
            Duration::new(100_000_000, 0).unwrap(),
            Fraction {
                numerator: 1,
                denominator: 3,
            },
        )
        .unwrap();
    }
}
