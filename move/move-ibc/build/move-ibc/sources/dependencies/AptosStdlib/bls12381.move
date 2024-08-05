/// Contains functions for:
///
///     The minimum-pubkey-size variant of [Boneh-Lynn-Shacham (BLS) signatures](https://en.wikipedia.org/wiki/BLS_digital_signature),
///     where public keys are BLS12-381 elliptic-curve points in $\mathbb{G}_1$ and signatures are in $\mathbb{G}_2$,
///     as per the [IETF BLS draft standard](https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-bls-signature#section-2.1).

module aptos_std::bls12381 {
    use std::option::{Self, Option};
    #[test_only]
    use std::error::invalid_argument;

    /// The signature size, in bytes
    const SIGNATURE_SIZE: u64 = 96;

    /// The public key size, in bytes
    const PUBLIC_KEY_NUM_BYTES: u64 = 48;

    /// The caller was supposed to input one or more public keys.
    const EZERO_PUBKEYS: u64 = 1;

    /// One of the given inputs has the wrong size.s
    const EWRONG_SIZE: u64 = 2;

    /// The number of signers does not match the number of messages to be signed.
    const E_NUM_SIGNERS_MUST_EQ_NUM_MESSAGES: u64 = 3;

    // TODO: Performance would increase if structs in this module are implemented natively via handles (similar to Table and
    // RistrettoPoint). This will avoid unnecessary (de)serialization. We would need to allow storage of these structs too.

    #[test_only]
    struct SecretKey has copy, drop {
        bytes: vector<u8>,
    }

    /// A *validated* public key that:
    ///   (1) is a point in the prime-order subgroup of the BLS12-381 elliptic curve, and
    ///   (2) is not the identity point
    ///
    /// This struct can be used to verify a normal (non-aggregated) signature.
    ///
    /// This struct can be combined with a ProofOfPossession struct in order to create a PublicKeyWithPop struct, which
    /// can be used to verify a multisignature.
    struct PublicKey has copy, drop, store {
        bytes: vector<u8>
    }

    /// A proof-of-possession (PoP).
    /// Given such a struct and a PublicKey struct, one can construct a PublicKeyWithPoP (see below).
    struct ProofOfPossession has copy, drop, store {
        bytes: vector<u8>
    }

    /// A *validated* public key that had a successfully-verified proof-of-possession (PoP).
    ///
    /// A vector of these structs can be either:
    ///   (1) used to verify an aggregate signature
    ///   (2) aggregated with other PublicKeyWithPoP structs into an AggrPublicKeysWithPoP, which in turn can be used
    ///       to verify a multisignature
    struct PublicKeyWithPoP has copy, drop, store {
        bytes: vector<u8>
    }

    /// An aggregation of public keys with verified PoPs, which can be used to verify multisignatures.
    struct AggrPublicKeysWithPoP has copy, drop, store {
        bytes: vector<u8>
    }

    /// A BLS signature. This can be either a:
    ///   (1) normal (non-aggregated) signature
    ///   (2) signature share (for a multisignature or aggregate signature)
    struct Signature has copy, drop, store {
        bytes: vector<u8>
    }

    /// An aggregation of BLS signatures. This can be either a:
    ///   (4) aggregated signature (i.e., an aggregation of signatures s_i, each on a message m_i)
    ///   (3) multisignature (i.e., an aggregation of signatures s_i, each on the same message m)
    ///
    /// We distinguish between a Signature type and a AggrOrMultiSignature type to prevent developers from interchangeably
    /// calling `verify_multisignature` and `verify_signature_share` to verify both multisignatures and signature shares,
    /// which could create problems down the line.
    struct AggrOrMultiSignature has copy, drop, store {
        bytes: vector<u8>
    }

    /// Creates a new public key from a sequence of bytes.
    public fun public_key_from_bytes(bytes: vector<u8>): Option<PublicKey> {
        if (validate_pubkey_internal(bytes)) {
            option::some(PublicKey {
                bytes
            })
        } else {
            option::none<PublicKey>()
        }
    }

    /// Serializes a public key into 48 bytes.
    public fun public_key_to_bytes(pk: &PublicKey): vector<u8> {
        pk.bytes
    }

    /// Creates a new proof-of-possession (PoP) which can be later used to create a PublicKeyWithPoP struct,
    public fun proof_of_possession_from_bytes(bytes: vector<u8>): ProofOfPossession {
        ProofOfPossession {
            bytes
        }
    }

    /// Serializes the signature into 96 bytes.
    public fun proof_of_possession_to_bytes(pop: &ProofOfPossession): vector<u8> {
        pop.bytes
    }

    /// Creates a PoP'd public key from a normal public key and a corresponding proof-of-possession.
    public fun public_key_from_bytes_with_pop(pk_bytes: vector<u8>, pop: &ProofOfPossession): Option<PublicKeyWithPoP> {
        if (verify_proof_of_possession_internal(pk_bytes, pop.bytes)) {
            option::some(PublicKeyWithPoP {
                bytes: pk_bytes
            })
        } else {
            option::none<PublicKeyWithPoP>()
        }
    }

    /// Creates a normal public key from a PoP'd public key.
    public fun public_key_with_pop_to_normal(pkpop: &PublicKeyWithPoP): PublicKey {
        PublicKey {
            bytes: pkpop.bytes
        }
    }

    /// Serializes a PoP'd public key into 48 bytes.
    public fun public_key_with_pop_to_bytes(pk: &PublicKeyWithPoP): vector<u8> {
        pk.bytes
    }

    /// Creates a new signature from a sequence of bytes. Does not check the signature for prime-order subgroup
    /// membership since that is done implicitly during verification.
    public fun signature_from_bytes(bytes: vector<u8>): Signature {
        Signature {
            bytes
        }
    }

    /// Serializes the signature into 96 bytes.
    public fun signature_to_bytes(sig: &Signature): vector<u8> {
        sig.bytes
    }

    /// Checks that the group element that defines a signature is in the prime-order subgroup.
    /// This check is implicitly performed when verifying any signature via this module, but we expose this functionality
    /// in case it might be useful for applications to easily dismiss invalid signatures early on.
    public fun signature_subgroup_check(signature: &Signature): bool {
        signature_subgroup_check_internal(signature.bytes)
    }

    /// Given a vector of public keys with verified PoPs, combines them into an *aggregated* public key which can be used
    /// to verify multisignatures using `verify_multisignature` and aggregate signatures using `verify_aggregate_signature`.
    /// Aborts if no public keys are given as input.
    public fun aggregate_pubkeys(public_keys: vector<PublicKeyWithPoP>): AggrPublicKeysWithPoP {
        let (bytes, success) = aggregate_pubkeys_internal(public_keys);
        assert!(success, std::error::invalid_argument(EZERO_PUBKEYS));

        AggrPublicKeysWithPoP {
            bytes
        }
    }

    /// Serializes an aggregate public key into 48 bytes.
    public fun aggregate_pubkey_to_bytes(apk: &AggrPublicKeysWithPoP): vector<u8> {
        apk.bytes
    }

