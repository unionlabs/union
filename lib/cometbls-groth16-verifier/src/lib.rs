// #![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use core::marker::PhantomData;
use std::fmt::Debug;

use ark_ff::{vec, vec::Vec, BigInt};
use byteorder::{BigEndian, ByteOrder};
use hex_literal::hex;
use sha3::Digest;
use unionlabs::{uint::U256, ByteArrayExt};

pub const HMAC_O: &[u8] = &hex!("1F333139281E100F5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C");
pub const HMAC_I: &[u8] = &hex!("75595B5342747A653636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636");
pub const PRIME_R_MINUS_ONE: U256 = U256::from_limbs([
    4891460686036598784,
    2896914383306846353,
    13281191951274694749,
    3486998266802970665,
]);

fn make_g1(x: BigInt<4>, y: BigInt<4>) -> substrate_bn::AffineG1 {
    substrate_bn::AffineG1::new(
        substrate_bn::Fq::from_u256(x.0.into()).unwrap(),
        substrate_bn::Fq::from_u256(y.0.into()).unwrap(),
    )
    .unwrap()
}

fn make_g2(x0: BigInt<4>, x1: BigInt<4>, y0: BigInt<4>, y1: BigInt<4>) -> substrate_bn::AffineG2 {
    substrate_bn::AffineG2::new(
        substrate_bn::Fq2::new(
            substrate_bn::Fq::from_u256(x0.0.into()).unwrap(),
            substrate_bn::Fq::from_u256(x1.0.into()).unwrap(),
        ),
        substrate_bn::Fq2::new(
            substrate_bn::Fq::from_u256(y0.0.into()).unwrap(),
            substrate_bn::Fq::from_u256(y1.0.into()).unwrap(),
        ),
    )
    .unwrap()
}

// TODO: this should be computed at compile time
pub fn pedersen_commitment_key() -> (substrate_bn::AffineG2, substrate_bn::AffineG2) {
    let g_raw = hex!("0DB410C824A5ADBD313D740A430630A107D57410B9BF6FAF5AFCFAAB2B2617FE1B42C82BA48ECE99B163761C96B995CCC15598BFBA746FC6E9DBAD445DDA796D1F34EBAF716B9210C884F8F07F0E08DEB6435B770E5E34B8244497D38840CE2B21EA2052643FFDCD11760154036BD266D6A261638E25AFB9917DA8E0C347C9B8");
    let g_root_sigma_neg_raw = hex!("247B0E5AF7C23D717F5C88E71545A7CD67052C3141DF9EBF8B1FFE7ADB1CBDC10C301FD6EA0C05EF4B9FB346AF88B7BA904A8EB37E87E412B04A002801B429A7161556EA8AE6D6B0B9E74133A53F5F15B2859611C982615E0D7937FD929EB90A2C07A459154070A4C140C7766C4034D1AF770F072C1A3C7E5E41B685AB9547A9");
    let G2Affine(_, g) = G2Affine::<BigEndian>::try_from(g_raw).expect("impossible");
    let G2Affine(_, g_root_sigma_neg) =
        G2Affine::<BigEndian>::try_from(g_root_sigma_neg_raw).expect("impossible");
    (g, g_root_sigma_neg)
}

