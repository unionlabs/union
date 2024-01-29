#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use core::{marker::PhantomData, ops::AddAssign};

use ark_ec::{pairing::Pairing, AffineRepr};
use ark_ff::{vec, vec::Vec, BigInt, PrimeField, QuadExtField};
use ark_groth16::{Groth16, Proof, VerifyingKey};
use ark_relations::r1cs::SynthesisError;
use ark_serialize::CanonicalDeserialize;
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use ethabi::ethereum_types::U256;
use hex_literal::hex;
use sha3::Digest;

pub const HMAC_O: &[u8] = &hex!("1F333139281E100F5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C");
pub const HMAC_I: &[u8] = &hex!("75595B5342747A653636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636");
pub const PRIME_R_MINUS_ONE: U256 = U256([
    4891460686036598784,
    2896914383306846353,
    13281191951274694749,
    3486998266802970665,
]);

fn make_g1(x: BigInt<4>, y: BigInt<4>) -> ark_bn254::G1Affine {
    ark_bn254::G1Affine::new(x.into(), y.into())
}

fn make_g2(x0: BigInt<4>, x1: BigInt<4>, y0: BigInt<4>, y1: BigInt<4>) -> ark_bn254::G2Affine {
    ark_bn254::G2Affine::new(
        QuadExtField {
            c0: x0.into(),
            c1: x1.into(),
        },
        QuadExtField {
            c0: y0.into(),
            c1: y1.into(),
        },
    )
}

pub fn universal_vk() -> VerifyingKey<ark_bn254::Bn254> {
    VerifyingKey::<ark_bn254::Bn254> {
        alpha_g1: make_g1(
            BigInt!("210400609118751867867594962339236416900807823190943555300977451252876367251"),
            BigInt!(
                "15114917314049487003769383074865116286536524761334960573881383182006812098667"
            ),
        ),
        beta_g2: make_g2(
            BigInt!("4025695320685928294502537638656612753817559258692794664435345502849231300067"),
            BigInt!(
                "18548846535245326130909941053625664549629964661821753718960511623672350665970"
            ),
            BigInt!(
                "10846430301459525961294403190013912919646029785598534122197567832498848258725"
            ),
            BigInt!(
                "14801712319754738293485998154637765616009986020917883574390603809142215597666"
            ),
        ),
        gamma_g2: make_g2(
            BigInt!("3871507673786634538856304899308535382710007469487017238115999457851260261753"),
            BigInt!("9263528541268382918393290127320839732843244267184360590957560203659557939807"),
            BigInt!(
                "21428157745127521242166523454127400279835355719334794879916725299573267377673"
            ),
            BigInt!(
                "16228735596538146886766390508590128116164825213870252854013185224046201333438"
            ),
        ),
        delta_g2: make_g2(
            BigInt!(
                "10511954649625640946194391595271440757788300118705902871380699323889590717070"
            ),
            BigInt!("3624887307974581652668788593721111206379446020443390411004753466558440004576"),
            BigInt!("7267520965177252745769584498003256794053985100783827924477069234086228875930"),
            BigInt!(
                "13871202899280183706927891825856880309055978294723749244553765869552018564080"
            ),
        ),
        gamma_abc_g1: vec![
            make_g1(
                BigInt!(
                    "7548860451745230603118483440705740915442110531593946049093378206268978489889"
                ),
                BigInt!(
                    "8642676949380445344876359442033848518648809724845273937857877647136985821580"
                ),
            ),
            make_g1(
                BigInt!(
                    "3761607900566975305271041912404145777243969522402194441451368484814115763984"
                ),
                BigInt!(
                    "5236577135937549420013214983430521575368103283652548157969360454835627955674"
                ),
            ),
            make_g1(
                BigInt!(
                    "6551227099151518169715147953582656472887860242160769774314802712412742643231"
                ),
                BigInt!(
                    "1819741224363792162320411058820080302711126963872887137191002940993034590657"
                ),
            ),
            make_g1(
                BigInt!(
                    "14381003547507154363999629345246039179209989264380273368866915523962951751353"
                ),
                BigInt!(
                    "11723454477524607914925823310578875758974497210070578907830734479226804006505"
                ),
            ),
            make_g1(
                BigInt!(
                    "9428637111213429505715745409259434820588419778432030319817634906362966081466"
                ),
                BigInt!(
                    "20385940124935492750321915374844475607796921194332338238981177318927052319236"
                ),
            ),
        ],
    }
}

