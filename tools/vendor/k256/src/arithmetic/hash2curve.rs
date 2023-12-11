use elliptic_curve::bigint::{ArrayEncoding, U256};
use elliptic_curve::consts::{U4, U48};
use elliptic_curve::generic_array::GenericArray;
use elliptic_curve::group::cofactor::CofactorGroup;
use elliptic_curve::hash2curve::{
    FromOkm, GroupDigest, Isogeny, IsogenyCoefficients, MapToCurve, OsswuMap, OsswuMapParams, Sgn0,
};
use elliptic_curve::subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};
use elliptic_curve::Field;

use crate::{AffinePoint, ProjectivePoint, Scalar, Secp256k1};

use super::FieldElement;

impl GroupDigest for Secp256k1 {
    type FieldElement = FieldElement;
}

impl FromOkm for FieldElement {
    type Length = U48;

    fn from_okm(data: &GenericArray<u8, Self::Length>) -> Self {
        // 0x0000000000000001000000000000000000000000000000000000000000000000
        const F_2_192: FieldElement = FieldElement::from_bytes_unchecked(&[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]);
        let d0 = FieldElement::from_bytes_unchecked(&[
            0, 0, 0, 0, 0, 0, 0, 0, data[0], data[1], data[2], data[3], data[4], data[5], data[6],
            data[7], data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15],
            data[16], data[17], data[18], data[19], data[20], data[21], data[22], data[23],
        ]);
        let d1 = FieldElement::from_bytes_unchecked(&[
            0, 0, 0, 0, 0, 0, 0, 0, data[24], data[25], data[26], data[27], data[28], data[29],
            data[30], data[31], data[32], data[33], data[34], data[35], data[36], data[37],
            data[38], data[39], data[40], data[41], data[42], data[43], data[44], data[45],
            data[46], data[47],
        ]);
        d0 * F_2_192 + d1
    }
}

impl Sgn0 for FieldElement {
    fn sgn0(&self) -> Choice {
        self.normalize().is_odd()
    }
}

impl OsswuMap for FieldElement {
    const PARAMS: OsswuMapParams<Self> = OsswuMapParams {
        // See section 8.7 in
        // <https://datatracker.ietf.org/doc/draft-irtf-cfrg-hash-to-curve/>
        c1: &[
            0xffff_ffff_bfff_ff0b,
            0xffff_ffff_ffff_ffff,
            0xffff_ffff_ffff_ffff,
            0x3fff_ffff_ffff_ffff,
        ],
        // 0x25e9711ae8c0dadc 0x46fdbcb72aadd8f4 0x250b65073012ec80 0xbc6ecb9c12973975
        c2: FieldElement::from_bytes_unchecked(&[
            0x25, 0xe9, 0x71, 0x1a, 0xe8, 0xc0, 0xda, 0xdc, 0x46, 0xfd, 0xbc, 0xb7, 0x2a, 0xad,
            0xd8, 0xf4, 0x25, 0x0b, 0x65, 0x07, 0x30, 0x12, 0xec, 0x80, 0xbc, 0x6e, 0xcb, 0x9c,
            0x12, 0x97, 0x39, 0x75,
        ]),
        // 0x3f8731abdd661adc 0xa08a5558f0f5d272 0xe953d363cb6f0e5d 0x405447c01a444533
        map_a: FieldElement::from_bytes_unchecked(&[
            0x3f, 0x87, 0x31, 0xab, 0xdd, 0x66, 0x1a, 0xdc, 0xa0, 0x8a, 0x55, 0x58, 0xf0, 0xf5,
            0xd2, 0x72, 0xe9, 0x53, 0xd3, 0x63, 0xcb, 0x6f, 0x0e, 0x5d, 0x40, 0x54, 0x47, 0xc0,
            0x1a, 0x44, 0x45, 0x33,
        ]),
        // 0x00000000000006eb
        map_b: FieldElement::from_bytes_unchecked(&[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x06, 0xeb,
        ]),
        // 0xffffffffffffffff 0xffffffffffffffff 0xffffffffffffffff 0xfffffffefffffc24
        z: FieldElement::from_bytes_unchecked(&[
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0xff, 0xff, 0xfc, 0x24,
        ]),
    };

