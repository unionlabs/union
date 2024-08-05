/// Exports MultiEd25519 multi-signatures in Move.
/// This module has the exact same interface as the Ed25519 module.

module aptos_std::multi_ed25519 {
    use std::bcs;
    use std::error;
    use std::features;
    use std::option::{Self, Option};
    use std::vector;
    use aptos_std::ed25519;

    //
    // Error codes
    //

    /// Wrong number of bytes were given as input when deserializing an Ed25519 public key.
    const E_WRONG_PUBKEY_SIZE: u64 = 1;

    /// Wrong number of bytes were given as input when deserializing an Ed25519 signature.
    const E_WRONG_SIGNATURE_SIZE: u64 = 2;

    /// The threshold must be in the range `[1, n]`, where n is the total number of signers.
    const E_INVALID_THRESHOLD_OR_NUMBER_OF_SIGNERS: u64 = 3;

    /// The native functions have not been rolled out yet.
    const E_NATIVE_FUN_NOT_AVAILABLE: u64 = 4;

    //
    // Constants
    //

    /// The identifier of the MultiEd25519 signature scheme, which is used when deriving Aptos authentication keys by hashing
    /// it together with an MultiEd25519 public key.
    const SIGNATURE_SCHEME_ID: u8 = 1;

    /// The size of an individual Ed25519 public key, in bytes.
    /// (A MultiEd25519 public key consists of several of these, plus the threshold.)
    const INDIVIDUAL_PUBLIC_KEY_NUM_BYTES: u64 = 32;

    /// The size of an individual Ed25519 signature, in bytes.
    /// (A MultiEd25519 signature consists of several of these, plus the signer bitmap.)
    const INDIVIDUAL_SIGNATURE_NUM_BYTES: u64 = 64;

    /// When serializing a MultiEd25519 public key, the threshold k will be encoded using this many bytes.
    const THRESHOLD_SIZE_BYTES: u64 = 1;

    /// When serializing a MultiEd25519 signature, the bitmap that indicates the signers will be encoded using this many
    /// bytes.
    const BITMAP_NUM_OF_BYTES: u64 = 4;

    /// Max number of ed25519 public keys allowed in multi-ed25519 keys
    const MAX_NUMBER_OF_PUBLIC_KEYS: u64 = 32;

    //
    // Structs
    //
    #[test_only]
    struct SecretKey has drop {
        bytes: vector<u8>,
    }

    /// An *unvalidated*, k out of n MultiEd25519 public key. The `bytes` field contains (1) several chunks of
    /// `ed25519::PUBLIC_KEY_NUM_BYTES` bytes, each encoding a Ed25519 PK, and (2) a single byte encoding the threshold k.
    /// *Unvalidated* means there is no guarantee that the underlying PKs are valid elliptic curve points of non-small
    /// order.
    struct UnvalidatedPublicKey has copy, drop, store {
        bytes: vector<u8>
    }

    /// A *validated* k out of n MultiEd25519 public key. *Validated* means that all the underlying PKs will be
    /// elliptic curve points that are NOT of small-order. It does not necessarily mean they will be prime-order points.
    /// This struct encodes the public key exactly as `UnvalidatedPublicKey`.
    ///
    /// For now, this struct is not used in any verification functions, but it might be in the future.
    struct ValidatedPublicKey has copy, drop, store {
        bytes: vector<u8>
    }

    /// A purported MultiEd25519 multi-signature that can be verified via `signature_verify_strict` or
    /// `signature_verify_strict_t`. The `bytes` field contains (1) several chunks of `ed25519::SIGNATURE_NUM_BYTES`
    /// bytes, each encoding a Ed25519 signature, and (2) a `BITMAP_NUM_OF_BYTES`-byte bitmap encoding the signer
    /// identities.
    struct Signature has copy, drop, store {
        bytes: vector<u8>
    }

    //
    // Functions
    //

