use cosmwasm_std::{Binary, Deps};
use ethereum_verifier::{error::InvalidSignature, verify::BlsVerify};
use unionlabs::{
    cosmwasm::wasm::union::custom_query::{query_fast_aggregate_verify, UnionCustomQuery},
    ensure,
    hash::{H384, H768},
};

pub struct VerificationContext<'a> {
    pub deps: Deps<'a, UnionCustomQuery>,
}

impl<'a> BlsVerify for VerificationContext<'a> {
    fn fast_aggregate_verify<'pk>(
        &self,
        public_keys: impl IntoIterator<Item = &'pk H384>,
        msg: Vec<u8>,
        signature: H768,
    ) -> Result<(), ethereum_verifier::error::Error> {
        let public_keys: Vec<_> = public_keys.into_iter().cloned().collect();

        let is_valid = query_fast_aggregate_verify(
            self.deps,
            public_keys
                .clone()
                .into_iter()
                .map(|x| Binary(x.into()))
                .collect(),
            msg.clone().into(),
            Binary(signature.into()),
        )
        .map_err(ethereum_verifier::error::Error::CustomQuery)?;

        ensure(
            is_valid,
            ethereum_verifier::error::Error::InvalidSignature(Box::new(InvalidSignature {
                public_keys,
                msg,
                signature,
            })),
        )
    }
}
