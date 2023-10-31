//! Test vectors for Argon2 password hashes in the PHC string format
//!
//! Adapted from: <https://github.com/P-H-C/phc-winner-argon2/blob/master/src/test.c>

#![cfg(all(feature = "alloc", feature = "password-hash"))]

use argon2::{
    Algorithm, Argon2, ParamsBuilder, PasswordHash, PasswordHasher, PasswordVerifier, Version,
};
use password_hash::{
    errors::{Error, InvalidValue},
    SaltString,
};

/// Valid password
pub const VALID_PASSWORD: &[u8] = b"password";

/// Invalid password
pub const INVALID_PASSWORD: &[u8] = b"sassword";

/// Password hashes for "password"
pub const VALID_PASSWORD_HASHES: &[&str] = &[
    "$argon2i$v=19$m=65536,t=1,p=1$c29tZXNhbHQAAAAAAAAAAA$+r0d29hqEB0yasKr55ZgICsQGSkl0v0kgwhd+U3wyRo",
    "$argon2id$v=19$m=262144,t=2,p=1$c29tZXNhbHQ$eP4eyR+zqlZX1y5xCFTkw9m5GYx0L5YWwvCFvtlbLow",
    "$argon2id$v=19$m=65536,t=2,p=1$c29tZXNhbHQ$CTFhFdXPJO1aFaMaO6Mm5c8y7cJHAph8ArZWb2GRPPc",
    "$argon2d$v=19$m=65536,t=2,p=1$YzI5dFpYTmhiSFFBQUFBQUFBQUFBQQ$Jxy74cswY2mq9y+u+iJcJy8EqOp4t/C7DWDzGwGB3IM",
    // Password with optional keyid
    "$argon2d$v=19$m=65536,t=2,p=1,keyid=8PDw8A$YzI5dFpYTmhiSFFBQUFBQUFBQUFBQQ$Jxy74cswY2mq9y+u+iJcJy8EqOp4t/C7DWDzGwGB3IM",
    // Password with optional data
    "$argon2d$v=16$m=32,t=2,p=3,data=Dw8PDw8P$AAAAAAAAAAA$KnH4gniiaFnDvlA1xev3yovC4cnrrI6tnHOYtmja90o",
    // Password with optional keyid&data
    "$argon2d$v=16$m=32,t=2,p=3,keyid=8PDw8A,data=Dw8PDw8P$AAAAAAAAAAA$KnH4gniiaFnDvlA1xev3yovC4cnrrI6tnHOYtmja90o",
];

#[test]
fn verifies_correct_password() {
    for hash_string in VALID_PASSWORD_HASHES {
        let hash = PasswordHash::new(hash_string).unwrap();
        assert_eq!(
            Argon2::default().verify_password(VALID_PASSWORD, &hash),
            Ok(())
        );
    }
}

#[test]
fn rejects_incorrect_password() {
    for hash_string in VALID_PASSWORD_HASHES {
        let hash = PasswordHash::new(hash_string).unwrap();
        assert!(Argon2::default()
            .verify_password(INVALID_PASSWORD, &hash)
            .is_err());
    }
}

// Test PHC string format according to spec
// see: https://github.com/P-H-C/phc-string-format/blob/master/phc-sf-spec.md#argon2-encoding

macro_rules! testcase_bad_encoding {
    ($name: ident, $err: expr, $hash: expr) => {
        #[test]
        fn $name() {
            let hash = PasswordHash::new($hash).unwrap();
            assert_eq!(
                Argon2::default().verify_password(b"password", &hash),
                Err($err)
            );
        }
    };
}

macro_rules! ignored_testcase_bad_encoding {
    ($name: ident, $err: expr, $hash: expr) => {
        #[test]
        #[ignore]
        fn $name() {
            let hash = PasswordHash::new($hash).unwrap();
            assert_eq!(
                Argon2::default().verify_password(b"password", &hash),
                Err($err)
            );
        }
    };
}

