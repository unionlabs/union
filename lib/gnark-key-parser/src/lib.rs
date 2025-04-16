use error::Error;
use substrate_bn::{
    arith::{self},
    AffineG1, AffineG2, Fq, Fq2, G1, G2,
};

mod error;

pub struct PedersenVerifyingKey {
    pub g: G2,
    pub g_root_sigma_neg: G2,
}

/// A verification key in the Groth16 SNARK.
pub struct VerifyingKey {
    /// The `alpha * G`, where `G` is the generator of `E::G1`.
    pub alpha_g1: G1,
    /// The `alpha * H`, where `H` is the generator of `E::G2`.
    pub beta_neg_g2: G2,
    /// The `gamma * H`, where `H` is the generator of `E::G2`.
    pub gamma_neg_g2: G2,
    /// The `delta * H`, where `H` is the generator of `E::G2`.
    pub delta_neg_g2: G2,
    /// The `gamma^{-1} * (beta * a_i + alpha * b_i + c_i) * H`, where `H` is the generator of `E::G1`.
    pub gamma_abc_g1: Vec<G1>,
    pub public_and_commitment_committed: Vec<Vec<u64>>,
    pub commitment_key: PedersenVerifyingKey,
}

const MASK: u8 = 0b11 << 6;
const UNCOMPRESSED: u8 = 0b00 << 6;
const COMPRESSED_INFINITY: u8 = 0b01 << 6;
const COMPRESSED_SMALLEST: u8 = 0b10 << 6;
const COMPRESSED_LARGEST: u8 = 0b11 << 6;

const G1_AFFINE_COMPRESSED_SIZE: usize = 32;
const G1_AFFINE_UNCOMPRESSED_SIZE: usize = G1_AFFINE_COMPRESSED_SIZE * 2;

const G2_AFFINE_COMPRESSED_SIZE: usize = 32 * 2;
const G2_AFFINE_UNCOMPRESSED_SIZE: usize = G2_AFFINE_COMPRESSED_SIZE * 2;

impl VerifyingKey {
    pub fn parse(buf: &[u8]) -> Result<(usize, Self), Error> {
        let mut cursor = 0;
        let (n_bytes, alpha_g1) = parse_affine_g1(buf)?;
        cursor += n_bytes;
        let (n_bytes, _g1_beta) = parse_affine_g1(&buf[cursor..])?;
        cursor += n_bytes;
        let (n_bytes, beta_g2) = parse_affine_g2(&buf[cursor..])?;
        cursor += n_bytes;
        let (n_bytes, gamma_g2) = parse_affine_g2(&buf[cursor..])?;
        cursor += n_bytes;
        let (n_bytes, _g1_delta) = parse_affine_g1(&buf[cursor..])?;
        cursor += n_bytes;
        let (n_bytes, delta_g2) = parse_affine_g2(&buf[cursor..])?;
        cursor += n_bytes;
        let (n_bytes, gamma_abc_g1) = parse_affine_g1_array(&buf[cursor..])?;
        cursor += n_bytes;
        let (n_bytes, public_and_commitment_committed) = parse_uint64_slice_slice(&buf[cursor..])?;
        cursor += n_bytes;
        let (n_bytes, g) = parse_affine_g2(&buf[cursor..])?;
        cursor += n_bytes;
        let (n_bytes, g_root_sigma_neg) = parse_affine_g2(&buf[cursor..])?;
        cursor += n_bytes;

        Ok((
            cursor,
            Self {
                alpha_g1: alpha_g1.into(),
                beta_neg_g2: -G2::from(beta_g2),
                gamma_neg_g2: -G2::from(gamma_g2),
                delta_neg_g2: -G2::from(delta_g2),
                gamma_abc_g1: gamma_abc_g1.into_iter().map(Into::into).collect(),
                public_and_commitment_committed,
                commitment_key: PedersenVerifyingKey {
                    g: g.into(),
                    g_root_sigma_neg: g_root_sigma_neg.into(),
                },
            },
        ))
    }
}