    /// Aggregates the input signatures into an aggregate-or-multi-signature structure, which can be later verified via
    /// `verify_aggregate_signature` or `verify_multisignature`. Returns `None` if zero signatures are given as input
    /// or if some of the signatures are not valid group elements.
    public fun aggregate_signatures(signatures: vector<Signature>): Option<AggrOrMultiSignature> {
        let (bytes, success) = aggregate_signatures_internal(signatures);
        if (success) {
            option::some(
                AggrOrMultiSignature {
                    bytes
                }
            )
        } else {
            option::none<AggrOrMultiSignature>()
        }
    }

    /// Serializes an aggregate-or-multi-signature into 96 bytes.
    public fun aggr_or_multi_signature_to_bytes(sig: &AggrOrMultiSignature): vector<u8> {
        sig.bytes
    }

    /// Deserializes an aggregate-or-multi-signature from 96 bytes.
    public fun aggr_or_multi_signature_from_bytes(bytes: vector<u8>): AggrOrMultiSignature {
        assert!(std::vector::length(&bytes) == SIGNATURE_SIZE, std::error::invalid_argument(EWRONG_SIZE));

        AggrOrMultiSignature {
            bytes
        }
    }


    /// Checks that the group element that defines an aggregate-or-multi-signature is in the prime-order subgroup.
    public fun aggr_or_multi_signature_subgroup_check(signature: &AggrOrMultiSignature): bool {
        signature_subgroup_check_internal(signature.bytes)
    }

    /// Verifies an aggregate signature, an aggregation of many signatures `s_i`, each on a different message `m_i`.
    public fun verify_aggregate_signature(
        aggr_sig: &AggrOrMultiSignature,
        public_keys: vector<PublicKeyWithPoP>,
        messages: vector<vector<u8>>,
    ): bool {
        verify_aggregate_signature_internal(aggr_sig.bytes, public_keys, messages)
    }

    /// Verifies a multisignature: an aggregation of many signatures, each on the same message `m`.
    public fun verify_multisignature(
        multisig: &AggrOrMultiSignature,
        aggr_public_key: &AggrPublicKeysWithPoP,
        message: vector<u8>
    ): bool {
        verify_multisignature_internal(multisig.bytes, aggr_public_key.bytes, message)
    }

    /// Verifies a normal, non-aggregated signature.
    public fun verify_normal_signature(
        signature: &Signature,
        public_key: &PublicKey,
        message: vector<u8>
    ): bool {
        verify_normal_signature_internal(signature.bytes, public_key.bytes, message)
    }

    /// Verifies a signature share in the multisignature share or an aggregate signature share.
    public fun verify_signature_share(
        signature_share: &Signature,
        public_key: &PublicKeyWithPoP,
        message: vector<u8>
    ): bool {
        verify_signature_share_internal(signature_share.bytes, public_key.bytes, message)
    }

    #[test_only]
    /// Generates a BLS key-pair: a secret key with its corresponding public key.
    public fun generate_keys(): (SecretKey, PublicKeyWithPoP) {
        let (sk_bytes, pk_bytes) = generate_keys_internal();
        let sk = SecretKey {
            bytes: sk_bytes
        };
        let pkpop = PublicKeyWithPoP {
            bytes: pk_bytes
        };
        (sk, pkpop)
    }

    #[test_only]
    /// Generates a BLS signature for a message with a signing key.
    public fun sign_arbitrary_bytes(signing_key: &SecretKey, message: vector<u8>): Signature {
        Signature {
            bytes: sign_internal(signing_key.bytes, message)
        }
    }

    #[test_only]
    /// Generates a multi-signature for a message with multiple signing keys.
    public fun multi_sign_arbitrary_bytes(signing_keys: &vector<SecretKey>, message: vector<u8>): AggrOrMultiSignature {
        let n = std::vector::length(signing_keys);
        let sigs = vector[];
        let i: u64 = 0;
        while (i < n) {
            let sig = sign_arbitrary_bytes(std::vector::borrow(signing_keys, i), message);
            std::vector::push_back(&mut sigs, sig);
            i = i + 1;
        };
        let multisig = aggregate_signatures(sigs);
        option::extract(&mut multisig)
    }

    #[test_only]
    /// Generates an aggregated signature over all messages in messages, where signing_keys[i] signs messages[i].
    public fun aggr_sign_arbitrary_bytes(signing_keys: &vector<SecretKey>, messages: &vector<vector<u8>>): AggrOrMultiSignature {
        let signing_key_count = std::vector::length(signing_keys);
        let message_count = std::vector::length(messages);
        assert!(signing_key_count == message_count, invalid_argument(E_NUM_SIGNERS_MUST_EQ_NUM_MESSAGES));
        let sigs = vector[];
        let i: u64 = 0;
        while (i < signing_key_count) {
            let sig = sign_arbitrary_bytes(std::vector::borrow(signing_keys, i), *std::vector::borrow(messages, i));
            std::vector::push_back(&mut sigs, sig);
            i = i + 1;
        };
        let aggr_sig = aggregate_signatures(sigs);
        option::extract(&mut aggr_sig)
    }

    #[test_only]
    /// Returns a mauled copy of a byte array.
    public fun maul_bytes(bytes: &vector<u8>): vector<u8> {
        let new_bytes = *bytes;
        let first_byte = std::vector::borrow_mut(&mut new_bytes, 0);
        *first_byte = *first_byte ^ 0xff;
        new_bytes
    }

    #[test_only]
    /// Returns a mauled copy of a normal signature.
    public fun maul_signature(sig: &Signature): Signature {
        Signature {
            bytes: maul_bytes(&signature_to_bytes(sig))
        }
    }

    #[test_only]
    /// Returns a mauled copy of an aggregated signature or a multi-signature.
    public fun maul_aggr_or_multi_signature(sig: &AggrOrMultiSignature): AggrOrMultiSignature {
        AggrOrMultiSignature {
            bytes: maul_bytes(&aggr_or_multi_signature_to_bytes(sig))
        }
    }

    #[test_only]
    /// Returns a mauled copy of a normal public key.
    public fun maul_public_key(pk: &PublicKey): PublicKey {
        PublicKey {
            bytes: maul_bytes(&public_key_to_bytes(pk))
        }
    }

    #[test_only]
    /// Returns a mauled copy of a PoP'd public key.
    public fun maul_public_key_with_pop(pk: &PublicKeyWithPoP): PublicKeyWithPoP {
        PublicKeyWithPoP {
            bytes: maul_bytes(&public_key_with_pop_to_bytes(pk))
        }
    }

    #[test_only]
    /// Returns a mauled copy of an aggregated public key.
    public fun maul_aggregated_public_key(pk: &AggrPublicKeysWithPoP): AggrPublicKeysWithPoP {
        AggrPublicKeysWithPoP {
            bytes: maul_bytes(&aggregate_pubkey_to_bytes(pk))
        }
    }

    #[test_only]
    /// Returns a mauled copy of a proof-of-possession.
    public fun maul_proof_of_possession(pop: &ProofOfPossession): ProofOfPossession {
        ProofOfPossession {
            bytes: maul_bytes(&proof_of_possession_to_bytes(pop))
        }
    }