    fn osswu(&self) -> (Self, Self) {
        let tv1 = self.square(); // u^2
        let tv3 = Self::PARAMS.z * tv1; // Z * u^2
        let mut tv2 = tv3.square(); // tv3^2
        let mut xd = tv2 + tv3; // tv3^2 + tv3
        let x1n = Self::PARAMS.map_b * (xd + Self::ONE); // B * (xd + 1)
        xd = (xd * Self::PARAMS.map_a.negate(1)).normalize(); // -A * xd

        let tv = Self::PARAMS.z * Self::PARAMS.map_a;
        xd.conditional_assign(&tv, xd.is_zero());

        tv2 = xd.square(); //xd^2
        let gxd = tv2 * xd; // xd^3
        tv2 *= Self::PARAMS.map_a; // A * tv2

        let mut gx1 = x1n * (tv2 + x1n.square()); //x1n *(tv2 + x1n^2)
        tv2 = gxd * Self::PARAMS.map_b; // B * gxd
        gx1 += tv2; // gx1 + tv2

        let mut tv4 = gxd.square(); // gxd^2
        tv2 = gx1 * gxd; // gx1 * gxd
        tv4 *= tv2;

        let y1 = tv4.pow_vartime(Self::PARAMS.c1) * tv2; // tv4^C1 * tv2
        let x2n = tv3 * x1n; // tv3 * x1n

        let y2 = y1 * Self::PARAMS.c2 * tv1 * self; // y1 * c2 * tv1 * u

        tv2 = y1.square() * gxd; //y1^2 * gxd

        let e2 = tv2.normalize().ct_eq(&gx1.normalize());

        // if e2 , x = x1, else x = x2
        let mut x = Self::conditional_select(&x2n, &x1n, e2);
        // xn / xd
        x *= xd.invert().unwrap();

        // if e2, y = y1, else y = y2
        let mut y = Self::conditional_select(&y2, &y1, e2);

        y.conditional_assign(&-y, self.sgn0() ^ y.sgn0());
        (x, y)
    }
}

impl MapToCurve for FieldElement {
    type Output = ProjectivePoint;

    fn map_to_curve(&self) -> Self::Output {
        let (rx, ry) = self.osswu();
        let (qx, qy) = FieldElement::isogeny(rx, ry);

        AffinePoint {
            x: qx,
            y: qy,
            infinity: 0,
        }
        .into()
    }
}

impl FromOkm for Scalar {
    type Length = U48;

    fn from_okm(data: &GenericArray<u8, Self::Length>) -> Self {
        const F_2_192: Scalar = Scalar(U256::from_be_hex(
            "0000000000000001000000000000000000000000000000000000000000000000",
        ));

        let mut d0 = GenericArray::default();
        d0[8..].copy_from_slice(&data[0..24]);
        let d0 = Scalar(U256::from_be_byte_array(d0));

        let mut d1 = GenericArray::default();
        d1[8..].copy_from_slice(&data[24..]);
        let d1 = Scalar(U256::from_be_byte_array(d1));

        d0 * F_2_192 + d1
    }
}

impl Isogeny for FieldElement {
    type Degree = U4;