pub fn parse_uint64_slice(buf: &[u8]) -> Result<(usize, Vec<u64>), Error> {
    let size = u32::from_be_bytes((&buf[0..4]).try_into().expect("impossible"));
    let mut items = Vec::new();
    let mut cursor = 4;
    for _ in 0..size {
        items.push(u64::from_be_bytes(
            (&buf[cursor..cursor + 8]).try_into().expect("impossible"),
        ));
        cursor += 8;
    }
    Ok((cursor, items))
}

pub fn parse_uint64_slice_slice(buf: &[u8]) -> Result<(usize, Vec<Vec<u64>>), Error> {
    let size = u32::from_be_bytes((&buf[0..4]).try_into().expect("impossible"));
    let mut items = Vec::new();
    let mut cursor = 4;
    for _ in 0..size {
        let (cur_read, value) = parse_uint64_slice(&buf[cursor..])?;
        cursor += cur_read;
        items.push(value);
    }
    Ok((cursor, items))
}

pub fn parse_affine_g1_array(buf: &[u8]) -> Result<(usize, Vec<AffineG1>), Error> {
    let size = u32::from_be_bytes((&buf[0..4]).try_into().expect("impossible"));
    let mut g1s = Vec::new();
    let mut n_read = 4;
    for _ in 0..size {
        let (cur_read, g1) = parse_affine_g1(&buf[n_read..])?;
        n_read += cur_read;
        g1s.push(g1);
    }

    Ok((n_read, g1s))
}

fn is_zeroed(first_byte: u8, rest: &[u8]) -> bool {
    first_byte == 0 && rest.iter().all(|x| *x == 0)
}

/// LexicographicallyLargest returns true if this element is strictly lexicographically
/// larger than its negation, false otherwise
/// See [in gnark](https://github.com/Consensys/gnark-crypto/blob/v0.12.1/ecc/bn254/fp/element.go#L290)
fn g1_lexicographically_largest(z: &Fq) -> bool {
    // z > (q-1) / 2
    z.into_u256()
        > arith::U256::from([
            11389680472494603939,
            14681934109093717318,
            15863968012492123182,
            1743499133401485332,
        ])
}

// LexicographicallyLargest returns true if this element is strictly lexicographically
// larger than its negation, false otherwise
/// See [in gnark](https://github.com/Consensys/gnark-crypto/blob/v0.12.1/ecc/bn254/internal/fptower/e2.go#L57)
fn g2_lexicographically_largest(z: &Fq2) -> bool {
    if z.real().is_zero() {
        g1_lexicographically_largest(&z.real())
    } else {
        g1_lexicographically_largest(&z.imaginary())
    }
}

/// Parse G1 element
///
/// [See in gnark](https://github.com/Consensys/gnark-crypto/blob/v0.12.1/ecc/bn254/marshal.go#L807)
pub fn parse_affine_g1(buf: &[u8]) -> Result<(usize, AffineG1), Error> {
    if buf.len() < G1_AFFINE_COMPRESSED_SIZE {
        return Err(Error::ShortBuffer);
    }

    let metadata = buf[0] & MASK;

    if metadata == UNCOMPRESSED && buf.len() < G1_AFFINE_UNCOMPRESSED_SIZE {
        return Err(Error::ShortBuffer);
    }

    if metadata == COMPRESSED_INFINITY {
        if !is_zeroed(buf[0], &buf[1..32]) {
            return Err(Error::InvalidInfinityEncoding);
        }

        return Ok((
            G1_AFFINE_COMPRESSED_SIZE,
            AffineG1::new(Fq::zero(), Fq::zero())?,
        ));
    }

    if metadata == UNCOMPRESSED {
        let x = Fq::from_slice(&buf[..32])?;
        let y = Fq::from_slice(&buf[32..64])?;

        return Ok((G1_AFFINE_UNCOMPRESSED_SIZE, AffineG1::new(x, y)?));
    }

    let mut buf_x: [u8; 32] = [0; 32];
    buf_x.copy_from_slice(&buf[..32]);
    buf_x[0] &= !MASK;

    let x = Fq::from_slice(&buf_x[..32])?;

    let y = x * x * x + G1::b();

    let Some(mut y) = y.sqrt() else {
        return Err(Error::NoSquareRoot);
    };

    if g1_lexicographically_largest(&y) {
        if metadata == COMPRESSED_SMALLEST {
            y = -y;
        }
    } else if metadata == COMPRESSED_LARGEST {
        y = -y;
    }

    let g1 = AffineG1::new(x, y)?;

    Ok((G1_AFFINE_COMPRESSED_SIZE, g1))
}

