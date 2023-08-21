use ethabi::{ParamType, Token};
use ethers::types::U256;
use tiny_keccak::Hasher;

lazy_static::lazy_static! {
    static ref HMAC_O: Vec<u8> = hex::decode("1F333139281E100F5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C").unwrap();
    static ref HMAC_I: Vec<u8> = hex::decode("75595B5342747A653636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636").unwrap();
    static ref PRIME_R: U256 = U256::from_dec_str("21888242871839275222246405745257275088548364400416034343698204186575808495617").unwrap();
    static ref PRIME_R_MINUS_ONE: U256 = *PRIME_R - 1;
}

pub fn verify_zkp(
    trusted_validators_hash: &[u8],
    untrusted_validators_hash: &[u8],
    message: &[u8],
    zkp: &[u8],
) -> bool {
    let (message_x, message_y) = hash_to_field2(message);

    let values = ethabi::decode(
        &[
            ParamType::FixedArray(Box::new(ParamType::Uint(256)), 2),
            ParamType::FixedArray(
                Box::new(ParamType::FixedArray(Box::new(ParamType::Uint(256)), 2)),
                2,
            ),
            ParamType::FixedArray(Box::new(ParamType::Uint(256)), 2),
            ParamType::Uint(256),
            ParamType::FixedArray(Box::new(ParamType::Uint(256)), 2),
        ],
        zkp,
    )
    .unwrap();

    let (a, b, c, commitment_hash, proof_commitment) = match &values[..] {
        [Token::FixedArray(a), Token::FixedArray(b), Token::FixedArray(c), Token::Uint(commitment_hash), Token::FixedArray(proof_commitment)] =>
        {
            let a = match &a[..] {
                [Token::Uint(a1), Token::Uint(a2)] => [*a1, *a2],
                _ => panic!("omg"),
            };

            let b = match &b[..] {
                [Token::FixedArray(b1), Token::FixedArray(b2)] => match (&b1[..], &b2[..]) {
                    (
                        [Token::Uint(b11), Token::Uint(b12)],
                        [Token::Uint(b21), Token::Uint(b22)],
                    ) => [[*b11, *b12], [*b21, *b22]],
                    _ => panic!(""),
                },
                _ => panic!("omg"),
            };

            let c = match &c[..] {
                [Token::Uint(c1), Token::Uint(c2)] => [*c1, *c2],
                _ => panic!("omg"),
            };

            let proof_commitment = match &proof_commitment[..] {
                [Token::Uint(pc1), Token::Uint(pc2)] => [*pc1, *pc2],
                _ => panic!("omg"),
            };

            (a, b, c, *commitment_hash, proof_commitment)
        }
        _ => panic!("omg"),
    };

    let packed_trusted_validators_hash =
        ethabi::encode(&[Token::FixedBytes(trusted_validators_hash.to_vec())]);

    let packed_untrusted_validators_hash =
        ethabi::encode(&[Token::FixedBytes(untrusted_validators_hash.to_vec())]);

    let inputs: [U256; 9] = [
        u128::from_le_bytes(packed_trusted_validators_hash[..16].try_into().unwrap()).into(),
        u128::from_le_bytes(packed_trusted_validators_hash[16..].try_into().unwrap()).into(),
        u128::from_le_bytes(packed_untrusted_validators_hash[..16].try_into().unwrap()).into(),
        u128::from_le_bytes(packed_untrusted_validators_hash[16..].try_into().unwrap()).into(),
        message_x,
        message_y,
        commitment_hash,
        proof_commitment[0],
        proof_commitment[1],
    ];

    verify_proof(a, b, c, inputs);

    true
}

fn verify_proof(_a: [U256; 2], _b: [[U256; 2]; 2], _c: [U256; 2], _input: [U256; 9]) -> bool {
    todo!()
}

fn hmac_keccak(message: &[u8]) -> [u8; 32] {
    let mut inner_hash = [0u8; 32];
    let mut hasher = tiny_keccak::Keccak::v256();
    hasher.update(&ethabi::encode(&[
        ethabi::Token::FixedBytes(HMAC_I.to_vec()),
        ethabi::Token::FixedBytes(message.to_vec()),
    ]));
    hasher.finalize(&mut inner_hash);

    let mut outer_hash = [0u8; 32];
    let mut hasher = tiny_keccak::Keccak::v256();
    hasher.update(&ethabi::encode(&[
        ethabi::Token::FixedBytes(HMAC_O.to_vec()),
        ethabi::Token::FixedBytes(inner_hash.to_vec()),
    ]));
    hasher.finalize(&mut outer_hash);

    outer_hash
}

fn hash_to_field2(message: &[u8]) -> (U256, U256) {
    (
        hash_to_field(&ethabi::encode(&[
            ethabi::Token::FixedBytes(vec![0]),
            ethabi::Token::Bytes(message.to_vec()),
        ])),
        hash_to_field(&ethabi::encode(&[
            ethabi::Token::FixedBytes(vec![1]),
            ethabi::Token::Bytes(message.to_vec()),
        ])),
    )
}

fn hash_to_field(message: &[u8]) -> U256 {
    U256::from(&hmac_keccak(message)) % *PRIME_R_MINUS_ONE + 1
}
