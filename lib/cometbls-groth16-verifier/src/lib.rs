#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use core::{marker::PhantomData, ops::AddAssign};

use ark_ec::{
    pairing::{Pairing, PairingOutput},
    AffineRepr, CurveGroup,
};
use ark_ff::{vec, vec::Vec, BigInt, PrimeField, QuadExtField};
use ark_groth16::{PreparedVerifyingKey, Proof, VerifyingKey};
use ark_relations::r1cs::SynthesisError;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
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

pub fn pedersen_commitment_key() -> (ark_bn254::G2Affine, ark_bn254::G2Affine) {
    let G2Affine(_, g) = G2Affine::<BigEndian, ark_bn254::Bn254>::try_from(hex!("0DB410C824A5ADBD313D740A430630A107D57410B9BF6FAF5AFCFAAB2B2617FE1B42C82BA48ECE99B163761C96B995CCC15598BFBA746FC6E9DBAD445DDA796D1F34EBAF716B9210C884F8F07F0E08DEB6435B770E5E34B8244497D38840CE2B21EA2052643FFDCD11760154036BD266D6A261638E25AFB9917DA8E0C347C9B8")).expect("impossible");
    let G2Affine(_, g_root_sigma_neg) = G2Affine::<BigEndian, ark_bn254::Bn254>::try_from(hex!("247B0E5AF7C23D717F5C88E71545A7CD67052C3141DF9EBF8B1FFE7ADB1CBDC10C301FD6EA0C05EF4B9FB346AF88B7BA904A8EB37E87E412B04A002801B429A7161556EA8AE6D6B0B9E74133A53F5F15B2859611C982615E0D7937FD929EB90A2C07A459154070A4C140C7766C4034D1AF770F072C1A3C7E5E41B685AB9547A9")).expect("impossible");
    (g, g_root_sigma_neg)
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

#[derive(Debug)]
pub struct ZKP<FromOrder, P: Pairing> {
    pub proof: Proof<P>,
    pub proof_commitment: P::G1Affine,
    pub proof_commitment_pok: P::G1Affine,
    pub _marker: PhantomData<FromOrder>,
}

// G1 + G2 + G1 + G1 + G1
pub const EXPECTED_PROOF_SIZE: usize = G1_SIZE + G2_SIZE + G1_SIZE + G1_SIZE + G1_SIZE;

// [a ... b ... c ... proof_commitment ... commitment_pok]
pub type RawZKP = [u8; EXPECTED_PROOF_SIZE];

impl<FromOrder: ByteOrder, P: Pairing> TryFrom<&[u8]> for ZKP<FromOrder, P>
where
    G1Affine<FromOrder, P>: TryFrom<[u8; G1_SIZE], Error = Error>,
    G2Affine<FromOrder, P>: TryFrom<[u8; G2_SIZE], Error = Error>,
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
        let G1Affine(_, proof_commitment) = G1Affine::<FromOrder, P>::try_from(
            value[G1_SIZE + G2_SIZE + G1_SIZE..G1_SIZE + G2_SIZE + G1_SIZE + G1_SIZE]
                .try_into()
                .expect("impossible"),
        )?;
        let G1Affine(_, proof_commitment_pok) = G1Affine::<FromOrder, P>::try_from(
            value[G1_SIZE + G2_SIZE + G1_SIZE + G1_SIZE
                ..G1_SIZE + G2_SIZE + G1_SIZE + G1_SIZE + G1_SIZE]
                .try_into()
                .expect("impossible"),
        )?;
        Ok(Self {
            proof: Proof { a, b, c },
            proof_commitment,
            proof_commitment_pok,
            _marker: PhantomData,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    EthAbiDecoding,
    InvalidPublicInput,
    InvalidPoint,
    InvalidProof(SynthesisError),
    InvalidVerifyingKey,
    InvalidCommitment,
    InvalidCommitmentPOK,
    InvalidRawProof,
}

pub fn verify_zkp(
    trusted_validators_hash: U256,
    untrusted_validators_hash: U256,
    message: &[u8],
    zkp: impl Into<Vec<u8>>,
) -> Result<(), Error> {
    let (g, g_root_sigma_neg) = pedersen_commitment_key();
    verify_generic_zkp::<ark_bn254::Bn254>(
        universal_vk(),
        trusted_validators_hash,
        untrusted_validators_hash,
        message,
        &g,
        &g_root_sigma_neg,
        ZKP::try_from(zkp.into().as_ref())?,
    )
}

fn hash_commitment<P: Pairing>(proof_commitment: &P::G1Affine) -> Result<U256, Error> {
    let mut buffer = [0u8; 64];
    proof_commitment
        .serialize_uncompressed(&mut buffer[..])
        .map_err(|_| Error::InvalidCommitment)?;
    // arkworks is little endian, gnark is big endian
    buffer[0..32].reverse();
    buffer[32..64].reverse();
    Ok(hash_to_field(&buffer))
}

fn verify_generic_zkp<P: Pairing>(
    vk: VerifyingKey<P>,
    trusted_validators_hash: U256,
    untrusted_validators_hash: U256,
    message: &[u8],
    g: &P::G2Affine,
    g_root_sigma_neg: &P::G2Affine,
    zkp: ZKP<BigEndian, P>,
) -> Result<(), Error> {
    let mut buffer = [0u8; 32];
    let mut decode_scalar = move |x: U256| -> Result<P::ScalarField, Error> {
        x.to_little_endian(&mut buffer);
        let value = <P::ScalarField as PrimeField>::BigInt::deserialize_uncompressed(&buffer[..])
            .map_err(|_| Error::InvalidPublicInput)?;
        <P::ScalarField as PrimeField>::from_bigint(value).ok_or(Error::InvalidPublicInput)
    };

    let commitment_hash = hash_commitment::<P>(&zkp.proof_commitment)?;

    let hashed_message = hash_to_field(message);

    let public_inputs: [P::ScalarField; 4] = [
        decode_scalar(trusted_validators_hash)?,
        decode_scalar(untrusted_validators_hash)?,
        decode_scalar(hashed_message)?,
        decode_scalar(commitment_hash)?,
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

    let pvk = PreparedVerifyingKey::from(vk);

    // Verify both the pedersen commitment and the zkp
    let PairingOutput(result) = P::final_exponentiation(P::multi_miller_loop(
        [
            <P::G1Affine as Into<P::G1Prepared>>::into(zkp.proof.a),
            prepared_inputs.into_affine().into(),
            zkp.proof.c.into(),
            // Pedersen commitment proof of knowledge
            // Symmetric to https://github.com/Consensys/gnark-crypto/blob/2e4aaaaefdbfdf06515663986ed884fed1b2177e/ecc/bn254/fr/pedersen/pedersen.go#L212-L224
            zkp.proof_commitment.into(),
            zkp.proof_commitment_pok.into(),
        ],
        [
            zkp.proof.b.into(),
            pvk.gamma_g2_neg_pc,
            pvk.delta_g2_neg_pc,
            // Pedersen key
            g.into(),
            g_root_sigma_neg.into(),
        ],
    ))
    .ok_or(Error::InvalidProof(SynthesisError::UnexpectedIdentity))?;

    if result == pvk.alpha_g1_beta_g2 {
        Ok(())
    } else {
        Err(Error::InvalidProof(SynthesisError::UnexpectedIdentity))
    }
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
                U256::from_str("17204F2B98C9E9A6C92C29AC7E19C1BF025530DEE72793868EE9B040CA00417B").unwrap(),
                U256::from_str("17204F2B98C9E9A6C92C29AC7E19C1BF025530DEE72793868EE9B040CA00417B").unwrap(),
                &hex!("650802113E0200000000000022480A207A3675198C63E4D7E49CD290929CA9B713B6FCB867EA023DB55BB9CA505946B212240801122009F221212558CB45E97A3A349E215937FF36B69E1EDE1F468A6C64C71F57A2E4320E756E696F6E2D6465766E65742D31"),
                hex!("195562CC376E9265A7FD89A086855C100173B717B0DEA58AC9F50120E9CBDD7402D59ADAC8A274C5DDB199915B03B5CFB7A91032A71723876F946A7662135D4912EB1FAD1FCA5E88AD1D9097870391D1D477F4CD2A26F27DB3CFC8B511922C482F374A4821BEE34818589A052995CC5994CE787538207F1BA0D595890EB96D751D947274566F6338FC14BB1728C9E42F47F9D47A8A7F46CFA341D3EC71F0A8E80ECDAA9E38B4D6090989B165E536C4332BDF470E860D85001362EC7B369DE0092FD13C85FE2A16247E574B759B7B8EBFE8C7ED19CE7520A693BD09FD604CA54E2FA277AC176ACEC9626313DA7022E8B8DB599E1B02C25DA90AD508AA315DA67C0EAF8A0F41C4CDC897A4941F3BFA7D0E0C2BDD3030D5B0025FB4030A31C886F417B2509E9ECFEA86AA22F75402599E72C21623E9C32A499D7B14B6DBC3A1251E119244B7DC12B54A74FBC3B23E7954435491D89AFA7ABF6F07E1DADE0B28F0DA1978EC72A2C2C0F1FE8DEDA8DD8DDA7E82454618C3DFF1341C9901456F7E656A")
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
                hex!("195562CC376E9265A7FD89A086855C100173B717B0DEA58AC9F50120E9CBDD7402D59ADAC8A274C5DDB199915B03B5CFB7A91032A71723876F946A7662135D4912EB1FAD1FCA5E88AD1D9097870391D1D477F4CD2A26F27DB3CFC8B511922C482F374A4821BEE34818589A052995CC5994CE787538207F1BA0D595890EB96D751D947274566F6338FC14BB1728C9E42F47F9D47A8A7F46CFA341D3EC71F0A8E80ECDAA9E38B4D6090989B165E536C4332BDF470E860D85001362EC7B369DE0092FD13C85FE2A16247E574B759B7B8EBFE8C7ED19CE7520A693BD09FD604CA54E2FA277AC176ACEC9626313DA7022E8B8DB599E1B02C25DA90AD508AA315DA67C0EAF8A0F41C4CDC897A4941F3BFA7D0E0C2BDD3030D5B0025FB4030A31C886F417B2509E9ECFEA86AA22F75402599E72C21623E9C32A499D7B14B6DBC3A1251E119244B7DC12B54A74FBC3B23E7954435491D89AFA7ABF6F07E1DADE0B28F0DA1978EC72A2C2C0F1FE8DEDA8DD8DDA7E82454618C3DFF1341C9901456F7E656A")
            ),
            Err(Error::InvalidProof(SynthesisError::UnexpectedIdentity))
        );
    }
}
