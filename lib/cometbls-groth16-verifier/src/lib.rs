#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use core::marker::PhantomData;

use ark_ff::vec;
use byteorder::{BigEndian, ByteOrder};
use cometbls_light_client_types::{light_header::LightHeader, ChainId};
use constants::*;
use hex_literal::hex;
use sha3::Digest;
use substrate_bn::G1;
use unionlabs::{hash::H256, uint::U256, ByteArrayExt};

mod constants;

pub const NB_PUBLIC_INPUTS: usize = 2;

pub const HMAC_O: &[u8] = &hex!("1F333139281E100F5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C");
pub const HMAC_I: &[u8] = &hex!("75595B5342747A653636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636");
pub const PRIME_R_MINUS_ONE: U256 = U256::from_limbs([
    4891460686036598784,
    2896914383306846353,
    13281191951274694749,
    3486998266802970665,
]);

const _: () = assert!(GAMMA_ABC_G1.len() == NB_PUBLIC_INPUTS + 1);

fn hmac_keccak(message: &[u8]) -> [u8; 32] {
    sha3::Keccak256::new()
        .chain_update(
            HMAC_O
                .iter()
                .copied()
                .chain(
                    sha3::Keccak256::new()
                        .chain_update(
                            HMAC_I
                                .iter()
                                .copied()
                                .chain(message.iter().copied())
                                .collect::<Vec<_>>(),
                        )
                        .finalize(),
                )
                .collect::<Vec<_>>(),
        )
        .finalize()
        .into()
}

// Union whitepaper: (1) H_{hmac_r}
fn hash_to_field(message: &[u8]) -> U256 {
    (U256::from_be_bytes(hmac_keccak(message)) % PRIME_R_MINUS_ONE) + U256::from(1u32)
}

// Gnark commitment hashing, we employ our custom hash_to_field in the prover itself
fn hash_commitment(proof_commitment: &substrate_bn::AffineG1) -> Result<U256, Error> {
    let mut buffer = [0u8; 64];
    proof_commitment
        .x()
        .to_big_endian(&mut buffer[0..32])
        .map_err(|_| Error::InvalidCommitment)?;
    proof_commitment
        .y()
        .to_big_endian(&mut buffer[32..64])
        .map_err(|_| Error::InvalidCommitment)?;
    Ok(hash_to_field(&buffer))
}

pub const FQ_SIZE: usize = 32;
pub const G1_SIZE: usize = 2 * FQ_SIZE;
pub const G2_SIZE: usize = 2 * G1_SIZE;

pub struct G1Affine<FromOrder: ByteOrder>(PhantomData<FromOrder>, substrate_bn::AffineG1);
pub type G1AffineBE = G1Affine<BigEndian>;

impl TryFrom<[u8; G1_SIZE]> for G1AffineBE {
    type Error = Error;
    fn try_from(value: [u8; G1_SIZE]) -> Result<Self, Self::Error> {
        Ok(G1Affine(
            PhantomData,
            substrate_bn::AffineG1::new(
                substrate_bn::Fq::from_slice(&value.array_slice::<0, FQ_SIZE>())
                    .map_err(|_| Error::InvalidPoint)?,
                substrate_bn::Fq::from_slice(&value.array_slice::<FQ_SIZE, FQ_SIZE>())
                    .map_err(|_| Error::InvalidPoint)?,
            )
            .map_err(|_| Error::InvalidPoint)?,
        ))
    }
}

pub struct G2Affine<FromOrder>(PhantomData<FromOrder>, substrate_bn::AffineG2);
pub type G2AffineBE = G2Affine<BigEndian>;

impl TryFrom<[u8; G2_SIZE]> for G2AffineBE {
    type Error = Error;
    fn try_from(value: [u8; G2_SIZE]) -> Result<Self, Self::Error> {
        Ok(G2Affine(
            PhantomData,
            substrate_bn::AffineG2::new(
                substrate_bn::Fq2::new(
                    substrate_bn::Fq::from_slice(&value.array_slice::<FQ_SIZE, FQ_SIZE>())
                        .map_err(|_| Error::InvalidPoint)?,
                    substrate_bn::Fq::from_slice(&value.array_slice::<0, FQ_SIZE>())
                        .map_err(|_| Error::InvalidPoint)?,
                ),
                substrate_bn::Fq2::new(
                    substrate_bn::Fq::from_slice(
                        &value.array_slice::<{ G1_SIZE + FQ_SIZE }, FQ_SIZE>(),
                    )
                    .map_err(|_| Error::InvalidPoint)?,
                    substrate_bn::Fq::from_slice(&value.array_slice::<G1_SIZE, FQ_SIZE>())
                        .map_err(|_| Error::InvalidPoint)?,
                ),
            )
            .map_err(|_| Error::InvalidPoint)?,
        ))
    }
}

