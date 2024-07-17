module IBC::ics23 {
    use std::vector;
    use std::option::Option;
    use std::hash;

    struct ExistenceProof has drop {
        key: vector<u8>,
        value: vector<u8>,
        leaf_prefix: vector<u8>,
        path: vector<InnerOp>
    }

    struct InnerOp has drop, copy {
        prefix: vector<u8>,
        suffix: vector<u8>,
        
    }

    struct NonExistenceProof has drop {
        key: vector<u8>,
        left: Option<ExistenceProof>,
        right: Option<ExistenceProof>,
    }

    struct ProofSpec has drop {
        child_size: u64,
        min_prefix_length: u64,
        max_prefix_length: u64,
    }

    fun iavl_proof_spec(): ProofSpec {
        ProofSpec {
            child_size: 33, 
            min_prefix_length: 4, 
            max_prefix_length: 12
        }
    }

    fun tm_proof_spec(): ProofSpec {
        ProofSpec {
            child_size: 32,
            min_prefix_length: 1, 
            max_prefix_length: 1
        }
    }

    fun verify_chained_membership(
        proofs: vector<ExistenceProof>,
        root: vector<u8>,
        prefix: vector<u8>,
        key: vector<u8>,
        value: vector<u8>,
    ): u64 {
        let (subroot, err) = calculate_existence_root(vector::borrow(&proofs, 0));
        if (err != 0) {
            return err
        };

        let err = verify_no_root_check(
            vector::borrow(&proofs, 0),
            iavl_proof_spec(),
            key,
            value,
        );

        if (err != 0) {
            return 1
        };

        verify_existence(
            vector::borrow(&proofs, 1),
            tm_proof_spec(),
            root,
            prefix,
            subroot,
        )
    }

    fun verify_existence(
        proof: &ExistenceProof,
        proof_spec: ProofSpec,
        commitment_root: vector<u8>,
        key: vector<u8>,
        value: vector<u8>,
    ): u64 {
        if (key != proof.key) {
            return 1
        };

        if (value != proof.value) {
            return 2
        };

        let err = check_against_spec(proof, proof_spec);
        if (err != 0) {
            return err
        };

        let (root, err) = calculate_existence_root(proof);
        if (err != 0) {
            return err
        };

        if (root != commitment_root) {
            return 3 
        };

        0
    }

    fun verify_no_root_check(
        proof: &ExistenceProof,
        proof_spec: ProofSpec,
        key: vector<u8>,
        value: vector<u8>,
    ): u64 {
        if (key != proof.key) {
            return 1
        };

        if (value != proof.value) {
            return 1
        };

        check_against_spec(proof, proof_spec)
    }

    fun check_against_spec(proof: &ExistenceProof, proof_spec: ProofSpec): u64 {
        if (vector::is_empty(&proof.leaf_prefix)) {
            return 4
        };

        if (*vector::borrow(&proof.leaf_prefix, 0) != 0) {
            return 5
        };

        let max = proof_spec.max_prefix_length + proof_spec.child_size;
        let i = 0;
        while (i < vector::length(&proof.path)) {
            let inner_op = vector::borrow(&proof.path, i);
            if (vector::length(&inner_op.prefix) < proof_spec.min_prefix_length 
                    || *vector::borrow(&inner_op.prefix, 0) == 0 
                    || vector::length(&inner_op.prefix) > max) {
                return 6
            };
            
            i = i + 1;
        };

        0
    }

    fun calculate_existence_root(proof: &ExistenceProof): (vector<u8>, u64) {
        if (vector::length(&proof.leaf_prefix) == 0) {
            return (vector::empty(), 7)
        };

        let (root, err) = apply_leaf_op(&proof.leaf_prefix, &proof.key, &proof.value);
        if (err != 0) {
            return (vector::empty(), err)
        };

        let i = 0;
        while (i < vector::length(&proof.path)) {
            (root, err) = apply_inner_op(*vector::borrow(&proof.path, i), root);
            if (err != 0) {
                return (vector::empty(), err)
            };
            i = i + 1;
        };

        (root, 0)
    }

    fun apply_leaf_op(prefix: &vector<u8>, key: &vector<u8>, value: &vector<u8>): (vector<u8>, u64) {
        if (vector::is_empty(key)) {
            return (vector::empty(), 8)
        };
        
        if (vector::is_empty(value)) {
            return (vector::empty(), 9)
        };

        let encoded_key = varint(vector::length(key));

        let hashed_value = hash::sha2_256(*value);
        let encoded_value = varint(32);

        let hash_data: vector<u8> = vector::empty();
        vector::append(&mut hash_data, *prefix);
        vector::append(&mut hash_data, encoded_key);
        vector::append(&mut hash_data, *key);
        vector::append(&mut hash_data, encoded_value);
        vector::append(&mut hash_data, hashed_value);

        (hash::sha2_256(hash_data), 0)
    }

    fun apply_inner_op(inner_op: InnerOp, child: vector<u8>): (vector<u8>, u64) {
        if (vector::is_empty(&child)) {
            return (vector::empty(), 10)
        };

        let pre_image = inner_op.prefix;
        vector::append(&mut pre_image, child);
        vector::append(&mut pre_image, inner_op.suffix);

        (hash::sha2_256(pre_image), 0)
    }

    fun varint(value: u64): vector<u8> {
        let buf: vector<u8> = vector::empty();
        let i = 0;
        while (i < 10) {
            if (value < 0x80) {
                vector::push_back(&mut buf, (value as u8));
                break
            } else {
                vector::push_back(&mut buf, (((value & 0x7F) | 0x80) as u8));
                value = value >> 7;
            };
            i = i + 1;
        };
        buf
    }

    #[test]
    fun verify_membership_left() {
        let proof = ExistenceProof {
            key: x"303142424373615a55715146735259436c6a5767",
            value: x"76616c75655f666f725f303142424373615a55715146735259436c6a5767",
            leaf_prefix: x"00",
            path: vector<InnerOp> [
                InnerOp {
                    prefix: x"01",
                    suffix: x"cb3131cd98b069efcc0e8c7e68da47370adbff32266d7fcd1b0580fdf3961266",
                },
                InnerOp {
                    prefix: x"01",
                    suffix: x"21d1205c1f8537205e8fb4b176f960b459d9131669968d59c456442f7673b68b",                    
                },
                InnerOp {
                    prefix: x"01",
                    suffix: x"b82a0e7f4434b3cedb87ea83eb5a70c7dc664c77b2fe21c6245f315e58fdf745",
                },
                InnerOp {
                    prefix: x"01",
                    suffix: x"bf0657a0e6fbd8f2043eb2cf751561adcf50547d16201224133eeb8d38145229",
                },
                InnerOp {
                    prefix: x"01",
                    suffix: x"6d47c03df91a4a0252055d116439d34b5b73f3a24d5cb3cf0d4b08caa540cac4",
                },
                InnerOp {
                    prefix: x"01",
                    suffix: x"d5d2926993fa15c7410ac4ee1f1d81afddfb0ab5f6f4706b05f407bc01638149",
                },
                InnerOp {
                    prefix: x"01",
                    suffix: x"540719b26a7301ad012ac45ebe716679e5595e5570d78be9b6da8d8591afb374",
                },
                InnerOp {
                    prefix: x"01",
                    suffix: x"fccaaa9950730e80b9ccf75ad2cfeab26ae750b8bd6ac1ff1c7a7502f3c64be2",
                },
                InnerOp {
                    prefix: x"01",
                    suffix: x"ecb61a6d70accb79c2325fb0b51677ed1561c91af5e10578c8294002fbb3c21e",
                },
                InnerOp {
                    prefix: x"01",
                    suffix: x"1b3bc1bd8d08af9f6199de84e95d646570cbd9b306a632a5acf617cbd7d1ab0a",
                },
            ]
        };

        let res = verify_existence(
            &proof,
            tm_proof_spec(),
            x"c569a38a5775bbda2051c34ae00894186f837c39d11dca55495b9aed14f17ddf",
            x"303142424373615a55715146735259436c6a5767",
            x"76616c75655f666f725f303142424373615a55715146735259436c6a5767"
        );

        assert!(res == 0, res);
    }
}
