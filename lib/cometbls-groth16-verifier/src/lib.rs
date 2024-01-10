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
            BigInt!("4545497642472568342690310908405747139240036052727362141781732401147427278332"),
            BigInt!(
                "21657279939237288836606942458710668831714385968774740438494149333436465843139"
            ),
        ),
        beta_g2: make_g2(
            BigInt!(
                "13700154589878825236434612482502516805936642904411498786936854624077294311682"
            ),
            BigInt!("7497643146587701237207141457042187540104153076302400103164161194096334760677"),
            BigInt!("9826602037751518929179716999401231701426363230642489106652396292955775994691"),
            BigInt!("494959300439117228384867537570789342664813284882712364066022590927983142487"),
        ),
        gamma_g2: make_g2(
            BigInt!(
                "20107534645331006032402749367045367765170696291609897560802407293332329737698"
            ),
            BigInt!("6135886662735635672007238208825068442340242201492563368708252608220727995665"),
            BigInt!("584217449480441780710130852604895480474315394916633289664060053699306106397"),
            BigInt!(
                "17134974117749564453678476337428610454531306634132123613923694771472873051567"
            ),
        ),
        delta_g2: make_g2(
            BigInt!("7466991077765871589299219136524534381311757366195842209075383099119159267653"),
            BigInt!("3993057849766236546786517975621342624904647686274232418256214891442175004595"),
            BigInt!(
                "17059631376675436953753993725010634849620319904450639403903900154330555520271"
            ),
            BigInt!(
                "13975627069505281795607371372114671724714107626672690650658467595074779383085"
            ),
        ),
        gamma_abc_g1: vec![
            make_g1(
                BigInt!(
                    "12730996440491230341898748596203954698739716661771354666644872057102948394726"
                ),
                BigInt!(
                    "18188119481706424113895919492547204030227563509791341513627568384483237465563"
                ),
            ),
            make_g1(
                BigInt!(
                    "8627654005047498327557788753897980447566216968617518507065934795873759856303"
                ),
                BigInt!(
                    "7258461021217822820323520100501249447378191264854934186351306877513723742793"
                ),
            ),
            make_g1(
                BigInt!(
                    "10867392565326439682947570558412590838055450106691458097719409041212951853401"
                ),
                BigInt!(
                    "3124325152732842906431467328196929469314595151752342394843391644384931489602"
                ),
            ),
            make_g1(
                BigInt!(
                    "6627862564104432829412837659942319893523740327889349003623985834967392523238"
                ),
                BigInt!(
                    "11980409132042083280769458186828234442115366931894286356450034429211995205398"
                ),
            ),
            make_g1(
                BigInt!(
                    "8352580944529539453233628007042528490297057973561012318225452772905637057834"
                ),
                BigInt!(
                    "16521805616951802411915576898364661283847250025318378340431083135006258712933"
                ),
            ),
            make_g1(
                BigInt!(
                    "12071952363228031783312741175393664539881674330807724365734090335572247236031"
                ),
                BigInt!(
                    "15697249904809157640137081638559691717147113859496833342722786814178099529209"
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
                U256::from_str("0x219BEFB142BED271ACC41A52BA3F412BD1418AED36474A76AEFBFAD12CC6B592").unwrap(),
                U256::from_str("0x219BEFB142BED271ACC41A52BA3F412BD1418AED36474A76AEFBFAD12CC6B592").unwrap(),
                &hex!("650802113E0200000000000022480A207022627E60ED78120D2FE8DC7ACDB58A2321B0304F8912D2DFB86CE038E23CA812240801122041B8793236EE0980E2EAF1A2FAD268C4A3D8979A0C432F06E284EEC5E74DD69C320E756E696F6E2D6465766E65742D31"),
                hex!("170904B6B6F8E61E6E2D357436C7C3671E0ADE5ECF97E7A7A644F4B9F4DCC865138AAB8106303EFD304171C3B1E5181E4772AE47A050E19D9074BFD59072EE971FAC59DE603A8E7A677C4B55BD96FB62566CABD242D787687A86820E7ECD270407748CFCC9A3C646DD773D06AF98AB4ACBDF0FD76069212ABFF40C73C1FE0017073337A045A075449995D9B64417D017B670950E6C4F0BE8D8DA1AAFC41C3F4F0DCC256FD3B003EC5FC4A1A8BF7638FE2F703322DD418CC05E2EABC2A5801EA70CBC6020CA6DE288936436764093B0B18F815382DCDD75CEDC5B9D2B366478681C3B99E0A7F21D90D856C8AA59693A800A144D730737C3FD3E21B5DEBD1FC2B228516CBE9584853821FAE64F1C903936C07392529A5FB430007B3EB94C7EEB2F26A8A6B5E8474E4C283A2FD7A9DAD63A78230696F6D882D654725E28198653902B1CECF46DC9C200AEB9DB432A8ACA40EECD8CBA27C32975F1B338B7EFFC2CBA")
            ),
            Ok(())
        );
    }
}
