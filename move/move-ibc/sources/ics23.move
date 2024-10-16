module IBC::ics23 {
    use std::vector;
    use std::option::Option;
    use std::hash;
    use IBC::bcs_utils::{Self, BcsBuf};
    #[test_only]
    use IBC::connection_end;
    use IBC::proto_utils;

    const E_EMPTY_LEAF_PREFIX: u64 = 35200;
    const E_EMPTY_LEAF_KEY: u64 = 35201;
    const E_EMPTY_INNER_KEY: u64 = 35202;
    const E_EMPTY_CHILD: u64 = 35203;
    const E_PROOF_KEY_MISMATCH: u64 = 35204;
    const E_PROOF_VALUE_MISMATCH: u64 = 35205;
    const E_COMMITMENT_ROOT_MISMATCH: u64 = 35206;
    const E_INVALID_LEAF_PREFIX: u64 = 35207;
    const E_INVALID_INNER_PREFIX: u64 = 35208;
    const E_EMPTY_INNER_VALUE: u64 = 35209;

    struct MembershipProof has drop {
        sub_proof: ExistenceProof,
        top_level_proof: ExistenceProof
    }

    struct ExistenceProof has drop, copy {
        key: vector<u8>,
        value: vector<u8>,
        leaf_prefix: vector<u8>,
        path: vector<InnerOp>
    }

    struct InnerOp has drop, copy {
        prefix: vector<u8>,
        suffix: vector<u8>
    }

    struct NonExistenceProof has drop {
        key: vector<u8>,
        left: Option<ExistenceProof>,
        right: Option<ExistenceProof>
    }

    struct ProofSpec has drop {
        child_size: u64,
        min_prefix_length: u64,
        max_prefix_length: u64
    }

    fun iavl_proof_spec(): ProofSpec {
        ProofSpec { child_size: 33, min_prefix_length: 4, max_prefix_length: 12 }
    }

    fun tm_proof_spec(): ProofSpec {
        ProofSpec { child_size: 32, min_prefix_length: 1, max_prefix_length: 1 }
    }

    public fun verify_membership(
        proof: MembershipProof,
        root: vector<u8>,
        prefix: vector<u8>,
        key: vector<u8>,
        value: vector<u8>
    ) {
        let subroot = calculate_existence_root(&proof.sub_proof);

        verify_no_root_check(
            &proof.sub_proof,
            iavl_proof_spec(),
            key,
            value
        );

        verify_existence(
            &proof.top_level_proof,
            tm_proof_spec(),
            root,
            prefix,
            subroot
        );
    }

    fun verify_existence(
        proof: &ExistenceProof,
        proof_spec: ProofSpec,
        commitment_root: vector<u8>,
        key: vector<u8>,
        value: vector<u8>
    ) {
        assert!(key == proof.key, E_PROOF_KEY_MISMATCH);

        assert!(value == proof.value, E_PROOF_VALUE_MISMATCH);

        check_against_spec(proof, proof_spec);

        let root = calculate_existence_root(proof);

        assert!(root == commitment_root, E_COMMITMENT_ROOT_MISMATCH);
    }

    fun verify_no_root_check(
        proof: &ExistenceProof,
        proof_spec: ProofSpec,
        key: vector<u8>,
        value: vector<u8>
    ) {
        assert!(key == proof.key, E_PROOF_KEY_MISMATCH);

        assert!(value == proof.value, E_PROOF_VALUE_MISMATCH);

        check_against_spec(proof, proof_spec);
    }

    fun check_against_spec(proof: &ExistenceProof, proof_spec: ProofSpec) {
        assert!(!vector::is_empty(&proof.leaf_prefix), E_EMPTY_LEAF_PREFIX);

        assert!(*vector::borrow(&proof.leaf_prefix, 0) == 0, E_INVALID_LEAF_PREFIX);

        let max = proof_spec.max_prefix_length + proof_spec.child_size;
        let i = 0;
        while (i < vector::length(&proof.path)) {
            let inner_op = vector::borrow(&proof.path, i);
            assert!(
                vector::length(&inner_op.prefix) >= proof_spec.min_prefix_length
                    && *vector::borrow(&inner_op.prefix, 0) != 0
                    || vector::length(&inner_op.prefix) <= max,
                E_INVALID_INNER_PREFIX
            );

            i = i + 1;
        };
    }

    fun calculate_existence_root(proof: &ExistenceProof): vector<u8> {
        assert!(!vector::is_empty(&proof.leaf_prefix), E_EMPTY_LEAF_PREFIX);

        let root = apply_leaf_op(&proof.leaf_prefix, &proof.key, &proof.value);

        let i = 0;
        while (i < vector::length(&proof.path)) {
            root = apply_inner_op(*vector::borrow(&proof.path, i), root);
            i = i + 1;
        };

        root
    }

    fun apply_leaf_op(
        prefix: &vector<u8>, key: &vector<u8>, value: &vector<u8>
    ): vector<u8> {
        assert!(!vector::is_empty(key), E_EMPTY_INNER_KEY);

        assert!(!vector::is_empty(value), E_EMPTY_INNER_VALUE);

        let encoded_key = proto_utils::encode_varint(vector::length(key));

        let hashed_value = hash::sha2_256(*value);
        let encoded_value = proto_utils::encode_varint(32);

        let hash_data: vector<u8> = vector::empty();
        vector::append(&mut hash_data, *prefix);
        vector::append(&mut hash_data, encoded_key);
        vector::append(&mut hash_data, *key);
        vector::append(&mut hash_data, encoded_value);
        vector::append(&mut hash_data, hashed_value);

        hash::sha2_256(hash_data)
    }

    fun apply_inner_op(inner_op: InnerOp, child: vector<u8>): vector<u8> {
        assert!(!vector::is_empty(&child), E_EMPTY_CHILD);

        let pre_image = inner_op.prefix;
        vector::append(&mut pre_image, child);
        vector::append(&mut pre_image, inner_op.suffix);

        hash::sha2_256(pre_image)
    }

    public fun decode_membership_proof(buf: vector<u8>): MembershipProof {
        let buf = bcs_utils::new(buf);

        MembershipProof {
            sub_proof: decode_existence_proof(&mut buf),
            top_level_proof: decode_existence_proof(&mut buf)
        }
    }

    public fun decode_existence_proof(buf: &mut BcsBuf): ExistenceProof {
        let key = bcs_utils::peel_bytes(buf);
        let value = bcs_utils::peel_bytes(buf);
        let leaf_prefix = bcs_utils::peel_bytes(buf);

        ExistenceProof {
            key,
            value,
            leaf_prefix,
            path: bcs_utils::peel_vector<InnerOp>(
                buf,
                |buf| InnerOp {
                    prefix: bcs_utils::peel_bytes(buf),
                    suffix: bcs_utils::peel_bytes(buf)
                }
            )
        }
    }

    #[test]
    fun verify_membership_left() {
        let proof = ExistenceProof {
            key: x"303142424373615a55715146735259436c6a5767",
            value: x"76616c75655f666f725f303142424373615a55715146735259436c6a5767",
            leaf_prefix: x"00",
            path: vector<InnerOp>[
                InnerOp {
                    prefix: x"01",
                    suffix: x"cb3131cd98b069efcc0e8c7e68da47370adbff32266d7fcd1b0580fdf3961266"
                },
                InnerOp {
                    prefix: x"01",
                    suffix: x"21d1205c1f8537205e8fb4b176f960b459d9131669968d59c456442f7673b68b"
                },
                InnerOp {
                    prefix: x"01",
                    suffix: x"b82a0e7f4434b3cedb87ea83eb5a70c7dc664c77b2fe21c6245f315e58fdf745"
                },
                InnerOp {
                    prefix: x"01",
                    suffix: x"bf0657a0e6fbd8f2043eb2cf751561adcf50547d16201224133eeb8d38145229"
                },
                InnerOp {
                    prefix: x"01",
                    suffix: x"6d47c03df91a4a0252055d116439d34b5b73f3a24d5cb3cf0d4b08caa540cac4"
                },
                InnerOp {
                    prefix: x"01",
                    suffix: x"d5d2926993fa15c7410ac4ee1f1d81afddfb0ab5f6f4706b05f407bc01638149"
                },
                InnerOp {
                    prefix: x"01",
                    suffix: x"540719b26a7301ad012ac45ebe716679e5595e5570d78be9b6da8d8591afb374"
                },
                InnerOp {
                    prefix: x"01",
                    suffix: x"fccaaa9950730e80b9ccf75ad2cfeab26ae750b8bd6ac1ff1c7a7502f3c64be2"
                },
                InnerOp {
                    prefix: x"01",
                    suffix: x"ecb61a6d70accb79c2325fb0b51677ed1561c91af5e10578c8294002fbb3c21e"
                },
                InnerOp {
                    prefix: x"01",
                    suffix: x"1b3bc1bd8d08af9f6199de84e95d646570cbd9b306a632a5acf617cbd7d1ab0a"
                }
            ]
        };

        verify_existence(
            &proof,
            tm_proof_spec(),
            x"c569a38a5775bbda2051c34ae00894186f837c39d11dca55495b9aed14f17ddf",
            x"303142424373615a55715146735259436c6a5767",
            x"76616c75655f666f725f303142424373615a55715146735259436c6a5767"
        );
    }

    #[test]
    fun test_verify_chained_membership() {
        let mem_proof =
            decode_membership_proof(
                x"18636f6e6e656374696f6e732f636f6e6e656374696f6e2d30460a0930382d7761736d2d3012140a0131120f4f524445525f554e4f524445524544180222210a0a636f6d6574626c732d30120c636f6e6e656374696f6e2d301a050a03696263040002ca0104260204ca012067b76c7b82d60ebee7f41dd11a02534c1a16efa70c217310356230dfd5ad0c202000260406aa0220fe0560ee5685e1c214bcb958f761a467858478ed4a2ddcf77cc0f27258248f9c200005060eaa02202120140ee5ef0cddcc422e389954ff959f52c905a7211e62e3a14f67199ad81e032226081aaa02203d62d598ecb60b8721fb2ace147909fb3c61c54dc7b54e04d028cc21e10d505a20000369626320552a1b22544e343a046985a0ae8cc625adc18a18b7669a64ae9e4c9ba6754f460100050101202cd8b50700950546180ad979135a8708c2ea2098fff6ade31b7e40eb5dcf7c0521012cf3feea58fcdb48b73c2cdd1b018c90c4078f924385675a0e9457168cd47ff10021016bd19d4e1e3d1d96827c449152c4bedc0d5d306e9696d3ca78983d6866891f31002101a9788106a88704540fe0ead349d99096acaae60826863dd426a530b82570b75700010120a2fac4bcd28e2655f7985c9aad923140076c1764bd862ebfa999f8ed2bacfbf7"
            );
        verify_membership(
            mem_proof,
            x"88be092a61a8033111d4625bdbdc48c814b7258a2ec560e731b9fd17780e45ed",
            b"ibc",
            b"connections/connection-0",
            connection_end::encode_proto(
                connection_end::new(
                    std::string::utf8(b"08-wasm-0"),
                    vector[
                        connection_end::new_version(
                            std::string::utf8(b"1"),
                            vector[std::string::utf8(b"ORDER_UNORDERED")]
                        )
                    ],
                    2,
                    0,
                    connection_end::new_counterparty(
                        std::string::utf8(b"cometbls-0"),
                        std::string::utf8(b"connection-0"),
                        b"ibc"
                    )
                )
            )
        );
    }
}
