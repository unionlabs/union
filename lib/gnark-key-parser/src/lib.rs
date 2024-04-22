use error::Error;
use substrate_bn::{
    arith::{self},
    AffineG1, AffineG2, Fq, Fq2, G1, G2,
};

mod error;

/// A verification key in the Groth16 SNARK.
pub struct VerifyingKey {
    /// The `alpha * G`, where `G` is the generator of `E::G1`.
    pub alpha_g1: AffineG1,
    /// The `alpha * H`, where `H` is the generator of `E::G2`.
    pub beta_g2: substrate_bn::AffineG2,
    /// The `gamma * H`, where `H` is the generator of `E::G2`.
    pub gamma_g2: substrate_bn::AffineG2,
    /// The `delta * H`, where `H` is the generator of `E::G2`.
    pub delta_g2: substrate_bn::AffineG2,
    /// The `gamma^{-1} * (beta * a_i + alpha * b_i + c_i) * H`, where `H` is the generator of `E::G1`.
    pub gamma_abc_g1: Vec<AffineG1>,
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
    pub fn parse(buf: &[u8]) -> Result<Self, Error> {
        let mut cursor = 0;
        let (n_bytes, alpha_g1) = parse_affine_g1(&buf[..]).unwrap();
        cursor += n_bytes;
        let (n_bytes, _g1_beta) = parse_affine_g1(&buf[cursor..]).unwrap();
        cursor += n_bytes;
        let (n_bytes, beta_g2) = parse_affine_g2(&buf[cursor..]).unwrap();
        cursor += n_bytes;
        let (n_bytes, gamma_g2) = parse_affine_g2(&buf[cursor..]).unwrap();
        cursor += n_bytes;
        let (n_bytes, _g1_delta) = parse_affine_g1(&buf[cursor..]).unwrap();
        cursor += n_bytes;
        let (n_bytes, delta_g2) = parse_affine_g2(&buf[cursor..]).unwrap();
        cursor += n_bytes;
        let (_, gamma_abc_g1) = parse_affine_g1_array(&buf[cursor..]).unwrap();

        Ok(Self {
            alpha_g1,
            beta_g2,
            gamma_g2,
            delta_g2,
            gamma_abc_g1,
        })
    }
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

    if metadata == UNCOMPRESSED {
        if buf.len() < G1_AFFINE_UNCOMPRESSED_SIZE {
            return Err(Error::ShortBuffer);
        }
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
    } else {
        if metadata == COMPRESSED_LARGEST {
            y = -y;
        }
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

    if metadata == UNCOMPRESSED {
        if buf.len() < G2_AFFINE_UNCOMPRESSED_SIZE {
            return Err(Error::ShortBuffer);
        }
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
    } else {
        if metadata == COMPRESSED_LARGEST {
            y = -y;
        }
    }

    Ok((G2_AFFINE_COMPRESSED_SIZE, AffineG2::new(x, y)?))
}

#[cfg(test)]
mod tests {
    use ark_ff::BigInt;

    use super::*;

    fn make_g1(x: BigInt<4>, y: BigInt<4>) -> substrate_bn::AffineG1 {
        substrate_bn::AffineG1::new(
            substrate_bn::Fq::from_u256(x.0.into()).unwrap(),
            substrate_bn::Fq::from_u256(y.0.into()).unwrap(),
        )
        .unwrap()
    }

