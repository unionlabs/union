/// Cryptographic hashes:
/// - Keccak-256: see https://keccak.team/keccak.html
///
/// In addition, SHA2-256 and SHA3-256 are available in `std::hash`. Note that SHA3-256 is a variant of Keccak: it is
/// NOT the same as Keccak-256.
///
/// Non-cryptograhic hashes:
/// - SipHash: an add-rotate-xor (ARX) based family of pseudorandom functions created by Jean-Philippe Aumasson and Daniel J. Bernstein in 2012
module aptos_std::aptos_hash {
    use std::bcs;
    use std::features;

    //
    // Constants
    //

    /// A newly-added native function is not yet enabled.
    const E_NATIVE_FUN_NOT_AVAILABLE: u64 = 1;

    //
    // Functions
    //

    /// Returns the (non-cryptographic) SipHash of `bytes`. See https://en.wikipedia.org/wiki/SipHash
    native public fun sip_hash(bytes: vector<u8>): u64;

    /// Returns the (non-cryptographic) SipHash of the BCS serialization of `v`. See https://en.wikipedia.org/wiki/SipHash
    public fun sip_hash_from_value<MoveValue>(v: &MoveValue): u64 {
        let bytes = bcs::to_bytes(v);

        sip_hash(bytes)
    }

    /// Returns the Keccak-256 hash of `bytes`.
    native public fun keccak256(bytes: vector<u8>): vector<u8>;

    /// Returns the SHA2-512 hash of `bytes`.
    public fun sha2_512(bytes: vector<u8>): vector<u8> {
        if(!features::sha_512_and_ripemd_160_enabled()) {
            abort(std::error::invalid_state(E_NATIVE_FUN_NOT_AVAILABLE))
        };

        sha2_512_internal(bytes)
    }

    /// Returns the SHA3-512 hash of `bytes`.
    public fun sha3_512(bytes: vector<u8>): vector<u8> {
        if(!features::sha_512_and_ripemd_160_enabled()) {
            abort(std::error::invalid_state(E_NATIVE_FUN_NOT_AVAILABLE))
        };

        sha3_512_internal(bytes)
    }


    /// Returns the RIPEMD-160 hash of `bytes`.
    ///
    /// WARNING: Only 80-bit security is provided by this function. This means an adversary who can compute roughly 2^80
    /// hashes will, with high probability, find a collision x_1 != x_2 such that RIPEMD-160(x_1) = RIPEMD-160(x_2).
    public fun ripemd160(bytes: vector<u8>): vector<u8> {
        if(!features::sha_512_and_ripemd_160_enabled()) {
            abort(std::error::invalid_state(E_NATIVE_FUN_NOT_AVAILABLE))
        };

        ripemd160_internal(bytes)
    }

    /// Returns the BLAKE2B-256 hash of `bytes`.
    public fun blake2b_256(bytes: vector<u8>): vector<u8> {
        if(!features::blake2b_256_enabled()) {
            abort(std::error::invalid_state(E_NATIVE_FUN_NOT_AVAILABLE))
        };

        blake2b_256_internal(bytes)
    }

    //
    // Private native functions
    //

    /// Returns the SHA2-512 hash of `bytes`.
    native fun sha2_512_internal(bytes: vector<u8>): vector<u8>;


    /// Returns the SHA3-512 hash of `bytes`.
    native fun sha3_512_internal(bytes: vector<u8>): vector<u8>;

    /// Returns the RIPEMD-160 hash of `bytes`.
    ///
    /// WARNING: Only 80-bit security is provided by this function. This means an adversary who can compute roughly 2^80
    /// hashes will, with high probability, find a collision x_1 != x_2 such that RIPEMD-160(x_1) = RIPEMD-160(x_2).
    native fun ripemd160_internal(bytes: vector<u8>): vector<u8>;

    /// Returns the BLAKE2B-256 hash of `bytes`.
    native fun blake2b_256_internal(bytes: vector<u8>): vector<u8>;

    //
    // Testing
    //

    #[test]
    fun keccak256_test() {
        let inputs = vector[
            b"testing",
            b"",
        ];

        let outputs = vector[
            x"5f16f4c7f149ac4f9510d9cf8cf384038ad348b3bcdc01915f95de12df9d1b02",
            x"c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470",
        ];

        let i = 0;
        while (i < std::vector::length(&inputs)) {
            let input = *std::vector::borrow(&inputs, i);
            let hash_expected = *std::vector::borrow(&outputs, i);
            let hash = keccak256(input);

            assert!(hash_expected == hash, 1);

            i = i + 1;
        };
    }

