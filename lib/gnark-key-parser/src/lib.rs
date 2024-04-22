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
        // let b64str = b"hwk883gUlTKCyXYA6XWZa8H9/xKIYZaJ0xEs0M5hQOMxiGpxocuX/8maSDmeCk3bhwk883gUlTKCyXYA6XWZa8H9/xKIYZaJ0xEs0M5hQOMxiGpxocuX/8maSDmeCk3bo5ViaDBdO7ZBxAhLSe5k/5TFQyF5Lv7KN2tLKnwgoWMqB16OL8WdbePIwTCuPtJNAFKoTZylLDbSf02kckMcZQDPF9iGh+JC99Pio74vDpwTEjUx5tQ99gNQwxULtztsqDRsPnEvKvLmsxHt8LQVBkEBm2PBJFY+OXf1MNW021viDBpR10mX4WQ6zrsGL5L0GY4cwf4tlbh+Obit+LnN/SQTnREf8fPpdKZ1sa/ui3pGi8lMT6io4D7Ujlwx2RdChwk883gUlTKCyXYA6XWZa8H9/xKIYZaJ0xEs0M5hQOMxiGpxocuX/8maSDmeCk3bkBF+isfMf77HCEGsZANw0hSrO2FGg14Sl26xLAIohdaW8O7gEaag8JdVAZ3OVLd5Df1NkZBEr753Xb8WwaXsJjE7qxwINL1KdqA4+EiYW4edb7+a9bbBeOPtb67ZxmFqAAAAAoMkzUv+KG8WoXszZI5NNMrbMLBDYP/xHunVgSWcix/kBrGlNozv1uFr0cmYZiij3YqToYs+EZa3dl2ILHx7H1n+b+Bjky/td2QduHVtf5t/Z9sKCfr+vOn12zVvOVz/6wAAAADAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

        let verifying_key = universal_vk();

        let file = include_bytes!("/home/aeryz/dev/union/union/vk.bin");
        let mut cursor = 0;
        let (n_bytes, g1_alpha) = parse_affine_g1(&file[..]).unwrap();
        assert_eq!(verifying_key.alpha_g1, g1_alpha);
        cursor += n_bytes;
        let (n_bytes, _g1_beta) = parse_affine_g1(&file[cursor..]).unwrap();
        cursor += n_bytes;
        let (n_bytes, g2_beta) = parse_affine_g2(&file[cursor..]).unwrap();
        cursor += n_bytes;
        let (n_bytes, g2_gamma) = parse_affine_g2(&file[cursor..]).unwrap();
        cursor += n_bytes;
        let (n_bytes, _g1_delta) = parse_affine_g1(&file[cursor..]).unwrap();
        cursor += n_bytes;
        let (n_bytes, g2_delta) = parse_affine_g2(&file[cursor..]).unwrap();
        cursor += n_bytes;
        let (_, g1_gamma_abc) = parse_affine_g1_array(&file[cursor..]).unwrap();

        assert_eq!(verifying_key.alpha_g1, g1_alpha);
        assert_eq!(verifying_key.beta_g2.x(), g2_beta.x());
        assert_eq!(verifying_key.beta_g2.y(), g2_beta.y());
        assert_eq!(verifying_key.gamma_g2.x(), g2_gamma.x());
        assert_eq!(verifying_key.gamma_g2.y(), g2_gamma.y());
        assert_eq!(verifying_key.delta_g2.x(), g2_delta.x());
        assert_eq!(verifying_key.delta_g2.y(), g2_delta.y());
        assert_eq!(verifying_key.gamma_abc_g1[0], g1_gamma_abc[0]);
        assert_eq!(verifying_key.gamma_abc_g1[1], g1_gamma_abc[1]);
        assert_eq!(verifying_key.gamma_abc_g1[2], g1_gamma_abc[2]);
    }
}
