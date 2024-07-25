use borsh::{BorshDeserialize, BorshSerialize};
use hex_literal::hex;
use near_sdk::env;
use unionlabs::{
    hash::H256, ibc::lightclients::cometbls::light_header::LightHeader, uint::U256, ByteArrayExt,
};

use crate::error::Error;

#[derive(Copy, Debug, Default, Clone, BorshSerialize, BorshDeserialize)]
pub struct Fq(pub [u8; 32]);

impl From<u64> for Fq {
    fn from(value: u64) -> Self {
        let mut buffer = [0u8; 32];
        buffer[0..8].copy_from_slice(&value.to_le_bytes());
        Fq(buffer)
    }
}

#[derive(Debug, Default, Clone, BorshSerialize, BorshDeserialize)]
pub struct Fq2(pub Fq, pub Fq);

pub const FQ_SIZE: usize = 32;
pub const G1_SIZE: usize = 2 * FQ_SIZE;
pub const G2_SIZE: usize = 2 * G1_SIZE;
pub const EXPECTED_PROOF_SIZE: usize = G1_SIZE + G2_SIZE + G1_SIZE + G1_SIZE + G1_SIZE;

pub const HMAC_O: &[u8] = &hex!("1F333139281E100F5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C");
pub const HMAC_I: &[u8] = &hex!("75595B5342747A653636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636");
pub const PRIME_R_MINUS_ONE: U256 = U256::from_limbs([
    4891460686036598784,
    2896914383306846353,
    13281191951274694749,
    3486998266802970665,
]);

pub const ALPHA_G1: G1Affine = G1Affine {
    x: Fq([
        153, 168, 24, 193, 103, 1, 111, 127, 109, 2, 216, 64, 5, 165, 237, 31, 124, 108, 25, 196,
        221, 241, 87, 51, 182, 122, 204, 1, 41, 7, 103, 9,
    ]),
    y: Fq([
        255, 129, 13, 157, 51, 116, 128, 128, 105, 193, 234, 30, 93, 38, 58, 144, 207, 129, 129,
        185, 139, 65, 88, 5, 121, 113, 118, 53, 122, 206, 199, 8,
    ]),
};
pub const BETA_G2: G2Affine = G2Affine {
    x: Fq2(
        Fq([
            116, 40, 132, 234, 24, 160, 14, 243, 24, 116, 213, 252, 85, 17, 177, 143, 169, 57, 29,
            198, 155, 151, 27, 137, 138, 45, 191, 198, 68, 3, 63, 21,
        ]),
        Fq([
            101, 109, 201, 47, 31, 148, 220, 23, 0, 38, 205, 128, 33, 46, 81, 96, 210, 83, 158,
            126, 139, 64, 136, 93, 29, 96, 183, 112, 210, 95, 53, 25,
        ]),
    ),
    y: Fq2(
        Fq([
            161, 102, 168, 244, 129, 112, 104, 183, 220, 39, 2, 48, 133, 154, 161, 86, 245, 174,
            18, 32, 215, 225, 172, 145, 34, 57, 191, 2, 102, 143, 210, 39,
        ]),
        Fq([
            118, 162, 9, 196, 98, 30, 188, 69, 94, 17, 6, 29, 53, 168, 161, 189, 86, 130, 108, 217,
            134, 186, 224, 104, 244, 98, 252, 218, 60, 228, 213, 34,
        ]),
    ),
};
pub const GAMMA_G2: G2Affine = G2Affine {
    x: Fq2(
        Fq([
            25, 182, 113, 158, 66, 196, 46, 209, 223, 70, 250, 8, 200, 112, 197, 36, 26, 82, 145,
            59, 101, 217, 180, 54, 121, 224, 137, 194, 224, 187, 22, 34,
        ]),
        Fq([
            207, 58, 72, 156, 167, 146, 127, 79, 129, 64, 10, 46, 189, 115, 154, 147, 91, 206, 179,
            34, 66, 100, 239, 248, 226, 72, 49, 26, 233, 107, 231, 32,
        ]),
    ),
    y: Fq2(
        Fq([
            58, 175, 246, 108, 37, 153, 187, 92, 145, 127, 238, 195, 150, 148, 55, 234, 53, 232,
            223, 200, 25, 159, 60, 223, 127, 58, 237, 187, 117, 107, 46, 39,
        ]),
        Fq([
            113, 5, 38, 216, 83, 140, 105, 78, 143, 175, 204, 198, 146, 97, 207, 62, 194, 205, 84,
            90, 225, 218, 199, 37, 240, 196, 96, 232, 96, 77, 239, 47,
        ]),
    ),
};
pub const DELTA_G2: G2Affine = G2Affine {
    x: Fq2(
        Fq([
            235, 4, 77, 219, 149, 30, 155, 40, 237, 167, 218, 147, 171, 163, 65, 239, 44, 150, 164,
            214, 24, 44, 167, 133, 163, 32, 24, 201, 200, 3, 212, 5,
        ]),
        Fq([
            252, 185, 240, 74, 49, 201, 136, 162, 245, 166, 71, 16, 255, 175, 225, 1, 131, 29, 97,
            71, 37, 155, 84, 228, 93, 71, 224, 209, 24, 76, 94, 41,
        ]),
    ),
    y: Fq2(
        Fq([
            7, 27, 211, 147, 97, 116, 151, 45, 70, 133, 59, 213, 58, 234, 80, 223, 14, 241, 63,
            128, 113, 63, 238, 42, 105, 160, 139, 179, 163, 166, 218, 5,
        ]),
        Fq([
            66, 178, 169, 55, 218, 119, 194, 159, 243, 241, 194, 245, 213, 52, 254, 226, 230, 16,
            152, 197, 183, 237, 181, 35, 116, 35, 185, 83, 114, 104, 79, 21,
        ]),
    ),
};
pub const PEDERSEN_G: G2Affine = G2Affine {
    x: Fq2(
        Fq([
            90, 229, 109, 192, 20, 168, 19, 119, 18, 244, 88, 70, 88, 186, 111, 126, 57, 12, 195,
            152, 146, 249, 126, 86, 202, 133, 152, 135, 216, 216, 240, 19,
        ]),
        Fq([
            135, 25, 189, 159, 250, 43, 186, 150, 57, 81, 218, 46, 8, 186, 146, 255, 193, 4, 155,
            162, 241, 253, 125, 127, 3, 176, 44, 19, 248, 246, 125, 37,
        ]),
    ),
    y: Fq2(
        Fq([
            52, 127, 186, 120, 64, 214, 31, 105, 58, 120, 83, 131, 39, 49, 242, 98, 233, 101, 60,
            206, 146, 127, 168, 224, 219, 180, 141, 197, 66, 6, 232, 21,
        ]),
        Fq([
            138, 121, 13, 200, 103, 254, 116, 205, 74, 105, 57, 58, 93, 104, 19, 238, 40, 245, 147,
            89, 234, 252, 14, 86, 172, 163, 199, 96, 204, 235, 96, 22,
        ]),
    ),
};

