#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use core::marker::PhantomData;

use ark_ff::{vec, BigInt};
use byteorder::{BigEndian, ByteOrder};
use hex_literal::hex;
use sha2::Sha256;
use sha3::Digest;
use substrate_bn::G1;
use unionlabs::{
    hash::H256, ibc::lightclients::cometbls::light_header::LightHeader, uint::U256, ByteArrayExt,
};

pub const NB_PUBLIC_INPUTS: usize = 2;

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
    let g_raw = hex!("257DF6F8132CB0037F7DFDF1A29B04C1FF92BA082EDA513996BA2BFA9FBD198713F0D8D8879885CA567EF99298C30C397E6FBA584658F4127713A814C06DE55A1660EBCC60C7A3AC560EFCEA5993F528EE13685D3A39694ACD74FE67C80D798A15E80642C58DB4DBE0A87F92CE3C65E962F231278353783A691FD64078BA7F34");
    let g_root_sigma_neg_raw = hex!("2FBFE141A7555CF7E3E86B092660B81CFB68A025AD817E45CEC0B0F2E2CA636802A104DF1C015F2307FA2859627098CDF9FDB521D61D323943343A12304E5BAF27DA3F93ECF3BFD0B3A3354AE2162A6C230C0E539B6D9F82C0826E2B006A59222C0838551CB9E5CF67DB57DE7E2250BB97807F6687F135A6EB910359BA7BDB8D");
    let G2Affine(_, g) = G2Affine::<BigEndian>::try_from(g_raw).expect("impossible");
    let G2Affine(_, g_root_sigma_neg) =
        G2Affine::<BigEndian>::try_from(g_root_sigma_neg_raw).expect("impossible");
    (g, g_root_sigma_neg)
}

// TODO: this should be computed at compile time
pub fn universal_vk() -> VerifyingKey {
    VerifyingKey {
        alpha_g1: make_g1(
            BigInt!("4252850302693242182654534639730627324742305503909561446344356971523664816281"),
            BigInt!("3971530409048238023625806606514600982127202826003358538821613170737831313919"),
        ),
        beta_g2: make_g2(
            BigInt!("9609903744775525881338738176064678545439912439219033822736570321349357348980"),
            BigInt!(
                "11402125448377072234752634956069960846261435348550776006069399216352815312229"
            ),
            BigInt!("3876014193556985028076276590285094449745398487447250532380698384573245200038"),
            BigInt!("6131692356384648492800758325058748831519318785594820705365176509549681793745"),
        ),
        gamma_g2: make_g2(
            BigInt!(
                "15418804173338388766896385877623893969695670309009587476846726795628238714393"
            ),
            BigInt!(
                "14882897597913405382982164467298010752166363844685258881581520272046793702095"
            ),
            BigInt!("4166025151148225057462107057100265181139888889391061071239248954005945470477"),
            BigInt!("206728492847877950288262169260916452585500374823256459470367014125967964118"),
        ),
        delta_g2: make_g2(
            BigInt!("2636161939055419322743684458857549714230849256995406138405588958157843793131"),
            BigInt!(
                "18711435617866698040659011365354165232283248284733617156044102129651710736892"
            ),
            BigInt!(
                "19240355865528042255113556794397480864884450537537107687508383548050491695680"
            ),
            BigInt!(
                "12249371269602120664445362627662636389936048209522657338249293583990077475589"
            ),
        ),
        gamma_abc_g1: vec![
            make_g1(
                BigInt!(
                    "17683074019270049519594214298171697666582975915064153618004061598086681825921"
                ),
                BigInt!(
                    "16826145467743906176166100307225491106961753217491843100452871479833450456070"
                ),
            ),
            make_g1(
                BigInt!(
                    "4999724750322169039879775285047941133298355297928988655266615607529011563466"
                ),
                BigInt!(
                    "8614448667589143428827059805500251818303043966026074735628377626634208993292"
                ),
            ),
            make_g1(
                BigInt!(
                    "1184807858330365651919114999096473332175166887333719856514157833289677967559"
                ),
                BigInt!(
                    "20327610427697660249999185524229068956160879388632193295649998184224119517657"
                ),
            ),
        ],
    }
}

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
    (U256::from_big_endian(hmac_keccak(message)) % PRIME_R_MINUS_ONE) + U256::from(1)
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
    InvalidChainId,
    InvalidHeight,
    InvalidTimestamp,
}

