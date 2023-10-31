//! Argon2 Known Answer Tests (KAT).
//!
//! Taken from the Argon2 reference implementation as well as
//! `draft-irtf-cfrg-argon2-12` Section 5:
//! <https://datatracker.ietf.org/doc/draft-irtf-cfrg-argon2/>

#![cfg(all(feature = "alloc", feature = "password-hash"))]

// TODO(tarcieri): test full set of vectors from the reference implementation:
// https://github.com/P-H-C/phc-winner-argon2/blob/master/src/test.c

use argon2::{
    Algorithm, Argon2, Error, Params, ParamsBuilder, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use hex_literal::hex;
use password_hash::SaltString;

/// Params used by the KATs.
fn example_params() -> Params {
    let mut builder = ParamsBuilder::new();
    builder.m_cost(32).unwrap();
    builder.t_cost(3).unwrap();
    builder.p_cost(4).unwrap();
    builder.data(&[0x04; 12]).unwrap();
    builder.params().unwrap()
}

/// =======================================
/// Argon2d version number 16
/// =======================================
/// Memory: 32 KiB, Iterations: 3, Parallelism: 4 lanes, Tag length: 32 bytes
/// Password[32]:
///     01 01 01 01 01 01 01 01 01 01 01 01 01 01 01 01
///     01 01 01 01 01 01 01 01 01 01 01 01 01 01 01 01
/// Salt[16]: 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02
/// Secret[8]: 03 03 03 03 03 03 03 03
/// Associated data[12]: 04 04 04 04 04 04 04 04 04 04 04 04
/// Pre-hashing digest:
///     ec dc 26 dc 6b dd 21 56 19 68 97 aa 8c c9 a0 4c
///     03 ed 07 cd 12 92 67 c5 3c a6 ae f7 76 a4 30 89
///     6a 09 80 54 e4 de c3 e0 2e cd 82 c4 7f 56 2c a2
///     73 d2 f6 97 8a 5c 05 41 1a 0c d0 9d 47 7b 7b 06
/// Tag[32]:
///     96 a9 d4 e5 a1 73 40 92 c8 5e 29 f4 10 a4 59 14
///     a5 dd 1f 5c bf 08 b2 67 0d a6 8a 02 85 ab f3 2b
#[test]
fn argon2d_v0x10() {
    let algorithm = Algorithm::Argon2d;
    let version = Version::V0x10;
    let params = example_params();
    let password = [0x01; 32];
    let salt = [0x02; 16];
    let secret = [0x03; 8];
    let expected_tag = hex!(
        "
        96 a9 d4 e5 a1 73 40 92 c8 5e 29 f4 10 a4 59 14
        a5 dd 1f 5c bf 08 b2 67 0d a6 8a 02 85 ab f3 2b
        "
    );

    let ctx = Argon2::new_with_secret(&secret, algorithm, version, params).unwrap();
    let mut out = [0u8; 32];
    ctx.hash_password_into(&password, &salt, &mut out).unwrap();

    assert_eq!(out, expected_tag);
}

/// =======================================
/// Argon2i version number 16
/// =======================================
/// Memory: 32 KiB, Iterations: 3, Parallelism: 4 lanes, Tag length: 32 bytes
/// Password[32]:
///     01 01 01 01 01 01 01 01 01 01 01 01 01 01 01 01
///     01 01 01 01 01 01 01 01 01 01 01 01 01 01 01 01
/// Salt[16]: 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02
/// Secret[8]: 03 03 03 03 03 03 03 03
/// Associated data[12]: 04 04 04 04 04 04 04 04 04 04 04 04
/// Pre-hashing digest:
///    1c dc ec c8 58 ca 1b 6d 45 c7 3c 78 d0 00 76 c5
///    ec fc 5e df 14 45 b4 43 73 97 b1 b8 20 83 ff bf
///    e3 c9 1a a8 f5 06 67 ad 8f b9 d4 e7 52 df b3 85
///    34 71 9f ba d2 22 61 33 7b 2b 55 29 81 44 09 af
/// Tag[32]:
///    87 ae ed d6 51 7a b8 30 cd 97 65 cd 82 31 ab b2
///    e6 47 a5 de e0 8f 7c 05 e0 2f cb 76 33 35 d0 fd
#[test]
fn argon2i_v0x10() {
    let algorithm = Algorithm::Argon2i;
    let version = Version::V0x10;
    let params = example_params();
    let password = [0x01; 32];
    let salt = [0x02; 16];
    let secret = [0x03; 8];
    let expected_tag = hex!(
        "
        87 ae ed d6 51 7a b8 30 cd 97 65 cd 82 31 ab b2
        e6 47 a5 de e0 8f 7c 05 e0 2f cb 76 33 35 d0 fd
        "
    );

    let ctx = Argon2::new_with_secret(&secret, algorithm, version, params).unwrap();
    let mut out = [0u8; 32];
    ctx.hash_password_into(&password, &salt, &mut out).unwrap();

    assert_eq!(out, expected_tag);
}

/// =======================================
/// Argon2id version number 16
/// =======================================
/// Memory: 32 KiB, Iterations: 3, Parallelism: 4 lanes, Tag length: 32 bytes
/// Password[32]:
///     01 01 01 01 01 01 01 01 01 01 01 01 01 01 01 01
///     01 01 01 01 01 01 01 01 01 01 01 01 01 01 01 01
/// Salt[16]: 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02
/// Secret[8]: 03 03 03 03 03 03 03 03
/// Associated data[12]: 04 04 04 04 04 04 04 04 04 04 04 04
/// Pre-hashing digest:
///     70 65 ab 9c 82 b5 f0 e8 71 28 c7 84 7a 02 1d 1e
///     59 aa 16 66 6f c8 b4 ef ac a3 86 3f bf d6 5e 0e
///     8b a6 f6 09 eb bc 9b 60 e2 78 22 c8 24 b7 50 6f
///     b9 f9 5b e9 0e e5 84 2a ac 6e d6 b7 da 67 30 44
/// Tag[32]:
///     b6 46 15 f0 77 89 b6 6b 64 5b 67 ee 9e d3 b3 77
///     ae 35 0b 6b fc bb 0f c9 51 41 ea 8f 32 26 13 c0
#[test]
fn argon2id_v0x10() {
    let algorithm = Algorithm::Argon2id;
    let version = Version::V0x10;
    let params = example_params();
    let password = [0x01; 32];
    let salt = [0x02; 16];
    let secret = [0x03; 8];
    let expected_tag = hex!(
        "
        b6 46 15 f0 77 89 b6 6b 64 5b 67 ee 9e d3 b3 77
        ae 35 0b 6b fc bb 0f c9 51 41 ea 8f 32 26 13 c0
        "
    );

    let ctx = Argon2::new_with_secret(&secret, algorithm, version, params).unwrap();
    let mut out = [0u8; 32];
    ctx.hash_password_into(&password, &salt, &mut out).unwrap();

    assert_eq!(out, expected_tag);
}

/// =======================================
/// Argon2d version number 19
/// =======================================
/// Memory: 32 KiB
/// Passes: 3
/// Parallelism: 4 lanes
/// Tag length: 32 bytes
/// Password[32]:
///     01 01 01 01 01 01 01 01
///     01 01 01 01 01 01 01 01
///     01 01 01 01 01 01 01 01
///     01 01 01 01 01 01 01 01
/// Salt[16]: 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02
/// Secret[8]: 03 03 03 03 03 03 03 03
/// Associated data[12]: 04 04 04 04 04 04 04 04 04 04 04 04
/// Pre-hashing digest:
///     b8 81 97 91 a0 35 96 60
///     bb 77 09 c8 5f a4 8f 04
///     d5 d8 2c 05 c5 f2 15 cc
///     db 88 54 91 71 7c f7 57
///     08 2c 28 b9 51 be 38 14
///     10 b5 fc 2e b7 27 40 33
///     b9 fd c7 ae 67 2b ca ac
///     5d 17 90 97 a4 af 31 09
/// Tag[32]:
///     51 2b 39 1b 6f 11 62 97
///     53 71 d3 09 19 73 42 94
///     f8 68 e3 be 39 84 f3 c1
///     a1 3a 4d b9 fa be 4a cb
#[test]
fn argon2d_v0x13() {
    let algorithm = Algorithm::Argon2d;
    let version = Version::V0x13;
    let params = example_params();
    let password = [0x01; 32];
    let salt = [0x02; 16];
    let secret = [0x03; 8];
    let expected_tag = hex!(
        "
        51 2b 39 1b 6f 11 62 97
        53 71 d3 09 19 73 42 94
        f8 68 e3 be 39 84 f3 c1
        a1 3a 4d b9 fa be 4a cb
        "
    );

    let ctx = Argon2::new_with_secret(&secret, algorithm, version, params).unwrap();
    let mut out = [0u8; 32];
    ctx.hash_password_into(&password, &salt, &mut out).unwrap();

    assert_eq!(out, expected_tag);
}

/// =======================================
/// Argon2i version number 19
/// =======================================
/// Memory: 32 KiB
/// Passes: 3
/// Parallelism: 4 lanes
/// Tag length: 32 bytes
/// Password[32]:
///     01 01 01 01 01 01 01 01
///     01 01 01 01 01 01 01 01
///     01 01 01 01 01 01 01 01
///     01 01 01 01 01 01 01 01
/// Salt[16]: 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02
/// Secret[8]: 03 03 03 03 03 03 03 03
/// Associated data[12]: 04 04 04 04 04 04 04 04 04 04 04 04
/// Pre-hashing digest:
///     c4 60 65 81 52 76 a0 b3
///     e7 31 73 1c 90 2f 1f d8
///     0c f7 76 90 7f bb 7b 6a
///     5c a7 2e 7b 56 01 1f ee
///     ca 44 6c 86 dd 75 b9 46
///     9a 5e 68 79 de c4 b7 2d
///     08 63 fb 93 9b 98 2e 5f
///     39 7c c7 d1 64 fd da a9
/// Tag[32]:
///     c8 14 d9 d1 dc 7f 37 aa
///     13 f0 d7 7f 24 94 bd a1
///     c8 de 6b 01 6d d3 88 d2
///     99 52 a4 c4 67 2b 6c e8
#[test]
fn argon2i_v0x13() {
    let algorithm = Algorithm::Argon2i;
    let version = Version::V0x13;
    let params = example_params();
    let password = [0x01; 32];
    let salt = [0x02; 16];
    let secret = [0x03; 8];
    let expected_tag = hex!(
        "
        c8 14 d9 d1 dc 7f 37 aa
        13 f0 d7 7f 24 94 bd a1
        c8 de 6b 01 6d d3 88 d2
        99 52 a4 c4 67 2b 6c e8
        "
    );

    let ctx = Argon2::new_with_secret(&secret, algorithm, version, params).unwrap();
    let mut out = [0u8; 32];
    ctx.hash_password_into(&password, &salt, &mut out).unwrap();

    assert_eq!(out, expected_tag);
}

/// =======================================
/// Argon2id version number 19
/// =======================================
/// Memory: 32 KiB, Passes: 3,
/// Parallelism: 4 lanes, Tag length: 32 bytes
/// Password[32]:
///     01 01 01 01 01 01 01 01 01 01 01 01 01 01 01 01
///     01 01 01 01 01 01 01 01 01 01 01 01 01 01 01 01
/// Salt[16]: 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02
/// Secret[8]: 03 03 03 03 03 03 03 03
/// Associated data[12]: 04 04 04 04 04 04 04 04 04 04 04 04
/// Pre-hashing digest:
///     28 89 de 48 7e b4 2a e5 00 c0 00 7e d9 25 2f 10
///     69 ea de c4 0d 57 65 b4 85 de 6d c2 43 7a 67 b8
///     54 6a 2f 0a cc 1a 08 82 db 8f cf 74 71 4b 47 2e
///     94 df 42 1a 5d a1 11 2f fa 11 43 43 70 a1 e9 97
/// Tag[32]:
///     0d 64 0d f5 8d 78 76 6c 08 c0 37 a3 4a 8b 53 c9
///     d0 1e f0 45 2d 75 b6 5e b5 25 20 e9 6b 01 e6 59
#[test]
fn argon2id_v0x13() {
    let algorithm = Algorithm::Argon2id;
    let version = Version::V0x13;
    let params = example_params();
    let password = [0x01; 32];
    let salt = [0x02; 16];
    let secret = [0x03; 8];
    let expected_tag = hex!(
        "
        0d 64 0d f5 8d 78 76 6c 08 c0 37 a3 4a 8b 53 c9
        d0 1e f0 45 2d 75 b6 5e b5 25 20 e9 6b 01 e6 59
        "
    );

    let ctx = Argon2::new_with_secret(&secret, algorithm, version, params).unwrap();
    let mut out = [0u8; 32];
    ctx.hash_password_into(&password, &salt, &mut out).unwrap();

    assert_eq!(out, expected_tag);
}

// =======================================
// Basic error checks
// =======================================

#[test]
fn salt_bad_length() {
    let ctx = Argon2::new(Algorithm::Argon2id, Version::V0x13, example_params());
    let mut out = [0u8; 32];

    let too_short_salt = [0u8; argon2::MIN_SALT_LEN - 1];
    let ret = ctx.hash_password_into(b"password", &too_short_salt, &mut out);
    assert_eq!(ret, Err(Error::SaltTooShort));

    // 4 GiB of RAM seems big, but as long as we ask for a zero-initialized vector
    // optimizations kicks in an nothing is really allocated
    let too_long_salt = vec![0u8; argon2::MAX_SALT_LEN + 1];
    let ret = ctx.hash_password_into(b"password", &too_long_salt, &mut out);
    assert_eq!(ret, Err(Error::SaltTooLong));
}

#[test]
fn output_bad_length() {
    let ctx = Argon2::new(Algorithm::Argon2id, Version::V0x13, example_params());
    let mut out = [0u8; Params::MIN_OUTPUT_LEN - 1];
    let ret = ctx.hash_password_into(b"password", b"diffsalt", &mut out);
    assert_eq!(ret, Err(Error::OutputTooShort));

    // 4 GiB of RAM seems big, but as long as we ask for a zero-initialized vector
    // optimizations kicks in an nothing is really allocated
    let mut out = vec![0u8; Params::MAX_OUTPUT_LEN + 1];
    let ret = ctx.hash_password_into(b"password", b"diffsalt", &mut out);
    assert_eq!(ret, Err(Error::OutputTooLong));
}

// =======================================
// Reference implementation's test suite
// =======================================
// Taken from https://github.com/P-H-C/phc-winner-argon2/blob/master/src/test.c

fn hashtest(
    algorithm: Algorithm,
    version: Version,
    t: u32,
    m: u32,
    p: u32,
    pwd: &[u8],
    salt: &[u8],
    expected_raw_hash: [u8; 32],
    expected_phc_hash: &str,
    alternative_phc_hash: &str,
) {
    let mut builder = ParamsBuilder::new();
    builder.t_cost(t).unwrap();
    builder.m_cost(1 << m).unwrap();
    builder.p_cost(p).unwrap();
    let params = builder.params().unwrap();
    let ctx = Argon2::new(algorithm, version, params);

    // Test raw hash
    let mut out = [0u8; 32];
    ctx.hash_password_into(pwd, salt, &mut out).unwrap();
    assert_eq!(out, expected_raw_hash);

    // Test hash encoding
    let salt_string = SaltString::b64_encode(&salt).unwrap();
    let phc_hash = ctx.hash_password(pwd, &salt_string).unwrap().to_string();
    assert_eq!(phc_hash, expected_phc_hash);

    let hash = PasswordHash::new(alternative_phc_hash).unwrap();
    assert!(Argon2::default().verify_password(pwd, &hash).is_ok());
}

macro_rules! testcase_good {
    ($name: ident, $algorithm: expr, $version: expr, $t: expr, $m: expr, $p: expr, $pwd: expr, $salt: expr, $expected_raw: expr, $expected_phc: expr) => {
        #[test]
        fn $name() {
            hashtest(
                $algorithm,
                $version,
                $t,
                $m,
                $p,
                $pwd,
                $salt,
                $expected_raw,
                $expected_phc,
                $expected_phc,
            )
        }
    };
    ($name: ident, $algorithm: expr, $version: expr, $t: expr, $m: expr, $p: expr, $pwd: expr, $salt: expr, $expected_raw: expr, $expected_phc: expr, $alternative_phc: expr) => {
        #[test]
        fn $name() {
            hashtest(
                $algorithm,
                $version,
                $t,
                $m,
                $p,
                $pwd,
                $salt,
                $expected_raw,
                $expected_phc,
                $alternative_phc,
            )
        }
    };
}

macro_rules! ignored_testcase_good {
    ($name: ident, $algorithm: expr, $version: expr, $t: expr, $m: expr, $p: expr, $pwd: expr, $salt: expr, $expected_raw: expr, $expected_phc: expr, $alternative_phc: expr) => {
        #[test]
        #[ignore]
        fn $name() {
            hashtest(
                $algorithm,
                $version,
                $t,
                $m,
                $p,
                $pwd,
                $salt,
                $expected_raw,
                $expected_phc,
                $alternative_phc,
            )
        }
    };
}

/* Argon2i V0x10: Multiple test cases for various input values */

// TODO: If version is not provided, verifier incorrectly uses version 0x13
ignored_testcase_good!(
    reference_argon2i_v0x10_2_16_1,
    Algorithm::Argon2i,
    Version::V0x10,
    2,
    16,
    1,
    b"password",
    b"somesalt",
    hex!("f6c4db4a54e2a370627aff3db6176b94a2a209a62c8e36152711802f7b30c694"),
    "$argon2i$v=16$m=65536,t=2,p=1$c29tZXNhbHQ$9sTbSlTio3Biev89thdrlKKiCaYsjjYVJxGAL3swxpQ",
    "$argon2i$m=65536,t=2,p=1$c29tZXNhbHQ$9sTbSlTio3Biev89thdrlKKiCaYsjjYVJxGAL3swxpQ"
);

// TODO: If version is not provided, verifier incorrectly uses version 0x13
#[ignore]
#[cfg(feature = "test_large_ram")]
testcase_good!(
    reference_argon2i_v0x10_2_20_1_large_ram,
    Algorithm::Argon2i,
    Version::V0x10,
    2,
    20,
    1,
    b"password",
    b"somesalt",
    hex!("9690ec55d28d3ed32562f2e73ea62b02b018757643a2ae6e79528459de8106e9"),
    "$argon2i$v=16$m=1048576,t=2,p=1$c29tZXNhbHQ$lpDsVdKNPtMlYvLnPqYrArAYdXZDoq5ueVKEWd6BBuk",
    "$argon2i$m=1048576,t=2,p=1$c29tZXNhbHQ$lpDsVdKNPtMlYvLnPqYrArAYdXZDoq5ueVKEWd6BBuk"
);

// TODO: If version is not provided, verifier incorrectly uses version 0x13
ignored_testcase_good!(
    reference_argon2i_v0x10_2_18_1,
    Algorithm::Argon2i,
    Version::V0x10,
    2,
    18,
    1,
    b"password",
    b"somesalt",
    hex!("3e689aaa3d28a77cf2bc72a51ac53166761751182f1ee292e3f677a7da4c2467"),
    "$argon2i$v=16$m=262144,t=2,p=1$c29tZXNhbHQ$Pmiaqj0op3zyvHKlGsUxZnYXURgvHuKS4/Z3p9pMJGc",
    "$argon2i$m=262144,t=2,p=1$c29tZXNhbHQ$Pmiaqj0op3zyvHKlGsUxZnYXURgvHuKS4/Z3p9pMJGc"
);

// TODO: If version is not provided, verifier incorrectly uses version 0x13
ignored_testcase_good!(
    reference_argon2i_v0x10_2_8_1,
    Algorithm::Argon2i,
    Version::V0x10,
    2,
    8,
    1,
    b"password",
    b"somesalt",
    hex!("fd4dd83d762c49bdeaf57c47bdcd0c2f1babf863fdeb490df63ede9975fccf06"),
    "$argon2i$v=16$m=256,t=2,p=1$c29tZXNhbHQ$/U3YPXYsSb3q9XxHvc0MLxur+GP960kN9j7emXX8zwY",
    "$argon2i$m=256,t=2,p=1$c29tZXNhbHQ$/U3YPXYsSb3q9XxHvc0MLxur+GP960kN9j7emXX8zwY"
);

// TODO: If version is not provided, verifier incorrectly uses version 0x13
ignored_testcase_good!(
    reference_argon2i_v0x10_2_8_2,
    Algorithm::Argon2i,
    Version::V0x10,
    2,
    8,
    2,
    b"password",
    b"somesalt",
    hex!("b6c11560a6a9d61eac706b79a2f97d68b4463aa3ad87e00c07e2b01e90c564fb"),
    "$argon2i$v=16$m=256,t=2,p=2$c29tZXNhbHQ$tsEVYKap1h6scGt5ovl9aLRGOqOth+AMB+KwHpDFZPs",
    "$argon2i$m=256,t=2,p=2$c29tZXNhbHQ$tsEVYKap1h6scGt5ovl9aLRGOqOth+AMB+KwHpDFZPs"
);

// TODO: If version is not provided, verifier incorrectly uses version 0x13
ignored_testcase_good!(
    reference_argon2i_v0x10_1_16_1,
    Algorithm::Argon2i,
    Version::V0x10,
    1,
    16,
    1,
    b"password",
    b"somesalt",
    hex!("81630552b8f3b1f48cdb1992c4c678643d490b2b5eb4ff6c4b3438b5621724b2"),
    "$argon2i$v=16$m=65536,t=1,p=1$c29tZXNhbHQ$gWMFUrjzsfSM2xmSxMZ4ZD1JCytetP9sSzQ4tWIXJLI",
    "$argon2i$m=65536,t=1,p=1$c29tZXNhbHQ$gWMFUrjzsfSM2xmSxMZ4ZD1JCytetP9sSzQ4tWIXJLI"
);

// TODO: If version is not provided, verifier incorrectly uses version 0x13
ignored_testcase_good!(
    reference_argon2i_v0x10_4_16_1,
    Algorithm::Argon2i,
    Version::V0x10,
    4,
    16,
    1,
    b"password",
    b"somesalt",
    hex!("f212f01615e6eb5d74734dc3ef40ade2d51d052468d8c69440a3a1f2c1c2847b"),
    "$argon2i$v=16$m=65536,t=4,p=1$c29tZXNhbHQ$8hLwFhXm6110c03D70Ct4tUdBSRo2MaUQKOh8sHChHs",
    "$argon2i$m=65536,t=4,p=1$c29tZXNhbHQ$8hLwFhXm6110c03D70Ct4tUdBSRo2MaUQKOh8sHChHs"
);

// TODO: If version is not provided, verifier incorrectly uses version 0x13
ignored_testcase_good!(
    reference_argon2i_v0x10_2_16_1_differentpassword,
    Algorithm::Argon2i,
    Version::V0x10,
    2,
    16,
    1,
    b"differentpassword",
    b"somesalt",
    hex!("e9c902074b6754531a3a0be519e5baf404b30ce69b3f01ac3bf21229960109a3"),
    "$argon2i$v=16$m=65536,t=2,p=1$c29tZXNhbHQ$6ckCB0tnVFMaOgvlGeW69ASzDOabPwGsO/ISKZYBCaM",
    "$argon2i$m=65536,t=2,p=1$c29tZXNhbHQ$6ckCB0tnVFMaOgvlGeW69ASzDOabPwGsO/ISKZYBCaM"
);

// TODO: If version is not provided, verifier incorrectly uses version 0x13
ignored_testcase_good!(
    reference_argon2i_v0x10_2_16_1_diffsalt,
    Algorithm::Argon2i,
    Version::V0x10,
    2,
    16,
    1,
    b"password",
    b"diffsalt",
    hex!("79a103b90fe8aef8570cb31fc8b22259778916f8336b7bdac3892569d4f1c497"),
    "$argon2i$v=16$m=65536,t=2,p=1$ZGlmZnNhbHQ$eaEDuQ/orvhXDLMfyLIiWXeJFvgza3vaw4kladTxxJc",
    "$argon2i$m=65536,t=2,p=1$ZGlmZnNhbHQ$eaEDuQ/orvhXDLMfyLIiWXeJFvgza3vaw4kladTxxJc"
);

/* Argon2i V0x10: Error state tests */

// TODO: If version is not provided, verifier incorrectly uses version 0x13
#[ignore]
#[test]
fn reference_argon2i_v0x10_mismatching_hash() {
    /* Handle an mismatching hash (the encoded password is "passwore") */
    let hash = PasswordHash::new(
        "$argon2i$m=65536,t=2,p=1$c29tZXNhbHQ$b2G3seW+uPzerwQQC+/E1K50CLLO7YXy0JRcaTuswRo",
    )
    .unwrap();
    assert_eq!(
        Argon2::default().verify_password(b"password", &hash),
        Err(password_hash::errors::Error::Password)
    );
}

/* Argon2i V0x13: Multiple test cases for various input values */

testcase_good!(
    reference_argon2i_v0x13_2_16_1,
    Algorithm::Argon2i,
    Version::V0x13,
    2,
    16,
    1,
    b"password",
    b"somesalt",
    hex!("c1628832147d9720c5bd1cfd61367078729f6dfb6f8fea9ff98158e0d7816ed0"),
    "$argon2i$v=19$m=65536,t=2,p=1$c29tZXNhbHQ$wWKIMhR9lyDFvRz9YTZweHKfbftvj+qf+YFY4NeBbtA"
);

#[cfg(feature = "test_large_ram")]
testcase_good!(
    reference_argon2i_v0x13_2_20_1,
    Algorithm::Argon2i,
    Version::V0x13,
    2,
    20,
    1,
    b"password",
    b"somesalt",
    hex!("d1587aca0922c3b5d6a83edab31bee3c4ebaef342ed6127a55d19b2351ad1f41"),
    "$argon2i$v=19$m=1048576,t=2,p=1$c29tZXNhbHQ$0Vh6ygkiw7XWqD7asxvuPE667zQu1hJ6VdGbI1GtH0E"
);

testcase_good!(
    reference_argon2i_v0x13_2_18_1,
    Algorithm::Argon2i,
    Version::V0x13,
    2,
    18,
    1,
    b"password",
    b"somesalt",
    hex!("296dbae80b807cdceaad44ae741b506f14db0959267b183b118f9b24229bc7cb"),
    "$argon2i$v=19$m=262144,t=2,p=1$c29tZXNhbHQ$KW266AuAfNzqrUSudBtQbxTbCVkmexg7EY+bJCKbx8s"
);

testcase_good!(
    reference_argon2i_v0x13_2_8_1,
    Algorithm::Argon2i,
    Version::V0x13,
    2,
    8,
    1,
    b"password",
    b"somesalt",
    hex!("89e9029f4637b295beb027056a7336c414fadd43f6b208645281cb214a56452f"),
    "$argon2i$v=19$m=256,t=2,p=1$c29tZXNhbHQ$iekCn0Y3spW+sCcFanM2xBT63UP2sghkUoHLIUpWRS8"
);

testcase_good!(
    reference_argon2i_v0x13_2_8_2,
    Algorithm::Argon2i,
    Version::V0x13,
    2,
    8,
    2,
    b"password",
    b"somesalt",
    hex!("4ff5ce2769a1d7f4c8a491df09d41a9fbe90e5eb02155a13e4c01e20cd4eab61"),
    "$argon2i$v=19$m=256,t=2,p=2$c29tZXNhbHQ$T/XOJ2mh1/TIpJHfCdQan76Q5esCFVoT5MAeIM1Oq2E"
);

testcase_good!(
    reference_argon2i_v0x13_1_16_1,
    Algorithm::Argon2i,
    Version::V0x13,
    1,
    16,
    1,
    b"password",
    b"somesalt",
    hex!("d168075c4d985e13ebeae560cf8b94c3b5d8a16c51916b6f4ac2da3ac11bbecf"),
    "$argon2i$v=19$m=65536,t=1,p=1$c29tZXNhbHQ$0WgHXE2YXhPr6uVgz4uUw7XYoWxRkWtvSsLaOsEbvs8"
);

testcase_good!(
    reference_argon2i_v0x13_4_16_1,
    Algorithm::Argon2i,
    Version::V0x13,
    4,
    16,
    1,
    b"password",
    b"somesalt",
    hex!("aaa953d58af3706ce3df1aefd4a64a84e31d7f54175231f1285259f88174ce5b"),
    "$argon2i$v=19$m=65536,t=4,p=1$c29tZXNhbHQ$qqlT1YrzcGzj3xrv1KZKhOMdf1QXUjHxKFJZ+IF0zls"
);

testcase_good!(
    reference_argon2i_v0x13_2_16_1_differentpassword,
    Algorithm::Argon2i,
    Version::V0x13,
    2,
    16,
    1,
    b"differentpassword",
    b"somesalt",
    hex!("14ae8da01afea8700c2358dcef7c5358d9021282bd88663a4562f59fb74d22ee"),
    "$argon2i$v=19$m=65536,t=2,p=1$c29tZXNhbHQ$FK6NoBr+qHAMI1jc73xTWNkCEoK9iGY6RWL1n7dNIu4"
);

testcase_good!(
    reference_argon2i_v0x13_2_16_1_diffsalt,
    Algorithm::Argon2i,
    Version::V0x13,
    2,
    16,
    1,
    b"password",
    b"diffsalt",
    hex!("b0357cccfbef91f3860b0dba447b2348cbefecadaf990abfe9cc40726c521271"),
    "$argon2i$v=19$m=65536,t=2,p=1$ZGlmZnNhbHQ$sDV8zPvvkfOGCw26RHsjSMvv7K2vmQq/6cxAcmxSEnE"
);

#[test]
fn reference_argon2i_v0x13_mismatching_hash() {
    /* Handle an mismatching hash (the encoded password is "passwore") */
    let hash = PasswordHash::new(
        "$argon2i$v=19$m=65536,t=2,p=1$c29tZXNhbHQ$8iIuixkI73Js3G1uMbezQXD0b8LG4SXGsOwoQkdAQIM",
    )
    .unwrap();
    assert_eq!(
        Argon2::default().verify_password(b"password", &hash),
        Err(password_hash::errors::Error::Password)
    );
}

/* Argon2id V0x13: Multiple test cases for various input values */

testcase_good!(
    reference_argon2id_v0x13_2_16_1,
    Algorithm::Argon2id,
    Version::V0x13,
    2,
    16,
    1,
    b"password",
    b"somesalt",
    hex!("09316115d5cf24ed5a15a31a3ba326e5cf32edc24702987c02b6566f61913cf7"),
    "$argon2id$v=19$m=65536,t=2,p=1$c29tZXNhbHQ$CTFhFdXPJO1aFaMaO6Mm5c8y7cJHAph8ArZWb2GRPPc"
);

testcase_good!(
    reference_argon2id_v0x13_2_18_1,
    Algorithm::Argon2id,
    Version::V0x13,
    2,
    18,
    1,
    b"password",
    b"somesalt",
    hex!("78fe1ec91fb3aa5657d72e710854e4c3d9b9198c742f9616c2f085bed95b2e8c"),
    "$argon2id$v=19$m=262144,t=2,p=1$c29tZXNhbHQ$eP4eyR+zqlZX1y5xCFTkw9m5GYx0L5YWwvCFvtlbLow"
);

testcase_good!(
    reference_argon2id_v0x13_2_8_1,
    Algorithm::Argon2id,
    Version::V0x13,
    2,
    8,
    1,
    b"password",
    b"somesalt",
    hex!("9dfeb910e80bad0311fee20f9c0e2b12c17987b4cac90c2ef54d5b3021c68bfe"),
    "$argon2id$v=19$m=256,t=2,p=1$c29tZXNhbHQ$nf65EOgLrQMR/uIPnA4rEsF5h7TKyQwu9U1bMCHGi/4"
);

testcase_good!(
    reference_argon2id_v0x13_2_8_2,
    Algorithm::Argon2id,
    Version::V0x13,
    2,
    8,
    2,
    b"password",
    b"somesalt",
    hex!("6d093c501fd5999645e0ea3bf620d7b8be7fd2db59c20d9fff9539da2bf57037"),
    "$argon2id$v=19$m=256,t=2,p=2$c29tZXNhbHQ$bQk8UB/VmZZF4Oo79iDXuL5/0ttZwg2f/5U52iv1cDc"
);

testcase_good!(
    reference_argon2id_v0x13_1_16_1,
    Algorithm::Argon2id,
    Version::V0x13,
    1,
    16,
    1,
    b"password",
    b"somesalt",
    hex!("f6a5adc1ba723dddef9b5ac1d464e180fcd9dffc9d1cbf76cca2fed795d9ca98"),
    "$argon2id$v=19$m=65536,t=1,p=1$c29tZXNhbHQ$9qWtwbpyPd3vm1rB1GThgPzZ3/ydHL92zKL+15XZypg"
);

testcase_good!(
    reference_argon2id_v0x13_4_16_1,
    Algorithm::Argon2id,
    Version::V0x13,
    4,
    16,
    1,
    b"password",
    b"somesalt",
    hex!("9025d48e68ef7395cca9079da4c4ec3affb3c8911fe4f86d1a2520856f63172c"),
    "$argon2id$v=19$m=65536,t=4,p=1$c29tZXNhbHQ$kCXUjmjvc5XMqQedpMTsOv+zyJEf5PhtGiUghW9jFyw"
);

testcase_good!(
    reference_argon2id_v0x13_2_16_1_differentpassword,
    Algorithm::Argon2id,
    Version::V0x13,
    2,
    16,
    1,
    b"differentpassword",
    b"somesalt",
    hex!("0b84d652cf6b0c4beaef0dfe278ba6a80df6696281d7e0d2891b817d8c458fde"),
    "$argon2id$v=19$m=65536,t=2,p=1$c29tZXNhbHQ$C4TWUs9rDEvq7w3+J4umqA32aWKB1+DSiRuBfYxFj94"
);

testcase_good!(
    reference_argon2id_v0x13_2_16_1_diffsalt,
    Algorithm::Argon2id,
    Version::V0x13,
    2,
    16,
    1,
    b"password",
    b"diffsalt",
    hex!("bdf32b05ccc42eb15d58fd19b1f856b113da1e9a5874fdcc544308565aa8141c"),
    "$argon2id$v=19$m=65536,t=2,p=1$ZGlmZnNhbHQ$vfMrBczELrFdWP0ZsfhWsRPaHppYdP3MVEMIVlqoFBw"
);
