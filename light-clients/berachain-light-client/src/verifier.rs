use cosmwasm_std::Deps;
use ethereum_light_client::custom_query::VerificationContext;
use ethereum_verifier::verify::BlsVerify;
use sha2::Digest;
use tendermint_verifier::types::HostFns;
use unionlabs::{
    cosmwasm::wasm::union::custom_query::UnionCustomQuery,
    tendermint::crypto::public_key::PublicKey,
};

pub struct Bls12_381Verifier<'a> {
    inner: VerificationContext<'a>,
}

impl<'a> Bls12_381Verifier<'a> {
    pub fn new(deps: Deps<'a, UnionCustomQuery>) -> Self {
        Self {
            inner: VerificationContext { deps },
        }
    }
}

impl<'a> HostFns for Bls12_381Verifier<'a> {
    fn verify_signature(&self, pubkey: &PublicKey, msg: &[u8], sig: &[u8]) -> bool {
        match pubkey {
            PublicKey::Bls12_381(ref key) => {
                let Ok(key) = key.try_into() else {
                    return false;
                };

                let Ok(sig) = sig.try_into() else {
                    return false;
                };

                let msg = if msg.len() > 32 {
                    sha2::Sha256::new().chain_update(msg).finalize().to_vec()
                } else {
                    msg.to_vec()
                };

                match self.inner.fast_aggregate_verify([&key], msg, sig) {
                    Ok(()) => true,
                    Err(_) => false,
                }
            }
            _ => false,
        }
    }

    fn verify_batch_signature(
        &self,
        pubkeys: &[PublicKey],
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