    #[test(fx = @aptos_std)]
    fun sha2_512_test(fx: signer) {
        // We need to enable the feature in order for the native call to be allowed.
        features::change_feature_flags_for_testing(&fx, vector[features::get_sha_512_and_ripemd_160_feature()], vector[]);

        let inputs = vector[
        b"testing",
        b"",
        ];

        // From https://emn178.github.io/online-tools/sha512.html
        let outputs = vector[
        x"521b9ccefbcd14d179e7a1bb877752870a6d620938b28a66a107eac6e6805b9d0989f45b5730508041aa5e710847d439ea74cd312c9355f1f2dae08d40e41d50",
        x"cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
        ];

        let i = 0;
        while (i < std::vector::length(&inputs)) {
            let input = *std::vector::borrow(&inputs, i);
            let hash_expected = *std::vector::borrow(&outputs, i);
            let hash = sha2_512(input);

            assert!(hash_expected == hash, 1);

            i = i + 1;
        };
    }

    #[test(fx = @aptos_std)]
    fun sha3_512_test(fx: signer) {
        // We need to enable the feature in order for the native call to be allowed.
        features::change_feature_flags_for_testing(&fx, vector[features::get_sha_512_and_ripemd_160_feature()], vector[]);
        let inputs = vector[
        b"testing",
        b"",
        ];

        // From https://emn178.github.io/online-tools/sha3_512.html
        let outputs = vector[
        x"881c7d6ba98678bcd96e253086c4048c3ea15306d0d13ff48341c6285ee71102a47b6f16e20e4d65c0c3d677be689dfda6d326695609cbadfafa1800e9eb7fc1",
        x"a69f73cca23a9ac5c8b567dc185a756e97c982164fe25859e0d1dcc1475c80a615b2123af1f5f94c11e3e9402c3ac558f500199d95b6d3e301758586281dcd26",
        ];

        let i = 0;
        while (i < std::vector::length(&inputs)) {
            let input = *std::vector::borrow(&inputs, i);
            let hash_expected = *std::vector::borrow(&outputs, i);
            let hash = sha3_512(input);

            assert!(hash_expected == hash, 1);

            i = i + 1;
        };
    }

    #[test(fx = @aptos_std)]
    fun ripemd160_test(fx: signer) {
        // We need to enable the feature in order for the native call to be allowed.
        features::change_feature_flags_for_testing(&fx, vector[features::get_sha_512_and_ripemd_160_feature()], vector[]);
        let inputs = vector[
        b"testing",
        b"",
        ];

        // From https://www.browserling.com/tools/ripemd160-hash
        let outputs = vector[
        x"b89ba156b40bed29a5965684b7d244c49a3a769b",
        x"9c1185a5c5e9fc54612808977ee8f548b2258d31",
        ];

        let i = 0;
        while (i < std::vector::length(&inputs)) {
            let input = *std::vector::borrow(&inputs, i);
            let hash_expected = *std::vector::borrow(&outputs, i);
            let hash = ripemd160(input);

            assert!(hash_expected == hash, 1);

            i = i + 1;
        };
    }

    #[test(fx = @aptos_std)]
    #[expected_failure(abort_code = 196609, location = Self)]
    fun blake2b_256_aborts(fx: signer) {
        // We disable the feature to make sure the `blake2b_256` call aborts
        features::change_feature_flags_for_testing(&fx, vector[], vector[features::get_blake2b_256_feature()]);

        blake2b_256(b"This will abort");
    }

    #[test(fx = @aptos_std)]
    fun blake2b_256_test(fx: signer) {
        // We need to enable the feature in order for the native call to be allowed.
        features::change_feature_flags_for_testing(&fx, vector[features::get_blake2b_256_feature()], vector[]);
        let inputs = vector[
        b"",
        b"testing",
        b"testing again", // empty message doesn't yield an output on the online generator
        ];

        // From https://www.toolkitbay.com/tkb/tool/BLAKE2b_256
        //
        // For computing the hash of an empty string, we use the following Python3 script:
        // ```
        //   #!/usr/bin/python3
        //
        //   import hashlib
        //
        //   print(hashlib.blake2b(b'', digest_size=32).hexdigest());
        // ```
        let outputs = vector[
        x"0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8",
        x"99397ff32ae348b8b6536d5c213f343d7e9fdeaa10e8a23a9f90ab21a1658565",
        x"1deab5a4eb7481453ca9b29e1f7c4be8ba44de4faeeafdf173b310cbaecfc84c",
        ];

        let i = 0;
        while (i < std::vector::length(&inputs)) {
            let input = *std::vector::borrow(&inputs, i);
            let hash_expected = *std::vector::borrow(&outputs, i);
            let hash = blake2b_256(input);

            assert!(hash_expected == hash, 1);

            i = i + 1;
        };
    }
}
