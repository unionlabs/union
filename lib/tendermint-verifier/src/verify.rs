#![allow(clippy::type_complexity)] // we use some funky functions in this file

use std::collections::BTreeMap;

use cometbft_types::{
    crypto::public_key::PublicKey,
    types::{
        block_id::BlockId, commit::Commit, commit_sig::CommitSig, signed_header::SignedHeader,
        validator_set::ValidatorSet,
    },
};
use tendermint_light_client_types::Fraction;
use unionlabs::{
    google::protobuf::{duration::Duration, timestamp::Timestamp},
    primitives::H160,
};

use crate::{
    error::Error,
    types::{HostFns, SignatureVerifier},
    utils::{canonical_vote_bytes, get_validator_by_address, header_expired, validators_hash},
};

/// Reference implementation: <https://github.com/cometbft/cometbft/blob/e820315631a81c230e4abe9bcede8e29382e8af5/light/verifier.go#L130>
#[allow(clippy::too_many_arguments)]
pub fn verify<V: HostFns>(
    trusted_header: &SignedHeader,
    trusted_vals: &ValidatorSet,     // height=X or height=X+1
    untrusted_header: &SignedHeader, // height=Y
    untrusted_vals: &ValidatorSet,   // height=Y
    trusting_period: Duration,
    now: Timestamp,
    max_clock_drift: Duration,
    trust_level: &Fraction,
    signature_verifier: &SignatureVerifier<V>,
) -> Result<(), Error> {
    // check adjacency in terms of block (header) height
    if untrusted_header.header.height.inner()
        != trusted_header
            .header
            .height
            .inner()
            .checked_add(1)
            .ok_or(Error::IntegerOverflow)?
    {
        verify_non_adjacent(
            trusted_header,
            trusted_vals,
            untrusted_header,
            untrusted_vals,
            trusting_period,
            now,
            max_clock_drift,
            trust_level,
            signature_verifier,
        )
    } else {
        verify_adjacent(
            trusted_header,
            untrusted_header,
            untrusted_vals,
            trusting_period,
            now,
            max_clock_drift,
            signature_verifier,
        )
    }
}

// TODO(aeryz): Official docs, change
/// verifies non-adjacent untrustedHeader against
/// trustedHeader. It ensures that:
///
/// 1. trustedHeader can still be trusted
/// 2. untrustedHeader is valid
/// 3. trustLevel ([1/3, 1]) of trustedHeaderVals signed correctly
/// 4. more than 2/3 of untrustedVals have signed h2
/// 5. headers are non-adjacent.
///
/// maxClockDrift defines how much untrustedHeader.Time can drift into the
/// future.
///
/// Reference implementation: <https://github.com/cometbft/cometbft/blob/e820315631a81c230e4abe9bcede8e29382e8af5/light/verifier.go#L30>
#[allow(clippy::too_many_arguments)]
pub fn verify_non_adjacent<V: HostFns>(
    trusted_header: &SignedHeader,
    trusted_vals: &ValidatorSet,     // height=X or height=X+1
    untrusted_header: &SignedHeader, // height=Y
    untrusted_vals: &ValidatorSet,   // height=Y
    trusting_period: Duration,
    now: Timestamp,
    max_clock_drift: Duration,
    trust_level: &Fraction,
    signature_verifier: &SignatureVerifier<V>,
) -> Result<(), Error> {
    // We only want this check to be done when the headers are not adjacent
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

    verify_commit_light_trusting(
        &trusted_header.header.chain_id,
        trusted_vals,
        &untrusted_header.commit,
        trust_level,
        signature_verifier,
    )?;

    verify_commit_light(
        untrusted_vals,
        &trusted_header.header.chain_id,
        &untrusted_header.commit.block_id,
        untrusted_header.header.height.inner(),
        &untrusted_header.commit,
        signature_verifier,
    )?;

    Ok(())
}