pub const PEDERSEN_G_ROOT_SIGMA_NEG: G2Affine = G2Affine {
    x: Fq2(
        Fq([
            175, 91, 78, 48, 18, 58, 52, 67, 57, 50, 29, 214, 33, 181, 253, 249, 205, 152, 112, 98,
            89, 40, 250, 7, 35, 95, 1, 28, 223, 4, 161, 2,
        ]),
        Fq([
            104, 99, 202, 226, 242, 176, 192, 206, 69, 126, 129, 173, 37, 160, 104, 251, 28, 184,
            96, 38, 9, 107, 232, 227, 247, 92, 85, 167, 65, 225, 191, 47,
        ]),
    ),
    y: Fq2(
        Fq([
            141, 219, 123, 186, 89, 3, 145, 235, 166, 53, 241, 135, 102, 127, 128, 151, 187, 80,
            34, 126, 222, 87, 219, 103, 207, 229, 185, 28, 85, 56, 8, 44,
        ]),
        Fq([
            34, 89, 106, 0, 43, 110, 130, 192, 130, 159, 109, 155, 83, 14, 12, 35, 108, 42, 22,
            226, 74, 53, 163, 179, 208, 191, 243, 236, 147, 63, 218, 39,
        ]),
    ),
};
pub const GAMMA_ABC_G1: [G1Affine; 3] = [
    G1Affine {
        x: Fq([
            129, 146, 83, 48, 148, 29, 83, 216, 206, 193, 196, 66, 16, 246, 200, 130, 254, 232, 44,
            74, 233, 124, 182, 75, 79, 134, 67, 39, 229, 67, 24, 39,
        ]),
        y: Fq([
            6, 36, 203, 115, 37, 168, 159, 234, 122, 210, 203, 222, 71, 138, 123, 163, 142, 202,
            24, 187, 161, 240, 36, 246, 114, 177, 248, 156, 198, 66, 51, 37,
        ]),
    },
    G1Affine {
        x: Fq([
            202, 75, 18, 93, 94, 26, 46, 192, 226, 38, 114, 67, 79, 190, 156, 160, 227, 202, 21,
            176, 194, 14, 22, 233, 2, 14, 214, 244, 113, 190, 13, 11,
        ]),
        y: Fq([
            12, 224, 112, 182, 168, 185, 95, 104, 112, 20, 216, 61, 224, 159, 158, 254, 51, 202,
            175, 22, 170, 146, 229, 236, 136, 131, 118, 211, 235, 154, 11, 19,
        ]),
    },
    G1Affine {
        x: Fq([
            199, 144, 196, 161, 145, 138, 177, 46, 126, 60, 54, 0, 91, 47, 92, 188, 245, 64, 140,
            237, 152, 3, 53, 113, 118, 12, 124, 244, 213, 147, 158, 2,
        ]),
        y: Fq([
            217, 241, 238, 106, 156, 19, 182, 235, 190, 46, 17, 218, 178, 63, 86, 0, 4, 15, 203,
            131, 59, 181, 121, 143, 174, 207, 157, 69, 16, 5, 241, 44,
        ]),
    },
];

