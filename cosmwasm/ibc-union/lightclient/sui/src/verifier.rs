use cosmwasm_std::{Deps, HashFunction};
use sui_verifier::SignatureVerification;

use crate::error::Error;

pub struct Verifier<'a> {
    pub deps: Deps<'a>,
}

impl<'a> SignatureVerification for Verifier<'a> {
    type Error = Error;

    fn verify_signature(
        &self,
        public_keys: &[sui_light_client_types::crypto::AuthorityPublicKeyBytes],
        msg: &[u8],
        signature: &sui_light_client_types::crypto::AggregateAuthoritySignature,
    ) -> Result<(), Self::Error> {
        let pubkeys = public_keys
            .into_iter()
            .flat_map(|x| x.0)
            .collect::<Vec<u8>>();

        let aggregate_pubkey = self.deps.api.bls12_381_aggregate_g2(&pubkeys).unwrap();

        let hashed_msg = self
            .deps
            .api
            .bls12_381_hash_to_g1(HashFunction::Sha256, &msg, Self::BLS_DST)
            .unwrap();

        let valid = self
            .deps
            .api
            .bls12_381_pairing_equality(
                signature.0.as_ref(),
                &Self::BLS_GENERATOR,
                &hashed_msg,
                &aggregate_pubkey,
            )
            .unwrap();

        Ok(())
    }
}