/// Reference implementation: <https://github.com/cometbft/cometbft/blob/e820315631a81c230e4abe9bcede8e29382e8af5/light/verifier.go#L92>
pub fn verify_adjacent<V: HostFns>(
    trusted_header: &SignedHeader,
    untrusted_header: &SignedHeader, // height=Y
    untrusted_vals: &ValidatorSet,   // height=Y
    trusting_period: Duration,
    now: Timestamp,
    max_clock_drift: Duration,
    signature_verifier: &SignatureVerifier<V>,
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
            next_validators_hash: untrusted_header.header.next_validators_hash,
            validators_hash: trusted_header.header.next_validators_hash,
        });
    }

    verify_commit_light(
        untrusted_vals,
        &trusted_header.header.chain_id,
        &untrusted_header.commit.block_id,
        untrusted_header.header.height.inner(),
        &untrusted_header.commit,
        signature_verifier,
    )?;

    Ok(())
}

/// Reference implementation: <https://github.com/cometbft/cometbft/blob/e820315631a81c230e4abe9bcede8e29382e8af5/types/validation.go#L82>
///
/// Note that this does not include the caching logic.
pub fn verify_commit_light<V: HostFns>(
    vals: &ValidatorSet,
    chain_id: &str,
    block_id: &BlockId,
    height: i64,
    commit: &Commit,
    signature_verifier: &SignatureVerifier<V>,
) -> Result<(), Error> {
    verify_basic_vals_and_commit(vals, commit, height, block_id)?;

    let voting_power_needed = u64::try_from(vals.total_voting_power)
        .map_err(|_| Error::NegativeVotingPower(vals.total_voting_power))?
        .checked_mul(2)
        .ok_or(Error::IntegerOverflow)?
        / 3;

    let filter_commit =
        |commit_sig: &CommitSig| -> Result<Option<(H160, Timestamp, Vec<u8>)>, Error> {
            match commit_sig {
                // only commits have a canonical vote, this filtering is required by `canonical_vote_bytes`
                CommitSig::Commit {
                    validator_address,
                    timestamp,
                    signature,
                } => Ok(Some((
                    *validator_address,
                    *timestamp,
                    signature.clone().into_vec(),
                ))),
                _ => Ok(None),
            }
        };

    if should_batch_verify(commit.signatures.len()) {
        verify_commit_batch(
            chain_id,
            vals,
            commit,
            voting_power_needed,
            filter_commit,
            true,
            signature_verifier,
        )
    } else {
        verify_commit_single(
            chain_id,
            vals,
            commit,
            voting_power_needed,
            filter_commit,
            true,
            signature_verifier,
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
            commit_block_id: Box::new(commit.block_id.clone()),
            block_id: Box::new(block_id.clone()),
        });
    }

    Ok(())
}

/// Reference implementation: <https://github.com/cometbft/cometbft/blob/e820315631a81c230e4abe9bcede8e29382e8af5/types/validation.go#L172>
///
/// Note that this does not include the caching logic.
pub fn verify_commit_light_trusting<V: HostFns>(
    chain_id: &str,
    vals: &ValidatorSet,
    commit: &Commit,
    trust_level: &Fraction,
    signature_verifier: &SignatureVerifier<V>,
) -> Result<(), Error> {
    // SAFETY: as u64 is safe here since we do `abs` which makes it always positive
    let total_voting_power_mul_by_numerator = vals
        .total_voting_power
        .unsigned_abs()
        .checked_mul(trust_level.numerator)
        .ok_or(Error::IntegerOverflow)?;
    let voting_power_needed = total_voting_power_mul_by_numerator / trust_level.denominator;

    // only use the commit signatures
    let filter_commit =
        |commit_sig: &CommitSig| -> Result<Option<(H160, Timestamp, Vec<u8>)>, Error> {
            match commit_sig {
                CommitSig::Commit {
                    validator_address,
                    timestamp,
                    signature,
                } => Ok(Some((
                    *validator_address,
                    *timestamp,
                    signature.clone().into_vec(),
                ))),
                _ => Ok(None),
            }
        };

    // attempt to batch verify commit. As the validator set doesn't necessarily
    // correspond with the validator set that signed the block we need to look
    // up by address rather than index.
    if should_batch_verify(commit.signatures.len()) {
        verify_commit_batch(
            chain_id,
            vals,
            commit,
            voting_power_needed,
            filter_commit,
            false,
            signature_verifier,
        )
    } else {
        verify_commit_single(
            chain_id,
            vals,
            commit,
            voting_power_needed,
            filter_commit,
            false,
            signature_verifier,
        )
    }
}

