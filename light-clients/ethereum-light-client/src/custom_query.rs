use cosmwasm_std::{Binary, Deps};
use ethereum_verifier::BlsVerify;
use unionlabs::{
    bls::{BlsPublicKey, BlsSignature},
    cosmwasm::wasm::union::custom_query::{query_fast_aggregate_verify, UnionCustomQuery},
};

pub struct VerificationContext<'a> {
    pub deps: Deps<'a, UnionCustomQuery>,
}

impl<'a> BlsVerify for VerificationContext<'a> {
    fn fast_aggregate_verify<'pk>(
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
            Binary(signature.clone().into()),
        )
        .map_err(|e| ethereum_verifier::Error::CustomError(e.to_string()))?;

        if is_valid {
            Ok(())
        } else {
            Err(ethereum_verifier::Error::CustomError(format!(
                "signature cannot be verified: public_keys: {:#?}, msg: {:#?}, signature: {}",
                public_keys_, msg, signature
            )))
        }
    }
}