    #[test_only]
    /// Generates a proof-of-possession (PoP) for the public key associated with the secret key `sk`.
    public fun generate_proof_of_possession(sk: &SecretKey): ProofOfPossession {
        ProofOfPossession {
            bytes: generate_proof_of_possession_internal(sk.bytes)
        }
    }

    //
    // Native functions
    //

    /// CRYPTOGRAPHY WARNING: This function assumes that the caller verified all public keys have a valid
    /// proof-of-possesion (PoP) using `verify_proof_of_possession`.
    ///
    /// Given a vector of serialized public keys, combines them into an aggregated public key, returning `(bytes, true)`,
    /// where `bytes` store the serialized public key.
    /// Aborts if no public keys are given as input.
    native fun aggregate_pubkeys_internal(public_keys: vector<PublicKeyWithPoP>): (vector<u8>, bool);


    /// CRYPTOGRAPHY WARNING: This function can be safely called without verifying that the input signatures are elements
    /// of the prime-order subgroup of the BLS12-381 curve.
    ///
    /// Given a vector of serialized signatures, combines them into an aggregate signature, returning `(bytes, true)`,
    /// where `bytes` store the serialized signature.
    /// Does not check the input signatures nor the final aggregated signatures for prime-order subgroup membership.
    /// Returns `(_, false)` if no signatures are given as input.
    /// Does not abort.
    native fun aggregate_signatures_internal(signatures: vector<Signature>): (vector<u8>, bool);

    /// Return `true` if the bytes in `public_key` are a valid BLS12-381 public key:
    ///  (1) it is NOT the identity point, and
    ///  (2) it is a BLS12-381 elliptic curve point, and
    ///  (3) it is a prime-order point
    /// Return `false` otherwise.
    /// Does not abort.
    native fun validate_pubkey_internal(public_key: vector<u8>): bool;

    /// Return `true` if the elliptic curve point serialized in `signature`:
    ///  (1) is NOT the identity point, and
    ///  (2) is a BLS12-381 elliptic curve point, and
    ///  (3) is a prime-order point
    /// Return `false` otherwise.
    /// Does not abort.
    native fun signature_subgroup_check_internal(signature: vector<u8>): bool;

    /// CRYPTOGRAPHY WARNING: First, this function assumes all public keys have a valid proof-of-possesion (PoP).
    /// This prevents both small-subgroup attacks and rogue-key attacks. Second, this function can be safely called
    /// without verifying that the aggregate signature is in the prime-order subgroup of the BLS12-381 curve.
    ///
    /// Returns `true` if the aggregate signature `aggsig` on `messages` under `public_keys` verifies (where `messages[i]`
    /// should be signed by `public_keys[i]`).
    ///
    /// Returns `false` if either:
    /// - no public keys or messages are given as input,
    /// - number of messages does not equal number of public keys
    /// - `aggsig` (1) is the identity point, or (2) is NOT a BLS12-381 elliptic curve point, or (3) is NOT a
    ///   prime-order point
    /// Does not abort.
    native fun verify_aggregate_signature_internal(
        aggsig: vector<u8>,
        public_keys: vector<PublicKeyWithPoP>,
        messages: vector<vector<u8>>,
    ): bool;

    /// CRYPTOGRAPHY WARNING: This function assumes verified proofs-of-possesion (PoP) for the public keys used in
    /// computing the aggregate public key. This prevents small-subgroup attacks and rogue-key attacks.
    ///
    /// Return `true` if the BLS `multisignature` on `message` verifies against the BLS aggregate public key `agg_public_key`.
    /// Returns `false` otherwise.
    /// Does not abort.
    native fun verify_multisignature_internal(
        multisignature: vector<u8>,
        agg_public_key: vector<u8>,
        message: vector<u8>
    ): bool;

    /// CRYPTOGRAPHY WARNING: This function WILL check that the public key is a prime-order point, in order to prevent
    /// library users from misusing the library by forgetting to validate public keys before giving them as arguments to
    /// this function.
    ///
    /// Returns `true` if the `signature` on `message` verifies under `public key`.
    /// Returns `false` otherwise.
    /// Does not abort.
    native fun verify_normal_signature_internal(
        signature: vector<u8>,
        public_key: vector<u8>,
        message: vector<u8>
    ): bool;

    /// Return `true` if the bytes in `public_key` are a valid bls12381 public key (as per `validate_pubkey`)
    /// *and* this public key has a valid proof-of-possesion (PoP).
    /// Return `false` otherwise.
    /// Does not abort.
    native fun verify_proof_of_possession_internal(
        public_key: vector<u8>,
        proof_of_possesion: vector<u8>
    ): bool;

    /// CRYPTOGRAPHY WARNING: Assumes the public key has a valid proof-of-possesion (PoP). This prevents rogue-key
    /// attacks later on during signature aggregation.
    ///
    /// Returns `true` if the `signature_share` on `message` verifies under `public key`.
    /// Returns `false` otherwise, similar to `verify_multisignature`.
    /// Does not abort.
    native fun verify_signature_share_internal(
        signature_share: vector<u8>,
        public_key: vector<u8>,
        message: vector<u8>
    ): bool;

    #[test_only]
    native fun generate_keys_internal(): (vector<u8>, vector<u8>);

    #[test_only]
    native fun sign_internal(sk: vector<u8>, msg: vector<u8>): vector<u8>;

    #[test_only]
    native fun generate_proof_of_possession_internal(sk: vector<u8>): vector<u8>;

    //
    // Constants and helpers for tests
    //

    /// Random signature generated by running `cargo test -- bls12381_sample_signature --nocapture --include-ignored` in `crates/aptos-crypto`.
    /// The message signed is "Hello Aptos!" and the associated SK is 07416693b6b32c84abe45578728e2379f525729e5b94762435a31e65ecc728da.
    const RANDOM_SIGNATURE: vector<u8> = x"a01a65854f987d3434149b7f08f70730e30b241984e8712bc2aca885d632aafced4c3f661209debb6b1c8601326623cc16ca2f6c9edc53b7b88b7435fb6b05ddece418d2c34dc6aca2f5a11a79e67774582c14084a01dcb7820e4cb4bad0ea8d";

    /// Random signature generated by running `cargo test -- bls12381_sample_signature --nocapture --include-ignored` in `crates/aptos-crypto`.
    /// The associated SK is 07416693b6b32c84abe45578728e2379f525729e5b94762435a31e65ecc728da.
    const RANDOM_PK: vector<u8> = x"8a53e7ae5270e3e765cd8a4032c2e77c6f7e87a44ebb85bf28a4d7865565698f975346714262f9e47c6f3e0d5d951660";

    //
    // Tests
    //

    #[test_only]
    fun get_random_aggsig(): AggrOrMultiSignature {
        assert!(signature_subgroup_check_internal(RANDOM_SIGNATURE), 1);

        AggrOrMultiSignature { bytes: RANDOM_SIGNATURE }
    }