    fn make_g2(
        x0: BigInt<4>,
        x1: BigInt<4>,
        y0: BigInt<4>,
        y1: BigInt<4>,
    ) -> substrate_bn::AffineG2 {
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

    pub fn universal_vk() -> VerifyingKey {
        VerifyingKey {
            alpha_g1: make_g1(
                BigInt!(
                    "4252850302693242182654534639730627324742305503909561446344356971523664816281"
                ),
                BigInt!(
                    "3971530409048238023625806606514600982127202826003358538821613170737831313919"
                ),
            ),
            beta_g2: make_g2(
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
            gamma_g2: make_g2(
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
            delta_g2: make_g2(
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
        }
    }
    #[test]
    fn it_works() {
        // vk.bin
        let file = hex::decode("8967072901cc7ab63357f1ddc4196c7c1feda50540d8026d7f6f0167c118a899d923def15f75234f2a6d53b566a2528441e98050b38803673e9179b834fc39a499355fd270b7601d5d88408b7e9e53d260512e2180cd260017dc941f2fc96d65153f0344c6bf2d8a891b979bc61d39a98fb11155fcd57418f30ea018ea842874a0e76be91a3148e2f8ef644222b3ce5b939a73bd2e0a40814f7f92a79c483acf2216bbe0c289e07936b4d9653b91521a24c570c808fa46dfd12ec4429e71b61999fcfb245459d63a4923b8f8c488d1e6af7ca358867b88eb0cdefe896c221f09e95e4c18d1e0475de4549b2547611d8301e1afff1047a6f5a288c9314af0b9fc05d403c8c91820a385a72c18d6a4962cef41a3ab93daa7ed289b1e95db4d04eb00000003e71843e52743864f4bb67ce94a2ce8fe82c8f61042c4c1ced8531d94305392818b0dbe71f4d60e02e9160ec2b015cae3a09cbe4f437226e2c02e1a5e5d124bcac29e93d5f47c0c7671350398ed8c40f5bc5c2f5b00363c7e2eb18a91a1c490c70000000100000000a57df6f8132cb0037f7dfdf1a29b04c1ff92ba082eda513996ba2bfa9fbd198713f0d8d8879885ca567ef99298c30c397e6fba584658f4127713a814c06de55aefbfe141a7555cf7e3e86b092660b81cfb68a025ad817e45cec0b0f2e2ca636802a104df1c015f2307fa2859627098cdf9fdb521d61d323943343a12304e5baf").unwrap();

        let verifying_key = universal_vk();

        let parsed_key = VerifyingKey::parse(&file[..]).unwrap();

        assert_eq!(verifying_key.alpha_g1, parsed_key.alpha_g1);
        assert_eq!(verifying_key.beta_g2.x(), parsed_key.beta_g2.x());
        assert_eq!(verifying_key.beta_g2.y(), parsed_key.beta_g2.y());
        assert_eq!(verifying_key.gamma_g2.x(), parsed_key.gamma_g2.x());
        assert_eq!(verifying_key.gamma_g2.y(), parsed_key.gamma_g2.y());
        assert_eq!(verifying_key.delta_g2.x(), parsed_key.delta_g2.x());
        assert_eq!(verifying_key.delta_g2.y(), parsed_key.delta_g2.y());
        assert_eq!(verifying_key.gamma_abc_g1, parsed_key.gamma_abc_g1);
    }

    #[test]
    fn build_script() {
        let file = hex::decode("8967072901cc7ab63357f1ddc4196c7c1feda50540d8026d7f6f0167c118a899d923def15f75234f2a6d53b566a2528441e98050b38803673e9179b834fc39a499355fd270b7601d5d88408b7e9e53d260512e2180cd260017dc941f2fc96d65153f0344c6bf2d8a891b979bc61d39a98fb11155fcd57418f30ea018ea842874a0e76be91a3148e2f8ef644222b3ce5b939a73bd2e0a40814f7f92a79c483acf2216bbe0c289e07936b4d9653b91521a24c570c808fa46dfd12ec4429e71b61999fcfb245459d63a4923b8f8c488d1e6af7ca358867b88eb0cdefe896c221f09e95e4c18d1e0475de4549b2547611d8301e1afff1047a6f5a288c9314af0b9fc05d403c8c91820a385a72c18d6a4962cef41a3ab93daa7ed289b1e95db4d04eb00000003e71843e52743864f4bb67ce94a2ce8fe82c8f61042c4c1ced8531d94305392818b0dbe71f4d60e02e9160ec2b015cae3a09cbe4f437226e2c02e1a5e5d124bcac29e93d5f47c0c7671350398ed8c40f5bc5c2f5b00363c7e2eb18a91a1c490c70000000100000000a57df6f8132cb0037f7dfdf1a29b04c1ff92ba082eda513996ba2bfa9fbd198713f0d8d8879885ca567ef99298c30c397e6fba584658f4127713a814c06de55aefbfe141a7555cf7e3e86b092660b81cfb68a025ad817e45cec0b0f2e2ca636802a104df1c015f2307fa2859627098cdf9fdb521d61d323943343a12304e5baf").unwrap();
        let parsed_key = VerifyingKey::parse(&file[..]).unwrap();
        let [alpha_g1_x0, alpha_g1_x1, alpha_g1_y0, alpha_g1_y1] =
            unsafe { std::mem::transmute::<AffineG1, [u128; 4]>(parsed_key.alpha_g1) };
        let [beta_g2_x00, beta_g2_x01, beta_g2_x10, beta_g2_x11, beta_g2_y00, beta_g2_y01, beta_g2_y10, beta_g2_y11] =
            unsafe { std::mem::transmute::<AffineG2, [u128; 8]>(parsed_key.beta_g2) };
        let [gamma_g2_x00, gamma_g2_x01, gamma_g2_x10, gamma_g2_x11, gamma_g2_y00, gamma_g2_y01, gamma_g2_y10, gamma_g2_y11] =
            unsafe { std::mem::transmute::<AffineG2, [u128; 8]>(parsed_key.gamma_g2) };
        let [delta_g2_x00, delta_g2_x01, delta_g2_x10, delta_g2_x11, delta_g2_y00, delta_g2_y01, delta_g2_y10, delta_g2_y11] =
            unsafe { std::mem::transmute::<AffineG2, [u128; 8]>(parsed_key.delta_g2) };

        let gamma_abc_size = parsed_key.gamma_abc_g1.len();
        let s: String = parsed_key
            .gamma_abc_g1
            .into_iter()
            .map(|g1| {
                let [g1_x0, g1_x1, g1_y0, g1_y1] =
                    unsafe { std::mem::transmute::<AffineG1, [u128; 4]>(g1) };
                format!(
                    r#"unsafe {{ core::mem::transmute::<[u128; 4], substrate_bn::AffineG1>
                        ([{g1_x0}, {g1_x1}, {g1_y0}, {g1_y1}]) }},
            "#
                )
            })
            .collect();

        let out = format!(
            r#"
                const ALPHA_G1: substrate_bn::AffineG1 = unsafe {{ core::mem::transmute::<[u128; 4], substrate_bn::AffineG1>([{alpha_g1_x0}, {alpha_g1_x1}, {alpha_g1_y0}, {alpha_g1_y1}]) }};
                const BETA_G2: substrate_bn::AffineG2 = unsafe {{ core::mem::transmute::<[u128; 8], substrate_bn::AffineG2>([{beta_g2_x00}, {beta_g2_x01}, {beta_g2_x10}, {beta_g2_x11}, {beta_g2_y00}, {beta_g2_y01}, {beta_g2_y10}, {beta_g2_y11}]) }};
                const GAMMA_G2: substrate_bn::AffineG2 = unsafe {{ core::mem::transmute::<[u128; 8], substrate_bn::AffineG2>([{gamma_g2_x00}, {gamma_g2_x01}, {gamma_g2_x10}, {gamma_g2_x11}, {gamma_g2_y00}, {gamma_g2_y01}, {gamma_g2_y10}, {gamma_g2_y11}]) }};
                const DELTA_G2: substrate_bn::AffineG2 = unsafe {{ core::mem::transmute::<[u128; 8], substrate_bn::AffineG2>([{delta_g2_x00}, {delta_g2_x01}, {delta_g2_x10}, {delta_g2_x11}, {delta_g2_y00}, {delta_g2_y01}, {delta_g2_y10}, {delta_g2_y11}]) }};
                const GAMMA_ABC_G1: [substrate_bn::AffineG1; {gamma_abc_size}] = [{s}];
            "#
        );

        println!("{out}");
    }
}
