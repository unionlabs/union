use unionlabs::hash::{H384, H768};

use crate::error::Error;

pub fn fast_aggregate_verify(
    public_keys: &[&H384],
    msg: &[u8],
    signature: &H768,
) -> Result<bool, Error> {
    __fast_aggregate_verify(public_keys, msg, signature, |pk| {
        milagro_bls::PublicKey::from_bytes(pk.as_ref())
    })
}

pub fn eth_aggregate_public_keys(public_keys: &[H384]) -> Result<H384, Error> {
    __eth_aggregate_public_keys(public_keys, |pk| {
        milagro_bls::PublicKey::from_bytes(pk.as_ref())
    })
}

pub fn fast_aggregate_verify_unchecked(
    public_keys: &[&H384],
    msg: &[u8],
    signature: &H768,
) -> Result<bool, Error> {
    __fast_aggregate_verify(public_keys, msg, signature, |pk| {
        milagro_bls::PublicKey::from_bytes_unchecked(pk.as_ref())
    })
}

pub fn eth_aggregate_public_keys_unchecked(public_keys: &[H384]) -> Result<H384, Error> {
    __eth_aggregate_public_keys(public_keys, |pk| {
        milagro_bls::PublicKey::from_bytes_unchecked(pk.as_ref())
    })
}

fn __fast_aggregate_verify(
    public_keys: &[&H384],
    msg: &[u8],
    signature: &H768,
    map_fn: fn(&H384) -> Result<milagro_bls::PublicKey, milagro_bls::AmclError>,
) -> Result<bool, Error> {
    let public_keys = public_keys
        .iter()
        .cloned()
        .map(map_fn)
        .collect::<Result<Vec<milagro_bls::PublicKey>, _>>()?;

    let public_keys: Vec<&milagro_bls::PublicKey> = public_keys.iter().collect();

    let signature = milagro_bls::Signature::from_bytes(signature.as_ref())?;

    let aggregate_signature = milagro_bls::AggregateSignature::aggregate(&[&signature]);
    let aggregate_pubkey =
        milagro_bls::AggregatePublicKey::aggregate(&public_keys).map_err(Error::Bls)?;

    Ok(aggregate_signature.fast_aggregate_verify_pre_aggregated(msg, &aggregate_pubkey))
}

fn __eth_aggregate_public_keys(
    public_keys: &[H384],
    map_fn: fn(&H384) -> Result<milagro_bls::PublicKey, milagro_bls::AmclError>,
) -> Result<H384, Error> {
    let public_keys = public_keys
        .iter()
        .map(map_fn)
        .collect::<Result<Vec<milagro_bls::PublicKey>, _>>()?;

    let public_keys: Vec<&milagro_bls::PublicKey> = public_keys.iter().collect();

    milagro_bls::AggregatePublicKey::aggregate(&public_keys)
        .map(|x| milagro_bls::PublicKey { point: x.point }.as_bytes().into())
        .map_err(Error::Bls)
}