    #[test_only]
    public fun generate_keys(threshold: u8, n: u8): (SecretKey, ValidatedPublicKey) {
        assert!(1 <= threshold && threshold <= n, error::invalid_argument(E_INVALID_THRESHOLD_OR_NUMBER_OF_SIGNERS));
        let (sk_bytes, pk_bytes) = generate_keys_internal(threshold, n);
        let sk = SecretKey {
            bytes: sk_bytes
        };
        let pk = ValidatedPublicKey {
            bytes: pk_bytes
        };
        (sk, pk)
    }

    #[test_only]
    public fun sign_arbitrary_bytes(sk: &SecretKey, msg: vector<u8>) : Signature {
        Signature {
            bytes: sign_internal(sk.bytes, msg)
        }
    }

    #[test_only]
    public fun sign_struct<T: drop>(sk: &SecretKey, data: T) : Signature {
        let encoded = ed25519::new_signed_message(data);
        Signature {
            bytes: sign_internal(sk.bytes, bcs::to_bytes(&encoded)),
        }
    }

    /// Parses the input 32 bytes as an *unvalidated* MultiEd25519 public key.
    ///
    /// NOTE: This function could have also checked that the # of sub-PKs is > 0, but it did not. However, since such
    /// invalid PKs are rejected during signature verification  (see `bugfix_unvalidated_pk_from_zero_subpks`) they
    /// will not cause problems.
    ///
    /// We could fix this API by adding a new one that checks the # of sub-PKs is > 0, but it is likely not a good idea
    /// to reproduce the PK validation logic in Move. We should not have done so in the first place. Instead, we will
    /// leave it as is and continue assuming `UnvalidatedPublicKey` objects could be invalid PKs that will safely be
    /// rejected during signature verification.
    public fun new_unvalidated_public_key_from_bytes(bytes: vector<u8>): UnvalidatedPublicKey {
        let len = vector::length(&bytes);
        let num_sub_pks = len / INDIVIDUAL_PUBLIC_KEY_NUM_BYTES;

        assert!(num_sub_pks <= MAX_NUMBER_OF_PUBLIC_KEYS, error::invalid_argument(E_WRONG_PUBKEY_SIZE));
        assert!(len % INDIVIDUAL_PUBLIC_KEY_NUM_BYTES == THRESHOLD_SIZE_BYTES, error::invalid_argument(E_WRONG_PUBKEY_SIZE));
        UnvalidatedPublicKey { bytes }
    }

    /// DEPRECATED: Use `new_validated_public_key_from_bytes_v2` instead. See `public_key_validate_internal` comments.
    ///
    /// (Incorrectly) parses the input bytes as a *validated* MultiEd25519 public key.
    public fun new_validated_public_key_from_bytes(bytes: vector<u8>): Option<ValidatedPublicKey> {
        // Note that `public_key_validate_internal` will check that `vector::length(&bytes) / INDIVIDUAL_PUBLIC_KEY_NUM_BYTES <= MAX_NUMBER_OF_PUBLIC_KEYS`.
        if (vector::length(&bytes) % INDIVIDUAL_PUBLIC_KEY_NUM_BYTES == THRESHOLD_SIZE_BYTES &&
            public_key_validate_internal(bytes)) {
            option::some(ValidatedPublicKey {
                bytes
            })
        } else {
            option::none<ValidatedPublicKey>()
        }
    }

    /// Parses the input bytes as a *validated* MultiEd25519 public key (see `public_key_validate_internal_v2`).
    public fun new_validated_public_key_from_bytes_v2(bytes: vector<u8>): Option<ValidatedPublicKey> {
        if (!features::multi_ed25519_pk_validate_v2_enabled()) {
            abort(error::invalid_state(E_NATIVE_FUN_NOT_AVAILABLE))
        };

        if (public_key_validate_v2_internal(bytes)) {
            option::some(ValidatedPublicKey {
                bytes
            })
        } else {
            option::none<ValidatedPublicKey>()
        }
    }

