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
    g: substrate_bn::AffineG2,
    g_root_sigma_neg: substrate_bn::AffineG2,
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
    let initial_point = substrate_bn::G1::from(GAMMA_ABC_G1[0]) + zkp.proof_commitment.into();
    let public_inputs_msm = public_inputs
        .into_iter()
        .zip(GAMMA_ABC_G1.into_iter().skip(1).map(substrate_bn::G1::from))
        .fold(initial_point, |s, (w_i, gamma_l_i)| s + gamma_l_i * w_i);

    let proof_a: G1 = zkp.proof.a.into();
    let proof_c: G1 = zkp.proof.c.into();
    let pc: G1 = zkp.proof_commitment.into();
    let pok: G1 = zkp.proof_commitment_pok.into();

    let pok_result = substrate_bn::pairing_batch(&[(pc, g.into()), (pok, g_root_sigma_neg.into())]);
    if pok_result != substrate_bn::Gt::one() {
        return Err(Error::InvalidPok);
    }

    let g16_result = substrate_bn::pairing_batch(&[
        (proof_a, zkp.proof.b.into()),
        (public_inputs_msm, -substrate_bn::G2::from(GAMMA_G2)),
        (proof_c, -substrate_bn::G2::from(DELTA_G2)),
        (G1::from(ALPHA_G1), -substrate_bn::G2::from(BETA_G2)),
    ]);
    if g16_result != substrate_bn::Gt::one() {
        Err(Error::InvalidProof)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use unionlabs::google::protobuf::timestamp::Timestamp;

    use super::*;

    #[test]
    fn test_ok() {
        assert_eq!(
            verify_zkp(
                &ChainId::from_string("union-devnet-1337").unwrap(),
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
    fn test_decode() {
        ZKP::try_from(hex!("1c9bc15a0c4541aff1d12780d6cf4ae2bdc6e3afafceae9d4fa36209fa323b68002e9c77c223d830e5df6a80cdd683f0986353933ee3179970fccc5d893219d30726f3b8c0dbe630b815b01b5557228a0dfeb0e0435bb0d15d1ccff7f6133fc110937d9fceee2f9052468c198fafeca89d524142a0efa9dc4df445853ce617302059018fef03dc34456ad201d2a5420a7d1c8fac57cb48cbe6709ac4da27d1eb250f73eab007d26cbff41ceb4564ab1cdfa83e9ee88be4f816dc841bbf2e90c80186ad9437fce7655c71b54addae1ccea429da3edba3232d073cb7e89ff2d27218556f1af0c446962ace932f637279dd0ad3ef1501fb6da39d5f68282f54bcf6094999672f3d8cbbf0409aef1048175ffff50b03a5154016d307a2ef425ffee509cd447b22ce6331c7a3473b2c6da1f9d550e8c3ab19bde65e699e07f4f2886c03ec4ff2faa0e342de7ac5daf32025acd6070c19ed8b007c121db0d955472c7d2e38d5a943d15bc902613029e4baa8c26034ff280e3a4d5468fcd6745afe53b5").as_slice()).unwrap();
    }

    #[test]
    fn test_err_969001_969006() {
        assert_eq!(
            verify_zkp(
                &ChainId::from_string("union-testnet-8").unwrap(),
                hex!("01a84dca649aa2df8de2f65a84c9092bbd5296b4bc54d818f844b28573d8e0be").into(),
                &LightHeader {
                    height: 969006.try_into().unwrap(),
                    time: Timestamp::from_str("2024-06-18T13:21:28.026113925Z").unwrap(),
                    validators_hash: hex!("01a84dca649aa2df8de2f65a84c9092bbd5296b4bc54d818f844b28573d8e0be").into(),
                    next_validators_hash: hex!("01a84dca649aa2df8de2f65a84c9092bbd5296b4bc54d818f844b28573d8e0be").into(),
                    app_hash: hex!("87822b2b2affeed1c7a67b15f73d9b4ad128d0984a7f0dca910d033092dac828").into(),
                },
                hex!("07c6767f0ec80904244a735ed6b0dba033fdcfaf92697438c560673c255331550201e520aa2202d20f2f4a264f03fde7faf072f053ff0c11630fc951e5956e9b0ac9b8301e712a57be7bd624659d937c3c42880b629d910743297fa444f626af2823ac2e190c3a8a78dbb2e8cc8c431c8536ce8d8fd8a2c192591e1559b5cbea04f330ec60397f60363457c00884d797cd7ff3a0e58fa27d2fed6eef6840e85a270f44f1ecdac406385b6b1ba933b21b5e5c390e09230d6710b30940434a39ab13db654f0bad779cce75d84f5cd302aa0feb83de879bd2ac96830bcacba82e9f0d00f4b0611ed108fe7e63217c3b058dd33d4a4de0307506eb20799f3c9db9f52fd45d1875ab9a5cdeecd0757d7d20a3af34f3182c6b53adc6076d914d2281c20cf63a90841b4f9aefc896a4e8defc01e76509a9b779b7e580a19143ff9af2591691b825825167febe9b5d762a17df13a0a4c547efa12c10b33d67886f505836213e3df47da08b551e92cb3db933ab06fd7dfbe84950bfba30e4f481ae35c3bc")
            ),
            Err(Error::InvalidProof)
        );
    }

    #[test]
    fn test_ok_969001_969002() {
        assert_eq!(
            verify_zkp(
                &ChainId::from_string("union-testnet-8").unwrap(),
                hex!("01a84dca649aa2df8de2f65a84c9092bbd5296b4bc54d818f844b28573d8e0be").into(),
                &LightHeader {
                    height: 969002.try_into().unwrap(),
                    time: Timestamp::from_str("2024-06-18T13:21:02.868708953Z").unwrap(),
                    validators_hash: hex!("01a84dca649aa2df8de2f65a84c9092bbd5296b4bc54d818f844b28573d8e0be").into(),
                    next_validators_hash: hex!("01a84dca649aa2df8de2f65a84c9092bbd5296b4bc54d818f844b28573d8e0be").into(),
                    app_hash: hex!("333f81c038816f109413eac1dc1cb8cef8facca1e9a49f21763f5dc84a375e14").into(),
                },
                hex!("02344d05cbb4f42548eadc621c46a3ae37f2ce23c12df83d1b490414bc20749a1fd5d4bd3b62a5b2cfae9f29686bfe1bc7a7c4bde72df168bdc1c1b0a3da1deb2a3f92896f5c37b4e3269aa84b47a67cad8b072350f794a15bac37608a5d549315e3850f18ddfa58ff9cfd5b2d133c3ac08d9f76e64611e6df4b6ba3d752e6f9054ec040028d1fd50d0f39eb60cb16326ba8876f5a47eea0c8b9c61461612bd518532a44ed88602a6e81177d08018fefadb2fedeac17ec26dae578532efb8a7905e1aca9429d9b8bfd7fb04e419c034258bc2d367e1c1a63936c67aca6767d5c1ba16ebb1dfccd919fa28d12255e6f9fcb98964682ca733bc591a25bd5a7993226daae60fea7d697b714916f9a6093f40a7a0e2a2a40b41b8741a98d5337b91f21a20866c16d94855c50593175e6d61481d56d08569ca55f8aa9f73277b3782a179b1bb01a269ae4eeacf273379099c641503f20830d6ef399867024b4f3c191120c8f0c1091387705c314ee6c5d8d23bf200649fe7b8dc2857db55f7bc5968c")
            ),
            Ok(())
        );
    }

    #[test]
    fn test_err_969001_969002() {
        assert_eq!(
            verify_zkp(
                &ChainId::from_string("union-testnet-8").unwrap(),
                hex!("01a84dca649aa2df8de2f65a84c9092bbd5296b4bc54d818f844b28573d8e0be").into(),
                &LightHeader {
                    height: 969002.try_into().unwrap(),
                    time: Timestamp::from_str("2024-06-18T13:21:02.868708953Z").unwrap(),
                    validators_hash: hex!("01a84dca649aa2df8de2f65a84c9092bbd5296b4bc54d818f844b28573d8e0be").into(),
                    next_validators_hash: hex!("01a84dca649aa2df8de2f65a84c9092bbd5296b4bc54d818f844b28573d8e0be").into(),
                    app_hash: hex!("333f81c038816f109413eac1dc1cb8cef8facca1e9a49f21763f5dc84a375e14").into(),
                },
                hex!("13b9571349f3624ca8027ceb742ac0582a3d27847b794f567c0e35dc551a8e3e1c791e8efdd146de4319a39089755754a3a3b08a4ab1d343576ed085b5c924f825f284dad24cddb3614e663b3b407af8d3ec55edad709dace9266996aa91466126eb14026de607692bb70f8f6750c6245a9491bba466245f49ee08fbdc57ed12096bcc416908750ce28317609680ca01b5731237d600162f790d0c7085a6b721022f966ae2f087062644fcd20024ac0641ca732388cf360ce8cc61ac0480c7cc26a09e5a8c2e1b728fd0a37e5532fcc44dcd389314a80e0fb191d148740e436a1e4b916c9862c7ccf9073bfcb3b5dd09a3903f619e79a7c04f89cc42619fe35a074ad1bbd03821f2622c67a1ab95486896592703a846dda6e6e3c2b6213aa4791fc58b6834c89cbea52b43c31ca8c4a44378f38d06d2baa04672f7006651c2431ed56b4cc18b0b0082d919813a0f0433942b8691ec70c6305705faef970ceef00ca817ffdf6c5bfa0eaf33951e6695bc537f8345cc8f03d9f234d44dec3ff8b4")
            ),
            Err(Error::InvalidProof)
        );
    }

    #[test]
    fn test_ok_968996_969001() {
        assert_eq!(
            verify_zkp(
                &ChainId::from_string("union-testnet-8").unwrap(),
                hex!("1deda64b1cc1319718f168b5aa8ed904b7d5b0ab932acdf6deae0ad9bd565a53").into(),
                &LightHeader {
                    height: 969001.try_into().unwrap(),
                    time: Timestamp::from_str("2024-06-18T13:20:56.784169335Z").unwrap(),
                    validators_hash: hex!("1deda64b1cc1319718f168b5aa8ed904b7d5b0ab932acdf6deae0ad9bd565a53").into(),
                    next_validators_hash: hex!("01a84dca649aa2df8de2f65a84c9092bbd5296b4bc54d818f844b28573d8e0be").into(),
                    app_hash: hex!("1818da4a8b1c430557a3018adc2bf9a06e56c3b530e5cce7709232e0f03bd9ab").into(),
                },
                hex!("086541c22b53d509d8369492d32683188f0b379950ea3c5da84aca2b331d911c163bc6e30c7610b6903832184d284399d140b316134202cfa53b695ed17db64e271a8ab10b015cc4562730180cc7af7d7509b64de00b5864ccef3ab6b5c187da1511c4af3392d5e4465cebeb3c92cad546ab6b5b7de08923ae756d4a49d972920ed4f1b33bde26016e753fe00e9ee8b37873e4df4696cce84baa34e444d6f9dc0021b25644dc22fd9414197dd9e094180eac33a5e6fc6d2e04e12df5baaae92815173080dedcafeb2789245e75f1c38ddaa4611273fa5eed1cb77f75aabace770186385a3a373190a9091147de95b3f11050152bc4376573ed454cfd703f1e7106edb33921b12717708fe03861534c812a5ea6c7e0ec428c02292f1e7dafb45901e8b29e0b18ba7cbfad2a7aef7db558f3eb49a943a379a03b1b976df912a0c329b66224da89f94e29c49b3c5070b86b23d9d23424246235088ea858a21340cc2d1120ac3dc25febd188abf16774ea49564f34bc769b6abd9295128c391dad18")
            ),
            Ok(())
        );
    }

    #[test]
    fn test_tampered_block() {
        assert_eq!(
            verify_zkp(
                &ChainId::from_string("union-devnet-1337").unwrap(),
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

    #[test]
    fn invalid_vk() {
        assert_eq!(
            verify_zkp(
                &ChainId::from_string("union-devnet-1").unwrap(),
                hex!("2f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500d").into(),
                &LightHeader {
                    height: 905.try_into().unwrap(),
                    time: Timestamp::from_str("2024-09-23T20:48:00.739712762Z").unwrap(),
                    validators_hash: hex!("2f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500d").into(),
                    next_validators_hash: hex!("2f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500d").into(),
                    app_hash: hex!("eddaa32275fbbf44c6a21e32b59b097bed5374be715eab22f093399a9700a1e4").into(),
                },
                hex!("1d530ee22263bc9e7008e3bd982c966b226d1018814e5b4d07597b4d35aea56b2ef63fdddb29fe06ef99cf645201a12e8b98b9ff7a7cec0819f696e17413294b0c638c4f946f4d4af4da8dd0815de2f5af8fd8612d1c98e9846846ea1ec78aac046df852b916de3fd8b3332bc3d23073e11b252b023711c18b19952507428da12e2baf74a03ca7bdc37edd0123e47f0a3a029f6da43a32dc6830e126b4ddf8712f2a0e021ac0f6414f171156f6a9019d6ea53cd30762c1e60d6a0e029778586c0cc1e2e13f7c45347a2a3ba82e43eccdc468fc8a05ba0a95fef26777872c27e42317f2c76c0a5f41e63088b8b394c5a7a3066809952f489718142107bd7b24572074be60bdb7611f1c916061a5ab3dc75a62b953a19650d839027a885801252a1e1cd84f8ba570047c2f1d220f26f7b11e69b7519f092d31ff954e92fd012a931ea2b4d20942376502043ba98e69f351f60b12e5a7ff180e5a1a966697d80696066694fa833420f5db7e3ae1b91dbce06fe2ffa1ea0a503af6a93f61ad7aa4f4")
            ),
            Err(Error::InvalidPok)
        );
    }
}
