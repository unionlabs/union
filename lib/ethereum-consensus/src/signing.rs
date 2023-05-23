use crate::crypto::{verify_signature, SecretKey};
use crate::primitives::{BlsPublicKey, BlsSignature, Domain, Root};
use crate::state_transition::Error;
use ssz_rs::prelude::*;

#[derive(Default, Debug, SimpleSerialize)]
pub struct SigningData {
    pub object_root: Root,
    pub domain: Domain,
}

pub fn compute_signing_root<T: SimpleSerialize>(
    ssz_object: &mut T,
    domain: Domain,
) -> Result<Root, Error> {
    let object_root = ssz_object.hash_tree_root()?;

    let mut s = SigningData {
        object_root,
        domain,
    };
    s.hash_tree_root().map_err(Error::Merkleization)
}

pub fn sign_with_domain<T: SimpleSerialize>(
    data: &mut T,
    signing_key: &SecretKey,
    domain: Domain,
) -> Result<BlsSignature, Error> {
    let signing_root = compute_signing_root(data, domain)?;
    Ok(signing_key.sign(signing_root.as_ref()))
}

pub fn verify_signed_data<T: SimpleSerialize>(
    data: &mut T,
    signature: &BlsSignature,
    public_key: &BlsPublicKey,
    domain: Domain,
) -> Result<(), Error> {
    let signing_root = compute_signing_root(data, domain)?;
    verify_signature(public_key, signing_root.as_ref(), signature).map_err(Into::into)
}