    // See section E.1 in
    // <https://datatracker.ietf.org/doc/draft-irtf-cfrg-hash-to-curve/>
    const COEFFICIENTS: IsogenyCoefficients<Self> = IsogenyCoefficients {
        xnum: &[
            FieldElement::from_bytes_unchecked(&[
                0x8e, 0x38, 0xe3, 0x8e, 0x38, 0xe3, 0x8e, 0x38, 0xe3, 0x8e, 0x38, 0xe3, 0x8e, 0x38,
                0xe3, 0x8e, 0x38, 0xe3, 0x8e, 0x38, 0xe3, 0x8e, 0x38, 0xe3, 0x8e, 0x38, 0xe3, 0x8d,
                0xaa, 0xaa, 0xa8, 0xc7,
            ]),
            FieldElement::from_bytes_unchecked(&[
                0x07, 0xd3, 0xd4, 0xc8, 0x0b, 0xc3, 0x21, 0xd5, 0xb9, 0xf3, 0x15, 0xce, 0xa7, 0xfd,
                0x44, 0xc5, 0xd5, 0x95, 0xd2, 0xfc, 0x0b, 0xf6, 0x3b, 0x92, 0xdf, 0xff, 0x10, 0x44,
                0xf1, 0x7c, 0x65, 0x81,
            ]),
            FieldElement::from_bytes_unchecked(&[
                0x53, 0x4c, 0x32, 0x8d, 0x23, 0xf2, 0x34, 0xe6, 0xe2, 0xa4, 0x13, 0xde, 0xca, 0x25,
                0xca, 0xec, 0xe4, 0x50, 0x61, 0x44, 0x03, 0x7c, 0x40, 0x31, 0x4e, 0xcb, 0xd0, 0xb5,
                0x3d, 0x9d, 0xd2, 0x62,
            ]),
            FieldElement::from_bytes_unchecked(&[
                0x8e, 0x38, 0xe3, 0x8e, 0x38, 0xe3, 0x8e, 0x38, 0xe3, 0x8e, 0x38, 0xe3, 0x8e, 0x38,
                0xe3, 0x8e, 0x38, 0xe3, 0x8e, 0x38, 0xe3, 0x8e, 0x38, 0xe3, 0x8e, 0x38, 0xe3, 0x8d,
                0xaa, 0xaa, 0xa8, 0x8c,
            ]),
        ],
        xden: &[
            FieldElement::from_bytes_unchecked(&[
                0xd3, 0x57, 0x71, 0x19, 0x3d, 0x94, 0x91, 0x8a, 0x9c, 0xa3, 0x4c, 0xcb, 0xb7, 0xb6,
                0x40, 0xdd, 0x86, 0xcd, 0x40, 0x95, 0x42, 0xf8, 0x48, 0x7d, 0x9f, 0xe6, 0xb7, 0x45,
                0x78, 0x1e, 0xb4, 0x9b,
            ]),
            FieldElement::from_bytes_unchecked(&[
                0xed, 0xad, 0xc6, 0xf6, 0x43, 0x83, 0xdc, 0x1d, 0xf7, 0xc4, 0xb2, 0xd5, 0x1b, 0x54,
                0x22, 0x54, 0x06, 0xd3, 0x6b, 0x64, 0x1f, 0x5e, 0x41, 0xbb, 0xc5, 0x2a, 0x56, 0x61,
                0x2a, 0x8c, 0x6d, 0x14,
            ]),
            FieldElement::from_bytes_unchecked(&[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x01,
            ]),
        ],
        ynum: &[
            FieldElement::from_bytes_unchecked(&[
                0x4b, 0xda, 0x12, 0xf6, 0x84, 0xbd, 0xa1, 0x2f, 0x68, 0x4b, 0xda, 0x12, 0xf6, 0x84,
                0xbd, 0xa1, 0x2f, 0x68, 0x4b, 0xda, 0x12, 0xf6, 0x84, 0xbd, 0xa1, 0x2f, 0x68, 0x4b,
                0x8e, 0x38, 0xe2, 0x3c,
            ]),
            FieldElement::from_bytes_unchecked(&[
                0xc7, 0x5e, 0x0c, 0x32, 0xd5, 0xcb, 0x7c, 0x0f, 0xa9, 0xd0, 0xa5, 0x4b, 0x12, 0xa0,
                0xa6, 0xd5, 0x64, 0x7a, 0xb0, 0x46, 0xd6, 0x86, 0xda, 0x6f, 0xdf, 0xfc, 0x90, 0xfc,
                0x20, 0x1d, 0x71, 0xa3,
            ]),
            FieldElement::from_bytes_unchecked(&[
                0x29, 0xa6, 0x19, 0x46, 0x91, 0xf9, 0x1a, 0x73, 0x71, 0x52, 0x09, 0xef, 0x65, 0x12,
                0xe5, 0x76, 0x72, 0x28, 0x30, 0xa2, 0x01, 0xbe, 0x20, 0x18, 0xa7, 0x65, 0xe8, 0x5a,
                0x9e, 0xce, 0xe9, 0x31,
            ]),
            FieldElement::from_bytes_unchecked(&[
                0x2f, 0x68, 0x4b, 0xda, 0x12, 0xf6, 0x84, 0xbd, 0xa1, 0x2f, 0x68, 0x4b, 0xda, 0x12,
                0xf6, 0x84, 0xbd, 0xa1, 0x2f, 0x68, 0x4b, 0xda, 0x12, 0xf6, 0x84, 0xbd, 0xa1, 0x2f,
                0x38, 0xe3, 0x8d, 0x84,
            ]),
        ],
        yden: &[
            FieldElement::from_bytes_unchecked(&[
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
                0xff, 0xff, 0xf9, 0x3b,
            ]),
            FieldElement::from_bytes_unchecked(&[
                0x7a, 0x06, 0x53, 0x4b, 0xb8, 0xbd, 0xb4, 0x9f, 0xd5, 0xe9, 0xe6, 0x63, 0x27, 0x22,
                0xc2, 0x98, 0x94, 0x67, 0xc1, 0xbf, 0xc8, 0xe8, 0xd9, 0x78, 0xdf, 0xb4, 0x25, 0xd2,
                0x68, 0x5c, 0x25, 0x73,
            ]),
            FieldElement::from_bytes_unchecked(&[
                0x64, 0x84, 0xaa, 0x71, 0x65, 0x45, 0xca, 0x2c, 0xf3, 0xa7, 0x0c, 0x3f, 0xa8, 0xfe,
                0x33, 0x7e, 0x0a, 0x3d, 0x21, 0x16, 0x2f, 0x0d, 0x62, 0x99, 0xa7, 0xbf, 0x81, 0x92,
                0xbf, 0xd2, 0xa7, 0x6f,
            ]),
            FieldElement::from_bytes_unchecked(&[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x01,
            ]),
        ],
    };
}

