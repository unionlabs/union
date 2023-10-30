use digest::{Digest, Update}; // trait
use k256::{
    ecdsa::recoverable,
    ecdsa::signature::{DigestVerifier, Signature as _}, // traits
    ecdsa::{Signature, VerifyingKey},                   // type aliases
    elliptic_curve::sec1::ToEncodedPoint,
};

use crate::errors::{CryptoError, CryptoResult};
use crate::identity_digest::Identity256;

/// Max length of a message hash for secp256k1 verification in bytes.
/// This is typically a 32 byte output of e.g. SHA-256 or Keccak256. In theory shorter values
/// are possible but currently not supported by the implementation. Let us know when you need them.
pub const MESSAGE_HASH_MAX_LEN: usize = 32;

/// ECDSA (secp256k1) parameters
/// Length of a serialized signature
pub const ECDSA_SIGNATURE_LEN: usize = 64;

/// Length of a serialized compressed public key
const ECDSA_COMPRESSED_PUBKEY_LEN: usize = 33;
/// Length of a serialized uncompressed public key
const ECDSA_UNCOMPRESSED_PUBKEY_LEN: usize = 65;
/// Max length of a serialized public key
pub const ECDSA_PUBKEY_MAX_LEN: usize = ECDSA_UNCOMPRESSED_PUBKEY_LEN;

