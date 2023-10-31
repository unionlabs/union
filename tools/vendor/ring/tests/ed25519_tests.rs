// Copyright 2015-2017 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

use ring::{
    error,
    signature::{self, Ed25519KeyPair, KeyPair},
    test, test_file,
};

/// Test vectors from BoringSSL.
#[test]
fn test_signature_ed25519() {
    test::run(test_file!("ed25519_tests.txt"), |section, test_case| {
        assert_eq!(section, "");
        let seed = test_case.consume_bytes("SEED");
        assert_eq!(32, seed.len());

        let public_key = test_case.consume_bytes("PUB");
        assert_eq!(32, public_key.len());

        let msg = test_case.consume_bytes("MESSAGE");

        let expected_sig = test_case.consume_bytes("SIG");

        {
            let key_pair = Ed25519KeyPair::from_seed_and_public_key(&seed, &public_key).unwrap();
            let actual_sig = key_pair.sign(&msg);
            assert_eq!(&expected_sig[..], actual_sig.as_ref());
        }

        // Test PKCS#8 generation, parsing, and private-to-public calculations.
        let rng = test::rand::FixedSliceRandom { bytes: &seed };
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
        assert_eq!(public_key, key_pair.public_key().as_ref());

        // Test Signature generation.
        let actual_sig = key_pair.sign(&msg);
        assert_eq!(&expected_sig[..], actual_sig.as_ref());

        // Test Signature verification.
        test_signature_verification(&public_key, &msg, &expected_sig, Ok(()));

        let mut tampered_sig = expected_sig;
        tampered_sig[0] ^= 1;

        test_signature_verification(&public_key, &msg, &tampered_sig, Err(error::Unspecified));

        Ok(())
    });
}

/// Test vectors from BoringSSL.
#[test]
fn test_signature_ed25519_verify() {
    test::run(
        test_file!("ed25519_verify_tests.txt"),
        |section, test_case| {
            assert_eq!(section, "");

            let public_key = test_case.consume_bytes("PUB");
            let msg = test_case.consume_bytes("MESSAGE");
            let sig = test_case.consume_bytes("SIG");
            let expected_result = match test_case.consume_string("Result").as_str() {
                "P" => Ok(()),
                "F" => Err(error::Unspecified),
                s => panic!("{:?} is not a valid result", s),
            };
            test_signature_verification(&public_key, &msg, &sig, expected_result);
            Ok(())
        },
    );
}

fn test_signature_verification(
    public_key: &[u8],
    msg: &[u8],
    sig: &[u8],
    expected_result: Result<(), error::Unspecified>,
) {
    assert_eq!(
        expected_result,
        signature::UnparsedPublicKey::new(&signature::ED25519, public_key).verify(msg, sig)
    );
}

#[test]
fn test_ed25519_from_seed_and_public_key_misuse() {
    const PRIVATE_KEY: &[u8] = include_bytes!("ed25519_test_private_key.bin");
    const PUBLIC_KEY: &[u8] = include_bytes!("ed25519_test_public_key.bin");

    assert!(Ed25519KeyPair::from_seed_and_public_key(PRIVATE_KEY, PUBLIC_KEY).is_ok());

    // Truncated private key.
    assert!(Ed25519KeyPair::from_seed_and_public_key(&PRIVATE_KEY[..31], PUBLIC_KEY).is_err());

    // Truncated public key.
    assert!(Ed25519KeyPair::from_seed_and_public_key(PRIVATE_KEY, &PUBLIC_KEY[..31]).is_err());

    // Swapped public and private key.
    assert!(Ed25519KeyPair::from_seed_and_public_key(PUBLIC_KEY, PRIVATE_KEY).is_err());
}

#[test]
fn test_ed25519_from_pkcs8_unchecked() {
    // Just test that we can parse the input.
    test::run(
        test_file!("ed25519_from_pkcs8_unchecked_tests.txt"),
        |section, test_case| {
            assert_eq!(section, "");
            let input = test_case.consume_bytes("Input");
            let error = test_case.consume_optional_string("Error");

            match (Ed25519KeyPair::from_pkcs8_maybe_unchecked(&input), error) {
                (Ok(_), None) => (),
                (Err(e), None) => panic!("Failed with error \"{}\", but expected to succeed", e),
                (Ok(_), Some(e)) => panic!("Succeeded, but expected error \"{}\"", e),
                (Err(actual), Some(expected)) => assert_eq!(actual.description_(), expected),
            };

            Ok(())
        },
    );
}

#[test]
fn test_ed25519_from_pkcs8() {
    // Just test that we can parse the input.
    test::run(
        test_file!("ed25519_from_pkcs8_tests.txt"),
        |section, test_case| {
            assert_eq!(section, "");
            let input = test_case.consume_bytes("Input");
            let error = test_case.consume_optional_string("Error");

            match (Ed25519KeyPair::from_pkcs8(&input), error) {
                (Ok(_), None) => (),
                (Err(e), None) => panic!("Failed with error \"{}\", but expected to succeed", e),
                (Ok(_), Some(e)) => panic!("Succeeded, but expected error \"{}\"", e),
                (Err(actual), Some(expected)) => assert_eq!(actual.description_(), expected),
            };

            Ok(())
        },
    );
}

#[test]
fn ed25519_test_public_key_coverage() {
    const PRIVATE_KEY: &[u8] = include_bytes!("ed25519_test_private_key.p8");
    const PUBLIC_KEY: &[u8] = include_bytes!("ed25519_test_public_key.der");
    const PUBLIC_KEY_DEBUG: &str =
        "PublicKey(\"5809e9fef6dcec58f0f2e3b0d67e9880a11957e083ace85835c3b6c8fbaf6b7d\")";

    let key_pair = signature::Ed25519KeyPair::from_pkcs8(PRIVATE_KEY).unwrap();

    // Test `AsRef<[u8]>`
    assert_eq!(key_pair.public_key().as_ref(), PUBLIC_KEY);

    // Test `Clone`.
    #[allow(clippy::clone_on_copy)]
    let _: <Ed25519KeyPair as KeyPair>::PublicKey = key_pair.public_key().clone();

    // Test `Copy`.
    let _: <Ed25519KeyPair as KeyPair>::PublicKey = *key_pair.public_key();

    // Test `Debug`.
    assert_eq!(PUBLIC_KEY_DEBUG, format!("{:?}", key_pair.public_key()));
    assert_eq!(
        format!(
            "Ed25519KeyPair {{ public_key: {:?} }}",
            key_pair.public_key()
        ),
        format!("{:?}", key_pair)
    );
}