#[derive(Copy, Debug, Default, Clone, BorshSerialize, BorshDeserialize)]
pub struct G1Affine {
    pub x: Fq,
    pub y: Fq,
}

impl G1Affine {
    pub const fn zero() -> Self {
        Self {
            x: Fq([0; 32]),
            y: Fq([0; 32]),
        }
    }
}

#[derive(Debug, Default, Clone, BorshSerialize, BorshDeserialize)]
pub struct G2Affine {
    pub x: Fq2,
    pub y: Fq2,
}

impl G2Affine {
    pub const fn zero() -> Self {
        Self {
            x: Fq2(Fq([0; 32]), Fq([0; 32])),
            y: Fq2(Fq([0; 32]), Fq([0; 32])),
        }
    }
}

pub struct Proof {
    pub a: G1Affine,
    pub b: G2Affine,
    pub c: G1Affine,
}

pub struct ZKP {
    pub proof: Proof,
    pub proof_commitment: G1Affine,
    pub proof_commitment_pok: G1Affine,
}

pub type RawZKP = [u8; EXPECTED_PROOF_SIZE];

impl TryFrom<&[u8]> for ZKP {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let value = RawZKP::try_from(value).map_err(|_| Error::InvalidZKPLength)?;

        Ok(Self {
            proof: Proof {
                a: G1Affine::from(value.array_slice::<0, G1_SIZE>()),
                b: G2Affine::from(value.array_slice::<G1_SIZE, G2_SIZE>()),
                c: G1Affine::from(value.array_slice::<{ G1_SIZE + G2_SIZE }, G1_SIZE>()),
            },
            proof_commitment: G1Affine::from(
                value.array_slice::<{ G1_SIZE + G2_SIZE + G1_SIZE }, G1_SIZE>(),
            ),
            proof_commitment_pok: G1Affine::from(
                value.array_slice::<{ G1_SIZE + G2_SIZE + G1_SIZE + G1_SIZE }, G1_SIZE>(),
            ),
        })
    }
}

impl From<[u8; G1_SIZE]> for G1Affine {
    fn from(value: [u8; G1_SIZE]) -> Self {
        let mut x = value.array_slice::<0, FQ_SIZE>();
        x.reverse();
        let mut y = value.array_slice::<FQ_SIZE, FQ_SIZE>();
        y.reverse();
        G1Affine { x: Fq(x), y: Fq(y) }
    }
}

impl From<[u8; G2_SIZE]> for G2Affine {
    fn from(value: [u8; G2_SIZE]) -> Self {
        let mut x_0 = value.array_slice::<FQ_SIZE, FQ_SIZE>();
        x_0.reverse();
        let mut x_1 = value.array_slice::<0, FQ_SIZE>();
        x_1.reverse();
        let mut y_0 = value.array_slice::<{ G1_SIZE + FQ_SIZE }, FQ_SIZE>();
        y_0.reverse();
        let mut y_1 = value.array_slice::<G1_SIZE, FQ_SIZE>();
        y_1.reverse();
        G2Affine {
            x: Fq2(Fq(x_0), Fq(x_1)),
            y: Fq2(Fq(y_0), Fq(y_1)),
        }
    }
}