pub fn verify_zkp(
    chain_id: &str,
    trusted_validators_hash: H256,
    header: &LightHeader,
    zkp: impl Into<Vec<u8>>,
) -> Result<(), Error> {
    let (g, g_root_sigma_neg) = pedersen_commitment_key();
    verify_generic_zkp_2(
        universal_vk(),
        chain_id,
        trusted_validators_hash,
        header,
        g,
        g_root_sigma_neg,
        ZKP::try_from(zkp.into().as_ref())?,
    )
}

fn affine_g1_to_bytes(g1_point: &substrate_bn::AffineG1) -> Result<[u8; 64], Error> {
    let mut buffer = [0; 64];
    g1_point
        .x()
        .to_big_endian(&mut buffer[..32])
        .map_err(|_| Error::InvalidPoint)?;
    g1_point
        .y()
        .to_big_endian(&mut buffer[32..])
        .map_err(|_| Error::InvalidPoint)?;
    Ok(buffer)
}

fn verify_generic_zkp_2(
    vk: VerifyingKey,
    chain_id: &str,
    trusted_validators_hash: H256,
    header: &LightHeader,
    g: substrate_bn::AffineG2,
    g_root_sigma_neg: substrate_bn::AffineG2,
    zkp: ZKP<BigEndian>,
) -> Result<(), Error> {
    if chain_id.len() > 31 {
        return Err(Error::InvalidChainId);
    }
    // Constant + public inputs
    if vk.gamma_abc_g1.len() != NB_PUBLIC_INPUTS + 1 {
        return Err(Error::InvalidVerifyingKey);
    }
    let decode_scalar = move |x: U256| -> Result<substrate_bn::Fr, Error> {
        substrate_bn::Fr::new(x.0 .0.into()).ok_or(Error::InvalidPublicInput)
    };
    let commitment_hash = hash_commitment(&zkp.proof_commitment)?;
    let mut inputs_hash = <[u8; 32]>::from(
        sha2::Sha256::new()
            .chain_update(
                vec![0u8; 32 - chain_id.len()]
                    .into_iter()
                    .chain(chain_id.bytes())
                    .collect::<Vec<_>>(),
            )
            .chain_update(
                U256::from(
                    u64::try_from(i64::from(header.height)).map_err(|_| Error::InvalidHeight)?,
                )
                .to_big_endian(),
            )
            .chain_update(
                U256::from(
                    u64::try_from(i64::from(header.time.seconds))
                        .map_err(|_| Error::InvalidTimestamp)?,
                )
                .to_big_endian(),
            )
            .chain_update(
                U256::from(
                    u64::try_from(i32::from(header.time.nanos))
                        .map_err(|_| Error::InvalidTimestamp)?,
                )
                .to_big_endian(),
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
        decode_scalar(U256::from_big_endian(inputs_hash))?,
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
        .zip(
            vk.gamma_abc_g1
                .into_iter()
                .skip(1)
                .map(substrate_bn::G1::from),
        )
        .fold(initial_point, |s, (w_i, gamma_l_i)| s + gamma_l_i * w_i);
    // TODO: the verifying key transformation, pedersen key decoding and this negations should all be done at compile time
    // TODO: random linear combination required?
    let r1 = Sha256::new()
        .chain_update(&affine_g1_to_bytes(&zkp.proof.a)?)
        .chain_update(&affine_g1_to_bytes(&zkp.proof.c)?)
        .chain_update(&affine_g1_to_bytes(&public_inputs_msm.into())?)
        .finalize();

    let r2 = Sha256::new()
        .chain_update(&affine_g1_to_bytes(&zkp.proof_commitment)?)
        .chain_update(&affine_g1_to_bytes(&zkp.proof_commitment_pok)?)
        .finalize();

    let r1 = substrate_bn::Fr::from_slice(&r1).unwrap();
    let r2 = substrate_bn::Fr::from_slice(&r2).unwrap();

    let result = substrate_bn::pairing_batch(&[
        (G1::try_from(zkp.proof.a).unwrap() * r1, zkp.proof.b.into()),
        (public_inputs_msm * r1, -substrate_bn::G2::from(vk.gamma_g2)),
        (
            G1::try_from(zkp.proof.c).unwrap() * r1,
            -substrate_bn::G2::from(vk.delta_g2),
        ),
        (
            G1::try_from(vk.alpha_g1).unwrap() * r1,
            -substrate_bn::G2::from(vk.beta_g2),
        ),
        // Verify pedersen proof of knowledge
        (G1::try_from(zkp.proof_commitment).unwrap() * r2, g.into()),
        (
            G1::try_from(zkp.proof_commitment_pok).unwrap() * r2,
            g_root_sigma_neg.into(),
        ),
    ]);
    if result != substrate_bn::Gt::one() {
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
                "union-devnet-1337",
                hex!("1B7EA0F1B3E574F8D50A12827CCEA43CFF858C2716AE05370CC40AE8EC521FD8").into(),
                &LightHeader {
                    height: 3405691582.try_into().unwrap(),
                    time: Timestamp {
                        seconds: 1710783278.try_into().unwrap(),
                        nanos: 499600406.try_into().unwrap()
                    },
                    validators_hash: hex!("1B7EA0F1B3E574F8D50A12827CCEA43CFF858C2716AE05370CC40AE8EC521FD8").into(),
                    next_validators_hash: hex!("1B7EA0F1B3E574F8D50A12827CCEA43CFF858C2716AE05370CC40AE8EC521FD8").into(),
                    app_hash: hex!("3A34FC963EEFAAE9B7C0D3DFF89180D91F3E31073E654F732340CEEDD77DD25B").into(),
                },
                hex!("294A48A750D5C2CF926516752FF484EEBE55FF26CF8A8A7536D98794CF062DB6214D0C9E5C6B164111927A1630889619DBBB40149D8E2D32898E7ACB765542CD0EB8A8E04CCC254C3BFDC2FCE627D59C3C05E2AC76E03977855DD889C1C9BA432FF7FF4DEFCB5286555D36D22DD073A859140508AF9B977F38EB9A604E99A5F6109D43A4AFA0AB161DA2B261DED80FBC0C36E57DE2001338941C834E3262CF751BC1BFC6EC27BB8E106BAAB976285BAC1D4AC38D1B759C8A2852D65CE239974F1275CC6765B3D174FD1122EFDE86137D19F07483FEF5244B1D74B2D9DC598AC32A5CA10E8837FBC89703F4D0D46912CF4AF82341C30C2A1F3941849CC011A56E18AD2162EEB71289B8821CC01875BC1E35E5FC1EBD9114C0B2C0F0D9A96C394001468C70A1716CA98EBE82B1E614D4D9B07292EBAD5B60E0C76FD1D58B485E7D1FB1E07F51A0C68E4CA59A399FCF0634D9585BE478E37480423681B984E96C0A1698D8FCB1DF51CAE023B045E114EED9CB233A5742D9E60E1097206EB20A5058")
            ),
            Ok(())
        );
    }

    #[test]
    fn test_tampered_block() {
        assert_eq!(
            verify_zkp(
                "union-devnet-1337",
                hex!("1B7EA0F1B3E574F8D50A12827CCEA43CFF858C2716AE05370CC40AE8EC521FD8").into(),
                &LightHeader {
                    height: 3405691583.try_into().unwrap(),
                    time: Timestamp {
                        seconds: 1710783278.try_into().unwrap(),
                        nanos: 499600406.try_into().unwrap()
                    },
                    validators_hash: hex!("1B7EA0F1B3E574F8D50A12827CCEA43CFF858C2716AE05370CC40AE8EC521FD8").into(),
                    next_validators_hash: hex!("1B7EA0F1B3E574F8D50A12827CCEA43CFF858C2716AE05370CC40AE8EC521FD8").into(),
                    app_hash: hex!("3A34FC963EEFAAE9B7C0D3DFF89180D91F3E31073E654F732340CEEDD77DD25B").into(),
                },
                hex!("294A48A750D5C2CF926516752FF484EEBE55FF26CF8A8A7536D98794CF062DB6214D0C9E5C6B164111927A1630889619DBBB40149D8E2D32898E7ACB765542CD0EB8A8E04CCC254C3BFDC2FCE627D59C3C05E2AC76E03977855DD889C1C9BA432FF7FF4DEFCB5286555D36D22DD073A859140508AF9B977F38EB9A604E99A5F6109D43A4AFA0AB161DA2B261DED80FBC0C36E57DE2001338941C834E3262CF751BC1BFC6EC27BB8E106BAAB976285BAC1D4AC38D1B759C8A2852D65CE239974F1275CC6765B3D174FD1122EFDE86137D19F07483FEF5244B1D74B2D9DC598AC32A5CA10E8837FBC89703F4D0D46912CF4AF82341C30C2A1F3941849CC011A56E18AD2162EEB71289B8821CC01875BC1E35E5FC1EBD9114C0B2C0F0D9A96C394001468C70A1716CA98EBE82B1E614D4D9B07292EBAD5B60E0C76FD1D58B485E7D1FB1E07F51A0C68E4CA59A399FCF0634D9585BE478E37480423681B984E96C0A1698D8FCB1DF51CAE023B045E114EED9CB233A5742D9E60E1097206EB20A5058")
            ),
            Err(Error::InvalidProof)
        );
    }
}