pub struct Proof {
    /// The `A` element in `G1`.
    pub a: substrate_bn::AffineG1,
    /// The `B` element in `G2`.
    pub b: substrate_bn::AffineG2,
    /// The `C` element in `G1`.
    pub c: substrate_bn::AffineG1,
}

pub struct ZKP<FromOrder> {
    pub proof: Proof,
    pub proof_commitment: substrate_bn::AffineG1,
    pub proof_commitment_pok: substrate_bn::AffineG1,
    pub _marker: PhantomData<FromOrder>,
}

// G1 + G2 + G1 + G1 + G1
pub const EXPECTED_PROOF_SIZE: usize = G1_SIZE + G2_SIZE + G1_SIZE + G1_SIZE + G1_SIZE;

// [a ... b ... c ... proof_commitment ... commitment_pok]
pub type RawZKP = [u8; EXPECTED_PROOF_SIZE];

impl<FromOrder: ByteOrder> TryFrom<&[u8]> for ZKP<FromOrder>
where
    G1Affine<FromOrder>: TryFrom<[u8; G1_SIZE], Error = Error>,
    G2Affine<FromOrder>: TryFrom<[u8; G2_SIZE], Error = Error>,
{
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let value = RawZKP::try_from(value).map_err(|_| Error::InvalidRawProof)?;
        let G1Affine(_, a) = G1Affine::<FromOrder>::try_from(value.array_slice::<0, G1_SIZE>())?;
        let G2Affine(_, b) =
            G2Affine::<FromOrder>::try_from(value.array_slice::<G1_SIZE, G2_SIZE>())?;
        let G1Affine(_, c) =
            G1Affine::<FromOrder>::try_from(value.array_slice::<{ G1_SIZE + G2_SIZE }, G1_SIZE>())?;
        let G1Affine(_, proof_commitment) = G1Affine::<FromOrder>::try_from(
            value.array_slice::<{ G1_SIZE + G2_SIZE + G1_SIZE }, G1_SIZE>(),
        )?;
        let G1Affine(_, proof_commitment_pok) = G1Affine::<FromOrder>::try_from(
            value.array_slice::<{ G1_SIZE + G2_SIZE + G1_SIZE + G1_SIZE }, G1_SIZE>(),
        )?;
        Ok(Self {
            proof: Proof { a, b, c },
            proof_commitment,
            proof_commitment_pok,
            _marker: PhantomData,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InvalidPublicInput,
    InvalidPoint,
    InvalidProof,
    InvalidPok,
    InvalidVerifyingKey,
    InvalidCommitment,
    InvalidRawProof,
    InvalidHeight,
    InvalidTimestamp,
    InvalidSliceLength,
}

pub fn verify_zkp(
    chain_id: &ChainId,
    trusted_validators_hash: H256,
    header: &LightHeader,
    zkp: impl Into<Vec<u8>>,
) -> Result<(), Error> {
    verify_generic_zkp_2(
        chain_id,
        trusted_validators_hash,
        header,
        PEDERSEN_G,
        PEDERSEN_G_ROOT_SIGMA_NEG,
        ZKP::try_from(zkp.into().as_ref())?,
    )
}

fn verify_generic_zkp_2(
    chain_id: &ChainId,
    trusted_validators_hash: H256,
    header: &LightHeader,
    g: substrate_bn::G2,
    g_root_sigma_neg: substrate_bn::G2,
    zkp: ZKP<BigEndian>,
) -> Result<(), Error> {
    // Constant + public inputs
    let decode_scalar = move |x: U256| -> Result<substrate_bn::Fr, Error> {
        substrate_bn::Fr::new(x.0 .0.into()).ok_or(Error::InvalidPublicInput)
    };
    let commitment_hash = hash_commitment(&zkp.proof_commitment)?;
    let mut inputs_hash = <[u8; 32]>::from(
        sha2::Sha256::new()
            .chain_update(
                vec![0u8; 32 - chain_id.as_str().len()]
                    .into_iter()
                    .chain(chain_id.as_str().bytes())
                    .collect::<Vec<_>>(),
            )
            .chain_update(
                U256::from(
                    u64::try_from(i64::from(header.height)).map_err(|_| Error::InvalidHeight)?,
                )
                .to_be_bytes(),
            )
            .chain_update(
                U256::from(
                    u64::try_from(i64::from(header.time.seconds))
                        .map_err(|_| Error::InvalidTimestamp)?,
                )
                .to_be_bytes(),
            )
            .chain_update(
                U256::from(
                    u64::try_from(i32::from(header.time.nanos))
                        .map_err(|_| Error::InvalidTimestamp)?,
                )
                .to_be_bytes(),
            )
            .chain_update(header.validators_hash)
            .chain_update(header.next_validators_hash)
            .chain_update(header.app_hash)
            .chain_update(trusted_validators_hash)
            .finalize(),
    );
    // drop the most significant byte to fit in bn254 F_r
    inputs_hash[0] = 0;
    let public_inputs: [substrate_bn::Fr; NB_PUBLIC_INPUTS] = [
        decode_scalar(U256::from_be_bytes(inputs_hash))?,
        decode_scalar(commitment_hash)?,
    ];
    let initial_point = GAMMA_ABC_G1[0] + zkp.proof_commitment.into();
    let public_inputs_msm = public_inputs
        .into_iter()
        .zip(GAMMA_ABC_G1.into_iter().skip(1))
        .fold(initial_point, |s, (w_i, gamma_l_i)| s + gamma_l_i * w_i);

    let proof_a: G1 = zkp.proof.a.into();
    let proof_c: G1 = zkp.proof.c.into();
    let pc: G1 = zkp.proof_commitment.into();
    let pok: G1 = zkp.proof_commitment_pok.into();

    let pok_result = substrate_bn::pairing_batch(&[(pc, g), (pok, g_root_sigma_neg)]);
    if pok_result != substrate_bn::Gt::one() {
        return Err(Error::InvalidPok);
    }

    let g16_result = substrate_bn::pairing_batch(&[
        (proof_a, zkp.proof.b.into()),
        (public_inputs_msm, GAMMA_NEG_G2),
        (proof_c, DELTA_NEG_G2),
        (ALPHA_G1, BETA_NEG_G2),
    ]);
    if g16_result != substrate_bn::Gt::one() {
        Err(Error::InvalidProof)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use unionlabs::google::protobuf::timestamp::Timestamp;

    use super::*;

    #[test]
    fn test_ok() {
        assert_eq!(
            verify_zkp(
                &ChainId::from_string("union-devnet-1337").unwrap(),
                hex!("20DDFE7A0F75C65D876316091ECCD494A54A2BB324C872015F73E528D53CB9C4").into(),
                &LightHeader {
                    height: 3405691582.try_into().unwrap(),
                    time: Timestamp {
                        seconds: 1732205251.try_into().unwrap(),
                        nanos: 998131342.try_into().unwrap()
                    },
                    validators_hash: hex!("20DDFE7A0F75C65D876316091ECCD494A54A2BB324C872015F73E528D53CB9C4").into(),
                    next_validators_hash: hex!("20DDFE7A0F75C65D876316091ECCD494A54A2BB324C872015F73E528D53CB9C4").into(),
                    app_hash: hex!("EE7E3E58F98AC95D63CE93B270981DF3EE54CA367F8D521ED1F444717595CD36").into(),
                },
                hex!("03CF56142A1E03D2445A82100FEAF70C1CD95A731ED85792AFFF5792EC0BDD2108991BB56F9043A269F88903DE616A9AB99A3C5AB778E566744B060456C5616C06BCE7F1930421768C2CBD79F88D08EC3A52D7C9A867064E973064385E9C945E02951190DD7CE1662546733DD540188C96E608CA750FEF36B39E2577833634C70AE6F1A6D00DC6C21446AAF285EF35D944E8782B131300574F9A889C7E708A2325E9A78013BBE869D38B19C602DAF69644C77D177E99ED76398BCEE13C61FDBF2E178A5BA028A36033E54D1D9A0071E82E04079A5305347EBAC6D66F6EBFA48B1DA1BF9DC5A51EFA292E1DC7B85D26F18422EB386C48CA75434039764448BB96268DDC2CF683DDCA4BD83DF21C5631CF784375EEBE77EABC2DE77886BF1D48392C9C52E063B4A7131EAB9ABBA12A9F26888BC37366D41AC7D4BAC0BF6755ACB009BF9F36F380B6D0EEAABF066503A1B6E01DCC965D968D7694E01B1755E6BDD21C7A80B41682748F9B7151714BE34AA79AAD48BBB2A84525F6CDF812658C6E4F")
            ),
            Ok(())
        );
    }

    #[test]
    fn test_decode() {
        ZKP::try_from(hex!("1c9bc15a0c4541aff1d12780d6cf4ae2bdc6e3afafceae9d4fa36209fa323b68002e9c77c223d830e5df6a80cdd683f0986353933ee3179970fccc5d893219d30726f3b8c0dbe630b815b01b5557228a0dfeb0e0435bb0d15d1ccff7f6133fc110937d9fceee2f9052468c198fafeca89d524142a0efa9dc4df445853ce617302059018fef03dc34456ad201d2a5420a7d1c8fac57cb48cbe6709ac4da27d1eb250f73eab007d26cbff41ceb4564ab1cdfa83e9ee88be4f816dc841bbf2e90c80186ad9437fce7655c71b54addae1ccea429da3edba3232d073cb7e89ff2d27218556f1af0c446962ace932f637279dd0ad3ef1501fb6da39d5f68282f54bcf6094999672f3d8cbbf0409aef1048175ffff50b03a5154016d307a2ef425ffee509cd447b22ce6331c7a3473b2c6da1f9d550e8c3ab19bde65e699e07f4f2886c03ec4ff2faa0e342de7ac5daf32025acd6070c19ed8b007c121db0d955472c7d2e38d5a943d15bc902613029e4baa8c26034ff280e3a4d5468fcd6745afe53b5").as_slice()).unwrap();
    }

    #[test]
    fn test_tampered_block() {
        assert_eq!(
            verify_zkp(
                &ChainId::from_string("union-devnet-1337").unwrap(),
                hex!("20DDFE7A0F75C65D876316091ECCD494A54A2BB324C872015F73E528D53CB9C4").into(),
                &LightHeader {
                    height: 3405691582.try_into().unwrap(),
                    time: Timestamp {
                        // tampered seconds
                        seconds: 1732205252.try_into().unwrap(),
                        nanos: 998131342.try_into().unwrap()
                    },
                    validators_hash: hex!("20DDFE7A0F75C65D876316091ECCD494A54A2BB324C872015F73E528D53CB9C4").into(),
                    next_validators_hash: hex!("20DDFE7A0F75C65D876316091ECCD494A54A2BB324C872015F73E528D53CB9C4").into(),
                    app_hash: hex!("EE7E3E58F98AC95D63CE93B270981DF3EE54CA367F8D521ED1F444717595CD36").into(),
                },
                hex!("03CF56142A1E03D2445A82100FEAF70C1CD95A731ED85792AFFF5792EC0BDD2108991BB56F9043A269F88903DE616A9AB99A3C5AB778E566744B060456C5616C06BCE7F1930421768C2CBD79F88D08EC3A52D7C9A867064E973064385E9C945E02951190DD7CE1662546733DD540188C96E608CA750FEF36B39E2577833634C70AE6F1A6D00DC6C21446AAF285EF35D944E8782B131300574F9A889C7E708A2325E9A78013BBE869D38B19C602DAF69644C77D177E99ED76398BCEE13C61FDBF2E178A5BA028A36033E54D1D9A0071E82E04079A5305347EBAC6D66F6EBFA48B1DA1BF9DC5A51EFA292E1DC7B85D26F18422EB386C48CA75434039764448BB96268DDC2CF683DDCA4BD83DF21C5631CF784375EEBE77EABC2DE77886BF1D48392C9C52E063B4A7131EAB9ABBA12A9F26888BC37366D41AC7D4BAC0BF6755ACB009BF9F36F380B6D0EEAABF066503A1B6E01DCC965D968D7694E01B1755E6BDD21C7A80B41682748F9B7151714BE34AA79AAD48BBB2A84525F6CDF812658C6E4F")
            ),
            Err(Error::InvalidProof)
        );
    }
}
