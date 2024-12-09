use cometbft_types::crypto::public_key::PublicKey;
use cosmwasm_std::{Deps, Empty, BLS12_381_G1_GENERATOR};
use sha2::Digest;
use tendermint_verifier::types::HostFns;

pub struct Bls12Verifier<'a> {
    pub deps: Deps<'a, Empty>,
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
