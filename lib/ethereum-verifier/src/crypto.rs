use ibc_types::bls::{BlsPublicKey, BlsSignature};

use crate::Error;

pub fn fast_aggregate_verify(
    public_keys: &[&BlsPublicKey],
    msg: &[u8],
    signature: &BlsSignature,
) -> Result<bool, Error> {
    let public_keys = public_keys
        .iter()
        .cloned()
        .map(milagro_bls::PublicKey::try_from)
        .collect::<Result<Vec<milagro_bls::PublicKey>, _>>()?;

    let public_keys: Vec<&milagro_bls::PublicKey> = public_keys.iter().collect();

    let signature: milagro_bls::Signature = signature.try_into()?;

    let aggregate_signature = milagro_bls::AggregateSignature::aggregate(&[&signature]);
    let aggregate_pubkey =
        milagro_bls::AggregatePublicKey::aggregate(&public_keys).map_err(Error::Bls)?;

    Ok(aggregate_signature.fast_aggregate_verify_pre_aggregated(msg, &aggregate_pubkey))
}

pub fn eth_aggregate_public_keys(public_keys: &[BlsPublicKey]) -> Result<BlsPublicKey, Error> {
    let public_keys = public_keys
        .iter()
        .map(milagro_bls::PublicKey::try_from)
        .collect::<Result<Vec<milagro_bls::PublicKey>, _>>()?;

    let public_keys: Vec<&milagro_bls::PublicKey> = public_keys.iter().collect();

    milagro_bls::AggregatePublicKey::aggregate(&public_keys)
        .map(|x| x.into())
        .map_err(Error::Bls)
}