    /// Parses the input bytes as a purported MultiEd25519 multi-signature.
    public fun new_signature_from_bytes(bytes: vector<u8>): Signature {
        assert!(vector::length(&bytes) % INDIVIDUAL_SIGNATURE_NUM_BYTES == BITMAP_NUM_OF_BYTES, error::invalid_argument(E_WRONG_SIGNATURE_SIZE));
        Signature { bytes }
    }

    /// Converts a ValidatedPublicKey to an UnvalidatedPublicKey, which can be used in the strict verification APIs.
    public fun public_key_to_unvalidated(pk: &ValidatedPublicKey): UnvalidatedPublicKey {
        UnvalidatedPublicKey {
            bytes: pk.bytes
        }
    }

    /// Moves a ValidatedPublicKey into an UnvalidatedPublicKey, which can be used in the strict verification APIs.
    public fun public_key_into_unvalidated(pk: ValidatedPublicKey): UnvalidatedPublicKey {
        UnvalidatedPublicKey {
            bytes: pk.bytes
        }
    }

    /// Serializes an UnvalidatedPublicKey struct to 32-bytes.
    public fun unvalidated_public_key_to_bytes(pk: &UnvalidatedPublicKey): vector<u8> {
        pk.bytes
    }

    /// Serializes a ValidatedPublicKey struct to 32-bytes.
    public fun validated_public_key_to_bytes(pk: &ValidatedPublicKey): vector<u8> {
        pk.bytes
    }

    /// Serializes a Signature struct to 64-bytes.
    public fun signature_to_bytes(sig: &Signature): vector<u8> {
        sig.bytes
    }

    /// DEPRECATED: Use `public_key_validate_v2` instead. See `public_key_validate_internal` comments.
    ///
    /// Takes in an *unvalidated* public key and attempts to validate it.
    /// Returns `Some(ValidatedPublicKey)` if successful and `None` otherwise.
    public fun public_key_validate(pk: &UnvalidatedPublicKey): Option<ValidatedPublicKey> {
        new_validated_public_key_from_bytes(pk.bytes)
    }

    /// Takes in an *unvalidated* public key and attempts to validate it.
    /// Returns `Some(ValidatedPublicKey)` if successful and `None` otherwise.
    public fun public_key_validate_v2(pk: &UnvalidatedPublicKey): Option<ValidatedPublicKey> {
        new_validated_public_key_from_bytes_v2(pk.bytes)
    }

    /// Verifies a purported MultiEd25519 `multisignature` under an *unvalidated* `public_key` on the specified `message`.
    /// This call will validate the public key by checking it is NOT in the small subgroup.
    public fun signature_verify_strict(
        multisignature: &Signature,
        public_key: &UnvalidatedPublicKey,
        message: vector<u8>
    ): bool {
        signature_verify_strict_internal(multisignature.bytes, public_key.bytes, message)
    }

    /// This function is used to verify a multi-signature on any BCS-serializable type T. For now, it is used to verify the
    /// proof of private key ownership when rotating authentication keys.
    public fun signature_verify_strict_t<T: drop>(multisignature: &Signature, public_key: &UnvalidatedPublicKey, data: T): bool {
        let encoded = ed25519::new_signed_message(data);

        signature_verify_strict_internal(multisignature.bytes, public_key.bytes, bcs::to_bytes(&encoded))
    }

    /// Derives the Aptos-specific authentication key of the given Ed25519 public key.
    public fun unvalidated_public_key_to_authentication_key(pk: &UnvalidatedPublicKey): vector<u8> {
        public_key_bytes_to_authentication_key(pk.bytes)
    }