    #[test_only]
    fun get_random_pk_with_pop(): PublicKeyWithPoP {
        assert!(validate_pubkey_internal(RANDOM_PK), 1);

        PublicKeyWithPoP {
            bytes: RANDOM_PK
        }
    }

    #[test]
    fun test_pubkey_validation() {
        // test low order points (in group for PK)
        assert!(option::is_none(&public_key_from_bytes(x"ae3cd9403b69c20a0d455fd860e977fe6ee7140a7f091f26c860f2caccd3e0a7a7365798ac10df776675b3a67db8faa0")), 1);
        assert!(option::is_none(&public_key_from_bytes(x"928d4862a40439a67fd76a9c7560e2ff159e770dcf688ff7b2dd165792541c88ee76c82eb77dd6e9e72c89cbf1a56a68")), 1);
        assert!(option::is_some(&public_key_from_bytes(x"b3e4921277221e01ed71284be5e3045292b26c7f465a6fcdba53ee47edd39ec5160da3b229a73c75671024dcb36de091")), 1);
    }

    #[test]
    #[expected_failure(abort_code = 65537, location = Self)]
    fun test_empty_pubkey_aggregation() {
        // First, make sure if no inputs are given, the function returns None
        // assert!(aggregate_pop_verified_pubkeys(vector::empty()) == option::none(), 1);
        aggregate_pubkeys(std::vector::empty());
    }

    #[test]
    fun test_pubkey_aggregation() {
        // Second, try some test-cases generated by running the following command in `crates/aptos-crypto`:
        //  $ cargo test -- sample_aggregate_pk_and_multisig --nocapture --include-ignored
        let pks = vector[
            PublicKeyWithPoP { bytes: x"92e201a806af246f805f460fbdc6fc90dd16a18d6accc236e85d3578671d6f6690dde22134d19596c58ce9d63252410a" },
            PublicKeyWithPoP { bytes: x"ab9df801c6f96ade1c0490c938c87d5bcc2e52ccb8768e1b5d14197c5e8bfa562783b96711b702dda411a1a9f08ebbfa" },
            PublicKeyWithPoP { bytes: x"b698c932cf7097d99c17bd6e9c9dc4eeba84278c621700a8f80ec726b1daa11e3ab55fc045b4dbadefbeef05c4182494" },
            PublicKeyWithPoP { bytes: x"934706a8b876d47a996d427e1526ce52c952d5ec0858d49cd262efb785b62b1972d06270b0a7adda1addc98433ad1843" },
            PublicKeyWithPoP { bytes: x"a4cd352daad3a0651c1998dfbaa7a748e08d248a54347544bfedd51a197e016bb6008e9b8e45a744e1a030cc3b27d2da" },
        ];

        // agg_pks[i] = \sum_{j <= i}  pk[j]
        let agg_pks = vector[
            AggrPublicKeysWithPoP { bytes: x"92e201a806af246f805f460fbdc6fc90dd16a18d6accc236e85d3578671d6f6690dde22134d19596c58ce9d63252410a" },
            AggrPublicKeysWithPoP { bytes: x"b79ad47abb441d7eda9b220a626df2e4e4910738c5f777947f0213398ecafae044ec0c20d552d1348347e9abfcf3eca1" },
            AggrPublicKeysWithPoP { bytes: x"b5f5eb6153ab5388a1a76343d714e4a2dcf224c5d0722d1e8e90c6bcead05c573fffe986460bd4000645a655bf52bc60" },
            AggrPublicKeysWithPoP { bytes: x"b922006ec14c183572a8864c31dc6632dccffa9f9c86411796f8b1b5a93a2457762c8e2f5ef0a2303506c4bca9a4e0bf" },
            AggrPublicKeysWithPoP { bytes: x"b53df1cfee2168f59e5792e710bf22928dc0553e6531dae5c7656c0a66fc12cb82fbb04863938c953dc901a5a79cc0f3" },
        ];

        let i = 0;
        let accum_pk = std::vector::empty<PublicKeyWithPoP>();
        while (i < std::vector::length(&pks)) {
            std::vector::push_back(&mut accum_pk, *std::vector::borrow(&pks, i));

            let apk = aggregate_pubkeys(accum_pk);

            // Make sure PKs were aggregated correctly
            assert!(apk == *std::vector::borrow(&agg_pks, i), 1);
            assert!(validate_pubkey_internal(apk.bytes), 1);

            i = i + 1;
        };
    }

    #[test]
    fun test_pubkey_validation_against_invalid_keys() {
        let (_sk, pk) = generate_keys();
        let pk_bytes = public_key_with_pop_to_bytes(&pk);
        assert!(option::is_some(&public_key_from_bytes(pk_bytes)), 1);
        assert!(option::is_none(&public_key_from_bytes(maul_bytes(&pk_bytes))), 1);
    }

