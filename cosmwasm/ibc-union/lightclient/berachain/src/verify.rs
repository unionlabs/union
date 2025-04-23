use cometbft_types::crypto::public_key::PublicKey;
use cosmwasm_std::{Deps, BLS12_381_G1_GENERATOR};
use tendermint_verifier::{error::Error, types::Verification};
use unionlabs::google::protobuf::timestamp::Timestamp;

pub const DST_POP_G2: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

pub struct Bls12381Verifier<'a> {
    deps: Deps<'a>,
    pubkeys: Vec<Vec<u8>>,
    msg: Option<Vec<u8>>,
    signature: Option<Vec<u8>>,
}

impl<'a> Bls12381Verifier<'a> {
    pub fn new(deps: Deps<'a>) -> Self {
        Self {
            deps,
            pubkeys: Vec::new(),
            msg: None,
            signature: None,
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
    #[error("multiple signatures are provided, the light client is only capable of verifying a single aggregated signature")]
    MultipleSignaturesAreProvided,
    #[error("message must be set before doing the verification")]
    MessageNotSet,
    #[error("signature must be set before doing the verification")]
    SignatureNotSet,
}

impl Into<Error> for VerificationError {
    fn into(self) -> Error {
        Error::ClientSpecific(Box::new(self))
    }
}

impl<'a> Verification for Bls12381Verifier<'a> {
    type Error = VerificationError;

    type CanonicalVoteProto = protos::cometbft::types::v1::CanonicalVote;

    fn filter_commit(
        &self,
        commit_sig: &cometbft_types::types::commit_sig::CommitSigRaw,
    ) -> Result<
        Option<(
            unionlabs::primitives::H160,
            unionlabs::google::protobuf::timestamp::Timestamp,
            Option<Vec<u8>>,
        )>,
        Self::Error,
    > {
        if commit_sig.block_id_flag == 4 {
            let Some(signature) = commit_sig.signature.clone() else {
                return Ok(None);
            };

            Ok(Some((
                commit_sig.validator_address,
                Timestamp::default(),
                Some(signature.into_vec()),
            )))
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
            return Err(VerificationError::InvalidPublicKeyType(
                "public key must be of type bls12381".to_string(),
            ));
        };
        self.pubkeys.push(public_key.into_vec());

        if msg.is_some() {
            if self.msg.is_some() {
                return Err(VerificationError::MultipleSignaturesAreProvided);
            }
            self.msg = msg;
        }

        if signature.is_some() {
            if self.signature.is_some() {
                return Err(VerificationError::MultipleSignaturesAreProvided);
            }
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

        let Some(msg) = self.msg.take() else {
            return Err(VerificationError::MessageNotSet);
        };

        let hashed_msg = self
            .deps
            .api
            .bls12_381_hash_to_g2(cosmwasm_std::HashFunction::Sha256, &msg, DST_POP_G2)
            .unwrap();

        let Some(signature) = &self.signature.take() else {
            return Err(VerificationError::SignatureNotSet);
        };

        let valid = self
            .deps
            .api
            .bls12_381_pairing_equality(&BLS12_381_G1_GENERATOR, signature, &pubkey, &hashed_msg)
            .unwrap();

        self.pubkeys = Vec::new();

        if valid {
            Ok(())
        } else {
            Err(VerificationError::SignatureVerificationFailed)
        }
    }
}