// TODO: this should be computed at compile time
pub fn universal_vk() -> VerifyingKey {
    VerifyingKey {
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
    (U256::from_big_endian(hmac_keccak(message)) % PRIME_R_MINUS_ONE) + U256::from(1)
}

pub const FQ_SIZE: usize = 32;
pub const G1_SIZE: usize = 2 * FQ_SIZE;
pub const G2_SIZE: usize = 2 * G1_SIZE;
pub const COMMITMENT_HASH_SIZE: usize = 32;

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

/// A verification key in the Groth16 SNARK.
pub struct VerifyingKey {
    /// The `alpha * G`, where `G` is the generator of `E::G1`.
    pub alpha_g1: substrate_bn::AffineG1,
    /// The `alpha * H`, where `H` is the generator of `E::G2`.
    pub beta_g2: substrate_bn::AffineG2,
    /// The `gamma * H`, where `H` is the generator of `E::G2`.
    pub gamma_g2: substrate_bn::AffineG2,
    /// The `delta * H`, where `H` is the generator of `E::G2`.
    pub delta_g2: substrate_bn::AffineG2,
    /// The `gamma^{-1} * (beta * a_i + alpha * b_i + c_i) * H`, where `H` is the generator of `E::G1`.
    pub gamma_abc_g1: Vec<substrate_bn::AffineG1>,
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

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidPublicInput,
    InvalidPoint,
    InvalidProof,
    InvalidVerifyingKey,
    InvalidCommitment,
    InvalidRawProof,
}

pub fn verify_zkp(
    trusted_validators_hash: U256,
    untrusted_validators_hash: U256,
    message: &[u8],
    zkp: impl Into<Vec<u8>>,
) -> Result<(), Error> {
    let (g, g_root_sigma_neg) = pedersen_commitment_key();
    verify_generic_zkp_2(
        universal_vk(),
        trusted_validators_hash,
        untrusted_validators_hash,
        message,
        g,
        g_root_sigma_neg,
        ZKP::try_from(zkp.into().as_ref())?,
    )
}

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

fn verify_generic_zkp_2(
    vk: VerifyingKey,
    trusted_validators_hash: U256,
    untrusted_validators_hash: U256,
    message: &[u8],
    g: substrate_bn::AffineG2,
    g_root_sigma_neg: substrate_bn::AffineG2,
    zkp: ZKP<BigEndian>,
) -> Result<(), Error> {
    let decode_scalar = move |x: U256| -> Result<substrate_bn::Fr, Error> {
        substrate_bn::Fr::new(x.0 .0.into()).ok_or(Error::InvalidPublicInput)
    };
    let commitment_hash = hash_commitment(&zkp.proof_commitment)?;
    let hashed_message = hash_to_field(message);
    let public_inputs: [substrate_bn::Fr; 4] = [
        decode_scalar(trusted_validators_hash)?,
        decode_scalar(untrusted_validators_hash)?,
        decode_scalar(hashed_message)?,
        decode_scalar(commitment_hash)?,
    ];
    let initial_point = substrate_bn::G1::from(
        vk.gamma_abc_g1
            .first()
            .copied()
            .ok_or(Error::InvalidVerifyingKey)?,
    ) + zkp.proof_commitment.into();
    let public_inputs_msm = public_inputs
        .into_iter()
        .zip(vk.gamma_abc_g1.into_iter().skip(1))
        .fold(
            initial_point,
            |state, (public_input, verifying_key_public_input)| {
                state + substrate_bn::G1::from(verifying_key_public_input) * public_input
            },
        );
    // TODO: the negation should be computed at compile time
    let result = substrate_bn::pairing_batch(&[
        (zkp.proof.a.into(), zkp.proof.b.into()),
        (public_inputs_msm, -substrate_bn::G2::from(vk.gamma_g2)),
        (zkp.proof.c.into(), -substrate_bn::G2::from(vk.delta_g2)),
        (zkp.proof_commitment.into(), g.into()),
        (zkp.proof_commitment_pok.into(), g_root_sigma_neg.into()),
    ]);
    // TODO: this pairing should be computed at compile time
    if result != substrate_bn::pairing(vk.alpha_g1.into(), vk.beta_g2.into()) {
        Err(Error::InvalidProof)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;

    #[test]
    fn test_ok() {
        assert!(matches!(
            verify_zkp(
                U256::from_str("17204F2B98C9E9A6C92C29AC7E19C1BF025530DEE72793868EE9B040CA00417B").unwrap(),
                U256::from_str("17204F2B98C9E9A6C92C29AC7E19C1BF025530DEE72793868EE9B040CA00417B").unwrap(),
                &hex!("650802113E0200000000000022480A207A3675198C63E4D7E49CD290929CA9B713B6FCB867EA023DB55BB9CA505946B212240801122009F221212558CB45E97A3A349E215937FF36B69E1EDE1F468A6C64C71F57A2E4320E756E696F6E2D6465766E65742D31"),
                hex!("195562CC376E9265A7FD89A086855C100173B717B0DEA58AC9F50120E9CBDD7402D59ADAC8A274C5DDB199915B03B5CFB7A91032A71723876F946A7662135D4912EB1FAD1FCA5E88AD1D9097870391D1D477F4CD2A26F27DB3CFC8B511922C482F374A4821BEE34818589A052995CC5994CE787538207F1BA0D595890EB96D751D947274566F6338FC14BB1728C9E42F47F9D47A8A7F46CFA341D3EC71F0A8E80ECDAA9E38B4D6090989B165E536C4332BDF470E860D85001362EC7B369DE0092FD13C85FE2A16247E574B759B7B8EBFE8C7ED19CE7520A693BD09FD604CA54E2FA277AC176ACEC9626313DA7022E8B8DB599E1B02C25DA90AD508AA315DA67C0EAF8A0F41C4CDC897A4941F3BFA7D0E0C2BDD3030D5B0025FB4030A31C886F417B2509E9ECFEA86AA22F75402599E72C21623E9C32A499D7B14B6DBC3A1251E119244B7DC12B54A74FBC3B23E7954435491D89AFA7ABF6F07E1DADE0B28F0DA1978EC72A2C2C0F1FE8DEDA8DD8DDA7E82454618C3DFF1341C9901456F7E656A")
            ),
            Ok(())
        ));
    }

    #[test]
    fn test_ko() {
        assert!(matches!(
            verify_zkp(
                U256::from_str("1BBACC23BE35969FFCEFC2892440045E83C3C78E81BF2D6473DD745A93835684").unwrap(),
                U256::from_str("1BBACC23BE35969FFCEFC2892440045E83C3C78E81BF2D6473DD745A93835684").unwrap(),
                &hex!("650802113E0200000000000022480A2053D3DC9F43757EEA63FC3B28C383074A111146B2DE7F73A198D29A6D6919DA6D12240801122023C8BED9455A38334F6462A3EAC87616CF51226F825A229FA23CA420E26730B9320E756E696F6E2D6465766E65742D31"),
                hex!("195562CC376E9265A7FD89A086855C100173B717B0DEA58AC9F50120E9CBDD7402D59ADAC8A274C5DDB199915B03B5CFB7A91032A71723876F946A7662135D4912EB1FAD1FCA5E88AD1D9097870391D1D477F4CD2A26F27DB3CFC8B511922C482F374A4821BEE34818589A052995CC5994CE787538207F1BA0D595890EB96D751D947274566F6338FC14BB1728C9E42F47F9D47A8A7F46CFA341D3EC71F0A8E80ECDAA9E38B4D6090989B165E536C4332BDF470E860D85001362EC7B369DE0092FD13C85FE2A16247E574B759B7B8EBFE8C7ED19CE7520A693BD09FD604CA54E2FA277AC176ACEC9626313DA7022E8B8DB599E1B02C25DA90AD508AA315DA67C0EAF8A0F41C4CDC897A4941F3BFA7D0E0C2BDD3030D5B0025FB4030A31C886F417B2509E9ECFEA86AA22F75402599E72C21623E9C32A499D7B14B6DBC3A1251E119244B7DC12B54A74FBC3B23E7954435491D89AFA7ABF6F07E1DADE0B28F0DA1978EC72A2C2C0F1FE8DEDA8DD8DDA7E82454618C3DFF1341C9901456F7E656A")
            ),
            Err(Error::InvalidProof)
        ));
    }

    #[test]
    fn test_ok_2() {
        assert_eq!(
            verify_zkp(
                U256::from_str("2f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500d").unwrap(),
                U256::from_str("2f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500d").unwrap(),
                &hex!("650802117c0000000000000022480a20ed341d012b198b8c6962209f30ac4a07c06d53ab258865aade613dcd5800aec5122408011220b1c27e9a68de8ddbc981319dea0ad31aa3e41f6759bd7200581eff9d1373ca9f320e756e696f6e2d6465766e65742d31"),
                hex!("07942610e1aeb229308405cd7fc0305f31129bb6d7a3f39b0b18ac0adc09e5301d05c7dfb6b1c21aeeae94928b7ddbf59c04454454b785bc430b8d825dc1a52f0444e6bddff7896fce6625c4fef776be5ef1dc9e539db05241b201e83e1ed2d02942b8a7c777ed5806508ab66547cfcff01f0a0aeffa773f32dfb9bf76c07e700c21088d0ed1f4aea52b7962ac5ffa2748d4b021bcafa5bcec2e1748130e64691dea0ac767b1fd72750c517f49da19aaa4e5e70591f9bdc1d177850275e2f1a90712c5ed8902568d20e34b2f3e224c3bcbefa57917efe64104d19767a419524f1eb315c1291e1eeaaf765a3f3c2f0ddd908b49cd2e5e776dc9b063fa62777dfc2a59e984c4b0a21d8afb790d9d06cb7d0cbef6b573eaa48398a8d0b731f3362c2f385771a9bce77c5e6cd66c074d36b6cb71cfe97c65dd75bcad2a9d91f899ee15256a75d3065bee14962a6b10b05b72ba616034803a76c8487fd9285f502c011eae9d47767324ea7d90ee9b4e8d9dbcad3cdc1759d3566e2351bd1176d3cd28")
            ),
            Ok(())
        );
    }
}