/// Parse G2 element
///
/// [See in gnark](https://github.com/Consensys/gnark-crypto/blob/v0.12.1/ecc/bn254/marshal.go#L1063)
pub fn parse_affine_g2(buf: &[u8]) -> Result<(usize, AffineG2), Error> {
    if buf.len() < G2_AFFINE_COMPRESSED_SIZE {
        return Err(Error::ShortBuffer);
    }

    let metadata = buf[0] & MASK;

    if metadata == UNCOMPRESSED && buf.len() < G2_AFFINE_UNCOMPRESSED_SIZE {
        return Err(Error::ShortBuffer);
    }

    if metadata == COMPRESSED_INFINITY {
        if !is_zeroed(buf[0] & !MASK, &buf[1..G2_AFFINE_COMPRESSED_SIZE]) {
            return Err(Error::InvalidInfinityEncoding);
        }
        return Ok((
            G2_AFFINE_COMPRESSED_SIZE,
            AffineG2::new(Fq2::zero(), Fq2::zero())?,
        ));
    }

    if metadata == UNCOMPRESSED {
        let x_1 = Fq::from_slice(&buf[..32])?;
        let x_0 = Fq::from_slice(&buf[32..64])?;

        let y_1 = Fq::from_slice(&buf[64..96])?;
        let y_0 = Fq::from_slice(&buf[96..128])?;

        return Ok((
            G2_AFFINE_UNCOMPRESSED_SIZE,
            AffineG2::new(Fq2::new(x_0, x_1), Fq2::new(y_0, y_1))?,
        ));
    }

    let mut buf_x: [u8; 32] = [0; 32];
    buf_x.copy_from_slice(&buf[..32]);

    buf_x[0] &= !MASK;
    let x_1 = Fq::from_slice(&buf_x[..32])?;
    let x_0 = Fq::from_slice(&buf[32..64])?;

    let x = Fq2::new(x_0, x_1);

    let y_squared = (x * x * x) + G2::b();

    let Some(mut y) = y_squared.sqrt() else {
        return Err(Error::NoSquareRoot);
    };

    if g2_lexicographically_largest(&y) {
        if metadata == COMPRESSED_SMALLEST {
            y = -y;
        }
    } else if metadata == COMPRESSED_LARGEST {
        y = -y;
    }

    Ok((G2_AFFINE_COMPRESSED_SIZE, AffineG2::new(x, y)?))
}

#[cfg(test)]
mod tests {
    use ark_ff::BigInt;
    use ark_serialize::CanonicalSerialize;
    use num_bigint::BigUint;

    use super::*;

    fn make_g1(x: BigInt<4>, y: BigInt<4>) -> substrate_bn::G1 {
        substrate_bn::AffineG1::new(
            substrate_bn::Fq::from_u256(x.0.into()).unwrap(),
            substrate_bn::Fq::from_u256(y.0.into()).unwrap(),
        )
        .unwrap()
        .into()
    }

    fn make_g2(x0: BigInt<4>, x1: BigInt<4>, y0: BigInt<4>, y1: BigInt<4>) -> substrate_bn::G2 {
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
        .into()
    }