impl CofactorGroup for ProjectivePoint {
    type Subgroup = ProjectivePoint;

    fn clear_cofactor(&self) -> Self::Subgroup {
        *self
    }

    fn into_subgroup(self) -> CtOption<Self::Subgroup> {
        CtOption::new(self, 1.into())
    }

    fn is_torsion_free(&self) -> Choice {
        1.into()
    }
}

#[cfg(test)]
mod tests {
    use crate::{FieldElement, Scalar, Secp256k1, U256};
    use elliptic_curve::{
        bigint::{ArrayEncoding, NonZero, U384},
        consts::U48,
        generic_array::GenericArray,
        group::cofactor::CofactorGroup,
        hash2curve::{FromOkm, GroupDigest, MapToCurve},
        Curve,
    };
    use hex_literal::hex;
    use proptest::{num::u64::ANY, prelude::ProptestConfig, proptest};

    #[test]
    fn hash_to_curve() {
        use elliptic_curve::hash2curve::{self, ExpandMsgXmd};
        use hex_literal::hex;
        use sha2::Sha256;

        struct TestVector {
            msg: &'static [u8],
            p_x: [u8; 32],
            p_y: [u8; 32],
            u_0: [u8; 32],
            u_1: [u8; 32],
            q0_x: [u8; 32],
            q0_y: [u8; 32],
            q1_x: [u8; 32],
            q1_y: [u8; 32],
        }

        const DST: &[u8] = b"QUUX-V01-CS02-with-secp256k1_XMD:SHA-256_SSWU_RO_";

        const TEST_VECTORS: [TestVector; 5] = [
            TestVector {
                msg: b"",
                p_x: hex!("c1cae290e291aee617ebaef1be6d73861479c48b841eaba9b7b5852ddfeb1346"),
                p_y: hex!("64fa678e07ae116126f08b022a94af6de15985c996c3a91b64c406a960e51067"),
                u_0: hex!("6b0f9910dd2ba71c78f2ee9f04d73b5f4c5f7fc773a701abea1e573cab002fb3"),
                u_1: hex!("1ae6c212e08fe1a5937f6202f929a2cc8ef4ee5b9782db68b0d5799fd8f09e16"),
                q0_x: hex!("74519ef88b32b425a095e4ebcc84d81b64e9e2c2675340a720bb1a1857b99f1e"),
                q0_y: hex!("c174fa322ab7c192e11748beed45b508e9fdb1ce046dee9c2cd3a2a86b410936"),
                q1_x: hex!("44548adb1b399263ded3510554d28b4bead34b8cf9a37b4bd0bd2ba4db87ae63"),
                q1_y: hex!("96eb8e2faf05e368efe5957c6167001760233e6dd2487516b46ae725c4cce0c6"),
            },
            TestVector {
                msg: b"abc",
                p_x: hex!("3377e01eab42db296b512293120c6cee72b6ecf9f9205760bd9ff11fb3cb2c4b"),
                p_y: hex!("7f95890f33efebd1044d382a01b1bee0900fb6116f94688d487c6c7b9c8371f6"),
                u_0: hex!("128aab5d3679a1f7601e3bdf94ced1f43e491f544767e18a4873f397b08a2b61"),
                u_1: hex!("5897b65da3b595a813d0fdcc75c895dc531be76a03518b044daaa0f2e4689e00"),
                q0_x: hex!("07dd9432d426845fb19857d1b3a91722436604ccbbbadad8523b8fc38a5322d7"),
                q0_y: hex!("604588ef5138cffe3277bbd590b8550bcbe0e523bbaf1bed4014a467122eb33f"),
                q1_x: hex!("e9ef9794d15d4e77dde751e06c182782046b8dac05f8491eb88764fc65321f78"),
                q1_y: hex!("cb07ce53670d5314bf236ee2c871455c562dd76314aa41f012919fe8e7f717b3"),
            },
            TestVector {
                msg: b"abcdef0123456789",
                p_x: hex!("bac54083f293f1fe08e4a70137260aa90783a5cb84d3f35848b324d0674b0e3a"),
                p_y: hex!("4436476085d4c3c4508b60fcf4389c40176adce756b398bdee27bca19758d828"),
                u_0: hex!("ea67a7c02f2cd5d8b87715c169d055a22520f74daeb080e6180958380e2f98b9"),
                u_1: hex!("7434d0d1a500d38380d1f9615c021857ac8d546925f5f2355319d823a478da18"),
                q0_x: hex!("576d43ab0260275adf11af990d130a5752704f79478628761720808862544b5d"),
                q0_y: hex!("643c4a7fb68ae6cff55edd66b809087434bbaff0c07f3f9ec4d49bb3c16623c3"),
                q1_x: hex!("f89d6d261a5e00fe5cf45e827b507643e67c2a947a20fd9ad71039f8b0e29ff8"),
                q1_y: hex!("b33855e0cc34a9176ead91c6c3acb1aacb1ce936d563bc1cee1dcffc806caf57"),
            },
            TestVector {
                msg: b"q128_qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq",
                p_x: hex!("e2167bc785333a37aa562f021f1e881defb853839babf52a7f72b102e41890e9"),
                p_y: hex!("f2401dd95cc35867ffed4f367cd564763719fbc6a53e969fb8496a1e6685d873"),
                u_0: hex!("eda89a5024fac0a8207a87e8cc4e85aa3bce10745d501a30deb87341b05bcdf5"),
                u_1: hex!("dfe78cd116818fc2c16f3837fedbe2639fab012c407eac9dfe9245bf650ac51d"),
                q0_x: hex!("9c91513ccfe9520c9c645588dff5f9b4e92eaf6ad4ab6f1cd720d192eb58247a"),
                q0_y: hex!("c7371dcd0134412f221e386f8d68f49e7fa36f9037676e163d4a063fbf8a1fb8"),
                q1_x: hex!("10fee3284d7be6bd5912503b972fc52bf4761f47141a0015f1c6ae36848d869b"),
                q1_y: hex!("0b163d9b4bf21887364332be3eff3c870fa053cf508732900fc69a6eb0e1b672"),
            },
            TestVector {
                msg: b"a512_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                p_x: hex!("e3c8d35aaaf0b9b647e88a0a0a7ee5d5bed5ad38238152e4e6fd8c1f8cb7c998"),
                p_y: hex!("8446eeb6181bf12f56a9d24e262221cc2f0c4725c7e3803024b5888ee5823aa6"),
                u_0: hex!("8d862e7e7e23d7843fe16d811d46d7e6480127a6b78838c277bca17df6900e9f"),
                u_1: hex!("68071d2530f040f081ba818d3c7188a94c900586761e9115efa47ae9bd847938"),
                q0_x: hex!("b32b0ab55977b936f1e93fdc68cec775e13245e161dbfe556bbb1f72799b4181"),
                q0_y: hex!("2f5317098360b722f132d7156a94822641b615c91f8663be69169870a12af9e8"),
                q1_x: hex!("148f98780f19388b9fa93e7dc567b5a673e5fca7079cd9cdafd71982ec4c5e12"),
                q1_y: hex!("3989645d83a433bc0c001f3dac29af861f33a6fd1e04f4b36873f5bff497298a"),
            },
        ];

        for test_vector in TEST_VECTORS {
            // in parts
            let mut u = [FieldElement::default(), FieldElement::default()];
            hash2curve::hash_to_field::<ExpandMsgXmd<Sha256>, FieldElement>(
                &[test_vector.msg],
                &[DST],
                &mut u,
            )
            .unwrap();
            assert_eq!(u[0].to_bytes().as_slice(), test_vector.u_0);
            assert_eq!(u[1].to_bytes().as_slice(), test_vector.u_1);

            let q0 = u[0].map_to_curve();
            let aq0 = q0.to_affine();
            assert_eq!(aq0.x.to_bytes().as_slice(), test_vector.q0_x);
            assert_eq!(aq0.y.to_bytes().as_slice(), test_vector.q0_y);

            let q1 = u[1].map_to_curve();
            let aq1 = q1.to_affine();
            assert_eq!(aq1.x.to_bytes().as_slice(), test_vector.q1_x);
            assert_eq!(aq1.y.to_bytes().as_slice(), test_vector.q1_y);

            let p = q0.clear_cofactor() + q1.clear_cofactor();
            let ap = p.to_affine();
            assert_eq!(ap.x.to_bytes().as_slice(), test_vector.p_x);
            assert_eq!(ap.y.to_bytes().as_slice(), test_vector.p_y);

            // complete run
            let pt = Secp256k1::hash_from_bytes::<ExpandMsgXmd<Sha256>>(&[test_vector.msg], &[DST])
                .unwrap();
            let apt = pt.to_affine();
            assert_eq!(apt.x.to_bytes().as_slice(), test_vector.p_x);
            assert_eq!(apt.y.to_bytes().as_slice(), test_vector.p_y);
        }
    }