    #[test]
    fun test_signature_aggregation() {
        // First, test empty aggregation
        assert!(option::is_none(&mut aggregate_signatures(vector[])), 1);

        // Second, try some test-cases generated by running the following command in `crates/aptos-crypto`:
        //  $ cargo test -- sample_aggregate_sigs --nocapture --include-ignored

        // Signatures of each signer i
        let sigs = vector[
            signature_from_bytes(x"a55ac2d64b4c1d141b15d876d3e54ad1eea07ee488e8287cce7cdf3eec551458ab5795ab196f8c112590346f7bc7c97e0053cd5be0f9bd74b93a87cd44458e98d125d6d5c6950ea5e62666beb34422ead79121f8cb0815dae41a986688d03eaf"),
            signature_from_bytes(x"90a639a44491191c46379a843266c293de3a46197714ead2ad3886233dd5c2b608b6437fa32fbf9d218b20f1cbfa7970182663beb9c148e2e9412b148e16abf283ffa51b8a536c0e55d61b2e5c849edc49f636c0ef07cb99f125cbcf602e22bb"),
            signature_from_bytes(x"9527d81aa15863ef3a3bf96bea6d58157d5063a93a6d0eb9d8b4f4bbda3b31142ec4586cb519da2cd7600941283d1bad061b5439703fd584295b44037a969876962ae1897dcc7cadf909d06faae213c4fef8e015dfb33ec109af02ab0c3f6833"),
            signature_from_bytes(x"a54d264f5cab9654b1744232c4650c42b29adf2b19bd00bbdaf4a4d792ee4dfd40a1fe1b067f298bcfd8ae4fdc8250660a2848bd4a80d96585afccec5c6cfa617033dd7913c9acfdf98a72467e8a5155d4cad589a72d6665be7cb410aebc0068"),
            signature_from_bytes(x"8d22876bdf73e6ad36ed98546018f6258cd47e45904b87c071e774a6ef4b07cac323258cb920b2fe2b07cca1f2b24bcb0a3194ec76f32edb92391ed2c39e1ada8919f8ea755c5e39873d33ff3a8f4fba21b1261c1ddb9d1688c2b40b77e355d1"),
        ];

        // multisigs[i] is a signature on "Hello, Aptoverse!" from signers 1 through i (inclusive)
        let multisigs = vector[
            AggrOrMultiSignature { bytes: x"a55ac2d64b4c1d141b15d876d3e54ad1eea07ee488e8287cce7cdf3eec551458ab5795ab196f8c112590346f7bc7c97e0053cd5be0f9bd74b93a87cd44458e98d125d6d5c6950ea5e62666beb34422ead79121f8cb0815dae41a986688d03eaf" },
            AggrOrMultiSignature { bytes: x"8f1949a06b95c3cb62898d861f889350c0d2cb740da513bfa195aa0ab8fa006ea2efe004a7bbbd9bb363637a279aed20132efd0846f520e7ee0e8ed847a1c6969bb986ad2239bcc9af561b6c2aa6d3016e1c722146471f1e28313de189fe7ebc" },
            AggrOrMultiSignature { bytes: x"ab5ad42bb8f350f8a6b4ae897946a05dbe8f2b22db4f6c37eff6ff737aebd6c5d75bd1abdfc99345ac8ec38b9a449700026f98647752e1c99f69bb132340f063b8a989728e0a3d82a753740bf63e5d8f51e413ebd9a36f6acbe1407a00c4b3e7" },
            AggrOrMultiSignature { bytes: x"ae307a0d055d3ba55ad6ec7094adef27ed821bdcf735fb509ab2c20b80952732394bc67ea1fd8c26ea963540df7448f8102509f7b8c694e4d75f30a43c455f251b6b3fd8b580b9228ffeeb9039834927aacefccd3069bef4b847180d036971cf" },
            AggrOrMultiSignature { bytes: x"8284e4e3983f29cb45020c3e2d89066df2eae533a01cb6ca2c4d466b5e02dd22467f59640aa120db2b9cc49e931415c3097e3d54ff977fd9067b5bc6cfa1c885d9d8821aef20c028999a1d97e783ae049d8fa3d0bbac36ce4ca8e10e551d3461" },
        ];

        let i = 0;
        let accum_sigs = std::vector::empty<Signature>();
        while (i < std::vector::length(&sigs)) {
            std::vector::push_back(&mut accum_sigs, *std::vector::borrow(&sigs, i));

            let multisig = option::extract(&mut aggregate_signatures(accum_sigs));

            // Make sure sigs were aggregated correctly
            assert!(multisig == *std::vector::borrow(&multisigs, i), 1);
            assert!(signature_subgroup_check_internal(multisig.bytes), 1);

            i = i + 1;
        };
    }

    #[test]
    fun test_empty_signature_aggregation() {
        assert!(option::is_none(&mut aggregate_signatures(vector[])), 1);
    }

    #[test]
    fun test_verify_multisig() {
        // Second, try some test-cases generated by running the following command in `crates/aptos-crypto`:
        //  $ cargo test -- sample_aggregate_pk_and_multisig --nocapture --include-ignored
        let pks = vector[
            PublicKeyWithPoP { bytes: x"92e201a806af246f805f460fbdc6fc90dd16a18d6accc236e85d3578671d6f6690dde22134d19596c58ce9d63252410a" },
            PublicKeyWithPoP { bytes: x"ab9df801c6f96ade1c0490c938c87d5bcc2e52ccb8768e1b5d14197c5e8bfa562783b96711b702dda411a1a9f08ebbfa" },
            PublicKeyWithPoP { bytes: x"b698c932cf7097d99c17bd6e9c9dc4eeba84278c621700a8f80ec726b1daa11e3ab55fc045b4dbadefbeef05c4182494" },
            PublicKeyWithPoP { bytes: x"934706a8b876d47a996d427e1526ce52c952d5ec0858d49cd262efb785b62b1972d06270b0a7adda1addc98433ad1843" },
            PublicKeyWithPoP { bytes: x"a4cd352daad3a0651c1998dfbaa7a748e08d248a54347544bfedd51a197e016bb6008e9b8e45a744e1a030cc3b27d2da" },
        ];

        // agg_pks[i] = \sum_{j <= i}  pk[j]
        let agg_pks = vector[
            AggrPublicKeysWithPoP { bytes: x"92e201a806af246f805f460fbdc6fc90dd16a18d6accc236e85d3578671d6f6690dde22134d19596c58ce9d63252410a" },
            AggrPublicKeysWithPoP { bytes: x"b79ad47abb441d7eda9b220a626df2e4e4910738c5f777947f0213398ecafae044ec0c20d552d1348347e9abfcf3eca1" },
            AggrPublicKeysWithPoP { bytes: x"b5f5eb6153ab5388a1a76343d714e4a2dcf224c5d0722d1e8e90c6bcead05c573fffe986460bd4000645a655bf52bc60" },
            AggrPublicKeysWithPoP { bytes: x"b922006ec14c183572a8864c31dc6632dccffa9f9c86411796f8b1b5a93a2457762c8e2f5ef0a2303506c4bca9a4e0bf" },
            AggrPublicKeysWithPoP { bytes: x"b53df1cfee2168f59e5792e710bf22928dc0553e6531dae5c7656c0a66fc12cb82fbb04863938c953dc901a5a79cc0f3" },
        ];

        // multisigs[i] is a signature on "Hello, Aptoverse!" under agg_pks[i]
        let multisigs = vector[
            AggrOrMultiSignature { bytes: x"ade45c67bff09ae57e0575feb0be870f2d351ce078e8033d847615099366da1299c69497027b77badb226ff1708543cd062597030c3f1553e0aef6c17e7af5dd0de63c1e4f1f9da68c966ea6c1dcade2cdc646bd5e8bcd4773931021ec5be3fd" },
            AggrOrMultiSignature { bytes: x"964af3d83436f6a9a382f34590c0c14e4454dc1de536af205319ce1ed417b87a2374863d5df7b7d5ed900cf91dffa7a105d3f308831d698c0d74fb2259d4813434fb86425db0ded664ae8f85d02ec1d31734910317d4155cbf69017735900d4d" },
            AggrOrMultiSignature { bytes: x"b523a31813e771e55aa0fc99a48db716ecc1085f9899ccadb64e759ecb481a2fb1cdcc0b266f036695f941361de773081729311f6a1bca9d47393f5359c8c87dc34a91f5dae335590aacbff974076ad1f910dd81750553a72ccbcad3c8cc0f07" },
            AggrOrMultiSignature { bytes: x"a945f61699df58617d37530a85e67bd1181349678b89293951ed29d1fb7588b5c12ebb7917dfc9d674f3f4fde4d062740b85a5f4927f5a4f0091e46e1ac6e41bbd650a74dd49e91445339d741e3b10bdeb9bc8bba46833e0011ff91fa5c77bd2" },
            AggrOrMultiSignature { bytes: x"b627b2cfd8ae59dcf5e58cc6c230ae369985fd096e1bc3be38da5deafcbed7d939f07cccc75383539940c56c6b6453db193f563f5b6e4fe54915afd9e1baea40a297fa7eda74abbdcd4cc5c667d6db3b9bd265782f7693798894400f2beb4637" },
        ];

        let i = 0;
        let accum_pk = std::vector::empty<PublicKeyWithPoP>();
        while (i < std::vector::length(&pks)) {
            std::vector::push_back(&mut accum_pk, *std::vector::borrow(&pks, i));

            let apk = aggregate_pubkeys(accum_pk);

            assert!(apk == *std::vector::borrow(&agg_pks, i), 1);

            assert!(verify_multisignature(std::vector::borrow(&multisigs, i), &apk, b"Hello, Aptoverse!"), 1);

            i = i + 1;
        };
    }

