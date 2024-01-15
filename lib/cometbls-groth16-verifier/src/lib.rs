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

// TODO: link whitepaper equation (hmac)
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
            BigInt!(
                "12953796731566255356153533186899970596541789506787316310421265866460204452345"
            ),
            BigInt!(
                "16108099332627179116882890117640860610766253379177782725877946988741043002151"
            ),
        ),
        beta_g2: make_g2(
            BigInt!("7189998629544061358868906102425391182345467937747171889044260956112296857453"),
            BigInt!(
                "11715211044976611849279736941659181461607821837429796658922621107593979258018"
            ),
            BigInt!("268303316233297557783682887628508342685752572403344995053486577590103398535"),
            BigInt!("4144094055252167352279214584912891444882296092131984956478324068851445564919"),
        ),
        gamma_g2: make_g2(
            BigInt!("3203739780556455486614737616951770238449562962366174174415509385502339079134"),
            BigInt!("330365480594874048579972851352786169022705988981774516328112713209916814425"),
            BigInt!("1160827755956593330229975476904769538358170226783719969166560221964705726473"),
            BigInt!(
                "10117748002270903361881103766639804088966132520100608744246025077047886374957"
            ),
        ),
        delta_g2: make_g2(
            BigInt!("144471853326950176158652078814987832244858457888532278798444997831177703256"),
            BigInt!(
                "11723967339734259367269684565753317343894480284660483851808778513760163502167"
            ),
            BigInt!(
                "13230225566375652551257855552370345586627664462416084709155088881134357226734"
            ),
            BigInt!("6917695229563553029365572247639515670362209203071184748187717255117343741604"),
        ),
        gamma_abc_g1: vec![
            make_g1(
                BigInt!(
                    "468243475977942096739227064799809074577932864561864594431724289332044119393"
                ),
                BigInt!(
                    "12026957193107468267989691684356505173830039075560970134183365962992276088502"
                ),
            ),
            make_g1(
                BigInt!(
                    "4273127142915912066836331589937887852131041396580330861495976561450995509060"
                ),
                BigInt!(
                    "20311891790436735379947440583419330671207702790700221333652972975201502172109"
                ),
            ),
            make_g1(
                BigInt!(
                    "5867078984367927991529260476370712193826388223706691841033290533650191497842"
                ),
                BigInt!(
                    "15457584854730416542120021991798916984793483604514831168874602434669080770632"
                ),
            ),
            make_g1(
                BigInt!(
                    "6073935183581261599921354767516829294802045150352674700000707907321520444286"
                ),
                BigInt!(
                    "19421513883482432722033354055257568460031664693915650865773106969145220560478"
                ),
            ),
            make_g1(
                BigInt!(
                    "6573761322005933095907247349767854226263237757268335098982485126002570113042"
                ),
                BigInt!(
                    "21648292561695958729986475933727235437209737383625151779025875934553286731278"
                ),
            ),
            make_g1(
                BigInt!(
                    "7850217296098862761033756178241744898548923761706289522462295413515747119164"
                ),
                BigInt!(
                    "15481433110471107159567305060748336299937224568483713663114311452391215471632"
                ),
            ),
        ],
    }
}

// TODO: link whitepaper equation
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

// TODO: link whitepaper equation
fn hash_to_field2(message: impl Iterator<Item = u8> + Clone) -> (U256, U256) {
    (
        hash_to_field(
            [0].into_iter()
                .chain(message.clone())
                .collect::<Vec<_>>()
                .as_ref(),
        ),
        hash_to_field([1].into_iter().chain(message).collect::<Vec<_>>().as_ref()),
    )
}

// TODO: link whitepaper equation
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

    let (message_x, message_y) = hash_to_field2(message.iter().copied());

    let public_inputs: [P::ScalarField; 5] = [
        decode_scalar(trusted_validators_hash),
        decode_scalar(untrusted_validators_hash),
        decode_scalar(message_x),
        decode_scalar(message_y),
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
    fn test() {
        assert_eq!(
            verify_zkp(
                U256::from_str("09539669AC74E3530F2782443258734DABAC8B8E036ECB9BC378361016892156").unwrap(),
                U256::from_str("09539669AC74E3530F2782443258734DABAC8B8E036ECB9BC378361016892156").unwrap(),
                &hex!("650802113E0200000000000022480A20B8C88CE47A2AC003E6736975F3AE14517A07E90CBF3293C8E073CF45FB371E79122408011220AC08E7599D7F5682B77E2293928D68A956B2A73E917E5B9F0DBA64B0ED8C2E2F320E756E696F6E2D6465766E65742D31"),
                hex!("21D80AACFCA03DC2B84881E3EF1A73C25D2D088E48AA35764A6B4485A78354F021C90A4CBAAB731658D13CE5152F147DF1734F0196031DAF918BF06DAEA1A4E9082959B87795E28482B4FE13AD4B777F9A2D4BFBC8C3FF2640A5DB5619A8F2DA04D6037DAEA584F0C93EDC769859BE695493F48813E491540C37587C2C3214490AE2C9DC087D8039CAF2BD181E289D60EA9AC8B4BF3411A9F9888DC9250525DD055143FE81924CF683CF8381167431A8CB0C984C9DB2BA13D6C9B2374FFD7323052586453C7C06E234B861E9E212EB4A8DF470BD9ADCDB759FED40E62004ECB8210E3A53A0D1F570552C5118521943BC2CC4BB1DA8A5877667A2800D4DF62665304E914F6631B3CE27C88F21E1E8FFAC6C0512D62AE00BEEA79F649BD6E139BD254011571644878C8A72D167D82B5F409360209E1B8E146457C1893769383F4F2F9C0E2EF22885F92672277AF244840CA6EB5298D74E73334BD88360D6B33681")
            ),
            Ok(())
        );
    }
}
