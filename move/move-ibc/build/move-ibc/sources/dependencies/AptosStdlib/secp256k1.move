/// This module implements ECDSA signatures based on the prime-order secp256k1 ellptic curve (i.e., cofactor is 1).

module aptos_std::secp256k1 {
    use std::option::Option;

    /// An error occurred while deserializing, for example due to wrong input size.
    const E_DESERIALIZE: u64 = 1;   // This code must be the same, if ever returned from the native Rust implementation.

    /// The size of a secp256k1-based ECDSA public key, in bytes.
    const RAW_PUBLIC_KEY_NUM_BYTES: u64 = 64;
    //const COMPRESSED_PUBLIC_KEY_SIZE: u64 = 33;

    /// The size of a secp256k1-based ECDSA signature, in bytes.
    const SIGNATURE_NUM_BYTES: u64 = 64;

    /// A 64-byte ECDSA public key.
    struct ECDSARawPublicKey has copy, drop, store {
        bytes: vector<u8>
    }

    /// A 64-byte ECDSA signature.
    struct ECDSASignature has copy, drop, store {
        bytes: vector<u8>
    }

    /// Constructs an ECDSASignature struct from the given 64 bytes.
    public fun ecdsa_signature_from_bytes(bytes: vector<u8>): ECDSASignature {
        assert!(std::vector::length(&bytes) == SIGNATURE_NUM_BYTES, std::error::invalid_argument(E_DESERIALIZE));
        ECDSASignature { bytes }
    }

    /// Constructs an ECDSARawPublicKey struct, given a 64-byte raw representation.
    public fun ecdsa_raw_public_key_from_64_bytes(bytes: vector<u8>): ECDSARawPublicKey {
        assert!(std::vector::length(&bytes) == RAW_PUBLIC_KEY_NUM_BYTES, std::error::invalid_argument(E_DESERIALIZE));
        ECDSARawPublicKey { bytes }
    }

    /// Serializes an ECDSARawPublicKey struct to 64-bytes.
    public fun ecdsa_raw_public_key_to_bytes(pk: &ECDSARawPublicKey): vector<u8> {
        pk.bytes
    }

    /// Serializes an ECDSASignature struct to 64-bytes.
    public fun ecdsa_signature_to_bytes(sig: &ECDSASignature): vector<u8> {
        sig.bytes
    }

    /// Recovers the signer's raw (64-byte) public key from a secp256k1 ECDSA `signature` given the `recovery_id` and the signed
    /// `message` (32 byte digest).
    ///
    /// Note that an invalid signature, or a signature from a different message, will result in the recovery of an
    /// incorrect public key. This recovery algorithm can only be used to check validity of a signature if the signer's
    /// public key (or its hash) is known beforehand.
    public fun ecdsa_recover(
        message: vector<u8>,
        recovery_id: u8,
        signature: &ECDSASignature,
    ): Option<ECDSARawPublicKey> {
        let (pk, success) = ecdsa_recover_internal(message, recovery_id, signature.bytes);
        if (success) {
            std::option::some(ecdsa_raw_public_key_from_64_bytes(pk))
        } else {
            std::option::none<ECDSARawPublicKey>()
        }
    }

    //
    // Native functions
    //

    /// Returns `(public_key, true)` if `signature` verifies on `message` under the recovered `public_key`
    /// and returns `([], false)` otherwise.
    native fun ecdsa_recover_internal(
        message: vector<u8>,
        recovery_id: u8,
        signature: vector<u8>
    ): (vector<u8>, bool);

    //
    // Tests
    //

    #[test]
    /// Test on a valid secp256k1 ECDSA signature created using sk = x"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    fun test_ecdsa_recover() {
        use std::hash;

        let pk = ecdsa_recover(
            hash::sha2_256(b"test aptos secp256k1"),
            0,
            &ECDSASignature { bytes: x"f7ad936da03f948c14c542020e3c5f4e02aaacd1f20427c11aa6e2fbf8776477646bba0e1a37f9e7c777c423a1d2849baafd7ff6a9930814a43c3f80d59db56f" },
        );
        assert!(std::option::is_some(&pk), 1);
        assert!(std::option::extract(&mut pk).bytes == x"4646ae5047316b4230d0086c8acec687f00b1cd9d1dc634f6cb358ac0a9a8ffffe77b4dd0a4bfb95851f3b7355c781dd60f8418fc8a65d14907aff47c903a559", 1);

        // Flipped bits; Signature stays valid
        let pk = ecdsa_recover(
            hash::sha2_256(b"test aptos secp256k1"),
            0,
            // NOTE: A '7' was flipped to an 'f' here
            &ECDSASignature { bytes: x"f7ad936da03f948c14c542020e3c5f4e02aaacd1f20427c11aa6e2fbf8776477646bba0e1a37f9e7c7f7c423a1d2849baafd7ff6a9930814a43c3f80d59db56f" },
        );
        assert!(std::option::is_some(&pk), 1);
        assert!(std::option::extract(&mut pk).bytes != x"4646ae5047316b4230d0086c8acec687f00b1cd9d1dc634f6cb358ac0a9a8ffffe77b4dd0a4bfb95851f3b7355c781dd60f8418fc8a65d14907aff47c903a559", 1);

        // Flipped bits; Signature becomes invalid
        let pk = ecdsa_recover(
            hash::sha2_256(b"test aptos secp256k1"),
            0,
            &ECDSASignature { bytes: x"ffad936da03f948c14c542020e3c5f4e02aaacd1f20427c11aa6e2fbf8776477646bba0e1a37f9e7c7f7c423a1d2849baafd7ff6a9930814a43c3f80d59db56f" },
        );
        assert!(std::option::is_none(&pk), 1);
    }
}