    pub fn universal_vk() -> VerifyingKey {
        const PEDERSEN_G: substrate_bn::AffineG2 = unsafe {
            core::mem::transmute::<[u128; 8], substrate_bn::AffineG2>([
                11811635544135052229933151055424244648,
                38109979931269619311736752979166931278,
                90391742616114872771404285750669801592,
                52127860733344004379550038853653983369,
                94283382067525625571866539274932825678,
                46211394867610833270624710233881181494,
                152610685216587622477368049102478160737,
                13810267963617699865883949649491963230,
            ])
        };
        const PEDERSEN_G_ROOT_SIGMA_NEG: substrate_bn::AffineG2 = unsafe {
            core::mem::transmute::<[u128; 8], substrate_bn::AffineG2>([
                72813077000167954255887915103454700534,
                12589073809616840933661747092291565935,
                337926906408442213219854203224585082877,
                59919365646951351584569269731816365136,
                12528477130055848686709301753705522532,
                49609059301297358295345786701940661278,
                57099380008375147203174978222963579184,
                6628981065425146778130938788258647964,
            ])
        };

        VerifyingKey {
            alpha_g1: make_g1(
                BigInt!(
                    "4252850302693242182654534639730627324742305503909561446344356971523664816281"
                ),
                BigInt!(
                    "3971530409048238023625806606514600982127202826003358538821613170737831313919"
                ),
            ),
            beta_neg_g2: -make_g2(
                BigInt!(
                    "9609903744775525881338738176064678545439912439219033822736570321349357348980"
                ),
                BigInt!(
                    "11402125448377072234752634956069960846261435348550776006069399216352815312229"
                ),
                BigInt!(
                    "3876014193556985028076276590285094449745398487447250532380698384573245200038"
                ),
                BigInt!(
                    "6131692356384648492800758325058748831519318785594820705365176509549681793745"
                ),
            ),
            gamma_neg_g2: -make_g2(
                BigInt!(
                    "15418804173338388766896385877623893969695670309009587476846726795628238714393"
                ),
                BigInt!(
                    "14882897597913405382982164467298010752166363844685258881581520272046793702095"
                ),
                BigInt!(
                    "4166025151148225057462107057100265181139888889391061071239248954005945470477"
                ),
                BigInt!(
                    "206728492847877950288262169260916452585500374823256459470367014125967964118"
                ),
            ),
            delta_neg_g2: -make_g2(
                BigInt!(
                    "2636161939055419322743684458857549714230849256995406138405588958157843793131"
                ),
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
            public_and_commitment_committed: Vec::new(),
            commitment_key: PedersenVerifyingKey {
                g: PEDERSEN_G.into(),
                g_root_sigma_neg: PEDERSEN_G_ROOT_SIGMA_NEG.into(),
            },
        }
    }

    #[test]
    fn it_works() {
        // vk.bin
        let file = hex::decode("8967072901cc7ab63357f1ddc4196c7c1feda50540d8026d7f6f0167c118a899d923def15f75234f2a6d53b566a2528441e98050b38803673e9179b834fc39a499355fd270b7601d5d88408b7e9e53d260512e2180cd260017dc941f2fc96d65153f0344c6bf2d8a891b979bc61d39a98fb11155fcd57418f30ea018ea842874a0e76be91a3148e2f8ef644222b3ce5b939a73bd2e0a40814f7f92a79c483acf2216bbe0c289e07936b4d9653b91521a24c570c808fa46dfd12ec4429e71b61999fcfb245459d63a4923b8f8c488d1e6af7ca358867b88eb0cdefe896c221f09e95e4c18d1e0475de4549b2547611d8301e1afff1047a6f5a288c9314af0b9fc05d403c8c91820a385a72c18d6a4962cef41a3ab93daa7ed289b1e95db4d04eb00000003e71843e52743864f4bb67ce94a2ce8fe82c8f61042c4c1ced8531d94305392818b0dbe71f4d60e02e9160ec2b015cae3a09cbe4f437226e2c02e1a5e5d124bcac29e93d5f47c0c7671350398ed8c40f5bc5c2f5b00363c7e2eb18a91a1c490c70000000100000000a57df6f8132cb0037f7dfdf1a29b04c1ff92ba082eda513996ba2bfa9fbd198713f0d8d8879885ca567ef99298c30c397e6fba584658f4127713a814c06de55aefbfe141a7555cf7e3e86b092660b81cfb68a025ad817e45cec0b0f2e2ca636802a104df1c015f2307fa2859627098cdf9fdb521d61d323943343a12304e5baf").unwrap();

        let verifying_key = universal_vk();

        let (n_read, parsed_key) = VerifyingKey::parse(&file[..]).unwrap();

        assert_eq!(n_read, file.len());

        assert_eq!(verifying_key.alpha_g1, parsed_key.alpha_g1);
        assert_eq!(verifying_key.beta_neg_g2.x(), parsed_key.beta_neg_g2.x());
        assert_eq!(verifying_key.beta_neg_g2.y(), parsed_key.beta_neg_g2.y());
        assert_eq!(verifying_key.gamma_neg_g2.x(), parsed_key.gamma_neg_g2.x());
        assert_eq!(verifying_key.gamma_neg_g2.y(), parsed_key.gamma_neg_g2.y());
        assert_eq!(verifying_key.delta_neg_g2.x(), parsed_key.delta_neg_g2.x());
        assert_eq!(verifying_key.delta_neg_g2.y(), parsed_key.delta_neg_g2.y());
        assert_eq!(verifying_key.gamma_abc_g1, parsed_key.gamma_abc_g1);
        assert_eq!(
            verifying_key.commitment_key.g.x(),
            parsed_key.commitment_key.g.x()
        );
        assert_eq!(
            verifying_key.commitment_key.g.y(),
            parsed_key.commitment_key.g.y()
        );
        assert_eq!(
            verifying_key.commitment_key.g_root_sigma_neg.x(),
            parsed_key.commitment_key.g_root_sigma_neg.x()
        );
        assert_eq!(
            verifying_key.commitment_key.g_root_sigma_neg.y(),
            parsed_key.commitment_key.g_root_sigma_neg.y()
        );
    }

    #[test]
    fn dump_aptos() {
        let file = hex::decode("e45229d9b076b3c0e8a4d70bde8c1cccffa08a9fae7557b165b3b0dbd653e2c7a3eba1776012a292e6780582e7a197913f375a4043e411358d4791b66f53c1d987090a82e8fabbd39299be24705b92cf208ee8b3487f6f2b39ff27978a29a1db2424bcc1f60a5472685fd50705b2809626e170120acaf441e133a2bd5e61d244998e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6edc3b40f6624e48a7a1d438b5f8f04347ddcaf3deacded6e5c1093e843d6c89a9887b8dbefa90bde075a26318e5066db729155514e3c06b888d4e03c56d82c97e602aca5d2a73f8d34e4b26eee3932365e6526c8d5e2f3347d679c2cb1867104dc00000003af5d8a3817f21d3e453573c90c3cc47b7ff235fad7bdfbd59bbd6ae5d153273eea81b98e1c997bd01a20893a08a46c6804493e838c1a0ff6c8c069ef5ab66b9a979496ce140df89ce35c5ee7fb496efdffda5e5d3b95ff9116e2e5df96b36ab70000000100000000998e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6edc7b8dbefa90bde075a26318e5066db729155514e3c06b888d4e03c56d82c97e602aca5d2a73f8d34e4b26eee3932365e6526c8d5e2f3347d679c2cb1867104dc").unwrap();
        let (_, parsed_key) = VerifyingKey::parse(&file[..]).unwrap();

        let parse_g1 = |g1: &G1| -> String {
            let mut g1x = [0u8; 32];
            let mut g1y = [0u8; 32];
            g1.x().to_big_endian(&mut g1x).unwrap();
            g1.y().to_big_endian(&mut g1y).unwrap();

            let mut out = Vec::new();
            ark_bn254::G1Affine::new(
                ark_bn254::Fq::from(BigUint::from_bytes_be(&g1x)),
                ark_bn254::Fq::from(BigUint::from_bytes_be(&g1y)),
            )
            .serialize_uncompressed(&mut out)
            .unwrap();

            hex::encode(out)
        };

        let print_g1 = |key: &str, g1: &G1| {
            println!("const {key}: vector<u8> = x\"{}\";", parse_g1(g1));
        };

        let print_g2 = |key: &str, g2: &G2| {
            let mut g2x1 = [0u8; 32];
            let mut g2x2 = [0u8; 32];
            let mut g2y1 = [0u8; 32];
            let mut g2y2 = [0u8; 32];
            let mut out = Vec::new();

            g2.x().real().to_big_endian(&mut g2x1).unwrap();
            g2.x().imaginary().to_big_endian(&mut g2x2).unwrap();
            g2.y().real().to_big_endian(&mut g2y1).unwrap();
            g2.y().imaginary().to_big_endian(&mut g2y2).unwrap();

            ark_bn254::G2Affine::new(
                ark_bn254::Fq2::new(
                    ark_bn254::Fq::from(BigUint::from_bytes_be(&g2x1)),
                    ark_bn254::Fq::from(BigUint::from_bytes_be(&g2x2)),
                ),
                ark_bn254::Fq2::new(
                    ark_bn254::Fq::from(BigUint::from_bytes_be(&g2y1)),
                    ark_bn254::Fq::from(BigUint::from_bytes_be(&g2y2)),
                ),
            )
            .serialize_compressed(&mut out)
            .unwrap();
            println!("const {key}: vector<u8> = x\"{}\";", hex::encode(out));
        };

        print_g1("ALPHA_G1", &parsed_key.alpha_g1);
        print_g2("BETA_G2", &parsed_key.beta_neg_g2);
        print_g2("GAMMA_G2", &parsed_key.gamma_neg_g2);
        print_g2("DELTA_G2", &parsed_key.delta_neg_g2);
        print_g2("PEDERSEN_G", &parsed_key.commitment_key.g);
        print_g2(
            "PEDERSEN_G_ROOT_SIGMA_NEG",
            &parsed_key.commitment_key.g_root_sigma_neg,
        );

        println!("const GAMMA_ABC_G1: vector<vector<u8>> = vector[");
        parsed_key.gamma_abc_g1.into_iter().for_each(|g1| {
            println!("\tx\"{}\",", parse_g1(&g1));
        });
        println!("];");
    }

    #[test]
    fn dump_evm() {
        let file = hex::decode("e45229d9b076b3c0e8a4d70bde8c1cccffa08a9fae7557b165b3b0dbd653e2c7a3eba1776012a292e6780582e7a197913f375a4043e411358d4791b66f53c1d987090a82e8fabbd39299be24705b92cf208ee8b3487f6f2b39ff27978a29a1db2424bcc1f60a5472685fd50705b2809626e170120acaf441e133a2bd5e61d244998e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6edc3b40f6624e48a7a1d438b5f8f04347ddcaf3deacded6e5c1093e843d6c89a9887b8dbefa90bde075a26318e5066db729155514e3c06b888d4e03c56d82c97e602aca5d2a73f8d34e4b26eee3932365e6526c8d5e2f3347d679c2cb1867104dc00000003af5d8a3817f21d3e453573c90c3cc47b7ff235fad7bdfbd59bbd6ae5d153273eea81b98e1c997bd01a20893a08a46c6804493e838c1a0ff6c8c069ef5ab66b9a979496ce140df89ce35c5ee7fb496efdffda5e5d3b95ff9116e2e5df96b36ab70000000100000000998e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6edc7b8dbefa90bde075a26318e5066db729155514e3c06b888d4e03c56d82c97e602aca5d2a73f8d34e4b26eee3932365e6526c8d5e2f3347d679c2cb1867104dc").unwrap();

        let (_, parsed_key) = VerifyingKey::parse(&file[..]).unwrap();

        let mut buffer = [0u8; 32];

        parsed_key.alpha_g1.x().to_big_endian(&mut buffer).unwrap();
        println!("uint256 constant ALPHA_X = 0x{};", hex::encode(buffer));
        parsed_key.alpha_g1.y().to_big_endian(&mut buffer).unwrap();
        println!("uint256 constant ALPHA_Y = 0x{};", hex::encode(buffer));

        parsed_key
            .beta_neg_g2
            .x()
            .real()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!("uint256 constant BETA_NEG_X_0 = 0x{};", hex::encode(buffer));
        parsed_key
            .beta_neg_g2
            .x()
            .imaginary()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!("uint256 constant BETA_NEG_X_1 = 0x{};", hex::encode(buffer));
        parsed_key
            .beta_neg_g2
            .y()
            .real()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!("uint256 constant BETA_NEG_Y_0 = 0x{};", hex::encode(buffer));
        parsed_key
            .beta_neg_g2
            .y()
            .imaginary()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!("uint256 constant BETA_NEG_Y_1 = 0x{};", hex::encode(buffer));

        parsed_key
            .gamma_neg_g2
            .x()
            .real()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!(
            "uint256 constant GAMMA_NEG_X_0 = 0x{};",
            hex::encode(buffer)
        );
        parsed_key
            .gamma_neg_g2
            .x()
            .imaginary()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!(
            "uint256 constant GAMMA_NEG_X_1 = 0x{};",
            hex::encode(buffer)
        );
        parsed_key
            .gamma_neg_g2
            .y()
            .real()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!(
            "uint256 constant GAMMA_NEG_Y_0 = 0x{};",
            hex::encode(buffer)
        );
        parsed_key
            .gamma_neg_g2
            .y()
            .imaginary()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!(
            "uint256 constant GAMMA_NEG_Y_1 = 0x{};",
            hex::encode(buffer)
        );

        parsed_key
            .delta_neg_g2
            .x()
            .real()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!(
            "uint256 constant DELTA_NEG_X_0 = 0x{};",
            hex::encode(buffer)
        );
        parsed_key
            .delta_neg_g2
            .x()
            .imaginary()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!(
            "uint256 constant DELTA_NEG_X_1 = 0x{};",
            hex::encode(buffer)
        );
        parsed_key
            .delta_neg_g2
            .y()
            .real()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!(
            "uint256 constant DELTA_NEG_Y_0 = 0x{};",
            hex::encode(buffer)
        );
        parsed_key
            .delta_neg_g2
            .y()
            .imaginary()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!(
            "uint256 constant DELTA_NEG_Y_1 = 0x{};",
            hex::encode(buffer)
        );

        parsed_key.gamma_abc_g1[0]
            .x()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!("uint256 constant CONSTANT_X = 0x{};", hex::encode(buffer));
        parsed_key.gamma_abc_g1[0]
            .y()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!("uint256 constant CONSTANT_Y = 0x{};", hex::encode(buffer));

        for (i, public) in parsed_key.gamma_abc_g1.into_iter().skip(1).enumerate() {
            public.x().to_big_endian(&mut buffer).unwrap();
            println!("uint256 constant PUB_{}_X = 0x{};", i, hex::encode(buffer));
            public.y().to_big_endian(&mut buffer).unwrap();
            println!("uint256 constant PUB_{}_Y = 0x{};", i, hex::encode(buffer));
        }

        parsed_key
            .commitment_key
            .g
            .x()
            .real()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!(
            "uint256 constant PEDERSEN_G_X_0 = 0x{};",
            hex::encode(buffer)
        );
        parsed_key
            .commitment_key
            .g
            .x()
            .imaginary()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!(
            "uint256 constant PEDERSEN_G_X_1 = 0x{};",
            hex::encode(buffer)
        );
        parsed_key
            .commitment_key
            .g
            .y()
            .real()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!(
            "uint256 constant PEDERSEN_G_Y_0 = 0x{};",
            hex::encode(buffer)
        );
        parsed_key
            .commitment_key
            .g
            .y()
            .imaginary()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!(
            "uint256 constant PEDERSEN_G_Y_1 = 0x{};",
            hex::encode(buffer)
        );

        parsed_key
            .commitment_key
            .g_root_sigma_neg
            .x()
            .real()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!(
            "uint256 constant PEDERSEN_G_ROOT_SIGMA_NEG_X_0 = 0x{};",
            hex::encode(buffer)
        );
        parsed_key
            .commitment_key
            .g_root_sigma_neg
            .x()
            .imaginary()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!(
            "uint256 constant PEDERSEN_G_ROOT_SIGMA_NEG_X_1 = 0x{};",
            hex::encode(buffer)
        );
        parsed_key
            .commitment_key
            .g_root_sigma_neg
            .y()
            .real()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!(
            "uint256 constant PEDERSEN_G_ROOT_SIGMA_NEG_Y_0 = 0x{};",
            hex::encode(buffer)
        );
        parsed_key
            .commitment_key
            .g_root_sigma_neg
            .y()
            .imaginary()
            .to_big_endian(&mut buffer)
            .unwrap();
        println!(
            "uint256 constant PEDERSEN_G_ROOT_SIGMA_NEG_Y_1 = 0x{};",
            hex::encode(buffer)
        );
    }
}
