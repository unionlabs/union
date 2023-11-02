use unionlabs::bls::{BlsPublicKey, BlsSignature};

use crate::Error;

pub fn fast_aggregate_verify(
    public_keys: &[&BlsPublicKey],
    msg: &[u8],
    signature: &BlsSignature,
) -> Result<bool, Error> {
    __fast_aggregate_verify(public_keys, msg, signature, |pk| {
        milagro_bls::PublicKey::from_bytes(pk.as_ref())
    })
}

pub fn eth_aggregate_public_keys(public_keys: &[BlsPublicKey]) -> Result<BlsPublicKey, Error> {
    __eth_aggregate_public_keys(public_keys, |pk| {
        milagro_bls::PublicKey::from_bytes(pk.as_ref())
    })
}

pub fn fast_aggregate_verify_unchecked(
    public_keys: &[&BlsPublicKey],
    msg: &[u8],
    signature: &BlsSignature,
) -> Result<bool, Error> {
    __fast_aggregate_verify(public_keys, msg, signature, |pk| {
        milagro_bls::PublicKey::from_bytes_unchecked(pk.as_ref())
    })
}

pub fn eth_aggregate_public_keys_unchecked(
    public_keys: &[BlsPublicKey],
) -> Result<BlsPublicKey, Error> {
    __eth_aggregate_public_keys(public_keys, |pk| {
        milagro_bls::PublicKey::from_bytes_unchecked(pk.as_ref())
    })
}

fn __fast_aggregate_verify(
    public_keys: &[&BlsPublicKey],
    msg: &[u8],
    signature: &BlsSignature,
    map_fn: fn(&BlsPublicKey) -> Result<milagro_bls::PublicKey, milagro_bls::AmclError>,
) -> Result<bool, Error> {
    let public_keys = public_keys
        .iter()
        .cloned()
        .map(map_fn)
        .collect::<Result<Vec<milagro_bls::PublicKey>, _>>()?;

    let public_keys: Vec<&milagro_bls::PublicKey> = public_keys.iter().collect();

    let signature: milagro_bls::Signature = signature.try_into()?;

    let aggregate_signature = milagro_bls::AggregateSignature::aggregate(&[&signature]);
    let aggregate_pubkey =
        milagro_bls::AggregatePublicKey::aggregate(&public_keys).map_err(Error::Bls)?;

    Ok(aggregate_signature.fast_aggregate_verify_pre_aggregated(msg, &aggregate_pubkey))
}

fn __eth_aggregate_public_keys(
    public_keys: &[BlsPublicKey],
    map_fn: fn(&BlsPublicKey) -> Result<milagro_bls::PublicKey, milagro_bls::AmclError>,
) -> Result<BlsPublicKey, Error> {
    let public_keys = public_keys
        .iter()
        .map(map_fn)
        .collect::<Result<Vec<milagro_bls::PublicKey>, _>>()?;

    let public_keys: Vec<&milagro_bls::PublicKey> = public_keys.iter().collect();

    milagro_bls::AggregatePublicKey::aggregate(&public_keys)
        .map(|x| x.into())
        .map_err(Error::Bls)
}