testcase_bad_encoding!(
    argon2i_invalid_encoding_invalid_version,
    Error::Version,
    "$argon2i$v=42$m=65536,t=2,p=1$c29tZXNhbHQ$9sTbSlTio3Biev89thdrlKKiCaYsjjYVJxGAL3swxpQ"
);
testcase_bad_encoding!(
    argon2i_invalid_encoding_m_not_a_number,
    Error::ParamValueInvalid(InvalidValue::InvalidChar('d')),
    "$argon2i$m=dummy,t=2,p=1$c29tZXNhbHQ$9sTbSlTio3Biev89thdrlKKiCaYsjjYVJxGAL3swxpQ"
);
testcase_bad_encoding!(
    argon2i_invalid_encoding_m_too_small,
    Error::ParamValueInvalid(InvalidValue::TooShort),
    "$argon2i$m=0,t=2,p=1$c29tZXNhbHQ$9sTbSlTio3Biev89thdrlKKiCaYsjjYVJxGAL3swxpQ"
);
testcase_bad_encoding!(
    argon2i_invalid_encoding_m_too_big,
    Error::ParamValueInvalid(InvalidValue::InvalidFormat),
    "$argon2i$m=4294967296,t=2,p=1$c29tZXNhbHQ$9sTbSlTio3Biev89thdrlKKiCaYsjjYVJxGAL3swxpQ"
);

testcase_bad_encoding!(
    argon2i_invalid_encoding_t_not_a_number,
    Error::ParamValueInvalid(InvalidValue::InvalidChar('d')),
    "$argon2i$m=65536,t=dummy,p=1$c29tZXNhbHQ$9sTbSlTio3Biev89thdrlKKiCaYsjjYVJxGAL3swxpQ"
);
testcase_bad_encoding!(
    argon2i_invalid_encoding_t_too_small,
    Error::ParamValueInvalid(InvalidValue::TooShort),
    "$argon2i$m=65536,t=0,p=1$c29tZXNhbHQ$9sTbSlTio3Biev89thdrlKKiCaYsjjYVJxGAL3swxpQ"
);
testcase_bad_encoding!(
    argon2i_invalid_encoding_t_too_big,
    Error::ParamValueInvalid(InvalidValue::InvalidFormat),
    "$argon2i$m=65536,t=4294967296,p=1$c29tZXNhbHQ$9sTbSlTio3Biev89thdrlKKiCaYsjjYVJxGAL3swxpQ"
);

testcase_bad_encoding!(
    argon2i_invalid_encoding_p_not_a_number,
    Error::ParamValueInvalid(InvalidValue::InvalidChar('d')),
    "$argon2i$m=65536,t=2,p=dummy$c29tZXNhbHQ$9sTbSlTio3Biev89thdrlKKiCaYsjjYVJxGAL3swxpQ"
);
testcase_bad_encoding!(
    argon2i_invalid_encoding_p_too_small,
    Error::ParamValueInvalid(InvalidValue::TooShort),
    "$argon2i$m=65536,t=2,p=0$c29tZXNhbHQ$9sTbSlTio3Biev89thdrlKKiCaYsjjYVJxGAL3swxpQ"
);
// TODO: Wrong error is returned, this should be changed in the `password-hash` crate
ignored_testcase_bad_encoding!(
    argon2i_invalid_encoding_p_too_big,
    Error::ParamValueInvalid(InvalidValue::InvalidFormat),
    "$argon2i$m=65536,t=2,p=256$c29tZXNhbHQ$9sTbSlTio3Biev89thdrlKKiCaYsjjYVJxGAL3swxpQ"
);