    /// Returns the number n of sub-PKs in an unvalidated t-out-of-n MultiEd25519 PK.
    /// If this `UnvalidatedPublicKey` would pass validation in `public_key_validate`, then the returned # of sub-PKs
    /// can be relied upon as correct.
    ///
    /// We provide this API as a cheaper alternative to calling `public_key_validate` and then `validated_public_key_num_sub_pks`
    /// when the input `pk` is known to be valid.
    public fun unvalidated_public_key_num_sub_pks(pk: &UnvalidatedPublicKey): u8 {
        let len = vector::length(&pk.bytes);

        ((len / INDIVIDUAL_PUBLIC_KEY_NUM_BYTES) as u8)
    }

    /// Returns the number t of sub-PKs in an unvalidated t-out-of-n MultiEd25519 PK (i.e., the threshold) or `None`
    /// if `bytes` does not correctly encode such a PK.
    public fun unvalidated_public_key_threshold(pk: &UnvalidatedPublicKey): Option<u8> {
        check_and_get_threshold(pk.bytes)
    }

    /// Derives the Aptos-specific authentication key of the given Ed25519 public key.
    public fun validated_public_key_to_authentication_key(pk: &ValidatedPublicKey): vector<u8> {
        public_key_bytes_to_authentication_key(pk.bytes)
    }

    /// Returns the number n of sub-PKs in a validated t-out-of-n MultiEd25519 PK.
    /// Since the format of this PK has been validated, the returned # of sub-PKs is guaranteed to be correct.
    public fun validated_public_key_num_sub_pks(pk: &ValidatedPublicKey): u8 {
        let len = vector::length(&pk.bytes);

        ((len / INDIVIDUAL_PUBLIC_KEY_NUM_BYTES) as u8)
    }

    /// Returns the number t of sub-PKs in a validated t-out-of-n MultiEd25519 PK (i.e., the threshold).
    public fun validated_public_key_threshold(pk: &ValidatedPublicKey): u8 {
        let len = vector::length(&pk.bytes);
        let threshold_byte = *vector::borrow(&pk.bytes, len - 1);

        threshold_byte
    }

    /// Checks that the serialized format of a t-out-of-n MultiEd25519 PK correctly encodes 1 <= n <= 32 sub-PKs.
    /// (All `ValidatedPublicKey` objects are guaranteed to pass this check.)
    /// Returns the threshold t <= n of the PK.
    public fun check_and_get_threshold(bytes: vector<u8>): Option<u8> {
        let len = vector::length(&bytes);
        if (len == 0) {
            return option::none<u8>()
        };

        let threshold_num_of_bytes = len % INDIVIDUAL_PUBLIC_KEY_NUM_BYTES;
        let num_of_keys = len / INDIVIDUAL_PUBLIC_KEY_NUM_BYTES;
        let threshold_byte = *vector::borrow(&bytes, len - 1);

        if (num_of_keys == 0 || num_of_keys > MAX_NUMBER_OF_PUBLIC_KEYS || threshold_num_of_bytes != 1) {
            return option::none<u8>()
        } else if (threshold_byte == 0 || threshold_byte > (num_of_keys as u8)) {
            return option::none<u8>()
        } else {
            return option::some(threshold_byte)
        }
    }

    /// Derives the Aptos-specific authentication key of the given Ed25519 public key.
    fun public_key_bytes_to_authentication_key(pk_bytes: vector<u8>): vector<u8> {
        vector::push_back(&mut pk_bytes, SIGNATURE_SCHEME_ID);
        std::hash::sha3_256(pk_bytes)
    }

    //
    // Native functions
    //

