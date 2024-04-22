use cosmwasm_std::{Binary, Deps};
use ethereum_verifier::BlsVerify;
use unionlabs::{
    bls::{BlsPublicKey, BlsSignature},
    cosmwasm::wasm::union::custom_query::{
        query_fast_aggregate_verify, query_verify, UnionCustomQuery,
    },
    ensure,
};

pub struct VerificationContext<'a> {
    pub deps: Deps<'a, UnionCustomQuery>,
}

impl<'a> BlsVerify for VerificationContext<'a> {
    fn aggregate_verify<'pk>(
        &self,
        public_keys: impl IntoIterator<Item = &'pk BlsPublicKey>,
        msg: Vec<u8>,
        signature: BlsSignature,
    ) -> Result<(), ethereum_verifier::Error> {
        let public_keys_: Vec<_> = public_keys.into_iter().cloned().collect();

        let is_valid = query_fast_aggregate_verify(
            self.deps,
            public_keys_
                .clone()
                .into_iter()
                .map(|x| Binary(x.into()))
                .collect(),
            msg.clone().into(),
            Binary(signature.into()),
        )
        .map_err(|e| ethereum_verifier::Error::CustomError(e.to_string()))?;

        ensure(
            is_valid,
            ethereum_verifier::Error::CustomError(format!(
                "signature cannot be verified: public_keys: {:#?}, msg: {:#?}, signature: {}",
                public_keys_, msg, signature
            )),
        )
    }

    fn verify(
        &self,
        aggregate_public_key: &BlsPublicKey,
        msg: Vec<u8>,
        signature: BlsSignature,
    ) -> Result<(), ethereum_verifier::Error> {
        let is_valid = query_verify(
            self.deps,
            Binary(aggregate_public_key.clone().into()),
            msg.clone().into(),
            Binary(signature.into()),
        )
        .map_err(|e| ethereum_verifier::Error::CustomError(e.to_string()))?;

        ensure(
            is_valid,
            ethereum_verifier::Error::CustomError(format!(
                "signature cannot be verified: public_key: {:#?}, msg: {:#?}, signature: {}",
                aggregate_public_key, msg, signature
            )),
        )
    }
}