/// ECDSA secp256k1 implementation.
///
/// This function verifies message hashes (typically, hashed unsing SHA-256) against a signature,
/// with the public key of the signer, using the secp256k1 elliptic curve digital signature
/// parametrization / algorithm.
///
/// The signature and public key are in "Cosmos" format:
/// - signature:  Serialized "compact" signature (64 bytes).
/// - public key: [Serialized according to SEC 2](https://www.oreilly.com/library/view/programming-bitcoin/9781492031482/ch04.html)
/// (33 or 65 bytes).
///
/// This implementation accepts both high-S and low-S signatures. Some applications
/// including Ethereum transactions consider high-S signatures invalid in order to
/// avoid malleability. If that's the case for your protocol, the signature needs
/// to be tested for low-S in addition to this verification.
pub fn secp256k1_verify(
    message_hash: &[u8],
    signature: &[u8],
    public_key: &[u8],
) -> CryptoResult<bool> {
    let message_hash = read_hash(message_hash)?;
    let signature = read_signature(signature)?;
    check_pubkey(public_key)?;

    // Already hashed, just build Digest container
    let message_digest = Identity256::new().chain(message_hash);

    let mut signature =
        Signature::from_bytes(&signature).map_err(|e| CryptoError::generic_err(e.to_string()))?;

    // High-S signatures require normalization since our verification implementation
    // rejects them by default. If we had a verifier that does not restrict to
    // low-S only, this step was not needed.
    if let Some(normalized) = signature.normalize_s() {
        signature = normalized;
    }

    let public_key = VerifyingKey::from_sec1_bytes(public_key)
        .map_err(|e| CryptoError::generic_err(e.to_string()))?;

    match public_key.verify_digest(message_digest, &signature) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Recovers a public key from a message hash and a signature.
///
/// This is required when working with Ethereum where public keys
/// are not stored on chain directly.
///
/// `recovery_param` must be 0 or 1. The values 2 and 3 are unsupported by this implementation,
/// which is the same restriction as Ethereum has (https://github.com/ethereum/go-ethereum/blob/v1.9.25/internal/ethapi/api.go#L466-L469).
/// All other values are invalid.
///
/// Returns the recovered pubkey in compressed form, which can be used
/// in secp256k1_verify directly.
///
/// This implementation accepts both high-S and low-S signatures. This is the
/// same behavior as Ethereum's `ecrecover`. The reason is that high-S signatures
/// may be perfectly valid if the application protocol does not disallow them.
/// Or as [EIP-2] put it "The ECDSA recover precompiled contract remains unchanged
/// and will keep accepting high s-values; this is useful e.g. if a contract
/// recovers old Bitcoin signatures.".
///
/// See also OpenZeppelin's [ECDSA.recover implementation](https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.8.1/contracts/utils/cryptography/ECDSA.sol#L138-L149)
/// which adds further restrictions to avoid potential signature malleability.
/// Please note that restricting signatures to low-S does not make signatures unique
/// in the sense that for each (pubkey, message) there is only one signature. The
/// signer can generate an arbitrary amount of valid signatures.
/// <https://medium.com/@simonwarta/signature-determinism-for-blockchain-developers-dbd84865a93e>
///
/// [EIP-2]: https://eips.ethereum.org/EIPS/eip-2
pub fn secp256k1_recover_pubkey(
    message_hash: &[u8],
    signature: &[u8],
    recovery_param: u8,
) -> Result<Vec<u8>, CryptoError> {
    let message_hash = read_hash(message_hash)?;
    let signature = read_signature(signature)?;

    let id =
        recoverable::Id::new(recovery_param).map_err(|_| CryptoError::invalid_recovery_param())?;

    // Compose extended signature
    let signature =
        Signature::from_bytes(&signature).map_err(|e| CryptoError::generic_err(e.to_string()))?;
    let extended_signature = recoverable::Signature::new(&signature, id)
        .map_err(|e| CryptoError::generic_err(e.to_string()))?;

    // Recover
    let message_digest = Identity256::new().chain(message_hash);
    let pubkey = extended_signature
        .recover_verifying_key_from_digest(message_digest)
        .map_err(|e| CryptoError::generic_err(e.to_string()))?;
    let encoded: Vec<u8> = pubkey.to_encoded_point(false).as_bytes().into();
    Ok(encoded)
}

/// Error raised when hash is not 32 bytes long
struct InvalidSecp256k1HashFormat;

impl From<InvalidSecp256k1HashFormat> for CryptoError {
    fn from(_original: InvalidSecp256k1HashFormat) -> Self {
        CryptoError::invalid_hash_format()
    }
}

fn read_hash(data: &[u8]) -> Result<[u8; 32], InvalidSecp256k1HashFormat> {
    data.try_into().map_err(|_| InvalidSecp256k1HashFormat)
}

/// Error raised when signature is not 64 bytes long (32 bytes r, 32 bytes s)
struct InvalidSecp256k1SignatureFormat;

impl From<InvalidSecp256k1SignatureFormat> for CryptoError {
    fn from(_original: InvalidSecp256k1SignatureFormat) -> Self {
        CryptoError::invalid_signature_format()
    }
}

fn read_signature(data: &[u8]) -> Result<[u8; 64], InvalidSecp256k1SignatureFormat> {
    data.try_into().map_err(|_| InvalidSecp256k1SignatureFormat)
}

/// Error raised when public key is not in one of the two supported formats:
/// 1. Uncompressed: 65 bytes starting with 0x04
/// 2. Compressed: 33 bytes starting with 0x02 or 0x03
struct InvalidSecp256k1PubkeyFormat;

impl From<InvalidSecp256k1PubkeyFormat> for CryptoError {
    fn from(_original: InvalidSecp256k1PubkeyFormat) -> Self {
        CryptoError::invalid_pubkey_format()
    }
}

fn check_pubkey(data: &[u8]) -> Result<(), InvalidSecp256k1PubkeyFormat> {
    let ok = match data.first() {
        Some(0x02) | Some(0x03) => data.len() == ECDSA_COMPRESSED_PUBKEY_LEN,
        Some(0x04) => data.len() == ECDSA_UNCOMPRESSED_PUBKEY_LEN,
        _ => false,
    };
    if ok {
        Ok(())
    } else {
        Err(InvalidSecp256k1PubkeyFormat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use hex_literal::hex;
    use k256::{
        ecdsa::signature::DigestSigner, // trait
        ecdsa::SigningKey,              // type alias
        elliptic_curve::rand_core::OsRng,
        elliptic_curve::sec1::ToEncodedPoint,
    };
    use serde::Deserialize;
    use sha2::Sha256;
    use std::fs::File;
    use std::io::BufReader;

    // For generic signature verification
    const MSG: &str = "Hello World!";

    // Cosmos secp256k1 signature verification
    // tendermint/PubKeySecp256k1 pubkey
    const COSMOS_SECP256K1_PUBKEY_BASE64: &str = "A08EGB7ro1ORuFhjOnZcSgwYlpe0DSFjVNUIkNNQxwKQ";

    const COSMOS_SECP256K1_MSG_HEX1: &str = "0a93010a90010a1c2f636f736d6f732e62616e6b2e763162657461312e4d736753656e6412700a2d636f736d6f7331706b707472653766646b6c366766727a6c65736a6a766878686c63337234676d6d6b38727336122d636f736d6f7331717970717870713971637273737a673270767871367273307a716733797963356c7a763778751a100a0575636f736d12073132333435363712650a4e0a460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a21034f04181eeba35391b858633a765c4a0c189697b40d216354d50890d350c7029012040a02080112130a0d0a0575636f736d12043230303010c09a0c1a0c73696d642d74657374696e672001";
    const COSMOS_SECP256K1_MSG_HEX2: &str = "0a93010a90010a1c2f636f736d6f732e62616e6b2e763162657461312e4d736753656e6412700a2d636f736d6f7331706b707472653766646b6c366766727a6c65736a6a766878686c63337234676d6d6b38727336122d636f736d6f7331717970717870713971637273737a673270767871367273307a716733797963356c7a763778751a100a0575636f736d12073132333435363712670a500a460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a21034f04181eeba35391b858633a765c4a0c189697b40d216354d50890d350c7029012040a020801180112130a0d0a0575636f736d12043230303010c09a0c1a0c73696d642d74657374696e672001";
    const COSMOS_SECP256K1_MSG_HEX3: &str = "0a93010a90010a1c2f636f736d6f732e62616e6b2e763162657461312e4d736753656e6412700a2d636f736d6f7331706b707472653766646b6c366766727a6c65736a6a766878686c63337234676d6d6b38727336122d636f736d6f7331717970717870713971637273737a673270767871367273307a716733797963356c7a763778751a100a0575636f736d12073132333435363712670a500a460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a21034f04181eeba35391b858633a765c4a0c189697b40d216354d50890d350c7029012040a020801180212130a0d0a0575636f736d12043230303010c09a0c1a0c73696d642d74657374696e672001";

    const COSMOS_SECP256K1_SIGNATURE_HEX1: &str = "c9dd20e07464d3a688ff4b710b1fbc027e495e797cfa0b4804da2ed117959227772de059808f765aa29b8f92edf30f4c2c5a438e30d3fe6897daa7141e3ce6f9";
    const COSMOS_SECP256K1_SIGNATURE_HEX2: &str = "525adc7e61565a509c60497b798c549fbf217bb5cd31b24cc9b419d098cc95330c99ecc4bc72448f85c365a4e3f91299a3d40412fb3751bab82f1940a83a0a4c";
    const COSMOS_SECP256K1_SIGNATURE_HEX3: &str = "f3f2ca73806f2abbf6e0fe85f9b8af66f0e9f7f79051fdb8abe5bb8633b17da132e82d577b9d5f7a6dae57a144efc9ccc6eef15167b44b3b22a57240109762af";

    // Test data originally from https://github.com/cosmos/cosmjs/blob/v0.24.0-alpha.22/packages/crypto/src/secp256k1.spec.ts#L195-L394
    const COSMOS_SECP256K1_TESTS_JSON: &str = "./testdata/secp256k1_tests.json";

    #[derive(Deserialize, Debug)]
    struct Encoded {
        message: String,
        message_hash: String,
        signature: String,
        #[serde(rename = "pubkey")]
        public_key: String,
    }

    #[test]
    fn test_secp256k1_verify() {
        // Explicit / external hashing
        let message_digest = Sha256::new().chain(MSG);
        let message_hash = message_digest.clone().finalize();

        // Signing
        let secret_key = SigningKey::random(&mut OsRng); // Serialize with `::to_bytes()`

        // Note: the signature type must be annotated or otherwise inferrable as
        // `Signer` has many impls of the `Signer` trait (for both regular and
        // recoverable signature types).
        let signature: Signature = secret_key.sign_digest(message_digest);

        let public_key = VerifyingKey::from(&secret_key); // Serialize with `::to_encoded_point()`

        // Verification (uncompressed public key)
        assert!(secp256k1_verify(
            &message_hash,
            signature.as_bytes(),
            public_key.to_encoded_point(false).as_bytes()
        )
        .unwrap());

        // Verification (compressed public key)
        assert!(secp256k1_verify(
            &message_hash,
            signature.as_bytes(),
            public_key.to_encoded_point(true).as_bytes()
        )
        .unwrap());

        // Wrong message fails
        let bad_message_hash = Sha256::new().chain(MSG).chain("\0").finalize();
        assert!(!secp256k1_verify(
            &bad_message_hash,
            signature.as_bytes(),
            public_key.to_encoded_point(false).as_bytes()
        )
        .unwrap());

        // Other pubkey fails
        let other_secret_key = SigningKey::random(&mut OsRng);
        let other_public_key = VerifyingKey::from(&other_secret_key);
        assert!(!secp256k1_verify(
            &message_hash,
            signature.as_bytes(),
            other_public_key.to_encoded_point(false).as_bytes()
        )
        .unwrap());
    }

    #[test]
    fn test_cosmos_secp256k1_verify() {
        let public_key = base64::decode(COSMOS_SECP256K1_PUBKEY_BASE64).unwrap();

        for ((i, msg), sig) in (1..)
            .zip(&[
                COSMOS_SECP256K1_MSG_HEX1,
                COSMOS_SECP256K1_MSG_HEX2,
                COSMOS_SECP256K1_MSG_HEX3,
            ])
            .zip(&[
                COSMOS_SECP256K1_SIGNATURE_HEX1,
                COSMOS_SECP256K1_SIGNATURE_HEX2,
                COSMOS_SECP256K1_SIGNATURE_HEX3,
            ])
        {
            let message = hex::decode(msg).unwrap();
            let signature = hex::decode(sig).unwrap();

            // Explicit hash
            let message_hash = Sha256::digest(message);

            // secp256k1_verify works
            let valid = secp256k1_verify(&message_hash, &signature, &public_key).unwrap();
            assert!(valid, "secp256k1_verify() failed (test case {i})",);
        }
    }

    #[test]
    fn test_cosmos_extra_secp256k1_verify() {
        // Open the file in read-only mode with buffer.
        let file = File::open(COSMOS_SECP256K1_TESTS_JSON).unwrap();
        let reader = BufReader::new(file);

        let codes: Vec<Encoded> = serde_json::from_reader(reader).unwrap();

        for (i, encoded) in (1..).zip(codes) {
            let message = hex::decode(&encoded.message).unwrap();
            let signature = hex::decode(&encoded.signature).unwrap();
            let public_key = hex::decode(&encoded.public_key).unwrap();

            let hash = hex::decode(&encoded.message_hash).unwrap();
            let message_hash = Sha256::digest(message);
            assert_eq!(hash.as_slice(), message_hash.as_slice());

            // secp256k1_verify() works
            let valid = secp256k1_verify(&message_hash, &signature, &public_key).unwrap();
            assert!(
                valid,
                "secp256k1_verify failed (test case {i} in {COSMOS_SECP256K1_TESTS_JSON})"
            );
        }
    }

    #[test]
    fn secp256k1_recover_pubkey_works() {
        // Test data from https://github.com/ethereumjs/ethereumjs-util/blob/v6.1.0/test/index.js#L496
        {
            let private_key =
                hex!("3c9229289a6125f7fdf1885a77bb12c37a8d3b4962d936f7e3084dece32a3ca1");
            let expected = SigningKey::from_bytes(&private_key)
                .unwrap()
                .verifying_key()
                .to_encoded_point(false)
                .as_bytes()
                .to_vec();
            let r_s = hex!("99e71a99cb2270b8cac5254f9e99b6210c6c10224a1579cf389ef88b20a1abe9129ff05af364204442bdb53ab6f18a99ab48acc9326fa689f228040429e3ca66");
            let recovery_param: u8 = 0;
            let message_hash =
                hex!("82ff40c0a986c6a5cfad4ddf4c3aa6996f1a7837f9c398e17e5de5cbd5a12b28");
            let pubkey = secp256k1_recover_pubkey(&message_hash, &r_s, recovery_param).unwrap();
            assert_eq!(pubkey, expected);
        }

        // Test data from https://github.com/randombit/botan/blob/2.9.0/src/tests/data/pubkey/ecdsa_key_recovery.vec
        // This is a high-s value (`0x81F1A4457589F30D76AB9F89E748A68C8A94C30FE0BAC8FB5C0B54EA70BF6D2F > 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0` is true)
        {
            let expected_x = "F3F8BB913AA68589A2C8C607A877AB05252ADBD963E1BE846DDEB8456942AEDC";
            let expected_y = "A2ED51F08CA3EF3DAC0A7504613D54CD539FC1B3CBC92453CD704B6A2D012B2C";
            let expected = hex::decode(format!("04{}{}", expected_x, expected_y)).unwrap();
            let r_s = hex!("E30F2E6A0F705F4FB5F8501BA79C7C0D3FAC847F1AD70B873E9797B17B89B39081F1A4457589F30D76AB9F89E748A68C8A94C30FE0BAC8FB5C0B54EA70BF6D2F");
            let recovery_param: u8 = 0;
            let message_hash =
                hex!("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
            let pubkey = secp256k1_recover_pubkey(&message_hash, &r_s, recovery_param).unwrap();
            assert_eq!(pubkey, expected);
        }

        // Test data calculated via Secp256k1.createSignature from @cosmjs/crypto
        {
            let expected = hex!("044a071e8a6e10aada2b8cf39fa3b5fb3400b04e99ea8ae64ceea1a977dbeaf5d5f8c8fbd10b71ab14cd561f7df8eb6da50f8a8d81ba564342244d26d1d4211595");
            let r_s = hex!("45c0b7f8c09a9e1f1cea0c25785594427b6bf8f9f878a8af0b1abbb48e16d0920d8becd0c220f67c51217eecfd7184ef0732481c843857e6bc7fc095c4f6b788");
            let recovery_param: u8 = 1;
            let message_hash =
                hex!("5ae8317d34d1e595e3fa7247db80c0af4320cce1116de187f8f7e2e099c0d8d0");
            let pubkey = secp256k1_recover_pubkey(&message_hash, &r_s, recovery_param).unwrap();
            assert_eq!(pubkey, expected);
        }

        let file = File::open(COSMOS_SECP256K1_TESTS_JSON).unwrap();
        let reader = BufReader::new(file);
        let codes: Vec<Encoded> = serde_json::from_reader(reader).unwrap();
        for (i, encoded) in (1..).zip(codes) {
            let message = hex::decode(&encoded.message).unwrap();
            let signature = hex::decode(&encoded.signature).unwrap();
            let public_key = hex::decode(&encoded.public_key).unwrap();

            let hash = hex::decode(&encoded.message_hash).unwrap();
            let message_hash = Sha256::digest(message);
            assert_eq!(hash.as_slice(), message_hash.as_slice());

            // Since the recovery param is missing in the test vectors, we try both 0 and 1
            let try0 = secp256k1_recover_pubkey(&message_hash, &signature, 0);
            let try1 = secp256k1_recover_pubkey(&message_hash, &signature, 1);
            match (try0, try1) {
                (Ok(recovered0), Ok(recovered1)) => {
                    // Got two different pubkeys. Without the recovery param, we don't know which one is the right one.
                    assert!(recovered0 == public_key || recovered1 == public_key)
                },
                (Ok(recovered), Err(_)) => assert_eq!(recovered, public_key),
                (Err(_), Ok(recovered)) => assert_eq!(recovered, public_key),
                (Err(_), Err(_)) => panic!("secp256k1_recover_pubkey failed (test case {i} in {COSMOS_SECP256K1_TESTS_JSON})"),
            }
        }
    }

    #[test]
    fn secp256k1_recover_pubkey_fails_for_invalid_recovery_param() {
        let r_s = hex!("45c0b7f8c09a9e1f1cea0c25785594427b6bf8f9f878a8af0b1abbb48e16d0920d8becd0c220f67c51217eecfd7184ef0732481c843857e6bc7fc095c4f6b788");
        let message_hash = hex!("5ae8317d34d1e595e3fa7247db80c0af4320cce1116de187f8f7e2e099c0d8d0");

        // 2 and 3 are explicitly unsupported
        let recovery_param: u8 = 2;
        match secp256k1_recover_pubkey(&message_hash, &r_s, recovery_param).unwrap_err() {
            CryptoError::InvalidRecoveryParam { .. } => {}
            err => panic!("Unexpected error: {}", err),
        }
        let recovery_param: u8 = 3;
        match secp256k1_recover_pubkey(&message_hash, &r_s, recovery_param).unwrap_err() {
            CryptoError::InvalidRecoveryParam { .. } => {}
            err => panic!("Unexpected error: {}", err),
        }

        // Other values are garbage
        let recovery_param: u8 = 4;
        match secp256k1_recover_pubkey(&message_hash, &r_s, recovery_param).unwrap_err() {
            CryptoError::InvalidRecoveryParam { .. } => {}
            err => panic!("Unexpected error: {}", err),
        }
        let recovery_param: u8 = 255;
        match secp256k1_recover_pubkey(&message_hash, &r_s, recovery_param).unwrap_err() {
            CryptoError::InvalidRecoveryParam { .. } => {}
            err => panic!("Unexpected error: {}", err),
        }
    }
}