    #[test]
    fun test_verify_multisignature_randomized() {
        let signer_count = 1;
        let max_signer_count = 5;
        let msg = b"hello world";
        while (signer_count <= max_signer_count) {
            // Generate key pairs.
            let signing_keys = vector[];
            let public_keys = vector[];
            let i = 0;
            while (i < signer_count) {
                let (sk, pk) = generate_keys();
                std::vector::push_back(&mut signing_keys, sk);
                std::vector::push_back(&mut public_keys, pk);
                i = i + 1;
            };

            // Generate multi-signature.
            let aggr_pk = aggregate_pubkeys(public_keys);
            let multisig = multi_sign_arbitrary_bytes(&signing_keys, msg);

            // Test signature verification.
            assert!(verify_multisignature(&multisig, &aggr_pk, msg), 1);
            assert!(!verify_multisignature(&maul_aggr_or_multi_signature(&multisig), &aggr_pk, msg), 1);
            assert!(!verify_multisignature(&multisig, &maul_aggregated_public_key(&aggr_pk), msg), 1);
            assert!(!verify_multisignature(&multisig, &aggr_pk, maul_bytes(&msg)), 1);

            // Also test signature aggregation.
            let signatures = vector[];
            let i = 0;
            while (i < signer_count) {
                let sk = std::vector::borrow(&signing_keys, i);
                let sig = sign_arbitrary_bytes(sk, msg);
                std::vector::push_back(&mut signatures, sig);
                i = i + 1;
            };
            let aggregated_signature = option::extract(&mut aggregate_signatures(signatures));
            assert!(aggr_or_multi_signature_subgroup_check(&aggregated_signature), 1);
            assert!(aggr_or_multi_signature_to_bytes(&aggregated_signature) == aggr_or_multi_signature_to_bytes(&multisig), 1);

            signer_count = signer_count + 1;
        }
    }

    #[test]
    fun test_verify_aggsig() {
        assert!(aggr_or_multi_signature_to_bytes(&aggr_or_multi_signature_from_bytes(RANDOM_SIGNATURE)) == RANDOM_SIGNATURE, 1);

        // First, make sure verification returns None when no inputs are given or |pks| != |msgs|
        assert!(verify_aggregate_signature(&get_random_aggsig(), vector[], vector[]) == false, 1);

        assert!(verify_aggregate_signature(
            &get_random_aggsig(),
            vector[ get_random_pk_with_pop() ],
            vector[]) == false, 1);

        assert!(verify_aggregate_signature(
            &get_random_aggsig(),
            vector[],
            vector[ x"ab" ]) == false, 1);

        assert!(verify_aggregate_signature(
            &get_random_aggsig(),
            vector[ get_random_pk_with_pop() ],
            vector[
                x"cd", x"ef"
            ]) == false, 1);

        assert!(verify_aggregate_signature(
            &get_random_aggsig(),
            vector[
                get_random_pk_with_pop(),
                get_random_pk_with_pop(),
                get_random_pk_with_pop(),
            ],
            vector[
                x"cd", x"ef"
            ]) == false, 1);

        // Second, try some test-cases generated by running the following command in `crates/aptos-crypto`:
        //  $ cargo test -- bls12381_sample_aggregate_pk_and_aggsig --nocapture --ignored

        // The signed messages are "Hello, Aptos <i>!", where <i> \in {1, ..., 5}
        let msgs = vector[
            x"48656c6c6f2c204170746f73203121",
            x"48656c6c6f2c204170746f73203221",
            x"48656c6c6f2c204170746f73203321",
            x"48656c6c6f2c204170746f73203421",
            x"48656c6c6f2c204170746f73203521",
        ];

        // Public key of signer i
        let pks = vector[
            PublicKeyWithPoP { bytes: x"b93d6aabb2b83e52f4b8bda43c24ea920bbced87a03ffc80f8f70c814a8b3f5d69fbb4e579ca76ee008d61365747dbc6" },
            PublicKeyWithPoP { bytes: x"b45648ceae3a983bcb816a96db599b5aef3b688c5753fa20ce36ac7a4f2c9ed792ab20af6604e85e42dab746398bb82c" },
            PublicKeyWithPoP { bytes: x"b3e4921277221e01ed71284be5e3045292b26c7f465a6fcdba53ee47edd39ec5160da3b229a73c75671024dcb36de091" },
            PublicKeyWithPoP { bytes: x"8463b8671c9775a7dbd98bf76d3deba90b5a90535fc87dc8c13506bb5c7bbd99be4d257e60c548140e1e30b107ff5822" },
            PublicKeyWithPoP { bytes: x"a79e3d0e9d04587a3b27d05efe5717da05fd93485dc47978c866dc70a01695c2efd247d1dd843a011a4b6b24079d7384" },
        ];

        // aggsigs[i] = \sum_{j <= i}  sigs[j], where sigs[j] is a signature on msgs[j] under pks[j]
        let aggsigs = vector[
            AggrOrMultiSignature { bytes: x"a2bc8bdebe6215ba74b5b53c5ed2aa0c68221a4adf868989ccdcfb62bb0eecc6537def9ee686a7960169c5917d25e5220177ed1c5e95ecfd68c09694062e76efcb00759beac874e4f9a715fd144210883bf9bb272f156b0a1fa15d0e9460f01f" },
            AggrOrMultiSignature { bytes: x"a523aa3c3f1f1074d968ffecf017c7b93ae5243006bf0abd2e45c036ddbec99302984b650ebe5ba306cda4071d281ba50a99ef0e66c3957fab94163296f9d673fc58a36de4276f82bfb1d9180b591df93b5c2804d40dd68cf0f72cd92f86442e" },
            AggrOrMultiSignature { bytes: x"abed10f464de74769121fc09715e59a3ac96a5054a43a9d43cc890a2d4d332614c74c7fb4cceef6d25f85c65dee337330f062f89f23fec9ecf7ce3193fbba2c886630d753be6a4513a4634428904b767af2f230c5cadbcb53a451dd9c7d977f6" },
            AggrOrMultiSignature { bytes: x"8362871631ba822742a31209fa4abce6dc94b741ac4725995459da2951324b51efbbf6bc3ab4681e547ebfbadd80e0360dc078c04188198f0acea26c12645ace9107a4a23cf8db46abc7a402637f16a0477c72569fc9966fe804ef4dc0e5e758" },
            AggrOrMultiSignature { bytes: x"a44d967935fbe63a763ce2dd2b16981f967ecd31e20d3266eef5517530cdc233c8a18273b6d9fd7f61dd39178826e3f115df4e7b304f2de17373a95ea0c9a14293dcfd6f0ef416e06fa23f6a3c850d638e4d8f97ab4562ef55d49a96a50baa13" },
        ];

        let i = 0;
        let msg_subset = std::vector::empty<vector<u8>>();
        let pk_subset = std::vector::empty<PublicKeyWithPoP>();
        while (i < std::vector::length(&pks)) {
            let aggsig = *std::vector::borrow(&aggsigs, i);

            std::vector::push_back(&mut pk_subset, *std::vector::borrow(&pks, i));
            std::vector::push_back(&mut msg_subset, *std::vector::borrow(&msgs, i));

            assert!(verify_aggregate_signature(&aggsig, pk_subset, msg_subset), 1);

            i = i + 1;
        };
    }

