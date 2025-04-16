use cometbft_types::crypto::public_key::PublicKey;
use cosmwasm_std::Deps;
use tendermint_verifier::types::HostFns;

pub struct Ed25519Verifier<'a> {
    deps: Deps<'a>,
}

impl<'a> Ed25519Verifier<'a> {
    pub fn new(deps: Deps<'a>) -> Self {
        Self { deps }
    }
}

#[allow(clippy::manual_unwrap_or)]
impl HostFns for Ed25519Verifier<'_> {
    fn verify_signature(&self, pubkey: &PublicKey, msg: &[u8], sig: &[u8]) -> bool {
        match pubkey {
            PublicKey::Ed25519(ref key) => self
                .deps
                .api
                .ed25519_verify(msg, sig, key)
                .unwrap_or_default(),
            _ => false,
        }
    }

    fn verify_batch_signature(
        &self,
        pubkeys: &[PublicKey],
        msgs: &[&[u8]],
        sigs: &[&[u8]],
    ) -> bool {
        let Ok(pubkeys) = pubkeys
            .iter()
            .map(|pk| match pk {
                PublicKey::Ed25519(pkey) => Ok(pkey.as_ref()),
                _ => Err(()),
            })
            .collect::<Result<Vec<_>, _>>()
        else {
            return false;
        };

        self.deps
            .api
            .ed25519_batch_verify(msgs, sigs, &pubkeys)
            .unwrap_or_default()
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
