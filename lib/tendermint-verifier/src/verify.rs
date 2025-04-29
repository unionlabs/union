#![allow(clippy::type_complexity)] // we use some funky functions in this file

use std::collections::BTreeMap;

use cometbft_types::types::{
    block_id::BlockId, commit::Commit, signed_header::SignedHeader, validator_set::ValidatorSet,
};
use tendermint_light_client_types::Fraction;
use unionlabs::google::protobuf::{duration::Duration, timestamp::Timestamp};

use crate::{
    error::Error,
    types::{ValidatorSig, Verification},
    utils::{canonical_vote_bytes, get_validator_by_address, header_expired, validators_hash},
};

/// Reference implementation: <https://github.com/cometbft/cometbft/blob/e820315631a81c230e4abe9bcede8e29382e8af5/light/verifier.go#L130>
#[allow(clippy::too_many_arguments)]
pub fn verify<V: Verification>(
    trusted_header: &SignedHeader,
    trusted_vals: &ValidatorSet,     // height=X or height=X+1
    untrusted_header: &SignedHeader, // height=Y
    untrusted_vals: &ValidatorSet,   // height=Y
    trusting_period: Duration,
    now: Timestamp,
    max_clock_drift: Duration,
    trust_level: &Fraction,
    signature_verifier: &mut V,
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
pub fn verify_non_adjacent<V: Verification>(
    trusted_header: &SignedHeader,
    trusted_vals: &ValidatorSet,     // height=X or height=X+1
    untrusted_header: &SignedHeader, // height=Y
    untrusted_vals: &ValidatorSet,   // height=Y
    trusting_period: Duration,
    now: Timestamp,
    max_clock_drift: Duration,
    trust_level: &Fraction,
    signature_verifier: &mut V,
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
pub fn verify_adjacent<V: Verification>(
    trusted_header: &SignedHeader,
    untrusted_header: &SignedHeader, // height=Y
    untrusted_vals: &ValidatorSet,   // height=Y
    trusting_period: Duration,
    now: Timestamp,
    max_clock_drift: Duration,
    signature_verifier: &mut V,
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
pub fn verify_commit_light<V: Verification>(
    vals: &ValidatorSet,
    chain_id: &str,
    block_id: &BlockId,
    height: i64,
    commit: &Commit,
    signature_verifier: &mut V,
) -> Result<(), Error> {
    verify_basic_vals_and_commit(vals, commit, height, block_id)?;

    let voting_power_needed = u64::try_from(vals.total_voting_power)
        .map_err(|_| Error::NegativeVotingPower(vals.total_voting_power))?
        .checked_mul(2)
        .ok_or(Error::IntegerOverflow)?
        / 3;

    verify_commit(
        chain_id,
        vals,
        commit,
        voting_power_needed,
        true,
        signature_verifier,
    )
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
pub fn verify_commit_light_trusting<V: Verification>(
    chain_id: &str,
    vals: &ValidatorSet,
    commit: &Commit,
    trust_level: &Fraction,
    signature_verifier: &mut V,
) -> Result<(), Error> {
    // SAFETY: as u64 is safe here since we do `abs` which makes it always positive
    let total_voting_power_mul_by_numerator = vals
        .total_voting_power
        .unsigned_abs()
        .checked_mul(trust_level.numerator)
        .ok_or(Error::IntegerOverflow)?;
    let voting_power_needed = total_voting_power_mul_by_numerator / trust_level.denominator;

    // attempt to batch verify commit. As the validator set doesn't necessarily
    // correspond with the validator set that signed the block we need to look
    // up by address rather than index.
    verify_commit(
        chain_id,
        vals,
        commit,
        voting_power_needed,
        false,
        signature_verifier,
    )
}

fn verify_commit<V: Verification>(
    chain_id: &str,
    vals: &ValidatorSet,
    commit: &Commit,
    voting_power_needed: u64,
    lookup_by_index: bool,
    signature_verifier: &mut V,
) -> Result<(), Error> {
    let mut tallied_voting_power: u64 = 0;
    let mut seen_vals: BTreeMap<usize, usize> = BTreeMap::new();

    for (i, commit_sig) in commit.signatures.iter().enumerate() {
        let Some(ValidatorSig {
            validator_address,
            timestamp,
            signature,
        }) = signature_verifier
            .filter_commit(commit_sig)
            .map_err(Into::into)?
        else {
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

        if let Some(signature) = signature {
            let vote_sign_bytes =
                canonical_vote_bytes::<V::CanonicalVoteProto>(commit, &timestamp, chain_id)?;

            signature_verifier
                .process_signature(val.pub_key.clone(), Some(vote_sign_bytes), Some(signature))
                .map_err(Into::into)?;
        } else {
            signature_verifier
                .process_signature(val.pub_key.clone(), None, None)
                .map_err(Into::into)?;
        }

        // If this signature counts then add the voting power of the validator
        // to the tally
        tallied_voting_power += val.voting_power.inner() as u64; // SAFE because within the bounds

        if tallied_voting_power > voting_power_needed {
            return Ok(());
        }
    }

    signature_verifier.finish().map_err(Into::into)?;

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

#[cfg(test)]
mod tests {
    use std::{fs, num::NonZeroU64};

    use cometbft_types::{
        crypto::public_key::PublicKey,
        types::{block_id_flag::BlockIdFlag, validator::Validator},
    };
    use cosmwasm_std::{testing::mock_dependencies, Deps, BLS12_381_G1_GENERATOR};
    use tendermint_light_client_types::Header;
    use unionlabs::option_unwrap;

    use super::*;
    use crate::types::ValidatorSig;

    pub struct Ed25519Verifier<'a> {
        deps: Deps<'a>,
        pubkeys: Vec<PublicKey>,
        msgs: Vec<Vec<u8>>,
        signatures: Vec<Vec<u8>>,
    }

    impl<'a> Ed25519Verifier<'a> {
        pub fn new(deps: Deps<'a>) -> Self {
            Self {
                deps,
                pubkeys: Vec::new(),
                msgs: Vec::new(),
                signatures: Vec::new(),
            }
        }
    }

    impl<'a> Verification for Ed25519Verifier<'a> {
        type Error = EmptyError;

        type CanonicalVoteProto = protos::tendermint::types::CanonicalVote;

        fn filter_commit(
            &self,
            commit_sig: &cometbft_types::types::commit_sig::CommitSigRaw,
        ) -> Result<Option<ValidatorSig>, Self::Error> {
            if commit_sig.block_id_flag == Into::<i32>::into(BlockIdFlag::Commit) {
                let Some(timestamp) = commit_sig.timestamp else {
                    return Ok(None);
                };
                let Some(signature) = commit_sig.signature.clone() else {
                    return Ok(None);
                };

                Ok(Some(ValidatorSig {
                    validator_address: commit_sig.validator_address,
                    timestamp,
                    signature: Some(signature.into_vec()),
                }))
            } else {
                Ok(None)
            }
        }

        fn process_signature(
            &mut self,
            public_key: PublicKey,
            msg: Option<Vec<u8>>,
            signature: Option<Vec<u8>>,
        ) -> Result<(), Self::Error> {
            let (Some(msg), Some(signature)) = (msg, signature) else {
                panic!("msg and signature must have been provided");
            };

            // TODO(aeryz): verify here
            self.pubkeys.push(public_key);
            self.msgs.push(msg);
            self.signatures.push(signature);

            Ok(())
        }

        fn finish(&mut self) -> Result<(), Self::Error> {
            let Ok(pubkeys) = self
                .pubkeys
                .iter()
                .map(|pk| match pk {
                    PublicKey::Ed25519(pkey) => Ok(pkey.as_ref()),
                    _ => Err(()),
                })
                .collect::<Result<Vec<_>, _>>()
            else {
                panic!("invalid pubkey type");
            };

            let ret = {
                let msgs: Vec<&[u8]> = self.msgs.iter().map(|x| x.as_slice()).collect();
                let sigs: Vec<&[u8]> = self.signatures.iter().map(|x| x.as_slice()).collect();

                if self
                    .deps
                    .api
                    .ed25519_batch_verify(&msgs, &sigs, &pubkeys)
                    .unwrap()
                {
                    Ok(())
                } else {
                    panic!("invalid signature")
                }
            };

            self.pubkeys = Vec::new();
            self.msgs = Vec::new();
            self.signatures = Vec::new();

            ret
        }
    }

    pub struct Bls12381Verifier<'a> {
        deps: Deps<'a>,
        pubkeys: Vec<Vec<u8>>,
        msg: Vec<u8>,
        signature: Vec<u8>,
    }

    impl<'a> Bls12381Verifier<'a> {
        pub fn new(deps: Deps<'a>) -> Self {
            Self {
                deps,
                pubkeys: Vec::new(),
                msg: Vec::new(),
                signature: Vec::new(),
            }
        }
    }

    pub struct EmptyError;

    impl Into<Error> for EmptyError {
        fn into(self) -> Error {
            Error::SignatureVerification
        }
    }

    impl<'a> Verification for Bls12381Verifier<'a> {
        type Error = EmptyError;

        type CanonicalVoteProto = protos::cometbft::types::v1::CanonicalVote;

        fn filter_commit(
            &self,
            commit_sig: &cometbft_types::types::commit_sig::CommitSigRaw,
        ) -> Result<Option<ValidatorSig>, Self::Error> {
            if commit_sig.block_id_flag == 4 {
                let Some(signature) = commit_sig.signature.clone() else {
                    return Ok(None);
                };

                Ok(Some(ValidatorSig {
                    validator_address: commit_sig.validator_address,
                    timestamp: Timestamp::default(),
                    signature: Some(signature.into_vec()),
                }))
            } else {
                Ok(None)
            }
        }

        fn process_signature(
            &mut self,
            public_key: PublicKey,
            msg: Option<Vec<u8>>,
            signature: Option<Vec<u8>>,
        ) -> Result<(), Self::Error> {
            // TODO(aeryz): verify here
            let PublicKey::Bls12_381(public_key) = public_key else {
                panic!("invalid public key type");
            };
            self.pubkeys.push(public_key.into_vec());

            if let Some(msg) = msg {
                self.msg = msg;
            }

            if let Some(signature) = signature {
                self.signature = signature;
            }

            Ok(())
        }

        fn finish(&mut self) -> Result<(), Self::Error> {
            let pubkeys = self
                .pubkeys
                .iter()
                .flatten()
                .map(|x| *x)
                .collect::<Vec<_>>();

            let pubkey = self.deps.api.bls12_381_aggregate_g1(&pubkeys).unwrap();

            pub const DST_POP_G2: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";
            let hashed_msg = self
                .deps
                .api
                .bls12_381_hash_to_g2(cosmwasm_std::HashFunction::Sha256, &self.msg, DST_POP_G2)
                .unwrap();

            let valid = self
                .deps
                .api
                .bls12_381_pairing_equality(
                    &BLS12_381_G1_GENERATOR,
                    self.signature.as_ref(),
                    &pubkey,
                    &hashed_msg,
                )
                .unwrap();

            self.pubkeys = Vec::new();
            self.msg = Vec::new();
            self.signature = Vec::new();

            if valid {
                Ok(())
            } else {
                panic!("invalid signature");
            }
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
            &mut Ed25519Verifier::new(mock_dependencies().as_ref()),
        )
        .unwrap();
    }

    #[test]
    fn verify_bera_works() {
        let commits = r#"{"height":"3805039","round":0,"block_id":{"hash":"FCB377B5B317BBE949E06D676D79B5515012D852FF4AC2D7B20DA1C0441C9078","parts":{"total":1,"hash":"741065469F353605417CA88B80D536FC355FC3C533936B7B9E9861B4CCD02ED4"}},"signatures":[{"block_id_flag":4,"validator_address":"0x09C4AB26B20EF50B371EAF4CB006D6D5B72A53B9","timestamp":"0001-01-01T00:00:00Z","signature":"gCCadb8QHnhhHBCvwgsFO4Z36LlA6qF6NmLXgITBddwUmSjxG6s+uuNkQqSWo8CXBR58HCQkjqlFXYk27RRlVf3zbqP6cc94MtJL+XG2jyfaNkmJXgocdHRkp3l/7Yfq"},{"block_id_flag":5,"validator_address":"0x0EB600A5EB0DCCD405B3C71953C727975E39EDE4","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x17348776DE5BC1F4BE6F1DB84042DAC57D71C890","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x27219ACFC8E974C0DB5E137CC42E8427553802FC","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x5951C4349AB792BFB3A63956512663CC3B733D6E","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":6,"validator_address":"0x76D5AF272836142878B608C2125DCE729190EF5F","timestamp":"0001-01-01T00:00:00Z","signature":"tvEZ7g279H1n9IQ1K7Qj8xNdT7VEj7pl2qzEGZ0R8LRrYiZMZf2p5ZevC12T+/90AWdtOVoLZhv15y2/jMUUfdkQVh0idTiG3QdqCAtRjOXyKNnSQjHHg8paDrzBROZr"},{"block_id_flag":5,"validator_address":"0x8CAED645FC96CDA80F0217B018B2129481050AD9","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x9DB42D740B0D0C6A08543D679BF50CED94CA8E3C","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xCB1DCA2CBFD59BDE8B61C9FC95EEB65D0EDED2B3","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xD00F2981BCE83818D2EECFF87F85000DBDD0AF08","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xDD20845E62583B038100A652D5AA8611D9D4026E","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x310D164F10AAE6E85D6CD32CC64DDC2F48F5919B","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x5E1F812F4D7607E3E844D01862D32E9C1A956CB0","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x8B7419E9E076797E9F1C2160218DEE06B8B4478D","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xCAF0EAD038B1A2F9E8E2762999535BD85380C64E","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xD13490E5D3BE85D637544484AEBB970C47BA9844","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x7DF743C6CE84A99660FB91239E53754416943671","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x6B3D9D5DFE82E0BF52D6045DF17F16560A027717","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xC0B3BFA566E1CBB93BF8B358F29C8F7B10120987","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xEA9508CAB5DA1954EBE6CFD1425CF0E2D6EC4B0A","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xEACF2006E60A59979E5850B71702DDB354A2048B","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x901FB60F9AF7935456867CFC8355B70FA226A069","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xE37FB6E5CFCB23F8AA412A79D793C6A04F54747D","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x8C611647259F4440DF572119AFFA58148FFAE0D0","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x6D4649CD64A83E777CE89E0B268654E56DAC1F96","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xB29EA710A83B6D518A5566AC8AB90C2979CACC16","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xEFB1A37176B9D0252B10D7F56851E68CF096B5CA","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x90106A700F58A4BF7EBE5C083BDBBAF8A8761E5B","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xC52C7CDD8E64EF9581473E2AE286A96FB6A444A9","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x1ED255BA54A031E4ACE64B4AC9B0CD999AF1AB71","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x616EABCEC6FB0DB1AC54C5461D952BFB46B45819","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x06BC43DC34E9F52CFBBDAD93F9FA743BF5278E7F","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x2C1B7662D046B2FCAF685C804037D3FC728E9E80","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x4C79631C9E3BACA9D266C8FB3AD6A5304431D055","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x6DE14F6BCFECCE0F3F86936B6DB580258E542047","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x8F7E52CBD156EDD9317042F67AE7ABD1D64C9E19","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xB08ADE9A00DFC0A0EACB84BF579B6B6C0C32A645","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xAC26D89FB38D59464EE04CF66A44F24CE64DA7D4","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x0E1982A8083CF41A0B0CDF3CC1A42F5540FC211A","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x0634E825DCD51BF722CCF02086899CDE1E862E5A","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x19395121376B12A00F8C6AD5FE8C3AD9A77CBFC1","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x19D48D2EBF5A4D5C31228C47828BDF6BFDD7B77A","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x1D4DCFB35BE65B5E09C4270F0B4E3329F967B4E2","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x2FBB6C9798950CDE7647CB68602498874336999F","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x39CCAF0EB8D31A9592ABA51B6705ABBBFC38086C","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x3AECCB026FAEA18F43E79FE116CB1B94E768075C","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x41462F6C19492B6FF4F5FAF7D6ED16E1264D1AAE","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x45C2BA0FF19BCB262FA5713E993FBA4408258BC1","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x4D21CFCB799E0C9B66CE566F6AB68BCCCBC75FDA","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x4F5F37B9DB46C29C48C0B2231BEAD3E14F04CD1A","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x64703E2FCEA0339EDE427510BE1642F4E53FC4D0","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x65697A8BAA1891DC3D466DFEDBE51EAC264936B7","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x7299ADB3E975B1A7B461F45C4CCA6AFA39DBC9A7","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0x8A93B739376EBA41191E593847B1C844E66F29CA","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xA49685C59E968B2B097B0CB09E8D08D6B7FCD69E","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xC9D2DCB223D4353076E6A0E7E82DD99E42980CCA","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xCCB476A8C39D26D81428AB9FBD848F9B17E1BCE0","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":1,"validator_address":"0x0000000000000000000000000000000000000000","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xE1E054EDC9AEBA25B10965C927AB35D173080DC9","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xF1D993F20EFE45757E62F9AEB6BB14B15DBF2AE2","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xF314C84E265C6CF2D834F0CAAD99900CF6857FB2","timestamp":"0001-01-01T00:00:00Z","signature":null},{"block_id_flag":5,"validator_address":"0xFDCD7DBB810E66664F7ACA241E05834DDB79A5E0","timestamp":"0001-01-01T00:00:00Z","signature":null}]}"#;

        let commit: Commit = serde_json::from_str(commits).unwrap();

        let validators = r#"[{"address":"09C4AB26B20EF50B371EAF4CB006D6D5B72A53B9","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"GYrfc21g7s39OrUTaneaojI1xEu23gje8gafc+lG3pY7NaaNWmbWEdOnzUUDXcqPB0fd2AnTvyRiaUGJkKCh6KpDAzpactRmmYnvo4JwxQ5nPb6+DtlX257Z0rugRpPO"},"voting_power":"10000000000000000","proposer_priority":"41747661734825856"},{"address":"0EB600A5EB0DCCD405B3C71953C727975E39EDE4","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"DMq6EL3cM6T40qzdeFk4eahMdGb2QfHZtCOLIO4tBwaJSz5VsHRAmMULm0gh2jIHCWPtVaWPFUAJTGmJLQZbTELa/l4AWbGrMLOqm+guY0O7JOK0Q1uK2V/4Yx+rNqbn"},"voting_power":"10000000000000000","proposer_priority":"-84498120106402191"},{"address":"17348776DE5BC1F4BE6F1DB84042DAC57D71C890","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"AxmTFc82689qULq1coANeTJINfroMqPakjjzmcOf7Oti3kEznqtMyPeabU5ry4JcBwfwjyj4XpnQKKrtRwjSm1tlH9mVBadyqvpmnnZAgjt4ZfivckaJSMESmI0DvB62"},"voting_power":"10000000000000000","proposer_priority":"81775085425086853"},{"address":"27219ACFC8E974C0DB5E137CC42E8427553802FC","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"CL4Sa/2k7uGQ5sAaIkJy7XBkJIUeIDeRxyea7strUDBZkB2zWxgh8e/k5rRF9cyfCb8r27m45fmxOlvJdyOp3EEHEKj5tNDW7ZAyiTXWacPfQoEjXdFCtJefkKjrsqQ8"},"voting_power":"10000000000000000","proposer_priority":"97067641437565583"},{"address":"5951C4349AB792BFB3A63956512663CC3B733D6E","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"A/1TcQt1whFb0KrBKLc565+p4mJgPazfg0Awq7G/TIpsALtysxTBI9d/T/QM1NSaCgz/u+e0aLH/zEKYQWYVBMJAky+Kn4V3JX6562r5yJbCh0BqLLKStdeAXwGO0j+n"},"voting_power":"10000000000000000","proposer_priority":"28339985187565583"},{"address":"76D5AF272836142878B608C2125DCE729190EF5F","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"ENZKsqirm1+qzpIl0gXUfcC4FVWSs1S4YBNJKPfznxX1TZCdrXiXhoq6bcfn7vbIBGMvuu/gVTCx2/215AswF9h3d/vhW2QYp7qepQVoReY+tC8jmRFrRMziu9FXTwa1"},"voting_power":"10000000000000000","proposer_priority":"65205496556254427"},{"address":"8CAED645FC96CDA80F0217B018B2129481050AD9","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"AMZzGA2XITwcNf479OaE3TU0uqsjWhBtH3G5yKN+TTegVtR1RpZP0HVQHf9/dq6vFLKigAhjoKUNHgk/f5ETVk8+0lbI0CIZjpoMPMvccy3PJJLkI+g4+Sein20ic9VY"},"voting_power":"10000000000000000","proposer_priority":"-33264807797728722"},{"address":"9DB42D740B0D0C6A08543D679BF50CED94CA8E3C","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"A9D5DN7arBRQyNwIz6ZErgL1kYVyBBwvyn34tVM2aCy17bzK3W53Ep3kJYROFH0CBYU5e8fS/pywwNjC9RqT+Pz/2yGRQA1Dy5mGRXivGiRggXVWcFQzFpHk7upct4tt"},"voting_power":"10000000000000000","proposer_priority":"-56680014812434417"},{"address":"CB1DCA2CBFD59BDE8B61C9FC95EEB65D0EDED2B3","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"B1qvACQbFMzYYXbkuu0XDfZzVSmv0POPAez+iBy7YTBYkioDcoFLln466eiA2IZYCmHfQwFI6sJ6I5mq51J8XQ8N527u4whQybgLSzHJTkSM2gH4YEyR6+SJGg+kEEp9"},"voting_power":"10000000000000000","proposer_priority":"-76633930138154254"},{"address":"D00F2981BCE83818D2EECFF87F85000DBDD0AF08","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"Eo1vZr/Zyx7xjaaEOtnbbBtux+MJNwXJUiTo8gIy8kPnpifQkUQ2DUwXddj6/bDnA/JGLBmsIamicyjCsL4MvoTBUfn94ddnaBx+2km5aN3YlrAJoyMCl9Zc+UTGDJjJ"},"voting_power":"10000000000000000","proposer_priority":"-83381022529458601"},{"address":"DD20845E62583B038100A652D5AA8611D9D4026E","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"DdyItadCEbgO0rjFFpuFOAt0KKiwozgaRHDVIgPrIw8Ta0IzcDCHgwgoZvcWsGZREGMouYVDoUvaengK/0FUQDjmn5/lYl+hhsgFxxtJ/EWOfL7qONECm8QugcEJv3dv"},"voting_power":"10000000000000000","proposer_priority":"97457724121395729"},{"address":"310D164F10AAE6E85D6CD32CC64DDC2F48F5919B","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"ARR8bFk4un80Qj+d1HO/exu4g27UftP/JYFGyU1vEK7XPLo7RmitvjLKC1OpkDTzGOPumIJ3lwQK8jBF5Y+b0uOWp47yBeZyEql707LzhRT4qub1/3AqLfMjd5+f4T6a"},"voting_power":"9790000000000000","proposer_priority":"-63433764812434417"},{"address":"5E1F812F4D7607E3E844D01862D32E9C1A956CB0","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"BoiN9JHozNxbuUC53aUfp0SVGFk4IMnk6QM6e4f16fjeu7pqT2ghhxGJaQatQM5xAazPreAt/aV1OloZFFc/0QNG+6kw0IChtPokHBgI7HEaBsyo8bwBT+J18NBqK4Wx"},"voting_power":"9790000000000000","proposer_priority":"68687620967741936"},{"address":"8B7419E9E076797E9F1C2160218DEE06B8B4478D","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"AVh1qeVU5Ebl/NRjJF9Ne9aGOxpfUdM6yCjQb5GFxXBfHQpEK1LfFC7nTzAKAVUfGcM1giQZ9YgK2zTle9nZL4io6j4IZA3pfmY499SXiitUhGbNgWjO/+viKcEuqO9v"},"voting_power":"8930000000000000","proposer_priority":"-43754260922728722"},{"address":"CAF0EAD038B1A2F9E8E2762999535BD85380C64E","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"CtvLDZnq1Woaa03vYqJfZzoQ6FI43nfHQWjV5uwdjzqkRaIr6BITA3eFelhe77K+Ckujgf7wlApsmr7Z1NbRSPU8A/sEKf2+kWbSaxkfiDNyvzUjOFfdZaK3Qor8qH+0"},"voting_power":"8550000000000000","proposer_priority":"-17931687830727100"},{"address":"D13490E5D3BE85D637544484AEBB970C47BA9844","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"CS6NIiXOvu0l5UpZ9BZ2EHBPxQcXCntlMlhP+n9mwd2iDIOi8Uj7X37GFPNA7bpgCLy/RoqOhDkPia56+7JXveQTHjfL3W9zv2JQBH2xGm1yHvYWQVbQ4K3ql1ANwsmt"},"voting_power":"8020000000000000","proposer_priority":"82756452934955051"},{"address":"7DF743C6CE84A99660FB91239E53754416943671","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"AcyiCU8B/MVPHpG50DBnGG8fWm0YJvH32Kh344tR42UBChQMi6Wff6nt15p3MNSkBMH2W+Mnzqg2MN/twIwe53k/cuG6CNmmJdgggsdNwDeyJaCdHkWe+c/0bEhl5Ql0"},"voting_power":"5830000000000000","proposer_priority":"-95941687830727100"},{"address":"6B3D9D5DFE82E0BF52D6045DF17F16560A027717","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"CVQ8PlAlGuNVg1SE+vfLONrPypQ7l/75ye4eRySCWKcZjaQXv4/Gdx+n7xaorewPA6MWfE1zpUqLp50ofSmffRye+02ysa4XXKq/r07bIEoSxRcdiciJUedqf9Zu/RL8"},"voting_power":"5000000000000000","proposer_priority":"-62470014812434417"},{"address":"C0B3BFA566E1CBB93BF8B358F29C8F7B10120987","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"BkldPAKxkhY6aM0pIc/u+EruIBi/20n90AY9KPetB/UX9DJtHNUV9ImmawVxUllHF/xjqKxwyi5F7jJ/i7TOrETOLMaLmuNIUjum/1CRi5S+1wF49h1/ffIq+wM8wa0z"},"voting_power":"5000000000000000","proposer_priority":"-9746435261927391"},{"address":"EA9508CAB5DA1954EBE6CFD1425CF0E2D6EC4B0A","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"BKz9OKE68SrdjYLh7whCxN/B5BdfrluKtzdw+QUMv2c8r9v22Ktnn+nqEyCPULSFBcIGg3QJemNkNX6NRC3eIJAWBFXm9D397gii7YgbT7F7Y/oDNTwOafCEkK8OzTOJ"},"voting_power":"3830000000000000","proposer_priority":"-101089772529458601"},{"address":"EACF2006E60A59979E5850B71702DDB354A2048B","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"E44iqd+KCVo1VMGn3qu54J+t27aoAcfT7+nWHzP58aLoChyouQkIsi5OS3phcVAFEkHG+D77FSkL809zVKWGPa4yUOmF7ttaFzOHpMQm2sXNHagm/nLvPm5zBczyHeQr"},"voting_power":"3760000000000000","proposer_priority":"7695279893597809"},{"address":"901FB60F9AF7935456867CFC8355B70FA226A069","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"Djs1vTeWUgGf6PNBxchUpg9VdyAWNAeB6NNXfisk4awNV4bUVFnNdWgQKB7/j3KcDNr+/Fkd0rMa7Zjs4scsaL58+bubCl1Ji0/AgpN2s1QHQgkJafb1i+f21BqLEgWB"},"voting_power":"3600000000000000","proposer_priority":"33489985187565583"},{"address":"E37FB6E5CFCB23F8AA412A79D793C6A04F54747D","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"FQFQf2dyjsewkpa/h/StEzr496/ACSRyegmx3twXyahow22QnP9ocwwdpm3PwehiGKm45csmb6M6KRxwpc9w41CoJpIN2i42ISWBDVZx62bNxLO8rHREaAUcw5qILeNd"},"voting_power":"2020000000000000","proposer_priority":"-68703719844121513"},{"address":"8C611647259F4440DF572119AFFA58148FFAE0D0","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"FmUrui8vO2tPz9/phH9qVgSWXQ66w3wfDDd02ghmFAq2lzT+7A6wdh+OCNgJyeErCHs8CzyRKqKZTJ1r+Hn7pqqGy1HIAvQmDjyol7e5ko8wxnN18g/39JYaNFBjyKRa"},"voting_power":"830000000000000","proposer_priority":"4388981404739276"},{"address":"6D4649CD64A83E777CE89E0B268654E56DAC1F96","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"CcvVQsc3zKS8M/HqUISoV6diAEL+N/0ybs9a62HyzglgQ80O1XukRpPPYGl4tWa6ChoteymmyidJDvIhwfc5tC0qKT99EgDFeLf7D51W32CRqkXJC0wFdoKctjaYV48+"},"voting_power":"790000000000000","proposer_priority":"-120169737753461015"},{"address":"B29EA710A83B6D518A5566AC8AB90C2979CACC16","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"DQqUpaK7Aonrsk5zIz9a5lCNY1Ai2KEsafzpl5F4O9dqs/0ztjBKX+QrhSAdsS8YBFL+1YflGfaJhND6IMnH+jN6vxMEDxk4kP8BM1zj6IFc00w/ZGcfVAtFBxUrfkGw"},"voting_power":"750000000000000","proposer_priority":"4449985187565583"},{"address":"EFB1A37176B9D0252B10D7F56851E68CF096B5CA","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"C/ttnrB4Xle0Ps/93V2QY3XdYn5zOBN6d8AniO1tVpsaCY/ywkmadA44KTCzm85IEEyXRS8b2Cf4xmF3Qspk7r9eoKF851rmJoyQeR46cnf9ovw+d5VvfZFm0xotvite"},"voting_power":"750000000000000","proposer_priority":"4419985187565583"},{"address":"90106A700F58A4BF7EBE5C083BDBBAF8A8761E5B","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"EGSbG8rhlxLpFE5ymq4uPziu2T1xTo6qwXvlIE8vWp/vvViCD6FEH+m84mjXJLPsFWAQ4lBiiBeAuHXhet9Uktg6m9N7rRctHcfr69p3RcCpTgg8zYMVE3fAin+feYlt"},"voting_power":"660000000000000","proposer_priority":"-119934797065044949"},{"address":"C52C7CDD8E64EF9581473E2AE286A96FB6A444A9","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"BloE/D+yNLaVlfxGSwBgTo4fOH1H3GvKLRUDqC0uIAYDNO8nXHZBNvE7h0R709CoAAbc55eLPbDAWY650LOhVzSiWQjJDiQGZQMnaodPe/HrBtk5zd8Nu93A9g6b2Ifv"},"voting_power":"560000000000000","proposer_priority":"49169985187565583"},{"address":"1ED255BA54A031E4ACE64B4AC9B0CD999AF1AB71","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"F3CE0rkrqj4B7Fzlf71SnTyOYJQcY3vcfIXZyizFZUXDy+qGh53L6UAvilXQKE57GU+ohHee8GZwBCtr3cnkCiAAYlsItliY8fubfKGUK/j9XtnnphWQFQLt/SbUPY9j"},"voting_power":"550000000000000","proposer_priority":"-89070014812434417"},{"address":"616EABCEC6FB0DB1AC54C5461D952BFB46B45819","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"C5jqc/OvqwQVSCn40SpTfL8xqLCPf444cPSQIAHwARI07Y+SGLQLTtcy0R2In2CdExtiQmB73wifeqeuab0Eso+wtfEQew9PdQYd9zzii7x+NzO2sKd/+LqhqF5CSV+w"},"voting_power":"550000000000000","proposer_priority":"50179985187565583"},{"address":"06BC43DC34E9F52CFBBDAD93F9FA743BF5278E7F","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"AyFTvz4JucqxRBRCWg6664ieIdIIcuu5kO2aYQLX3H8wF9Ron5MajpbZGL3rGE4bCzYYk+w0mnb0MImdZs7jZputCG7GCkoyQPZ8x0eZldZGttrmxfie0gTi40m2Mzyz"},"voting_power":"500000000000000","proposer_priority":"-47240014812434417"},{"address":"2C1B7662D046B2FCAF685C804037D3FC728E9E80","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"GBEabuVp8b4AVVHYx24EKqvLW/TEkdF3RForX7MN7ce/bbOeOHgxxnOb3kEpxq2lDMVRABiwdQj2Ugm/6xtmLOxw1DX+9lYnSIdTP5/EC8mSX0/C5oYFvr1lYxst16VC"},"voting_power":"500000000000000","proposer_priority":"-52580014812434417"},{"address":"4C79631C9E3BACA9D266C8FB3AD6A5304431D055","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"FgBSxVCcqigCGPPs89p7pb9OwguX5sUnAN2TUV706WOBOqkqhzHJ4TexAn28dxAvCwzBINbhypK9BpKy5mNaSHMIjwieRMx9rTas7uFS46FkWDECipnwnrEAgGX9hsNC"},"voting_power":"500000000000000","proposer_priority":"-53280014812434417"},{"address":"6DE14F6BCFECCE0F3F86936B6DB580258E542047","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"AU+jeh1MZTJr8JHTuw5WB/3olGRx7ZI+aBUHf6xygtiYIN1Ph2XkCFyoJH+5nstrEfeNQ4sabcJ0pOOXdIBWxuQC34iCrAiKV/6h6HNBp71n28Q+spq5zdLyyHFjp8TA"},"voting_power":"500000000000000","proposer_priority":"75409985187565583"},{"address":"8F7E52CBD156EDD9317042F67AE7ABD1D64C9E19","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"AkfCH3PDMn8x+O4Nu0ARDvCoMK+JuDmG3d+S0b0aaLiPkFC8ag8UBct1+7RX4+g7F5ag/BG0s+ve19rMuez1jOxYByefy1oFZTZElpJLn8iENwRUjQIrvQ+d9j1LG9PE"},"voting_power":"500000000000000","proposer_priority":"73809985187565583"},{"address":"B08ADE9A00DFC0A0EACB84BF579B6B6C0C32A645","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"CYhPyVq8uCc212j8itT99MshEklpdK4FRA2DXQ6TIWZDriErNl+22dJQHXbQkl7zAnCWXYjmhW8hIW3Yq/7ce5rrPbRjC0Gqnnj0aSTosGfwb9s4AM3OIo5+8baA1SRw"},"voting_power":"500000000000000","proposer_priority":"72769985187565583"},{"address":"AC26D89FB38D59464EE04CF66A44F24CE64DA7D4","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"GCp5HXw9cu+mdZ4CUHhTRiZtbHDtiBQk7GOtTQYJBJg7xXkD+hM6m8AMLW+bEpZND7LfxrqL6L40wa4ilxdKEI4su4rs3YghWDX54VaoB2UyU8Vg4BeRNV3ozBa5WR7h"},"voting_power":"470000000000000","proposer_priority":"8642377631154370"},{"address":"0E1982A8083CF41A0B0CDF3CC1A42F5540FC211A","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"FyZslMSQZiR5qRiLBwyG5QXfTxZwFog69BFMjBpxQp6PJwwMLxt0N1iA+Bgot2jADCXB47VWXBu6uuVJGD9N8yPdeSU/qPmVTDw86HkfpHuZtCKrRbYdONnJ02Snwlr3"},"voting_power":"410000000000000","proposer_priority":"-22320014812434417"},{"address":"0634E825DCD51BF722CCF02086899CDE1E862E5A","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"CSM9VMxU5ny6uUk683vL7aqd75LBRjdpS/iPZ1sszQpsmKjd4AnW2A9RXxGohFUXDMPZ1qxO/L59VE2fam7usUXbY4oA4YOBaPjECjx7tjIzw4Mxnz72MvG75X/HU/6B"},"voting_power":"250000000000000","proposer_priority":"74539985187565583"},{"address":"19395121376B12A00F8C6AD5FE8C3AD9A77CBFC1","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"BHqmvRV9+mq29BE4SmYsPU1Ph0UhfqgZhHDW8xNb1xlY/YJCVv2/FrIUlOwwgLfuEaoutJru4CWKH0fcplYZsPX5LgoqiC5NkugSjeXHuBAp1LhBeXoyLjxkKrXN9kd/"},"voting_power":"250000000000000","proposer_priority":"63749985187565583"},{"address":"19D48D2EBF5A4D5C31228C47828BDF6BFDD7B77A","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"FXeqB/NboWUbJm9/MW7RzRCQfQBk4B28tQLsjDm8H8AazeB/R1WtDUFIMvouK1YpGd0X0TbYwCDiH062884AVaaLM1+8J7G9huh11KEdJQnBu2DGkx2juZdrKF6RN0mx"},"voting_power":"250000000000000","proposer_priority":"63529985187565583"},{"address":"1D4DCFB35BE65B5E09C4270F0B4E3329F967B4E2","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"A1Ocoo4P100qPExVJ0C+d9aRTK0tjsFlg0ksxX6M+jWMYuMcyRBrFwDMFplihVpvDnfwL1wsTXHtBG/lQjntx5vkyss0fTo92658BELNJ1yLVxiJC1LUxsZ0ERLTInbX"},"voting_power":"250000000000000","proposer_priority":"63529985187565583"},{"address":"2FBB6C9798950CDE7647CB68602498874336999F","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"ADBYMfgcaIpoFOv52bhC4oogtCWGTEgPq29S5BVBhT6rOed8poJI0Ja9TVSZskt+A/+mSH8NeGqPQeClkt+s0w7IXsEYS4pz4w/L6iQnCsfi/NWX6euLS7fJA1jYqNp8"},"voting_power":"250000000000000","proposer_priority":"38009985187565583"},{"address":"39CCAF0EB8D31A9592ABA51B6705ABBBFC38086C","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"Ccfw1Mus5WKUtiT2+hsnvqpa/J2rcHI4PF9j9jbp94+ehoqC5mhKcgaBmUYfHxwVC6edjUpeWRYVvAuj2L0txxTbcWNTwtcD6CKH6FcbanSC5afcYGEzdMLyrKHJaE9N"},"voting_power":"250000000000000","proposer_priority":"37359985187565583"},{"address":"3AECCB026FAEA18F43E79FE116CB1B94E768075C","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"B/MYYGXz8YpLIbZh9AlBuULa05Bi2d85nZShiHDgYA8cyrUe5xyl1vlxt5wuUdbeDbLaHgDUFoQE60oNF1jCsuTu1/lW4+mNns3fXSMO7PROhkiAgVcu0Mh5nwMpRTcE"},"voting_power":"250000000000000","proposer_priority":"31669985187565583"},{"address":"41462F6C19492B6FF4F5FAF7D6ED16E1264D1AAE","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"B2mFjKvL4vojmZfmduN48gN6Z5oc9nRnnEDkEP+p5ykheBwkr3zHmWk2FVnohhXeEp/OSvv5GjdBK9UUnHpWMezNaiL/DOzOnyBCPBxlrwfXNYJAicA6vPaGzKBHadFn"},"voting_power":"250000000000000","proposer_priority":"17879985187565583"},{"address":"45C2BA0FF19BCB262FA5713E993FBA4408258BC1","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"EvqlH5zxqMgYRZA0lB2WVEW9oVshJmH3Twk5Ty/r5Awfj+66DO3K/UrSipRK+yssFxULWXOgDVKzgQwN1LoPNecfCwh7Rt5jQLpSsiaqaXkZKOFwfl6vmw+4m1sEwDzS"},"voting_power":"250000000000000","proposer_priority":"17869985187565583"},{"address":"4D21CFCB799E0C9B66CE566F6AB68BCCCBC75FDA","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"DbDC80dfvpAxDnsu1ghUba56pdD+fz0ndcLhirDoc2i8jGcN4CH3dA8vgubVSoExBt1V6BdzHxPNkAntv77z2zCVhOlxNPgRJKnxFmK8oGmymNxGVNMffgCRBTFU0Ddq"},"voting_power":"250000000000000","proposer_priority":"17579985187565583"},{"address":"4F5F37B9DB46C29C48C0B2231BEAD3E14F04CD1A","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"FAzIcB00TEEFSj0Ci7X18HAvfCKLIr3JCBVBgIwQGAjT2ywTDWQ1nv3XWJNluPBbFz6mrtELxVv9ZHorQN5i7NICAs7H8u3vHIRvOab+qsh+tic323Xmcpt20z8vS/rZ"},"voting_power":"250000000000000","proposer_priority":"16479985187565583"},{"address":"64703E2FCEA0339EDE427510BE1642F4E53FC4D0","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"AUjdtHiikc9qFvuYGV4wdqrTGcqIIbSu9pyNHUXI7UnWEQmhy8AptY8i9btghU/kBZw6jfFdp7CUc1AcUE+n1UwdTmBXYsR42Wi2iGLlCU1+13nNFx5SoAvCAsQWwxR9"},"voting_power":"250000000000000","proposer_priority":"15779985187565583"},{"address":"65697A8BAA1891DC3D466DFEDBE51EAC264936B7","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"EwEr3Pa6qHwXN98DxfrHyLtEcoL7yl0t5Ccm7GfyN9ZqL4Z4U+MuvSlQEPB18i6VCDZdUqbzWlOWr4OgY3/KQOEBpVJD3SvJnmgZa2VZw8/qtnbqUe7DeONvLOUnhROg"},"voting_power":"250000000000000","proposer_priority":"15479985187565583"},{"address":"7299ADB3E975B1A7B461F45C4CCA6AFA39DBC9A7","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"BhlLTcX3yL0AZmy+GpSQ6uMwb/OXxNGYOV3ZOBAPw4KOLQcqG/byVimfnNuNK0LwF8n1aNmM9BdvohjDqlB/1iFQ8GNbgIkxHwlUkv9/C1tk5kavqBoj4uK9h+GVwKI2"},"voting_power":"250000000000000","proposer_priority":"1779985187565583"},{"address":"8A93B739376EBA41191E593847B1C844E66F29CA","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"CVBOiObpab0xoZTHyB3w5EtGRNDbga1xLVUCPFxuGsZWVY8nyPi6xlwf5yxZfmV4FiSu3FmeRYm8r39/WxoWiU6kRDhKpvHkYzdCbZ8w6AZOaMb6dgDfGMelkPqmZTvq"},"voting_power":"250000000000000","proposer_priority":"1779985187565583"},{"address":"A49685C59E968B2B097B0CB09E8D08D6B7FCD69E","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"GL7vQFPQl6oVJ6cDAQIaNj/TuSdUpEaTL1yX59qpyNC88u0xXnpVd1qLvl6+E0/7AMxG7T56AVXgxICoxHje7ls30spnhedGcMfgZsUWUMpRTw5OVDXmfK2q0j4jg37C"},"voting_power":"250000000000000","proposer_priority":"389985187565583"},{"address":"C9D2DCB223D4353076E6A0E7E82DD99E42980CCA","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"AjKoG16DS4F9sB2F7hPjZVK0hBNiYofeURtsibe4/0pEjoZXE/0hyY8UZ6WP5u/lEjW4ItlXkNjQoG4t5asOMDbccBtl3SB32CeDMEm+zf6kauz7GFFcBmQcDBcTRG9+"},"voting_power":"250000000000000","proposer_priority":"-5910014812434417"},{"address":"CCB476A8C39D26D81428AB9FBD848F9B17E1BCE0","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"Fk3QYy0O13LdC1omnpgJvlt0LfCtRGNVLHtRvu9rr2AhNg22CwQo/xxMOGYcArKDBen050Z8Mk8Dg3yu64aGrWQg5OMONfB0f2/UYqgQw8VGnCqiuPeCnZ8N+TWy3F5S"},"voting_power":"250000000000000","proposer_priority":"-9760014812434417"},{"address":"D94347C994432FFC131B1FA33A61159E5FAB7D5F","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"EVgLQSQx/3ocyhYEXwmWdApeUuvRrv9jWUFdmYhriaNQaza9ca8TKl3LmJ56ibWNENZu/Il03uJwt/XyUmbgT/HADDMYj9vvqo/mAmXZ5ihJ6dX7wgf/NDIEcy6RqsE4"},"voting_power":"250000000000000","proposer_priority":"-9760014812434417"},{"address":"E1E054EDC9AEBA25B10965C927AB35D173080DC9","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"A64AD0DAbBrxxA1xM3Cl+/LMKP3WoDs2t1Gh6yilaZ7qRp17M3mBCB+sxzjDJgkdEaKX5fr0eMOjzXGJA+NHii2oC3qUhV9U9k6JqJypid3YQUB231qCtXUWoetuw9U+"},"voting_power":"250000000000000","proposer_priority":"-10090014812434417"},{"address":"F1D993F20EFE45757E62F9AEB6BB14B15DBF2AE2","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"ElXjF/fmQV6rBTGAyONrX2MSwebuHohKJsSQqkEhWVV0ZW7YnIPg1q/V4rM8ALEBGbd1P4yL9KviqczOrx6u8abbp13BmWXRBy5PF+2bkJj2d6gRSo9w6LpY26oaxei8"},"voting_power":"250000000000000","proposer_priority":"-15930014812434417"},{"address":"F314C84E265C6CF2D834F0CAAD99900CF6857FB2","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"GLckC2qmAoPvJzOEDcCZaxoWtos4+cvYPlt92lFtR7gnlEnxTFtqPSvY50gWziyuDJXsu29AXuMTzPNObw/oCFMRGoT0kQaeocmaKhsSqMS7c4xQKrGZjHJfzSl3CBTK"},"voting_power":"250000000000000","proposer_priority":"-24320014812434417"},{"address":"FDCD7DBB810E66664F7ACA241E05834DDB79A5E0","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"C8p4faS+iwk8Dou55iMR7mFFkUDNYWqtSNvKsjOhY9qFmUhpmD50ZY+lOvWVQy30ALxPLiDTVxqATTfrZrHSaomkud0/wCHQhrVs+Pnq4/RgPlmN0O2NYQ+m0lAKFn+h"},"voting_power":"250000000000000","proposer_priority":"-46980014812434417"}]"#;

        let validators: Vec<Validator> = serde_json::from_str(validators).unwrap();
        let validator: Validator = serde_json::from_str(r#"{"address":"7DF743C6CE84A99660FB91239E53754416943671","pub_key":{"type":"cometbft/PubKeyBls12_381","value":"AcyiCU8B/MVPHpG50DBnGG8fWm0YJvH32Kh344tR42UBChQMi6Wff6nt15p3MNSkBMH2W+Mnzqg2MN/twIwe53k/cuG6CNmmJdgggsdNwDeyJaCdHkWe+c/0bEhl5Ql0"},"voting_power":"5830000000000000","proposer_priority":"-95941687830727100"}"#).unwrap();

        verify_commit(
            "mainnet-beacon-80094",
            &ValidatorSet {
                validators,
                proposer: validator,
                total_voting_power: 10,
            },
            &commit,
            10,
            true,
            &mut Bls12381Verifier::new(mock_dependencies().as_ref()),
        )
        .unwrap();
    }
}