    #[test]
    fun test_verify_aggregated_signature_randomized() {
        let signer_count = 1;
        let max_signer_count = 5;
        while (signer_count <= max_signer_count) {
            // Generate key pairs and messages.
            let signing_keys = vector[];
            let public_keys = vector[];
            let messages: vector<vector<u8>> = vector[];
            let i = 0;
            while (i < signer_count) {
                let (sk, pk) = generate_keys();
                std::vector::push_back(&mut signing_keys, sk);
                std::vector::push_back(&mut public_keys, pk);
                let msg: vector<u8> = vector[104, 101, 108, 108, 111, 32, 97, 112, 116, 111, 115, 32, 117, 115, 101, 114, 32, 48+(i as u8)]; //"hello aptos user {i}"
                std::vector::push_back(&mut messages, msg);
                i = i + 1;
            };

            // Maul messages and public keys.
            let mauled_public_keys = vector[maul_public_key_with_pop(std::vector::borrow(&public_keys, 0))];
            let mauled_messages = vector[maul_bytes(std::vector::borrow(&messages, 0))];
            let i = 1;
            while (i < signer_count) {
                let pk = std::vector::borrow(&public_keys, i);
                let msg = std::vector::borrow(&messages, i);
                std::vector::push_back(&mut mauled_public_keys, *pk);
                std::vector::push_back(&mut mauled_messages, *msg);
                i = i + 1;
            };

            // Generate aggregated signature.
            let aggrsig = aggr_sign_arbitrary_bytes(&signing_keys, &messages);

            // Test signature verification.
            assert!(verify_aggregate_signature(&aggrsig, public_keys, messages), 1);
            assert!(!verify_aggregate_signature(&maul_aggr_or_multi_signature(&aggrsig), public_keys, messages), 1);
            assert!(!verify_aggregate_signature(&aggrsig, mauled_public_keys, messages), 1);
            assert!(!verify_aggregate_signature(&aggrsig, public_keys, mauled_messages), 1);

            // Also test signature aggregation.
            let signatures = vector[];
            let i = 0;
            while (i < signer_count) {
                let sk = std::vector::borrow(&signing_keys, i);
                let msg = std::vector::borrow(&messages, i);
                let sig = sign_arbitrary_bytes(sk, *msg);
                std::vector::push_back(&mut signatures, sig);
                i = i + 1;
            };
            let aggrsig_another = option::extract(&mut aggregate_signatures(signatures));
            assert!(aggr_or_multi_signature_to_bytes(&aggrsig_another) == aggr_or_multi_signature_to_bytes(&aggrsig), 1);

            signer_count = signer_count + 1;
        }
    }

    #[test]
    /// Tests verification of a random BLS signature created using sk = x""
    fun test_verify_normal_and_verify_sigshare() {
        // Test case generated by running `cargo test -- bls12381_sample_signature --nocapture --include-ignored` in
        // `crates/aptos-crypto`
        // =============================================================================================================
        // SK:        077c8a56f26259215a4a245373ab6ddf328ac6e00e5ea38d8700efa361bdc58d

        let message = b"Hello Aptos!";

        // First, test signatures that verify
        let ok = verify_normal_signature(
            &signature_from_bytes(x"b01ce4632e94d8c611736e96aa2ad8e0528a02f927a81a92db8047b002a8c71dc2d6bfb94729d0973790c10b6ece446817e4b7543afd7ca9a17c75de301ae835d66231c26a003f11ae26802b98d90869a9e73788c38739f7ac9d52659e1f7cf7"),
            &option::extract(&mut public_key_from_bytes(x"94209a296b739577cb076d3bfb1ca8ee936f29b69b7dae436118c4dd1cc26fd43dcd16249476a006b8b949bf022a7858")),
            message,
        );
        assert!(ok == true, 1);

        let pk = option::extract(&mut public_key_from_bytes(x"94209a296b739577cb076d3bfb1ca8ee936f29b69b7dae436118c4dd1cc26fd43dcd16249476a006b8b949bf022a7858"));
        let pk_with_pop = PublicKeyWithPoP { bytes: pk.bytes };

        let ok = verify_signature_share(
            &signature_from_bytes(x"b01ce4632e94d8c611736e96aa2ad8e0528a02f927a81a92db8047b002a8c71dc2d6bfb94729d0973790c10b6ece446817e4b7543afd7ca9a17c75de301ae835d66231c26a003f11ae26802b98d90869a9e73788c38739f7ac9d52659e1f7cf7"),
            &pk_with_pop,
            message,
        );
        assert!(ok == true, 1);

        // Second, test signatures that do NOT verify
        let sigs = vector[
            Signature { bytes: x"a01ce4632e94d8c611736e96aa2ad8e0528a02f927a81a92db8047b002a8c71dc2d6bfb94729d0973790c10b6ece446817e4b7543afd7ca9a17c75de301ae835d66231c26a003f11ae26802b98d90869a9e73788c38739f7ac9d52659e1f7cf7" },
            Signature { bytes: x"b01ce4632e94d8c611736e96aa2ad8e0528a02f927a81a92db8047b002a8c71dc2d6bfb94729d0973790c10b6ece446817e4b7543afd7ca9a17c75de301ae835d66231c26a003f11ae26802b98d90869a9e73788c38739f7ac9d52659e1f7cf7" },
            Signature { bytes: x"b01ce4632e94d8c611736e96aa2ad8e0528a02f927a81a92db8047b002a8c71dc2d6bfb94729d0973790c10b6ece446817e4b7543afd7ca9a17c75de301ae835d66231c26a003f11ae26802b98d90869a9e73788c38739f7ac9d52659e1f7cf7" },
        ];
        let pks = vector[
            x"94209a296b739577cb076d3bfb1ca8ee936f29b69b7dae436118c4dd1cc26fd43dcd16249476a006b8b949bf022a7858",
            x"ae4851bb9e7782027437ed0e2c026dd63b77a972ddf4bd9f72bcc218e327986568317e3aa9f679c697a2cb7cebf992f3",
            x"82ed7bb5528303a2e306775040a7309e0bd597b70d9949d8c6198a01a7be0b00079320ebfeaf7bbd5bfe86809940d252",
        ];
        let messages = vector[
            b"Hello Aptos!",
            b"Hello Aptos!",
            b"Bello Aptos!",
        ];

        let i = 0;
        while (i < std::vector::length(&pks)) {
            let sig = std::vector::borrow(&sigs, i);
            let pk = *std::vector::borrow(&pks, i);
            let msg = *std::vector::borrow(&messages, i);

            let pk = option::extract(&mut public_key_from_bytes(pk));

            let notok = verify_normal_signature(
                sig,
                &pk,
                msg,
            );
            assert!(notok == false, 1);

            let notok = verify_signature_share(
                sig,
                &PublicKeyWithPoP { bytes: pk.bytes },
                msg,
            );
            assert!(notok == false, 1);

            i = i + 1;
        }
    }