testcase_bad_encoding!(
    argon2i_invalid_encoding_keyid_not_b64,
    Error::B64Encoding(base64ct::Error::InvalidEncoding),
    "$argon2i$m=65536,t=2,p=1,keyid=dummy$c29tZXNhbHQ$9sTbSlTio3Biev89thdrlKKiCaYsjjYVJxGAL3swxpQ"
);
testcase_bad_encoding!(
    argon2i_invalid_encoding_data_not_b64,
    Error::B64Encoding(base64ct::Error::InvalidEncoding),
    "$argon2i$m=65536,t=2,p=1,data=dummy$c29tZXNhbHQ$9sTbSlTio3Biev89thdrlKKiCaYsjjYVJxGAL3swxpQ"
);

#[test]
fn check_decoding_supports_out_of_order_parameters() {
    // parameters order is not mandatory
    let hash = "$argon2d$v=16$m=32,t=2,p=3,keyid=8PDw8A,data=Dw8PDw8P$AAAAAAAAAAA$KnH4gniiaFnDvlA1xev3yovC4cnrrI6tnHOYtmja90o";
    let hash = PasswordHash::new(hash).unwrap();
    assert!(Argon2::default()
        .verify_password(b"password", &hash)
        .is_ok());
}

// TODO: Fix default parameters for decoding !
#[test]
#[ignore]
fn check_decoding_supports_default_parameters() {
    // parameters order is not mandatory
    let hash = "$argon2i$p=2,m=256,t=2$c29tZXNhbHQ$tsEVYKap1h6scGt5ovl9aLRGOqOth+AMB+KwHpDFZPs";
    let hash = PasswordHash::new(hash).unwrap();
    assert!(Argon2::default()
        .verify_password(b"password", &hash)
        .is_ok());
}

// m/t/p parameters are NOT optional according to spec

// TODO: Wrong error is returned, this should be changed in the `password-hash` crate
ignored_testcase_bad_encoding!(
    argon2i_invalid_encoding_missing_m,
    Error::ParamValueInvalid(InvalidValue::InvalidFormat),
    "$argon2d$v=16$t=2,p=3$8PDw8PDw8PA$Xv5daH0zPuKO3c9tMBG/WOIUsDrPqq815/xyQTukNxY"
);
// TODO: Wrong error is returned, this should be changed in the `password-hash` crate
ignored_testcase_bad_encoding!(
    argon2i_invalid_encoding_missing_t,
    Error::ParamValueInvalid(InvalidValue::InvalidFormat),
    "$argon2d$v=16$m=32,p=3$8PDw8PDw8PA$Xv5daH0zPuKO3c9tMBG/WOIUsDrPqq815/xyQTukNxY"
);
// TODO: Wrong error is returned, this should be changed in the `password-hash` crate
ignored_testcase_bad_encoding!(
    argon2i_invalid_encoding_missing_p,
    Error::ParamValueInvalid(InvalidValue::InvalidFormat),
    "$argon2d$v=16$m=32,t=2$8PDw8PDw8PA$Xv5daH0zPuKO3c9tMBG/WOIUsDrPqq815/xyQTukNxY"
);

// Missing&invalid id/salt/hash fields is handled by `PasswordHash` so no need to test that here

#[test]
fn check_hash_encoding_parameters_order() {
    let mut builder = ParamsBuilder::new();
    builder.m_cost(32).unwrap();
    builder.t_cost(2).unwrap();
    builder.p_cost(3).unwrap();
    builder.data(&[0x0f; 6]).unwrap();
    builder.keyid(&[0xf0; 4]).unwrap();
    let ctx = Argon2::new(
        Algorithm::Argon2d,
        Version::V0x10,
        builder.params().unwrap(),
    );

    let salt = vec![0; 8];
    let password = b"password";
    let salt_string = SaltString::b64_encode(&salt).unwrap();
    let password_hash = ctx
        .hash_password(password, &salt_string)
        .unwrap()
        .to_string();

    // The parameters shall appear in the m,t,p,keyid,data order
    assert_eq!(password_hash, "$argon2d$v=16$m=32,t=2,p=3,keyid=8PDw8A,data=Dw8PDw8P$AAAAAAAAAAA$KnH4gniiaFnDvlA1xev3yovC4cnrrI6tnHOYtmja90o");
}
