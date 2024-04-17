use substrate_bn::{
    arith::{U256, U512},
    AffineG1, AffineG2, Fq, Fq2, G1, G2,
};

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

// TODO(aeryz): enum?
const MASK: u8 = 0b11 << 6;
const UNCOMPRESSED: u8 = 0b00 << 6;
const COMPRESSED_INFINITY: u8 = 0b01 << 6;
const COMPRESSED_SMALLEST: u8 = 0b10 << 6;
const COMPRESSED_LARGEST: u8 = 0b11 << 6;

const G1_AFFINE_COMPRESSED_SIZE: usize = 32;
const G1_AFFINE_UNCOMPRESSED_SIZE: usize = G1_AFFINE_COMPRESSED_SIZE * 2;

const G2_AFFINE_COMPRESSED_SIZE: usize = 32 * 2;
const G2_AFFINE_UNCOMPRESSED_SIZE: usize = G2_AFFINE_COMPRESSED_SIZE * 2;

pub fn decode_g1_affine(data: &[u8]) -> AffineG1 {
    if data.len() < 32 {
        panic!("cannot parse");
    }

    // if data[0] & MASK != UNCOMPRESSED {
    //     let g1 = AffineG1::new(Fq::zero(), Fq::zero()).unwrap();
    //     Fq::from_slice()
    // } else {
    //     todo!()
    // }
    todo!()
}

pub fn parse_affine_g1_array(buf: &[u8]) -> (usize, Vec<AffineG1>) {
    let size = u32::from_be_bytes((&buf[0..4]).try_into().unwrap());
    let mut g1s = Vec::new();
    let mut n_read = 4;
    for _ in 0..size {
        println!("parsing");
        let (cur_read, g1) = affine_g1_set_bytes(&buf[n_read..]);
        n_read += cur_read;
        g1s.push(g1);
    }

    (n_read, g1s)
}

pub fn affine_g1_set_bytes(buf: &[u8]) -> (usize, AffineG1) {
    if buf.len() < G1_AFFINE_COMPRESSED_SIZE {
        panic!("");
    }

    let metadata = buf[0] & MASK;

    if metadata == UNCOMPRESSED {
        if buf.len() < G1_AFFINE_UNCOMPRESSED_SIZE {
            panic!("short");
        }
    }

    if metadata == COMPRESSED_INFINITY {
        println!("here2");
        // TODO(aeryz): compressed infinity
    }

    if metadata == UNCOMPRESSED {
        let x = Fq::from_slice(&buf[..32]).unwrap();
        let y = Fq::from_slice(&buf[32..64]).unwrap();

        return (64, AffineG1::new(x, y).unwrap());
    }

    let mut buf_x: [u8; 33] = [0; 33];
    buf_x[2..33].copy_from_slice(&buf[1..32]);
    buf_x[1] = buf[0] & !MASK;

    println!("right before");
    buf_x[0] = if metadata == COMPRESSED_LARGEST {
        2
    } else if metadata == COMPRESSED_SMALLEST {
        3
    } else {
        panic!("invalid encoding");
    };

    let g1 = G1::from_compressed(&buf_x[..]).unwrap();

    (32, AffineG1::from_jacobian(g1).unwrap())
}

pub fn affine_g2_set_bytes(buf: &[u8]) -> (usize, AffineG2) {
    if buf.len() < G2_AFFINE_COMPRESSED_SIZE {
        panic!("");
    }

    let metadata = buf[0] & MASK;

    if metadata == UNCOMPRESSED {
        if buf.len() < G2_AFFINE_UNCOMPRESSED_SIZE {
            panic!("short");
        }
    }

    if metadata == COMPRESSED_INFINITY {
        println!("here2");
        // TODO(aeryz): compressed infinity
    }

    if metadata == UNCOMPRESSED {
        let x_1 = Fq::from_slice(&buf[..32]).unwrap();
        let x_0 = Fq::from_slice(&buf[32..64]).unwrap();

        let y_1 = Fq::from_slice(&buf[64..96]).unwrap();
        let y_0 = Fq::from_slice(&buf[96..128]).unwrap();

        return (
            128,
            AffineG2::new(Fq2::new(x_0, x_1), Fq2::new(y_0, y_1)).unwrap(),
        );
    }

    // let mut buf_x: [u8; 65] = [0; 65];
    // buf_x[2..65].copy_from_slice(&buf[1..64]);
    // buf_x[1] = buf[0] & !MASK;

    // buf_x[0] = if metadata == COMPRESSED_LARGEST {
    //     11
    // } else if metadata == COMPRESSED_SMALLEST {
    //     10
    // } else {
    //     panic!("invalid encoding");
    // };

    // let g2 = G2::from_compressed(&buf_x[..]).unwrap();

    // (64, AffineG2::from_jacobian(g2).unwrap())

    let mut buf_x: [u8; 32] = [0; 32];
    buf_x.copy_from_slice(&buf[..32]);

    buf_x[0] &= !MASK;
    let x_1 = Fq::from_slice(&buf_x[..32]).unwrap();
    let x_0 = Fq::from_slice(&buf[32..64]).unwrap();

    let x = Fq2::new(x_0, x_1);

    let y_squared = (x * x * x) + G2::b();

    if let Some(mut y) = y_squared.sqrt() {
        let c0: U256 = (y.real()).into_u256();
        let c1: U256 = (y.imaginary()).into_u256();
        let lhs = U512::new(&c1, &c0, &Fq::modulus());

        let y_neg = -y;

        let c0: U256 = (y_neg.real()).into_u256();
        let c1: U256 = (y_neg.imaginary()).into_u256();
        let rhs = U512::new(&c1, &c0, &Fq::modulus());

        let y_gt = lhs > rhs;

        if metadata == COMPRESSED_LARGEST {
            if !y_gt {
                y = y_neg;
            }
        } else if metadata == COMPRESSED_SMALLEST {
            if y_gt {
                y = y_neg;
            }
        } else {
            panic!("invalid encoding");
        }

        // TODO(aeryz): smallest largest
        return (64, AffineG2::new(x, y).unwrap());
    } else {
        panic!("no square")
    }
}

#[cfg(test)]
mod tests {
    use ark_ff::BigInt;
    use substrate_bn::G1;

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

        let file = include_bytes!("/home/aeryz/dev/union/union/vk.bin");
        let mut cursor = 0;
        let (n_bytes, g1_alpha) = affine_g1_set_bytes(&file[..]);
        cursor += n_bytes;
        let (n_bytes, _g1_beta) = affine_g1_set_bytes(&file[cursor..]);
        cursor += n_bytes;
        let (n_bytes, g2_beta) = affine_g2_set_bytes(&file[cursor..]);
        cursor += n_bytes;
        let (n_bytes, g2_gamma) = affine_g2_set_bytes(&file[cursor..]);
        cursor += n_bytes;
        let (n_bytes, _g1_delta) = affine_g1_set_bytes(&file[cursor..]);
        cursor += n_bytes;
        let (n_bytes, g2_delta) = affine_g2_set_bytes(&file[cursor..]);
        cursor += n_bytes;
        let (_, g1_gamma_abc) = parse_affine_g1_array(&file[cursor..]);

        let verifying_key = universal_vk();

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