    #[test]
    fn from_okm_fuzz() {
        let mut wide_order = GenericArray::default();
        wide_order[16..].copy_from_slice(&Secp256k1::ORDER.to_be_byte_array());
        let wide_order = NonZero::new(U384::from_be_byte_array(wide_order)).unwrap();

        let simple_from_okm = move |data: GenericArray<u8, U48>| -> Scalar {
            let data = U384::from_be_slice(&data);

            let scalar = data % wide_order;
            let reduced_scalar = U256::from_be_slice(&scalar.to_be_byte_array()[16..]);

            Scalar(reduced_scalar)
        };

        proptest!(ProptestConfig::with_cases(1000), |(b0 in ANY, b1 in ANY, b2 in ANY, b3 in ANY, b4 in ANY, b5 in ANY)| {
            let mut data = GenericArray::default();
            data[..8].copy_from_slice(&b0.to_be_bytes());
            data[8..16].copy_from_slice(&b1.to_be_bytes());
            data[16..24].copy_from_slice(&b2.to_be_bytes());
            data[24..32].copy_from_slice(&b3.to_be_bytes());
            data[32..40].copy_from_slice(&b4.to_be_bytes());
            data[40..].copy_from_slice(&b5.to_be_bytes());

            let from_okm = Scalar::from_okm(&data);
            let simple_from_okm = simple_from_okm(data);
            assert_eq!(from_okm, simple_from_okm);
        });
    }
}