fn hmac_keccak(message: &[u8]) -> [u8; 32] {
    let mut hasher = sha3::Keccak256::new();
    hasher.update(
        HMAC_I
            .iter()
            .copied()
            .chain(message.iter().copied())
            .collect::<Vec<_>>(),
    );
    let inner_hash = hasher.finalize();

    let mut hasher = sha3::Keccak256::new();
    hasher.update(HMAC_O.iter().copied().chain(inner_hash).collect::<Vec<_>>());

    hasher.finalize().into()
}

// Union whitepaper: (1) H_{hmac_r}
fn hash_to_field(message: &[u8]) -> U256 {
    (U256::from(&hmac_keccak(message)) % PRIME_R_MINUS_ONE) + 1
}

pub const G1_SIZE: usize = 64;
pub const G2_SIZE: usize = 2 * G1_SIZE;
pub const COMMITMENT_HASH_SIZE: usize = 32;

pub struct G1Affine<FromOrder: ByteOrder, P: Pairing>(PhantomData<FromOrder>, P::G1Affine);
pub type G1AffineBE<P> = G1Affine<BigEndian, P>;
pub type G1AffineLE<P> = G1Affine<LittleEndian, P>;

impl<P: Pairing> TryFrom<[u8; G1_SIZE]> for G1AffineBE<P> {
    type Error = Error;
    fn try_from(mut value: [u8; G1_SIZE]) -> Result<Self, Self::Error> {
        value[0..32].reverse();
        value[32..64].reverse();
        let G1Affine(_, point) = G1Affine::<LittleEndian, P>::try_from(value)?;
        Ok(G1Affine(PhantomData, point))
    }
}

impl<P: Pairing> TryFrom<[u8; G1_SIZE]> for G1AffineLE<P> {
    type Error = Error;
    fn try_from(value: [u8; G1_SIZE]) -> Result<Self, Self::Error> {
        let point = <P::G1Affine as CanonicalDeserialize>::deserialize_uncompressed(value.as_ref())
            .map_err(|_| Error::InvalidPoint)?;
        Ok(G1Affine(PhantomData, point))
    }
}

pub struct G2Affine<FromOrder, P: Pairing>(PhantomData<FromOrder>, P::G2Affine);
pub type G2AffineBE<P> = G2Affine<BigEndian, P>;
pub type G2AffineLE<P> = G2Affine<LittleEndian, P>;

impl<P: Pairing> TryFrom<[u8; G2_SIZE]> for G2AffineBE<P> {
    type Error = Error;
    fn try_from(mut value: [u8; G2_SIZE]) -> Result<Self, Self::Error> {
        value[0..64].reverse();
        value[64..128].reverse();
        let G2Affine(_, point) = G2Affine::<LittleEndian, P>::try_from(value)?;
        Ok(G2Affine(PhantomData, point))
    }
}

impl<P: Pairing> TryFrom<[u8; G2_SIZE]> for G2AffineLE<P> {
    type Error = Error;
    fn try_from(value: [u8; G2_SIZE]) -> Result<Self, Self::Error> {
        let point = <P::G2Affine as CanonicalDeserialize>::deserialize_uncompressed(value.as_ref())
            .map_err(|_| Error::InvalidPoint)?;
        Ok(G2Affine(PhantomData, point))
    }
}

pub struct CommitmentHash<FromOrder>(PhantomData<FromOrder>, U256);
pub type CommitmentHashBE = CommitmentHash<BigEndian>;
pub type CommitmentHashLE = CommitmentHash<LittleEndian>;

impl From<[u8; COMMITMENT_HASH_SIZE]> for CommitmentHashBE {
    fn from(value: [u8; COMMITMENT_HASH_SIZE]) -> Self {
        CommitmentHash(PhantomData, U256::from_big_endian(&value))
    }
}

impl From<[u8; COMMITMENT_HASH_SIZE]> for CommitmentHashLE {
    fn from(value: [u8; COMMITMENT_HASH_SIZE]) -> Self {
        CommitmentHash(PhantomData, U256::from_little_endian(&value))
    }
}

#[derive(Debug)]
pub struct ZKP<FromOrder, P: Pairing> {
    pub proof: Proof<P>,
    pub commitment_hash: U256,
    pub proof_commitment: P::G1Affine,
    pub _marker: PhantomData<FromOrder>,
}

