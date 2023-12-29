use ed25519_zebra::{batch, Signature, VerificationKey};
use rand_core::OsRng;

use crate::errors::{CryptoError, CryptoResult};

/// Length of a serialized public key
pub const EDDSA_PUBKEY_LEN: usize = 32;

/// EdDSA ed25519 implementation.
///
/// This function verifies messages against a signature, with the public key of the signer,
/// using the ed25519 elliptic curve digital signature parametrization / algorithm.
///
/// The maximum currently supported message length is 4096 bytes.
/// The signature and public key are in [Tendermint](https://docs.tendermint.com/v0.32/spec/blockchain/encoding.html#public-key-cryptography)
/// format:
/// - signature: raw ED25519 signature (64 bytes).
/// - public key: raw ED25519 public key (32 bytes).
pub fn ed25519_verify(message: &[u8], signature: &[u8], public_key: &[u8]) -> CryptoResult<bool> {
    // Validation
    let signature = read_signature(signature)?;
    let pubkey = read_pubkey(public_key)?;

    // Verification
    match VerificationKey::try_from(pubkey)
        .and_then(|vk| vk.verify(&Signature::from(signature), message))
    {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Performs batch Ed25519 signature verification.
///
/// Batch verification asks whether all signatures in some set are valid, rather than asking whether
/// each of them is valid. This allows sharing computations among all signature verifications,
/// performing less work overall, at the cost of higher latency (the entire batch must complete),
/// complexity of caller code (which must assemble a batch of signatures across work-items),
/// and loss of the ability to easily pinpoint failing signatures.
///
/// This batch verification implementation is adaptive, in the sense that it detects multiple
/// signatures created with the same verification key, and automatically coalesces terms
/// in the final verification equation.
///
/// In the limiting case where all signatures in the batch are made with the same verification key,
/// coalesced batch verification runs twice as fast as ordinary batch verification.
///
/// Three Variants are suppported in the input for convenience:
///  - Equal number of messages, signatures, and public keys: Standard, generic functionality.
///  - One message, and an equal number of signatures and public keys: Multiple digital signature
/// (multisig) verification of a single message.
///  - One public key, and an equal number of messages and signatures: Verification of multiple
/// messages, all signed with the same private key.
///
/// Any other variants of input vectors result in an error.
///
/// Notes:
///  - The "one-message, with zero signatures and zero public keys" case, is considered the empty
/// case.
///  - The "one-public key, with zero messages and zero signatures" case, is considered the empty
/// case.
///  - The empty case (no messages, no signatures and no public keys) returns true.
pub fn ed25519_batch_verify(
    messages: &[&[u8]],
    signatures: &[&[u8]],
    public_keys: &[&[u8]],
) -> CryptoResult<bool> {
    // Structural checks
    let messages_len = messages.len();
    let signatures_len = signatures.len();
    let public_keys_len = public_keys.len();

    let mut messages = messages.to_vec();
    let mut public_keys = public_keys.to_vec();
    if messages_len == signatures_len && messages_len == public_keys_len { // We're good to go
    } else if messages_len == 1 && signatures_len == public_keys_len {
        // Replicate message, for multisig
        messages = messages.repeat(signatures_len);
    } else if public_keys_len == 1 && messages_len == signatures_len {
        // Replicate pubkey
        public_keys = public_keys.repeat(messages_len);
    } else {
        return Err(CryptoError::batch_err(
            "Mismatched / erroneous number of messages / signatures / public keys",
        ));
    }
    debug_assert_eq!(messages.len(), signatures_len);
    debug_assert_eq!(messages.len(), public_keys.len());

    let mut batch = batch::Verifier::new();

    for ((&message, &signature), &public_key) in messages
        .iter()
        .zip(signatures.iter())
        .zip(public_keys.iter())
    {
        // Validation
        let signature = read_signature(signature)?;
        let pubkey = read_pubkey(public_key)?;

        // Enqueing
        batch.queue((pubkey.into(), signature.into(), message));
    }

    // Batch verification
    match batch.verify(OsRng) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Error raised when signature is not 64 bytes long
struct InvalidEd25519SignatureFormat;

impl From<InvalidEd25519SignatureFormat> for CryptoError {
    fn from(_original: InvalidEd25519SignatureFormat) -> Self {
        CryptoError::invalid_signature_format()
    }
}

fn read_signature(data: &[u8]) -> Result<[u8; 64], InvalidEd25519SignatureFormat> {
    data.try_into().map_err(|_| InvalidEd25519SignatureFormat)
}

/// Error raised when pubkey is not 32 bytes long
struct InvalidEd25519PubkeyFormat;

impl From<InvalidEd25519PubkeyFormat> for CryptoError {
    fn from(_original: InvalidEd25519PubkeyFormat) -> Self {
        CryptoError::invalid_pubkey_format()
    }
}

fn read_pubkey(data: &[u8]) -> Result<[u8; 32], InvalidEd25519PubkeyFormat> {
    data.try_into().map_err(|_| InvalidEd25519PubkeyFormat)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_zebra::SigningKey;
    use serde::Deserialize;

    // For generic signature verification
    const MSG: &str = "Hello World!";

    // Cosmos ed25519 signature verification
    // TEST 1 from https://tools.ietf.org/html/rfc8032#section-7.1
    const COSMOS_ED25519_MSG: &str = "";
    const COSMOS_ED25519_PRIVATE_KEY_HEX: &str =
        "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
    const COSMOS_ED25519_PUBLIC_KEY_HEX: &str =
        "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
    const COSMOS_ED25519_SIGNATURE_HEX: &str = "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b";

    // Test data from https://tools.ietf.org/html/rfc8032#section-7.1
    const COSMOS_ED25519_TESTS_JSON: &str = "./testdata/ed25519_tests.json";

    #[derive(Deserialize, Debug)]
    struct Encoded {
        #[serde(rename = "privkey")]
        #[allow(dead_code)]
        private_key: String,
        #[serde(rename = "pubkey")]
        public_key: String,
        message: String,
        signature: String,
    }

    fn read_cosmos_sigs() -> Vec<Encoded> {
        use std::fs::File;
        use std::io::BufReader;

        // Open the file in read-only mode with buffer.
        let file = File::open(COSMOS_ED25519_TESTS_JSON).unwrap();
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).unwrap()
    }

    #[test]
    fn test_ed25519_verify() {
        let message = MSG.as_bytes();
        // Signing
        let secret_key = SigningKey::new(OsRng);
        let signature = secret_key.sign(message);

        let public_key = VerificationKey::from(&secret_key);

        // Serialization. Types can be converted to raw byte arrays with From/Into
        let signature_bytes: [u8; 64] = signature.into();
        let public_key_bytes: [u8; 32] = public_key.into();

        // Verification
        assert!(ed25519_verify(message, &signature_bytes, &public_key_bytes).unwrap());

        // Wrong message fails
        let bad_message = [message, b"\0"].concat();
        assert!(!ed25519_verify(&bad_message, &signature_bytes, &public_key_bytes).unwrap());

        // Other pubkey fails
        let other_secret_key = SigningKey::new(OsRng);
        let other_public_key = VerificationKey::from(&other_secret_key);
        let other_public_key_bytes: [u8; 32] = other_public_key.into();
        assert!(!ed25519_verify(message, &signature_bytes, &other_public_key_bytes).unwrap());
    }

    #[test]
    fn test_cosmos_ed25519_verify() {
        let secret_key = SigningKey::try_from(
            hex::decode(COSMOS_ED25519_PRIVATE_KEY_HEX)
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        let public_key = VerificationKey::try_from(
            hex::decode(COSMOS_ED25519_PUBLIC_KEY_HEX)
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        let signature = secret_key.sign(COSMOS_ED25519_MSG.as_bytes());

        let signature_bytes: [u8; 64] = signature.into();
        let public_key_bytes: [u8; 32] = public_key.into();

        assert_eq!(
            signature_bytes,
            hex::decode(COSMOS_ED25519_SIGNATURE_HEX)
                .unwrap()
                .as_slice()
        );

        assert!(ed25519_verify(
            COSMOS_ED25519_MSG.as_bytes(),
            &signature_bytes,
            &public_key_bytes
        )
        .unwrap());
    }

    #[test]
    fn test_cosmos_extra_ed25519_verify() {
        let codes = read_cosmos_sigs();

        for (i, encoded) in (1..).zip(codes) {
            let message = hex::decode(&encoded.message).unwrap();

            let signature = hex::decode(&encoded.signature).unwrap();

            let public_key = hex::decode(&encoded.public_key).unwrap();

            // ed25519_verify() works
            assert!(
                ed25519_verify(&message, &signature, &public_key).unwrap(),
                "verify() failed (test case {})",
                i
            );
        }
    }

    #[test]
    fn test_cosmos_ed25519_batch_verify() {
        let codes = read_cosmos_sigs();

        let mut messages: Vec<Vec<u8>> = vec![];
        let mut signatures: Vec<Vec<u8>> = vec![];
        let mut public_keys: Vec<Vec<u8>> = vec![];

        for encoded in codes {
            let message = hex::decode(&encoded.message).unwrap();
            messages.push(message);

            let signature = hex::decode(&encoded.signature).unwrap();
            signatures.push(signature);

            let public_key = hex::decode(&encoded.public_key).unwrap();
            public_keys.push(public_key);
        }

        let messages: Vec<&[u8]> = messages.iter().map(|m| m.as_slice()).collect();
        let signatures: Vec<&[u8]> = signatures.iter().map(|m| m.as_slice()).collect();
        let public_keys: Vec<&[u8]> = public_keys.iter().map(|m| m.as_slice()).collect();

        // ed25519_batch_verify() works
        assert!(ed25519_batch_verify(&messages, &signatures, &public_keys).unwrap());
    }

    // structural tests
    #[test]
    fn test_cosmos_ed25519_batch_verify_empty_works() {
        let messages: Vec<&[u8]> = vec![];
        let signatures: Vec<&[u8]> = vec![];
        let public_keys: Vec<&[u8]> = vec![];

        // ed25519_batch_verify() works for empty msgs / sigs / pubkeys
        assert!(ed25519_batch_verify(&messages, &signatures, &public_keys).unwrap());
    }

    #[test]
    fn test_cosmos_ed25519_batch_verify_wrong_number_of_items_errors() {
        let codes = read_cosmos_sigs();

        let mut messages: Vec<Vec<u8>> = vec![];
        let mut signatures: Vec<Vec<u8>> = vec![];
        let mut public_keys: Vec<Vec<u8>> = vec![];

        for encoded in codes {
            let message = hex::decode(&encoded.message).unwrap();
            messages.push(message);

            let signature = hex::decode(&encoded.signature).unwrap();
            signatures.push(signature);

            let public_key = hex::decode(&encoded.public_key).unwrap();
            public_keys.push(public_key);
        }

        let mut messages: Vec<&[u8]> = messages.iter().map(|m| m.as_slice()).collect();
        let mut signatures: Vec<&[u8]> = signatures.iter().map(|m| m.as_slice()).collect();
        let mut public_keys: Vec<&[u8]> = public_keys.iter().map(|m| m.as_slice()).collect();

        // Check the whole set passes
        assert!(ed25519_batch_verify(&messages, &signatures, &public_keys).unwrap());

        // Remove one message
        let msg = messages.pop().unwrap();

        let res = ed25519_batch_verify(&messages, &signatures, &public_keys);
        match res.unwrap_err() {
            CryptoError::BatchErr { msg, .. } => assert_eq!(
                msg,
                "Mismatched / erroneous number of messages / signatures / public keys"
            ),
            _ => panic!("Wrong error message"),
        }

        // Restore messages
        messages.push(msg);

        // Remove one signature
        let sig = signatures.pop().unwrap();

        let res = ed25519_batch_verify(&messages, &signatures, &public_keys);
        match res.unwrap_err() {
            CryptoError::BatchErr { msg, .. } => assert_eq!(
                msg,
                "Mismatched / erroneous number of messages / signatures / public keys"
            ),
            _ => panic!("Wrong error message"),
        }

        // Restore signatures
        signatures.push(sig);

        // Remove one public key
        let pubkey = public_keys.pop().unwrap();

        let res = ed25519_batch_verify(&messages, &signatures, &public_keys);
        match res.unwrap_err() {
            CryptoError::BatchErr { msg, .. } => assert_eq!(
                msg,
                "Mismatched / erroneous number of messages / signatures / public keys"
            ),
            _ => panic!("Wrong error message"),
        }

        // Restore public keys
        public_keys.push(pubkey);

        // Add one message
        messages.push(messages[0]);

        let res = ed25519_batch_verify(&messages, &signatures, &public_keys);
        match res.unwrap_err() {
            CryptoError::BatchErr { msg, .. } => assert_eq!(
                msg,
                "Mismatched / erroneous number of messages / signatures / public keys"
            ),
            _ => panic!("Wrong error message"),
        }

        // Restore messages
        messages.pop();

        // Add one signature
        signatures.push(signatures[0]);
        let res = ed25519_batch_verify(&messages, &signatures, &public_keys);
        match res.unwrap_err() {
            CryptoError::BatchErr { msg, .. } => assert_eq!(
                msg,
                "Mismatched / erroneous number of messages / signatures / public keys"
            ),
            _ => panic!("Wrong error message"),
        }

        // Restore signatures
        signatures.pop();

        // Add one public keys
        public_keys.push(public_keys[0]);
        let res = ed25519_batch_verify(&messages, &signatures, &public_keys);
        match res.unwrap_err() {
            CryptoError::BatchErr { msg, .. } => assert_eq!(
                msg,
                "Mismatched / erroneous number of messages / signatures / public keys"
            ),
            _ => panic!("Wrong error message"),
        }
    }

    #[test]
    fn test_cosmos_ed25519_batch_verify_one_msg_different_number_of_sigs_pubkeys_errors() {
        let codes = read_cosmos_sigs();

        let mut messages: Vec<Vec<u8>> = vec![];
        let mut signatures: Vec<Vec<u8>> = vec![];
        let mut public_keys: Vec<Vec<u8>> = vec![];

        for encoded in codes {
            let message = hex::decode(&encoded.message).unwrap();
            messages.push(message);

            let signature = hex::decode(&encoded.signature).unwrap();
            signatures.push(signature);

            let public_key = hex::decode(&encoded.public_key).unwrap();
            public_keys.push(public_key);
        }

        let mut messages: Vec<&[u8]> = messages.iter().map(|m| m.as_slice()).collect();
        let mut signatures: Vec<&[u8]> = signatures.iter().map(|m| m.as_slice()).collect();
        let mut public_keys: Vec<&[u8]> = public_keys.iter().map(|m| m.as_slice()).collect();

        // Check the whole set passes
        assert!(ed25519_batch_verify(&messages, &signatures, &public_keys).unwrap());

        // Just one message
        messages.truncate(1);

        // Check (in passing) this fails verification
        assert!(!ed25519_batch_verify(&messages, &signatures, &public_keys).unwrap());

        // Remove one sig
        let sig = signatures.pop().unwrap();

        let res = ed25519_batch_verify(&messages, &signatures, &public_keys);
        match res.unwrap_err() {
            CryptoError::BatchErr { msg, .. } => assert_eq!(
                msg,
                "Mismatched / erroneous number of messages / signatures / public keys"
            ),
            _ => panic!("Wrong error message"),
        }

        // Restore signatures
        signatures.push(sig);

        // Remove one public key
        let pubkey = public_keys.pop().unwrap();

        let res = ed25519_batch_verify(&messages, &signatures, &public_keys);
        match res.unwrap_err() {
            CryptoError::BatchErr { msg, .. } => assert_eq!(
                msg,
                "Mismatched / erroneous number of messages / signatures / public keys"
            ),
            _ => panic!("Wrong error message"),
        }

        // Restore public keys
        public_keys.push(pubkey);
    }

    #[test]
    fn test_cosmos_ed25519_batch_verify_one_pubkey_different_number_of_msgs_sigs_errors() {
        let codes = read_cosmos_sigs();

        let mut messages: Vec<Vec<u8>> = vec![];
        let mut signatures: Vec<Vec<u8>> = vec![];
        let mut public_keys: Vec<Vec<u8>> = vec![];

        for encoded in codes {
            let message = hex::decode(&encoded.message).unwrap();
            messages.push(message);

            let signature = hex::decode(&encoded.signature).unwrap();
            signatures.push(signature);

            let public_key = hex::decode(&encoded.public_key).unwrap();
            public_keys.push(public_key);
        }

        let mut messages: Vec<&[u8]> = messages.iter().map(|m| m.as_slice()).collect();
        let mut signatures: Vec<&[u8]> = signatures.iter().map(|m| m.as_slice()).collect();
        let mut public_keys: Vec<&[u8]> = public_keys.iter().map(|m| m.as_slice()).collect();

        // Check the whole set passes
        assert!(ed25519_batch_verify(&messages, &signatures, &public_keys).unwrap());

        // Just one public key
        public_keys.truncate(1);

        // Check (in passing) this fails verification
        assert!(!ed25519_batch_verify(&messages, &signatures, &public_keys).unwrap());

        // Remove one sig
        let sig = signatures.pop().unwrap();

        let res = ed25519_batch_verify(&messages, &signatures, &public_keys);
        match res.unwrap_err() {
            CryptoError::BatchErr { msg, .. } => assert_eq!(
                msg,
                "Mismatched / erroneous number of messages / signatures / public keys"
            ),
            _ => panic!("Wrong error message"),
        }

        // Restore signatures
        signatures.push(sig);

        // Remove one msg
        let msg = messages.pop().unwrap();

        let res = ed25519_batch_verify(&messages, &signatures, &public_keys);
        match res.unwrap_err() {
            CryptoError::BatchErr { msg, .. } => assert_eq!(
                msg,
                "Mismatched / erroneous number of messages / signatures / public keys"
            ),
            _ => panic!("Wrong error message"),
        }

        // Restore messages
        messages.push(msg);
    }

    #[test]
    fn test_cosmos_ed25519_batch_verify_one_msg_zero_sigs_pubkeys_works() {
        let codes = read_cosmos_sigs();

        let mut messages: Vec<Vec<u8>> = vec![];
        // Zero sigs / pubkeys
        let signatures: Vec<&[u8]> = vec![];
        let public_keys: Vec<&[u8]> = vec![];

        // Just one message
        for encoded in codes[..1].iter() {
            let message = hex::decode(&encoded.message).unwrap();
            messages.push(message);
        }
        let messages: Vec<&[u8]> = messages.iter().map(|m| m.as_slice()).collect();

        // ed25519_batch_verify() works for empty sigs / pubkeys
        assert!(ed25519_batch_verify(&messages, &signatures, &public_keys).unwrap());
    }

    #[test]
    fn test_cosmos_ed25519_batch_verify_one_pubkey_zero_msgs_sigs_works() {
        let codes = read_cosmos_sigs();

        // Zero msgs / sigs
        let messages: Vec<&[u8]> = vec![];
        let signatures: Vec<&[u8]> = vec![];
        let mut public_keys: Vec<Vec<u8>> = vec![];

        // Just one public key
        for encoded in codes[..1].iter() {
            let public_key = hex::decode(&encoded.public_key).unwrap();
            public_keys.push(public_key);
        }
        let public_keys: Vec<&[u8]> = public_keys.iter().map(|m| m.as_slice()).collect();

        // ed25519_batch_verify() works for empty msgs / sigs
        assert!(ed25519_batch_verify(&messages, &signatures, &public_keys).unwrap());
    }
}
