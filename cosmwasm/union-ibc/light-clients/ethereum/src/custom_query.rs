use cosmwasm_std::{Binary, Deps, Empty, HashFunction, BLS12_381_G1_GENERATOR};
use ethereum_sync_protocol::BlsVerify;
use unionlabs::bls::{BlsPublicKey, BlsSignature};

pub const DST_POP_G2: &[u8] = b"BLS_POP_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";

pub struct VerificationContext<'a> {
    pub deps: Deps<'a, Empty>,
}

impl BlsVerify for VerificationContext<'_> {
    fn fast_aggregate_verify<'pk>(
        &self,
        public_keys: impl IntoIterator<Item = &'pk BlsPublicKey>,
        msg: Vec<u8>,
        signature: BlsSignature,
    ) -> Result<(), ethereum_sync_protocol::error::Error> {
        let pubkey = self
            .deps
            .api
            .bls12_381_aggregate_g1(
                public_keys
                    .into_iter()
                    .map(|x| x.0)
                    .flatten()
                    .collect::<Vec<u8>>()
                    .as_slice(),
            )
            .unwrap();

        let msg_hashed = self
            .deps
            .api
            .bls12_381_hash_to_g2(HashFunction::Sha256, &msg, DST_POP_G2)
            .unwrap();

        let res = self
            .deps
            .api
            .bls12_381_pairing_equality(&BLS12_381_G1_GENERATOR, &signature.0, &pubkey, &msg_hashed)
            .unwrap();

        if res {
            Ok(())
        } else {
            Err(ethereum_sync_protocol::error::Error::Crypto)
        }
    }
}
