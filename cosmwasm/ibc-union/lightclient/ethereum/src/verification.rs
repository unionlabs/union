use cosmwasm_std::{Deps, Empty, HashFunction, BLS12_381_G1_GENERATOR};
use ethereum_sync_protocol::{BlsVerify, DST_POP_G2};
use unionlabs::primitives::{H384, H768};

pub struct VerificationContext<'a> {
    pub deps: Deps<'a, Empty>,
}

impl BlsVerify for VerificationContext<'_> {
    fn fast_aggregate_verify<'pk>(
        &self,
        public_keys: impl IntoIterator<Item = &'pk H384>,
        msg: Vec<u8>,
        signature: H768,
    ) -> Result<(), ethereum_sync_protocol::error::Error> {
        let pubkeys = public_keys
            .into_iter()
            .flat_map(|x| *x)
            .collect::<Vec<u8>>();

        let pubkey = self
            .deps
            .api
            .bls12_381_aggregate_g1(&pubkeys)
            .map_err(|e| {
                ethereum_sync_protocol::error::Error::ClientSignatureVerification(e.to_string())
            })?;

        let hashed_msg = self
            .deps
            .api
            .bls12_381_hash_to_g2(HashFunction::Sha256, &msg, DST_POP_G2)
            .map_err(|e| {
                ethereum_sync_protocol::error::Error::ClientSignatureVerification(e.to_string())
            })?;

        let valid = self
            .deps
            .api
            .bls12_381_pairing_equality(
                &BLS12_381_G1_GENERATOR,
                signature.as_ref(),
                &pubkey,
                &hashed_msg,
            )
            .map_err(|e| {
                ethereum_sync_protocol::error::Error::ClientSignatureVerification(e.to_string())
            })?;

        if valid {
            Ok(())
        } else {
            Err(ethereum_sync_protocol::error::Error::Crypto)
        }
    }
}