// G1 + G2 + G1 + U256 + G1
pub const EXPECTED_PROOF_SIZE: usize = G1_SIZE + G2_SIZE + G1_SIZE + COMMITMENT_HASH_SIZE + G1_SIZE;

// [a ... b ... c ... commitment_hash ... proof_commitment]
pub type RawZKP = [u8; EXPECTED_PROOF_SIZE];

impl<FromOrder: ByteOrder, P: Pairing> TryFrom<&[u8]> for ZKP<FromOrder, P>
where
    G1Affine<FromOrder, P>: TryFrom<[u8; G1_SIZE], Error = Error>,
    G2Affine<FromOrder, P>: TryFrom<[u8; G2_SIZE], Error = Error>,
    CommitmentHash<FromOrder>: From<[u8; COMMITMENT_HASH_SIZE]>,
{
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let value = RawZKP::try_from(value).map_err(|_| Error::InvalidRawProof)?;
        // sadly, no const sub-slicing...
        let G1Affine(_, a) =
            G1Affine::<FromOrder, P>::try_from(value[0..G1_SIZE].try_into().expect("impossible"))?;
        let G2Affine(_, b) = G2Affine::<FromOrder, P>::try_from(
            value[G1_SIZE..G1_SIZE + G2_SIZE]
                .try_into()
                .expect("impossible"),
        )?;
        let G1Affine(_, c) = G1Affine::<FromOrder, P>::try_from(
            value[G1_SIZE + G2_SIZE..G1_SIZE + G2_SIZE + G1_SIZE]
                .try_into()
                .expect("impossible"),
        )?;
        let CommitmentHash(_, commitment_hash) = CommitmentHash::<FromOrder>::from(
            value[G1_SIZE + G2_SIZE + G1_SIZE..G1_SIZE + G2_SIZE + G1_SIZE + COMMITMENT_HASH_SIZE]
                .try_into()
                .expect("impossible"),
        );
        let G1Affine(_, proof_commitment) = G1Affine::<FromOrder, P>::try_from(
            value[G1_SIZE + G2_SIZE + G1_SIZE + COMMITMENT_HASH_SIZE
                ..G1_SIZE + G2_SIZE + G1_SIZE + COMMITMENT_HASH_SIZE + G1_SIZE]
                .try_into()
                .expect("impossible"),
        )?;
        Ok(Self {
            proof: Proof { a, b, c },
            commitment_hash,
            proof_commitment,
            _marker: PhantomData,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    EthAbiDecoding,
    InvalidPoint,
    InvalidProof(SynthesisError),
    InvalidVerifyingKey,
    InvalidRawProof,
}

pub fn verify_zkp(
    trusted_validators_hash: U256,
    untrusted_validators_hash: U256,
    message: &[u8],
    zkp: impl Into<Vec<u8>>,
) -> Result<(), Error> {
    verify_generic_zkp::<ark_bn254::Bn254>(
        universal_vk(),
        trusted_validators_hash,
        untrusted_validators_hash,
        message,
        ZKP::try_from(zkp.into().as_ref())?,
    )
}

fn verify_generic_zkp<P: Pairing>(
    vk: VerifyingKey<P>,
    trusted_validators_hash: U256,
    untrusted_validators_hash: U256,
    message: &[u8],
    zkp: ZKP<BigEndian, P>,
) -> Result<(), Error> {
    let mut buffer = [0u8; 32];
    let mut decode_scalar = move |x: U256| {
        x.to_little_endian(&mut buffer);
        // NOTE: This would silently fail if the input do not fit the scalar
        // field, which is unlikely to happen unless the parameters have been
        // tampered. The pairing check would obviously fail in this case.
        <P::ScalarField as PrimeField>::from_le_bytes_mod_order(&buffer)
    };

    let hashed_message = hash_to_field(message);

    let public_inputs: [P::ScalarField; 4] = [
        decode_scalar(trusted_validators_hash),
        decode_scalar(untrusted_validators_hash),
        decode_scalar(hashed_message),
        decode_scalar(zkp.commitment_hash),
    ];

    let mut prepared_inputs = vk
        .gamma_abc_g1
        .first()
        .ok_or(Error::InvalidVerifyingKey)?
        .into_group();
    // Gnark specific, we need to aggregate the proof commitment
    // See https://github.com/ConsenSys/gnark/issues/652
    prepared_inputs.add_assign(zkp.proof_commitment);
    for (i, b) in public_inputs
        .into_iter()
        .zip(vk.gamma_abc_g1.iter().skip(1))
    {
        prepared_inputs.add_assign(&b.mul_bigint(i.into_bigint()));
    }

    Groth16::<P>::verify_proof_with_prepared_inputs(&vk.into(), &zkp.proof, &prepared_inputs)
        .map_err(Error::InvalidProof)
        .and_then(|v| {
            if v {
                Ok(())
            } else {
                Err(Error::InvalidProof(SynthesisError::UnexpectedIdentity))
            }
        })
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use ethabi::ethereum_types::U256;

    use super::*;

    #[test]
    fn test_ok() {
        assert_eq!(
            verify_zkp(
                U256::from_str("1BBACC23BE35969FFCEFC2892440045E83C3C78E81BF2D6473DD745A93835684").unwrap(),
                U256::from_str("1BBACC23BE35969FFCEFC2892440045E83C3C78E81BF2D6473DD745A93835684").unwrap(),
                &hex!("650802113E0200000000000022480A2053D3DC9F43757EEA63FC3B28C383074A111146B2DE7F73A198D29A6D6919DA6D12240801122023C8BED9455A38334F6462A3EAC87616CF51226F825A229FA23CA420E26730B9320E756E696F6E2D6465766E65742D31"),
                hex!("1BFF5A73FF68B0DAF1F9A29DAD3675762BC346BB36DB701FB5DC80D7FE49C6BE08919D9E16A64159D119278B3BB0EF17DD3703CC3BB6E2FE9E54E84C638EE21829F7FB01533159D70B95F032A99E712D7E06BCCE645D7701615DA5EBAEA75088271F5ECA5E9DAC8D4B1A3CEC0A423F856940D02E5F71B5D9A7C980239987C0D224E1CB5AFE824E7201D959E1B21CBCEE6E0B9BD3B33667741BF7FE201A9A8BA414ACA69E03C3C5DED496B65F02469BC7941AD20E832678BDACF37BD0F7E1A4C50DB32B4A5871AC0584912C09A103FDE335D0A20134C6E2AE52419C84BE775A6C2B4C38DE85412682F66ED405023E9CCEC604B3854B800DFB346EAABF676E844A0EFB9D293D0CC163BAD528AD73B91EDDCDA5FCE49D6ACBA8417D9DC514E1B9FA0C55ADB722120C67CF120CF00225E6B7842A5561772AE1095E30E8FCD7A513370FAD55D57F7794DB096BD7FF7E4373410D04194BE1A1818EFFE5D09F9E08CF8D")
            ),
            Ok(())
        );
    }

    #[test]
    fn test_ko() {
        assert_eq!(
            verify_zkp(
                U256::from_str("1BBACC23BE35969FFCEFC2892440045E83C3C78E81BF2D6473DD745A93835684").unwrap(),
                U256::from_str("1BBACC23BE35969FFCEFC2892440045E83C3C78E81BF2D6473DD745A93835684").unwrap(),
                &hex!("650802113E0200000000000022480A2053D3DC9F43757EEA63FC3B28C383074A111146B2DE7F73A198D29A6D6919DA6D12240801122023C8BED9455A38334F6462A3EAC87616CF51226F825A229FA23CA420E26730B9320E756E696F6E2D6465766E65742D31"),
                hex!("21D80AACFCA03DC2B84881E3EF1A73C25D2D088E48AA35764A6B4485A78354F021C90A4CBAAB731658D13CE5152F147DF1734F0196031DAF918BF06DAEA1A4E9082959B87795E28482B4FE13AD4B777F9A2D4BFBC8C3FF2640A5DB5619A8F2DA04D6037DAEA584F0C93EDC769859BE695493F48813E491540C37587C2C3214490AE2C9DC087D8039CAF2BD181E289D60EA9AC8B4BF3411A9F9888DC9250525DD055143FE81924CF683CF8381167431A8CB0C984C9DB2BA13D6C9B2374FFD7323052586453C7C06E234B861E9E212EB4A8DF470BD9ADCDB759FED40E62004ECB8210E3A53A0D1F570552C5118521943BC2CC4BB1DA8A5877667A2800D4DF62665304E914F6631B3CE27C88F21E1E8FFAC6C0512D62AE00BEEA79F649BD6E139BD254011571644878C8A72D167D82B5F409360209E1B8E146457C1893769383F4F2F9C0E2EF22885F92672277AF244840CA6EB5298D74E73334BD88360D6B33681")
            ),
            Err(Error::InvalidProof(SynthesisError::UnexpectedIdentity))
        );
    }
}
