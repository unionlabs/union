use cosmwasm_std::Deps;
use tendermint_verifier::types::HostFns;
use unionlabs::tendermint::crypto::public_key::PublicKey;

pub struct Ed25519Verifier<'a> {
    deps: Deps<'a>,
}

impl<'a> Ed25519Verifier<'a> {
    pub fn new(deps: Deps<'a>) -> Self {
        Self { deps }
    }
}

impl<'a> HostFns for Ed25519Verifier<'a> {
    fn verify_signature(&self, pubkey: &PublicKey, msg: &[u8], sig: &[u8]) -> bool {
        match pubkey {
            PublicKey::Ed25519(ref key) => match self.deps.api.ed25519_verify(msg, sig, &key) {
                Ok(res) => res,
                Err(_) => false,
            },
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
                PublicKey::Ed25519(pkey) => Ok(pkey.as_slice()),
                _ => Err(()),
            })
            .collect::<Result<Vec<_>, _>>()
        else {
            return false;
        };

        match self.deps.api.ed25519_batch_verify(msgs, sigs, &pubkeys) {
            Ok(res) => res,
            Err(_) => false,
        }
    }
}