fn verify_commit_single<V: HostFns>(
    chain_id: &str,
    vals: &ValidatorSet,
    commit: &Commit,
    voting_power_needed: u64,
    filter_commit: fn(&CommitSig) -> Result<Option<(H160, Timestamp, Vec<u8>)>, Error>,
    lookup_by_index: bool,
    signature_verifier: &SignatureVerifier<V>,
) -> Result<(), Error> {
    verify_commit(
        chain_id,
        vals,
        commit,
        voting_power_needed,
        filter_commit,
        lookup_by_index,
        |pubkey, msg, signature| {
            if signature_verifier
                .verifier
                .verify_signature(pubkey, &msg, signature.as_ref())
            {
                Ok(())
            } else {
                Err(Error::SignatureVerification)
            }
        },
    )
}

fn verify_commit_batch<V: HostFns>(
    chain_id: &str,
    vals: &ValidatorSet,
    commit: &Commit,
    voting_power_needed: u64,
    filter_commit: fn(&CommitSig) -> Result<Option<(H160, Timestamp, Vec<u8>)>, Error>,
    lookup_by_index: bool,
    signature_verifier: &SignatureVerifier<V>,
) -> Result<(), Error> {
    let mut pubkeys = Vec::new();
    let mut msgs = Vec::new();
    let mut signatures = Vec::new();
    verify_commit(
        chain_id,
        vals,
        commit,
        voting_power_needed,
        filter_commit,
        lookup_by_index,
        |pubkey, msg, signature| {
            // TODO(aeryz): ensure same key here
            pubkeys.push(pubkey.clone());
            msgs.push(msg);
            signatures.push(signature);
            Ok(())
        },
    )?;

    if signature_verifier.verifier.verify_batch_signature(
        &pubkeys,
        &msgs.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
        &signatures.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
    ) {
        Ok(())
    } else {
        Err(Error::SignatureVerification)
    }
}

fn verify_commit<F: FnMut(&PublicKey, Vec<u8>, Vec<u8>) -> Result<(), Error>>(
    chain_id: &str,
    vals: &ValidatorSet,
    commit: &Commit,
    voting_power_needed: u64,
    filter_commit: fn(&CommitSig) -> Result<Option<(H160, Timestamp, Vec<u8>)>, Error>,
    lookup_by_index: bool,
    mut signature_handle: F,
) -> Result<(), Error> {
    let mut tallied_voting_power: u64 = 0;
    let mut seen_vals: BTreeMap<usize, usize> = BTreeMap::new();

    for (i, commit_sig) in commit.signatures.iter().enumerate() {
        let Some((validator_address, timestamp, signature)) = filter_commit(commit_sig)? else {
            continue;
        };

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

            // `insert` returns the value if there already exists a value
            if seen_vals.insert(val_idx, i).is_some() {
                return Err(Error::DoubleVote(validator_address));
            }

            val
        };

        let vote_sign_bytes = canonical_vote_bytes(commit, commit_sig, &timestamp, chain_id)?;

        signature_handle(&val.pub_key, vote_sign_bytes, signature)?;

        // If this signature counts then add the voting power of the validator
        // to the tally
        tallied_voting_power += val.voting_power.inner() as u64; // SAFE because within the bounds

        if tallied_voting_power > voting_power_needed {
            return Ok(());
        }
    }

    Err(Error::NotEnoughVotingPower {
        have: tallied_voting_power,
        need: voting_power_needed,
    })
}