    #[test]
    fun test_verify_normal_signature_or_signature_share_randomized() {
        let (sk, pkpop) = generate_keys();
        let pk = public_key_with_pop_to_normal(&pkpop);

        let msg = b"hello world";
        let sig = sign_arbitrary_bytes(&sk, msg);
        assert!(verify_normal_signature(&sig, &pk, msg), 1);
        assert!(!verify_normal_signature(&maul_signature(&sig), &pk, msg), 1);
        assert!(!verify_normal_signature(&sig, &maul_public_key(&pk), msg), 1);
        assert!(!verify_normal_signature(&sig, &pk, maul_bytes(&msg)), 1);

        assert!(verify_signature_share(&sig, &pkpop, msg), 1);
        assert!(!verify_signature_share(&maul_signature(&sig), &pkpop, msg), 1);
        assert!(!verify_signature_share(&sig, &maul_public_key_with_pop(&pkpop), msg), 1);
        assert!(!verify_signature_share(&sig, &pkpop, maul_bytes(&msg)), 1);
    }

    #[test]
    /// Tests verification of random BLS proofs-of-possession (PoPs)
    fun test_verify_pop() {
        // Test case generated by running `cargo test -- sample_pop --nocapture --include-ignored` in `crates/aptos-crypto`
        // =============================================================================================================

        let pks = vector[
            x"808864c91ae7a9998b3f5ee71f447840864e56d79838e4785ff5126c51480198df3d972e1e0348c6da80d396983e42d7",
            x"8843843c76d167c02842a214c21277bad0bfd83da467cb5cf2d3ee67b2dcc7221b9fafa6d430400164012580e0c34d27",
            x"a23b524d4308d46e43ee8cbbf57f3e1c20c47061ad9c3f915212334ea6532451dd5c01d3d3ada6bea10fe180b2c3b450",
            x"a2aaa3eae1df3fc36365491afa1da5181acbb03801afd1430f04bb3b3eb18036f8b756b3508e4caee04beff50d455d1c",
            x"84985b7e983dbdaddfca1f0b7dad9660bb39fff660e329acec15f69ac48c75dfa5d2df9f0dc320e4e7b7658166e0ac1c",
        ];

        let pops = vector[
            proof_of_possession_from_bytes(x"ab42afff92510034bf1232a37a0d31bc8abfc17e7ead9170d2d100f6cf6c75ccdcfedbd31699a112b4464a06fd636f3f190595863677d660b4c5d922268ace421f9e86e3a054946ee34ce29e1f88c1a10f27587cf5ec528d65ba7c0dc4863364"),
            proof_of_possession_from_bytes(x"a6da5f2bc17df70ce664cff3e3a3e09d17162e47e652032b9fedc0c772fd5a533583242cba12095602e422e579c5284b1735009332dbdd23430bbcf61cc506ae37e41ff9a1fc78f0bc0d99b6bc7bf74c8f567dfb59079a035842bdc5fa3a0464"),
            proof_of_possession_from_bytes(x"b8eef236595e2eab34d3c1abdab65971f5cfa1988c731ef62bd63c9a9ad3dfc9259f4f183bfffbc8375a38ba62e1c41a11173209705996ce889859bcbb3ddd7faa3c4ea3d8778f30a9ff814fdcfea1fb163d745c54dfb4dcc5a8cee092ee0070"),
            proof_of_possession_from_bytes(x"a03a12fab68ad59d85c15dd1528560eff2c89250070ad0654ba260fda4334da179811d2ecdaca57693f80e9ce977d62011e3b1ee7bb4f7e0eb9b349468dd758f10fc35d54e0d0b8536ca713a77a301944392a5c192b6adf2a79ae2b38912dc98"),
            proof_of_possession_from_bytes(x"8899b294f3c066e6dfb59bc0843265a1ccd6afc8f0f38a074d45ded8799c39d25ee0376cd6d6153b0d4d2ff8655e578b140254f1287b9e9df4e2aecc5b049d8556a4ab07f574df68e46348fd78e5298b7913377cf5bb3cf4796bfc755902bfdd"),
        ];

        assert!(std::vector::length(&pks) == std::vector::length(&pops), 1);

        let i = 0;
        while (i < std::vector::length(&pks)) {
            let opt_pk = public_key_from_bytes_with_pop(*std::vector::borrow(&pks, i), std::vector::borrow(&pops, i));
            assert!(option::is_some(&opt_pk), 1);

            i = i + 1;
        };

        // assert first PK's PoP does not verify against modifed PK' = 0xa0 | PK[1:]
        let opt_pk = public_key_from_bytes_with_pop(
            x"a08864c91ae7a9998b3f5ee71f447840864e56d79838e4785ff5126c51480198df3d972e1e0348c6da80d396983e42d7",
            &proof_of_possession_from_bytes(x"ab42afff92510034bf1232a37a0d31bc8abfc17e7ead9170d2d100f6cf6c75ccdcfedbd31699a112b4464a06fd636f3f190595863677d660b4c5d922268ace421f9e86e3a054946ee34ce29e1f88c1a10f27587cf5ec528d65ba7c0dc4863364"));
        assert!(option::is_none(&opt_pk), 1);

        // assert first PK's PoP does not verify if modifed as pop' = 0xb0 | pop[1:]
        let opt_pk = public_key_from_bytes_with_pop(
            x"808864c91ae7a9998b3f5ee71f447840864e56d79838e4785ff5126c51480198df3d972e1e0348c6da80d396983e42d7",
            &proof_of_possession_from_bytes(x"bb42afff92510034bf1232a37a0d31bc8abfc17e7ead9170d2d100f6cf6c75ccdcfedbd31699a112b4464a06fd636f3f190595863677d660b4c5d922268ace421f9e86e3a054946ee34ce29e1f88c1a10f27587cf5ec528d65ba7c0dc4863364"));
        assert!(option::is_none(&opt_pk), 1);
    }

    #[test]
    fun test_verify_pop_randomized() {
        let (sk, pk) = generate_keys();
        let pk_bytes = public_key_with_pop_to_bytes(&pk);
        let pop = generate_proof_of_possession(&sk);
        assert!(option::is_some(&public_key_from_bytes_with_pop(pk_bytes, &pop)), 1);
        assert!(option::is_none(&public_key_from_bytes_with_pop(pk_bytes, &maul_proof_of_possession(&pop))), 1);
        assert!(option::is_none(&public_key_from_bytes_with_pop(maul_bytes(&pk_bytes), &pop)), 1);
    }
}