    /// DEPRECATED: Use `public_key_validate_internal_v2` instead. This function was NOT correctly implemented:
    ///
    ///  1. It does not check that the # of sub public keys is > 0, which leads to invalid `ValidatedPublicKey` objects
    ///     against which no signature will verify, since `signature_verify_strict_internal` will reject such invalid PKs.
    ///     This is not a security issue, but a correctness issue. See `bugfix_validated_pk_from_zero_subpks`.
    ///  2. It charges too much gas: if the first sub-PK is invalid, it will charge for verifying all remaining sub-PKs.
    ///
    /// DEPRECATES:
    ///  - new_validated_public_key_from_bytes
    ///  - public_key_validate
    ///
    /// Return `true` if the bytes in `public_key` can be parsed as a valid MultiEd25519 public key: i.e., all underlying
    /// PKs pass point-on-curve and not-in-small-subgroup checks.
    /// Returns `false` otherwise.
    native fun public_key_validate_internal(bytes: vector<u8>): bool;

    /// Return `true` if the bytes in `public_key` can be parsed as a valid MultiEd25519 public key: i.e., all underlying
    /// sub-PKs pass point-on-curve and not-in-small-subgroup checks.
    /// Returns `false` otherwise.
    native fun public_key_validate_v2_internal(bytes: vector<u8>): bool;

    /// Return true if the MultiEd25519 `multisignature` on `message` verifies against the MultiEd25519 `public_key`.
    /// Returns `false` if either:
    /// - The PKs in `public_key` do not all pass points-on-curve or not-in-small-subgroup checks,
    /// - The signatures in `multisignature` do not all pass points-on-curve or not-in-small-subgroup checks,
    /// - the `multisignature` on `message` does not verify.
    native fun signature_verify_strict_internal(
        multisignature: vector<u8>,
        public_key: vector<u8>,
        message: vector<u8>
    ): bool;

    #[test_only]
    native fun generate_keys_internal(threshold: u8, n: u8): (vector<u8>,vector<u8>);

    #[test_only]
    native fun sign_internal(sk: vector<u8>, message: vector<u8>): vector<u8>;

    //
    // Tests
    //

    #[test_only]
    struct TestMessage has copy, drop {
        foo: vector<u8>,
        bar: u64,
    }

    #[test_only]
    public fun maul_first_signature(sig: &mut Signature) {
        let first_sig_byte = vector::borrow_mut(&mut sig.bytes, 0);
        *first_sig_byte = *first_sig_byte ^ 0xff;
    }


    #[test(fx = @std)]
    fun bugfix_validated_pk_from_zero_subpks(fx: signer) {
        features::change_feature_flags_for_testing(&fx, vector[ features::multi_ed25519_pk_validate_v2_feature()], vector[]);
        let bytes = vector<u8>[1u8];
        assert!(vector::length(&bytes) == 1, 1);

        // Try deserializing a MultiEd25519 `ValidatedPublicKey` with 0 Ed25519 sub-PKs and 1 threshold byte.
        // This would ideally NOT succeed, but it currently does. Regardless, such invalid PKs will be safely dismissed
        // during signature verification.
        let some = new_validated_public_key_from_bytes(bytes);
        assert!(option::is_none(&check_and_get_threshold(bytes)), 1);   // ground truth
        assert!(option::is_some(&some), 2);                             // incorrect

        // In contrast, the v2 API will fail deserializing, as it should.
        let none = new_validated_public_key_from_bytes_v2(bytes);
        assert!(option::is_none(&none), 3);
    }

    #[test(fx = @std)]
    fun test_validated_pk_without_threshold_byte(fx: signer) {
        features::change_feature_flags_for_testing(&fx, vector[ features::multi_ed25519_pk_validate_v2_feature()], vector[]);

        let (_, subpk) = ed25519::generate_keys();
        let bytes = ed25519::validated_public_key_to_bytes(&subpk);
        assert!(vector::length(&bytes) == INDIVIDUAL_PUBLIC_KEY_NUM_BYTES, 1);

        // Try deserializing a MultiEd25519 `ValidatedPublicKey` with 1 Ed25519 sub-PKs but no threshold byte, which
        // will not succeed,
        let none = new_validated_public_key_from_bytes(bytes);
        assert!(option::is_none(&check_and_get_threshold(bytes)), 1);   // ground truth
        assert!(option::is_none(&none), 2);                             // correct

        // Similarly, the v2 API will also fail deserializing.
        let none = new_validated_public_key_from_bytes_v2(bytes);
        assert!(option::is_none(&none), 3);                             // also correct
    }