/// Reference implementation: <https://github.com/cometbft/cometbft/blob/e820315631a81c230e4abe9bcede8e29382e8af5/light/verifier.go#L148>
fn verify_new_headers_and_vals(
    untrusted_header: &SignedHeader, // height=Y
    untrusted_vals: &ValidatorSet,   // height=Y
    trusted_header: &SignedHeader,
    now: Timestamp,
    max_clock_drift: Duration,
) -> Result<(), Error> {
    // SH HEADER VALIDATE BASIC
    // TODO(aeryz): move these untrusted_header.validate_basic related checks to elsewhere, this function gets too bloated
    if untrusted_header.commit.height != untrusted_header.header.height {
        return Err(Error::SignedHeaderCommitHeightMismatch {
            sh_height: untrusted_header.header.height.inner(),
            commit_height: untrusted_header.commit.height.inner(),
        });
    }

    let untrusted_header_hash = untrusted_header
        .header
        .calculate_merkle_root()
        .ok_or(Error::InvalidHeader)?;
    let commit_hash = untrusted_header
        .commit
        .block_id
        .hash
        .ok_or(Error::MissingBlockIdHash)?;
    if untrusted_header_hash != commit_hash {
        return Err(Error::SignedHeaderCommitHashMismatch {
            sh_hash: untrusted_header_hash.into_encoding(),
            commit_hash: commit_hash.into_encoding(),
        });
    }

    if untrusted_header.header.chain_id != trusted_header.header.chain_id {
        return Err(Error::ChainIdMismatch {
            untrusted_header_chain_id: untrusted_header.header.chain_id.clone(),
            trusted_header_chain_id: trusted_header.header.chain_id.clone(),
        });
    }
    // SH HEADER VALIDATE BASIC END

    // we can only update using a latter header
    if untrusted_header.header.height <= trusted_header.header.height {
        return Err(Error::UntrustedHeaderHeightIsLE {
            untrusted_header_height: untrusted_header.header.height.inner(),
            trusted_header_height: trusted_header.header.height.inner(),
        });
    }

    // a header with a greater height can never have <= time
    if untrusted_header.header.time <= trusted_header.header.time {
        return Err(Error::UntrustedHeaderTimestampIsLE {
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

    let untrusted_validators_hash = validators_hash(untrusted_vals);
    if untrusted_header.header.validators_hash != untrusted_validators_hash {
        return Err(Error::UntrustedValidatorSetMismatch {
            expected: untrusted_header.header.validators_hash,
            found: untrusted_validators_hash.into_encoding(),
        });
    }

    Ok(())
}

fn should_batch_verify(signatures_len: usize) -> bool {
    signatures_len >= 2
}

#[cfg(test)]
mod tests {
    use std::{fs, num::NonZeroU64};

    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    use tendermint_light_client_types::Header;
    use unionlabs::option_unwrap;

    use super::*;

    struct EdVerifier;

    impl HostFns for EdVerifier {
        fn verify_signature(&self, pubkey: &PublicKey, msg: &[u8], sig: &[u8]) -> bool {
            let PublicKey::Ed25519(pubkey) = pubkey else {
                panic!("invalid pubkey");
            };
            let key: VerifyingKey =
                VerifyingKey::from_bytes(&pubkey.as_ref().try_into().unwrap()).unwrap();
            let signature: Signature = Signature::from_bytes(sig.try_into().unwrap());
            key.verify(msg, &signature).is_ok()
        }

        fn verify_batch_signature(
            &self,
            pubkeys: &[PublicKey],
            msgs: &[&[u8]],
            sigs: &[&[u8]],
        ) -> bool {
            let mut signatures = Vec::new();
            let mut keys = Vec::new();

            for (pubkey, signature) in pubkeys.iter().zip(sigs.iter()) {
                let PublicKey::Ed25519(pubkey) = pubkey else {
                    panic!("invalid pubkey");
                };
                let key: VerifyingKey =
                    VerifyingKey::from_bytes(pubkey.as_ref().try_into().unwrap()).unwrap();
                let signature: Signature =
                    Signature::from_bytes(&<[u8; 64]>::try_from(*signature).unwrap());
                signatures.push(signature);
                keys.push(key)
            }

            ed25519_dalek::verify_batch(msgs, &signatures, &keys).is_ok()
        }
    }

    #[test]
    fn verify_works() {
        let initial_header: Header =
            serde_json::from_str(&fs::read_to_string("src/test/288.json").unwrap()).unwrap();
        let update_header: Header =
            serde_json::from_str(&fs::read_to_string("src/test/291.json").unwrap()).unwrap();

        verify(
            &initial_header.signed_header,
            &initial_header.validator_set,
            &update_header.signed_header,
            &update_header.validator_set,
            Duration::new(315576000000, 0).unwrap(),
            update_header.signed_header.header.time,
            Duration::new(100_000_000, 0).unwrap(),
            &Fraction {
                numerator: 1,
                denominator: const { option_unwrap!(NonZeroU64::new(3)) },
            },
            &SignatureVerifier::new(EdVerifier),
        )
        .unwrap();
    }
}
