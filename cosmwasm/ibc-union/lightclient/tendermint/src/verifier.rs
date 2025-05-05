use cometbft_types::{crypto::public_key::PublicKey, types::block_id_flag::BlockIdFlag};
use cosmwasm_std::Deps;
use tendermint_verifier::{
    error::Error,
    types::{ValidatorSig, Verification},
};

pub struct Ed25519Verifier<'a> {
    deps: Deps<'a>,
    pubkeys: Vec<Vec<u8>>,
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

#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("timestamp must be set for the committed validators")]
    TimestampMustBeSet,
    #[error("signature must exist for the committed validators")]
    SignatureMustExist,
    #[error("message must exist for all commits")]
    MessageMustExist,
    #[error("invalid public key type: {0}")]
    InvalidPublicKeyType(String),
    #[error("inner verification")]
    InnerVerification(#[source] cosmwasm_std::VerificationError),
    #[error("signature verification failed")]
    SignatureVerificationFailed,
}

impl From<VerificationError> for Error {
    fn from(value: VerificationError) -> Self {
        Error::ClientSpecific(Box::new(value))
    }
}

impl<'a> Verification for Ed25519Verifier<'a> {
    type Error = VerificationError;

    type CanonicalVoteProto = protos::tendermint::types::CanonicalVote;

    fn filter_commit(
        &self,
        commit_sig: &cometbft_types::types::commit_sig::CommitSigRaw,
    ) -> Result<Option<ValidatorSig>, Self::Error> {
        if commit_sig.block_id_flag == Into::<i32>::into(BlockIdFlag::Commit) {
            let timestamp = commit_sig
                .timestamp
                .ok_or(VerificationError::TimestampMustBeSet)?;
            let signature = commit_sig
                .signature
                .clone()
                .ok_or(VerificationError::SignatureMustExist)?;

            Ok(Some(ValidatorSig {
                validator_address: commit_sig.validator_address.into_encoding(),
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
        let msg = msg.ok_or(VerificationError::MessageMustExist)?;
        let signature = signature.ok_or(VerificationError::SignatureMustExist)?;

        let PublicKey::Ed25519(public_key) = public_key else {
            return Err(VerificationError::InvalidPublicKeyType(
                "public key type must be ed25519".to_string(),
            ));
        };

        // TODO(aeryz): verify here
        self.pubkeys.push(public_key.into_vec());
        self.msgs.push(msg);
        self.signatures.push(signature);

        Ok(())
    }

    fn finish(&mut self) -> Result<(), Self::Error> {
        let pubkeys: Vec<&[u8]> = self.pubkeys.iter().map(|x| x.as_slice()).collect();
        let msgs: Vec<&[u8]> = self.msgs.iter().map(|x| x.as_slice()).collect();
        let sigs: Vec<&[u8]> = self.signatures.iter().map(|x| x.as_slice()).collect();

        let ret = if self
            .deps
            .api
            .ed25519_batch_verify(&msgs, &sigs, &pubkeys)
            .map_err(VerificationError::InnerVerification)?
        {
            Ok(())
        } else {
            Err(VerificationError::SignatureVerificationFailed)
        };

        self.pubkeys = Vec::new();
        self.msgs = Vec::new();
        self.signatures = Vec::new();

        ret
    }
}
#[cfg(feature = "bls")]
pub mod bls {
    use cometbft_types::crypto::public_key::PublicKey;
    use cosmwasm_std::{Deps, BLS12_381_G1_GENERATOR};
    use sha2::Digest;
    use tendermint_verifier::types::HostFns;

    pub struct Bls12Verifier<'a> {
        pub(crate) deps: Deps<'a>,
    }

    impl<'a> Bls12Verifier<'a> {
        pub fn new(deps: Deps<'a>) -> Self {
            Self { deps }
        }
    }

    impl HostFns for Bls12Verifier<'_> {
        fn verify_signature(
            &self,
            pubkey: &cometbft_types::crypto::public_key::PublicKey,
            msg: &[u8],
            sig: &[u8],
        ) -> bool {
            match pubkey {
                PublicKey::Bls12_381(ref pubkey) => {
                    let msg = if msg.len() > 32 {
                        sha2::Sha256::new().chain_update(msg).finalize().to_vec()
                    } else {
                        msg.to_vec()
                    };

                    let valid = self.deps.api.bls12_381_pairing_equality(
                        &BLS12_381_G1_GENERATOR,
                        sig,
                        pubkey,
                        &msg,
                    );

                    valid.is_ok()
                }
                _ => false,
            }
        }

        fn verify_batch_signature(
            &self,
            pubkeys: &[cometbft_types::crypto::public_key::PublicKey],
            msgs: &[&[u8]],
            sigs: &[&[u8]],
        ) -> bool {
            if pubkeys.len() != msgs.len() || pubkeys.len() != sigs.len() {
                return false;
            }

            for ((key, msg), sig) in pubkeys.iter().zip(msgs).zip(sigs) {
                if !self.verify_signature(key, msg, sig) {
                    return false;
                }
            }

            true
        }
    }
}
