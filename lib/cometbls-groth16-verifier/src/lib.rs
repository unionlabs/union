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
                "19681918582342826141927615585844819827950494091197079841581098590160509489088"
            ),
            BigInt!(
                "18976290249472753264792873488771466812990993964894861063003977176791880491271"
            ),
        ),
        beta_g2: make_g2(
            BigInt!(
                "17542740552152507448113209307107151415915067720344615090625491194497459342657"
            ),
            BigInt!("7391419840357209888406550113304609596117324320456425532340186750677647200951"),
            BigInt!("2096098177649336914352130549614133988628743230280002744893263955761858843171"),
            BigInt!("4677079465946489457182021046711835395185270130772240179395185648392131208623"),
        ),
        gamma_g2: make_g2(
            BigInt!(
                "18606218405301761142065379515313210013062685838824185304765852768028043703753"
            ),
            BigInt!(
                "14540190418613230568675456016157166803361906410442369269514923787931816842661"
            ),
            BigInt!(
                "12936954089909944910505634392074782488818125866327899713345517976848668940364"
            ),
            BigInt!(
                "13075501156799383604449750948266619544543946431193320977624603555722073460251"
            ),
        ),
        delta_g2: make_g2(
            BigInt!("4060446808760699692477462845230990229944734548192291022910719993807902355759"),
            BigInt!(
                "17803970575871171031178686612122420011629668206026599803865929512658387807614"
            ),
            BigInt!("4763598941158436116656275326473539450011503349766865816034780567774742318513"),
            BigInt!(
                "14258428007760852895551143871140524567103085048652562171285997361131304666100"
            ),
        ),
        gamma_abc_g1: vec![
            make_g1(
                BigInt!(
                    "18676861125246766292059080199576268981667767278300819763274799276376054409743"
                ),
                BigInt!(
                    "5269797328666185490526867808814966151140271775451395274640052553630677159076"
                ),
            ),
            make_g1(
                BigInt!(
                    "3010349418202885908760025883515590778403141726894708222433169071368055690912"
                ),
                BigInt!(
                    "20724571387755619214201948546999886629454427058875835531981815961969686023639"
                ),
            ),
            make_g1(
                BigInt!(
                    "1718980496599153571806495443921791801530740535933073284474040850386158191735"
                ),
                BigInt!(
                    "3288376032837046783397899352143814445169932711782482341330476711768756263890"
                ),
            ),
            make_g1(
                BigInt!(
                    "9266521894078168597926726825960443668976816125222306871429246198851182099011"
                ),
                BigInt!(
                    "9416966066664703605394453818829209487654794520205974695819389893969431707374"
                ),
            ),
            make_g1(
                BigInt!(
                    "13194582768609510874189454527180276310818912484460263820189470814556014162264"
                ),
                BigInt!(
                    "15983647339013447433771242507224193645257463334651420839328305715367829062538"
                ),
            ),
            make_g1(
                BigInt!(
                    "13160686484300787492313686811371534896624215839999346591796239441200125629208"
                ),
                BigInt!(
                    "11709584278193617231017776985640196897412209200566866495381859539145549732339"
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
                U256::from_str("0x0472116C575F0FECF44ED4F91C34E9E7B67CE8C911FC67F304C2B804330B61F0").unwrap(),
                U256::from_str("0x0472116C575F0FECF44ED4F91C34E9E7B67CE8C911FC67F304C2B804330B61F0").unwrap(),
                &hex!("650802113E0200000000000022480A20DFAD1A5E2BB2B94BD7ED5F4F85199E0DDD95FB4687CFBF19B36865845BD16E20122408011220E32B1FA520CE4F9D0C1A2C80D51FB1F09B9C241101BE70D5CE0DC0F11B009863320E756E696F6E2D6465766E65742D31"),
                hex!("19EB187C4EA70DA41CE1C2E9C81D7F4ACC9B8CD85A98AD59A64BD9D820A1D4E41DA150358EC5AFCBA7407D66AFABD59B1B92095D6A797572FAE6FE9C9A500E7C284C0AF8310181C072C1002961BE007DFC68DFDBD9CCF930410153829DA0E00D20ECDCE1DF5674D69A70BFB0891CE61888AADC6B362E27C6E4C9D1905ED327402EEE2EB635235068CBCBAB909D9A14A8992E74DE17D69269FC60CC448A832F8E1BCA6C3426A79029508AEE21DBB718F016FEC33A0A500EA14A05E3E19548634F04011075515FF876980559B120DC247A190BF699C2BA19ECDA92220AADAF3F5A1D502B8552BCC4BBDBA3C53EA39FAA5D6A7D23E06AE8F52AEECCED971D7D353613F7C750995DDD6BCCBB3E3185CF5B53EADAA6FA412665A0910FF709E49CFDFB0530B479EE5F0774588EE7397F7E050A9F4C87DB7989E2557E76353BF98CF7B922EB5DBE6D66AEA64C33AF7D6469BF3EFD5ACA10936CED35DAE692D489AAF8C2")
            ),
            Ok(())
        );
    }
}