pub fn verify_zkp(
    chain_id: &str,
    trusted_validators_hash: H256,
    header: &LightHeader,
    zkp: impl Into<Vec<u8>>,
) -> Result<(), Error> {
    let zkp = ZKP::try_from(zkp.into().as_ref())?;

    let commitment_hash = hash_commitment(&zkp.proof_commitment)?;

    let mut inputs_hash = env::sha256_array(
        &vec![0u8; 32 - chain_id.len()]
            .into_iter()
            .chain(chain_id.bytes())
            .chain(
                U256::from(
                    u64::try_from(i64::from(header.height)).map_err(|_| Error::InvalidHeight)?,
                )
                .to_be_bytes(),
            )
            .chain(
                U256::from(
                    u64::try_from(i64::from(header.time.seconds))
                        .map_err(|_| Error::InvalidTimestamp)?,
                )
                .to_be_bytes(),
            )
            .chain(
                U256::from(
                    u64::try_from(i32::from(header.time.nanos))
                        .map_err(|_| Error::InvalidTimestamp)?,
                )
                .to_be_bytes(),
            )
            .chain(header.validators_hash)
            .chain(header.next_validators_hash)
            .chain(header.app_hash)
            .chain(trusted_validators_hash)
            .collect::<Vec<_>>(),
    );

    // drop the most significant byte to fit in bn254 F_r
    inputs_hash[0] = 0;
    inputs_hash.reverse();

    let public_inputs = [Fq(inputs_hash), commitment_hash];

    let initial_point: G1Affine = borsh::from_slice(&env::alt_bn128_g1_sum(
        &borsh::to_vec(&[
            (false, GAMMA_ABC_G1[0].clone()),
            (false, zkp.proof_commitment.clone()),
        ])
        .unwrap(),
    ))
    .unwrap();

    let public_inputs_msm = public_inputs
        .into_iter()
        .zip(GAMMA_ABC_G1.into_iter().skip(1))
        .fold(initial_point, |s, (w_i, gamma_l_i)| {
            let mul: G1Affine = borsh::from_slice(&env::alt_bn128_g1_multiexp(
                &borsh::to_vec(&[(gamma_l_i, w_i)]).unwrap(),
            ))
            .unwrap();

            borsh::from_slice::<G1Affine>(&env::alt_bn128_g1_sum(
                &borsh::to_vec(&[(false, s), (false, mul)]).unwrap(),
            ))
            .unwrap()
        });

    // TODO(aeryz): either split the pairing or apply fiat-shamir transformation
    if env::alt_bn128_pairing_check(
        &borsh::to_vec(&[
            (zkp.proof.a, zkp.proof.b),
            (public_inputs_msm, GAMMA_G2),
            (zkp.proof.c, DELTA_G2),
            (ALPHA_G1, BETA_G2),
            (zkp.proof_commitment, PEDERSEN_G),
            (zkp.proof_commitment_pok, PEDERSEN_G_ROOT_SIGMA_NEG),
        ])
        .unwrap(),
    ) {
        Ok(())
    } else {
        env::panic_str("pairing check failed");
    }
}

fn hmac_keccak(message: &[u8]) -> [u8; 32] {
    env::keccak256_array(
        &HMAC_O
            .iter()
            .copied()
            .chain(env::keccak256(
                &HMAC_I
                    .iter()
                    .copied()
                    .chain(message.iter().copied())
                    .collect::<Vec<_>>(),
            ))
            .collect::<Vec<_>>(),
    )
}

fn hash_commitment(proof_commitment: &G1Affine) -> Result<Fq, Error> {
    let mut buffer = [0u8; 64];
    buffer[0..32].copy_from_slice(&proof_commitment.x.0);
    buffer[0..32].reverse();
    buffer[32..64].copy_from_slice(&proof_commitment.y.0);
    buffer[32..64].reverse();
    let hmac = hmac_keccak(&buffer);
    let mut out = ((U256::from_be_bytes(hmac) % PRIME_R_MINUS_ONE) + U256::from(1)).to_be_bytes();
    out.reverse();
    Ok(Fq(out))
}
