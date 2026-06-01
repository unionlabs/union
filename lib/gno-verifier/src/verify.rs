use std::collections::HashMap;

use gno_light_client_types::Fraction;
use gno_types::{BlockId, Commit, SignedHeader, ValidatorSet};
use unionlabs::{
    bounded::BoundedI64,
    ensure,
    google::protobuf::{duration::Duration, timestamp::Timestamp},
};

use crate::{
    error::{
        Error, TrustedValidatorsVerifyCommitError, VerifyLightCommitError,
        VerifyNewHeaderAndValsError,
    },
    types::SignatureVerifier,
    utils::{header_expired, validators_hash},
};

#[allow(clippy::too_many_arguments)]
pub fn verify<V: SignatureVerifier>(
    trusted_header: &SignedHeader,
    trusted_vals: &ValidatorSet,     // height=X or height=X+1
    untrusted_header: &SignedHeader, // height=Y
    untrusted_vals: &ValidatorSet,   // height=Y
    trusting_period: Duration,
    now: Timestamp,
    max_clock_drift: Duration,
    trust_level: &Fraction,
    signature_verifier: &V,
) -> Result<(), Error> {
    // check adjacency in terms of block(header) height
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

/// VerifyNonAdjacent verifies non-adjacent untrustedHeader against
/// trustedHeader. It ensures that:
///
/// a) trustedHeader can still be trusted (if not, ErrOldHeaderExpired is returned)
/// b) untrustedHeader is valid (if not, ErrInvalidHeader is returned)
/// c) trustLevel ([1/3, 1]) of trustedHeaderVals (or trustedHeaderNextVals)
///    signed correctly (if not, ErrNewValSetCantBeTrusted is returned)
/// d) more than 2/3 of untrustedVals have signed h2
///    (otherwise, ErrInvalidHeader is returned)
/// e) headers are non-adjacent.
///
/// maxClockDrift defines how much untrustedHeader.Time can drift into the
/// future.
///
/// Source: <https://github.com/atomone-hub/atomone/blob/5e3a5d733d818c1fd3d8b08aac9baf329737d27d/modules/10-gno/verifier.go#L34>
#[allow(clippy::too_many_arguments)]
pub fn verify_non_adjacent<V: SignatureVerifier>(
    trusted_header: &SignedHeader,
    trusted_vals: &ValidatorSet,     // height=X or height=X+1
    untrusted_header: &SignedHeader, // height=Y
    untrusted_vals: &ValidatorSet,   // height=Y
    trusting_period: Duration,
    now: Timestamp,
    max_clock_drift: Duration,
    trust_level: &Fraction,
    signature_verifier: &V,
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

    verify_light_commit(
        trusted_vals,
        &trusted_header.header.chain_id,
        &untrusted_header.commit.block_id,
        untrusted_header.header.height,
        &untrusted_header.commit,
        trust_level,
        signature_verifier,
    )?;

    validator_set_verify_commit(
        untrusted_vals,
        &trusted_header.header.chain_id,
        &untrusted_header.commit.block_id,
        untrusted_header.header.height,
        &untrusted_header.commit,
        signature_verifier,
    )?;

    Ok(())
}

fn validator_set_verify_commit<V: SignatureVerifier>(
    untrusted_vals: &ValidatorSet,
    chain_id: &str,
    block_id: &BlockId,
    height: BoundedI64<0>,
    commit: &Commit,
    signature_verifier: &V,
) -> Result<(), TrustedValidatorsVerifyCommitError> {
    use TrustedValidatorsVerifyCommitError::*;

    commit.validate_basic()?;

    ensure(
        untrusted_vals.validators.len() == commit.precommits.len(),
        InvalidCommitPrecommitsError {
            expected: untrusted_vals.validators.len(),
            actual: commit.precommits.len(),
        },
    )?;

    ensure(
        height == commit.height(),
        InvalidCommitHeightError {
            expected: height,
            actual: commit.height(),
        },
    )?;

    ensure(
        block_id == &commit.block_id,
        InvalidCommitWrongBlockId {
            expected: *block_id,
            actual: commit.block_id,
        },
    )?;

    let mut tallied_voting_power = 0;

    for (idx, precommit) in commit.precommits.iter().enumerate() {
        let Some(precommit) = precommit else {
            continue; // OK, some precommits can be missing.
        };

        let Some((_, val)) = untrusted_vals.get_by_index(idx) else {
            panic!("???")
        };

        // Validate signature.
        let precommit_sign_bytes = commit.vote_sign_bytes(chain_id.to_owned(), idx);
        ensure(
            signature_verifier.verify_signature(
                &val.pub_key,
                &precommit_sign_bytes,
                &precommit.signature,
            ),
            InvalidSignature {
                vote: precommit.clone(),
            },
        )?;

        // Good precommit!
        if block_id == &precommit.block_id {
            tallied_voting_power += val.voting_power.inner();
        }
        // else {
        // It's OK that the BlockID doesn't match.  We include stray
        // precommits to measure validator availability.
        // }
    }

    let needed_voting_power = untrusted_vals.total_voting_power() * 2 / 3;
    if tallied_voting_power > needed_voting_power {
        Ok(())
    } else {
        Err(TooMuchChangeError {
            got: tallied_voting_power,
            needed: needed_voting_power + 1,
        })
    }
}

// VerifyAdjacent verifies directly adjacent untrustedHeader against
// trustedHeader. It ensures that:
//
//	a) trustedHeader can still be trusted (if not, ErrOldHeaderExpired is returned)
//	b) untrustedHeader is valid (if not, ErrInvalidHeader is returned)
//	c) untrustedHeader.ValidatorsHash equals trustedHeader.NextValidatorsHash
//	d) more than 2/3 of new validators (untrustedVals) have signed h2
//	  (otherwise, ErrInvalidHeader is returned)
//	e) headers are adjacent.
//
// maxClockDrift defines how much untrustedHeader.Time can drift into the
// future.
pub fn verify_adjacent<V: SignatureVerifier>(
    trusted_header: &SignedHeader,
    untrusted_header: &SignedHeader, // height=Y
    untrusted_vals: &ValidatorSet,   // height=Y
    trusting_period: Duration,
    now: Timestamp,
    max_clock_drift: Duration,
    signature_verifier: &V,
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

    ensure(
        untrusted_header.header.validators_hash == trusted_header.header.next_validators_hash,
        Error::NextValidatorsHashMismatch {
            next_validators_hash: untrusted_header.header.next_validators_hash.into_encoding(),
            validators_hash: trusted_header.header.next_validators_hash.into_encoding(),
        },
    )?;

    validator_set_verify_commit(
        untrusted_vals,
        &trusted_header.header.chain_id,
        &untrusted_header.commit.block_id,
        untrusted_header.header.height,
        &untrusted_header.commit,
        signature_verifier,
    )?;

    Ok(())
}

/// Source: <https://github.com/atomone-hub/atomone/blob/5e3a5d733d818c1fd3d8b08aac9baf329737d27d/modules/10-gno/verifier.go#L191>
pub fn verify_light_commit<V: SignatureVerifier>(
    vals: &ValidatorSet,
    chain_id: &str,
    block_id: &BlockId,
    height: BoundedI64<0>,
    commit: &Commit,
    trust_level: &Fraction,
    signature_verifier: &V,
) -> Result<(), VerifyLightCommitError> {
    use VerifyLightCommitError::*;

    commit.validate_basic()?;

    ensure(
        height == commit.height(),
        InvalidCommitHeight {
            expected: height,
            actual: commit.height(),
        },
    )?;

    ensure(
        block_id == &commit.block_id,
        InvalidBlockId {
            want: *block_id,
            got: commit.block_id,
        },
    )?;

    let mut tallied_voting_power = 0;
    let mut seen = HashMap::new();

    for (idx, precommit) in commit.precommits.iter().enumerate() {
        let Some(precommit) = precommit else {
            continue; // OK, some precommits can be missing.
        };

        // Look up by address since the commit may be from a different height
        // whose validator set has a different ordering/composition.
        let Some((val_idx, val)) = vals.get_by_address(&precommit.validator_address) else {
            continue; // not in trusted set
        };
        if seen.contains_key(&val_idx) {
            continue; // already counted
        }
        seen.insert(val_idx, true);

        // Validate signature.
        let precommit_sign_bytes = commit.vote_sign_bytes(chain_id.to_owned(), idx);
        if !signature_verifier.verify_signature(
            &val.pub_key,
            &precommit_sign_bytes,
            &precommit.signature,
        ) {
            return Err(InvalidSignature {
                vote: precommit.clone(),
            });
        }
        // Good precommit!
        if block_id == &precommit.block_id {
            tallied_voting_power += val.voting_power.inner();
        }
        // else {
        // It's OK that the BlockID doesn't match.  We include stray
        // precommits to measure validator availability.
        // }
    }

    // safely calculate voting power needed.
    let Some(total_voting_power_mul_by_numerator) = vals
        .total_voting_power()
        .checked_mul(trust_level.numerator as i64)
    else {
        return Err(VotingPowerOverflow);
    };

    let voting_power_needed =
        total_voting_power_mul_by_numerator / (trust_level.denominator.get() as i64);

    if tallied_voting_power > voting_power_needed {
        Ok(())
    } else {
        Err(InsufficientTrustedVotingPower {
            got: tallied_voting_power,
            min: voting_power_needed,
        })
    }
}

/// Source: <https://github.com/atomone-hub/atomone/blob/5e3a5d733d818c1fd3d8b08aac9baf329737d27d/modules/10-gno/verifier.go#L150>
fn verify_new_headers_and_vals(
    untrusted_header: &SignedHeader, // height=Y
    untrusted_vals: &ValidatorSet,   // height=Y
    trusted_header: &SignedHeader,
    now: Timestamp,
    max_clock_drift: Duration,
) -> Result<(), VerifyNewHeaderAndValsError> {
    use VerifyNewHeaderAndValsError::*;

    // TODO: SignedHeader.ValidateBasic checks

    ensure(
        untrusted_header.header.height > trusted_header.header.height,
        NewHeaderHeightMustBeGreater {
            untrusted_header_height: untrusted_header.header.height,
            trusted_header_height: trusted_header.header.height,
        },
    )?;

    ensure(
        untrusted_header.header.time > trusted_header.header.time,
        NewHeaderTimeMustBeGreater {
            untrusted_header_time: untrusted_header.header.time,
            trusted_header_time: trusted_header.header.time,
        },
    )?;

    let drift_timestamp = now
        .checked_add(max_clock_drift)
        .expect("probably won't happen");

    ensure(
        untrusted_header.header.time < drift_timestamp,
        NewHeaderFromFuture {
            untrusted_header_time: untrusted_header.header.time,
            now,
            max_clock_drift,
        },
    )?;

    let untrusted_validators_hash = validators_hash(untrusted_vals);
    ensure(
        untrusted_header.header.validators_hash == untrusted_validators_hash,
        UntrustedValidatorSetMismatch {
            untrusted_header_validators_hash: untrusted_header
                .header
                .validators_hash
                .into_encoding(),
            untrusted_validators_hash: untrusted_validators_hash.into_encoding(),
            untrusted_header_height: untrusted_header.header.height,
        },
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, num::NonZeroU64};

    use gno_light_client_types::Header;
    use gno_types::PublicKey;
    use unionlabs::ErrorReporter;

    use super::*;

    struct SigVerifier;

    impl SignatureVerifier for SigVerifier {
        fn verify_signature(&self, pub_key: &PublicKey, msg: &[u8], sig: &[u8]) -> bool {
            match pub_key {
                PublicKey::Ed25519(pub_key) => {
                    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

                    let key =
                        VerifyingKey::from_bytes(&pub_key.as_ref().try_into().unwrap()).unwrap();
                    let signature = Signature::from_bytes(sig.try_into().unwrap());
                    key.verify(msg, &signature).is_ok()
                }
                PublicKey::Secp256k1(pub_key) => {
                    use k256::ecdsa::{Signature, VerifyingKey, signature::Verifier};

                    let key = VerifyingKey::from_sec1_bytes(pub_key).unwrap();
                    let signature = Signature::from_slice(sig).unwrap();
                    key.verify(msg, &signature).is_ok()
                }
                PublicKey::Multisig { .. } => {
                    panic!("well this doesn't make much sense now does it")
                }
            }
        }
    }

    #[test]
    fn verify_works() {
        let header: Header = serde_json::from_str(
            &fs::read_to_string("testdata/mainnet-header-1008284-1008285.json").unwrap(),
        )
        .unwrap();

        let trusted_header: SignedHeader = serde_json::from_str(
            &fs::read_to_string("testdata/mainnet-signed-header-1008284.json").unwrap(),
        )
        .unwrap();

        let res = verify(
            &trusted_header,
            &header.trusted_validators,
            &header.signed_header,
            &header.validator_set,
            Duration::new(315576000000, 0).unwrap(),
            header.signed_header.header.time, // now
            Duration::new(100_000_000, 0).unwrap(),
            &Fraction {
                numerator: 1,
                denominator: const { NonZeroU64::new(3).unwrap() },
            },
            &SigVerifier,
        );

        if let Err(err) = res {
            panic!("{}", ErrorReporter(err))
        }
    }
}
