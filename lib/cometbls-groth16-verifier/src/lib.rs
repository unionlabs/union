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
            BigInt!("843318045813904051851782814226569312224716668519879443037991679093301257400"),
            BigInt!(
                "15416107080929723745166446798814682078427563000162261911323849488726431649665"
            ),
        ),
        beta_g2: make_g2(
            BigInt!(
                "12045932065395624483191137563968354138074439042345149261057471855853342274450"
            ),
            BigInt!(
                "12933749417029078157681900380950146806593248644656848159050519236558381927908"
            ),
            BigInt!(
                "12805391782821906459838976144615356395473755968328066550080865894857140667287"
            ),
            BigInt!(
                "14584513895685150942959102461876800210772112671021072073441290868967091648340"
            ),
        ),
        gamma_g2: make_g2(
            BigInt!("8834401517279732426430709303690144409212027987281176083510451411579725021792"),
            BigInt!("5046190406338174773452605841724188049492775755571037890518087582412737236350"),
            BigInt!(
                "11026156038608598948216491715974624610876929196483098124997221084075368692359"
            ),
            BigInt!("4464838308205636397249900137912138755175447112293553356760701796674912600456"),
        ),
        delta_g2: make_g2(
            BigInt!("6081458280594160167006539403251703157253227316242605315149917857620252166561"),
            BigInt!(
                "19972418504378784918799069027339170180958910678041257330604520110151384463379"
            ),
            BigInt!("2810121513128221299816874771631171735906165699020735693821679570964133725116"),
            BigInt!("507659798917269612242723082454748958422267533387353773424850690004991936681"),
        ),
        gamma_abc_g1: vec![
            make_g1(
                BigInt!(
                    "11480604022502486320552383084244624890381895090315703237207761641596379587022"
                ),
                BigInt!(
                    "18194717846527842697597101854227780426448444481406927484453657628859660838012"
                ),
            ),
            make_g1(
                BigInt!(
                    "17151101065438037966418446912782326714022524609709696117185269486796039712846"
                ),
                BigInt!(
                    "20153978099716141223629974435089176387221033706146080785832322758462404157605"
                ),
            ),
            make_g1(
                BigInt!(
                    "4678289054354856791632961819188703697251080339511868428756468986114497080410"
                ),
                BigInt!(
                    "14896883393560813725625670193715860991949118639991676227159535433685188744128"
                ),
            ),
            make_g1(
                BigInt!(
                    "9861092238854536479945771789292920457542225544584991490958628460109410999807"
                ),
                BigInt!(
                    "11463793619454015701756134623742521947898582646550901437275441814334362034435"
                ),
            ),
            make_g1(
                BigInt!(
                    "11893178039948407756040467155600718551687885088832926431141932892836179895466"
                ),
                BigInt!(
                    "9336098954712618302413022925991187147043963777349552077325468595033619593827"
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
                U256::from_str("2A55A1417E3E50AD714B5E1C85A9432CCC3B1A477C524E64D25D4173EDEE1C05").unwrap(),
                U256::from_str("2A55A1417E3E50AD714B5E1C85A9432CCC3B1A477C524E64D25D4173EDEE1C05").unwrap(),
                &hex!("650802113E0200000000000022480A205245B762DA04AEE98BB41B8CA4F55A4E6C795931317BFCE196A643C8708A736112240801122012384DA4CDB8E223D9E581590B39D4E8B8B3D7702F159578E2E4E55703DCE783320E756E696F6E2D6465766E65742D31"),
                hex!("1F8483CFE28CC94F5B90CBD73A3BD3592F07338F3BA834D7FB739B1A0B313A6913FA014685598BCE33F05E196F6865786DEAE1F4F3DA7511E6E2911FDEC40342260D687282CCBBFF470C96866316924A507E43AA442AB4C5B5AFB8F83C87FBFC09295422479D6D4D96037BD6758A08F7ED0E9639EC37CD50CDAC6431EA4D74D602DDBEACC10A3ABCB4038F9C46168A8C1B354C3B378DFF7FF0E6DC91CF714A8E130D25238407C17D20E89B488E9EA14649E626D04B08DF0722AEEF6C1EBFD69B19722A0D6D52C4CB4182459939EABA60C5B539BBBF97F5A17DF0D43DBD16F6EB303C91E2291AEEF9C5EF19277D8010DEFBC2583D61C8141B36F3B57170BC489C0369E2274B22FCE30D28DE47CA4EBC334D04AE7AD29D5BBEFE19C317995FE3D2217518822C57A50257DBEC2D9F87AA10FF8CF97779A3BB90C088F51D29FB2314279FB97CEDFC91D68BEB55F3F975AA8DB9FBEFCAE24EDD022B74BDEE66E83498")
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