    #[test(fx = @std)]
    fun test_validated_pk_from_small_order_subpk(fx: signer) {
        features::change_feature_flags_for_testing(&fx, vector[ features::multi_ed25519_pk_validate_v2_feature()], vector[]);
        let torsion_point_with_threshold_1 = vector<u8>[
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1,
        ];

        assert!(option::extract(&mut check_and_get_threshold(torsion_point_with_threshold_1)) == 1, 1);

        // Try deserializing a MultiEd25519 `ValidatedPublicKey` with 1 Ed25519 sub-PKs and 1 threshold byte, as it should,
        // except the sub-PK is of small order. This should not succeed,
        let none = new_validated_public_key_from_bytes(torsion_point_with_threshold_1);
        assert!(option::is_none(&none), 2);

        // Similarly, the v2 API will also fail deserializing.
        let none = new_validated_public_key_from_bytes_v2(torsion_point_with_threshold_1);
        assert!(option::is_none(&none), 3);
    }

    #[test]
    fun test_gen_sign_verify() {
        let thresholds = vector[1, 1, 2, 2, 3, 15,]; // the thresholds, implicitly encoded in the public keys
        let party_counts = vector[1, 2, 2, 3, 10, 32,];
        let test_case_count = vector::length(&party_counts);
        let test_case_idx = 0;

        while (test_case_idx < test_case_count) {
            let threshold = *vector::borrow(&thresholds, test_case_idx);
            let group_size = *vector::borrow(&party_counts, test_case_idx);

            let (sk, pk) = generate_keys(threshold, group_size);
            assert!(validated_public_key_threshold(&pk) == threshold, 1);
            assert!(validated_public_key_num_sub_pks(&pk) == group_size, 2);
            assert!(public_key_validate_v2_internal(pk.bytes), 3);

            let upk = public_key_into_unvalidated(pk);
            assert!(option::extract(&mut unvalidated_public_key_threshold(&upk)) == threshold, 4);
            assert!(unvalidated_public_key_num_sub_pks(&upk) == group_size, 5);

            let msg1 = b"Hello Aptos!";
            let sig1 = sign_arbitrary_bytes(&sk, msg1);
            assert!(signature_verify_strict(&sig1, &upk, msg1), 6);

            let obj2 = TestMessage {
                foo: b"Hello Move!",
                bar: 64,
            };
            let sig2 = sign_struct(&sk, copy obj2);
            assert!(signature_verify_strict_t(&sig2, &upk, copy obj2), 7);

            test_case_idx = test_case_idx + 1;
        }
    }

    #[test]
    fun test_threshold_not_met_rejection() {
        let (sk,pk) = generate_keys(4, 5);
        assert!(validated_public_key_threshold(&pk) == 4, 1);
        assert!(validated_public_key_num_sub_pks(&pk) == 5, 2);
        assert!(public_key_validate_v2_internal(pk.bytes), 3);

        let upk = public_key_into_unvalidated(pk);
        assert!(option::extract(&mut unvalidated_public_key_threshold(&upk)) == 4, 4);
        assert!(unvalidated_public_key_num_sub_pks(&upk) == 5, 5);

        let msg1 = b"Hello Aptos!";
        let sig1 = sign_arbitrary_bytes(&sk, msg1);
        maul_first_signature(&mut sig1);
        assert!(!signature_verify_strict(&sig1, &upk, msg1), 6);

        let obj2 = TestMessage {
            foo: b"Hello Move!",
            bar: 64,
        };
        let sig2 = sign_struct(&sk, copy obj2);
        maul_first_signature(&mut sig2);
        assert!(!signature_verify_strict_t(&sig2, &upk, copy obj2), 7);
    }
}
