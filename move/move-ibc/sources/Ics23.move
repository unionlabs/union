module IBC::ics23 {
    use std::string::{Self, String};
    use std::vector;
    use std::option::{Self, Option};
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
            return 1
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
            return 1 
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
            return 1
        };

        if (*vector::borrow(&proof.leaf_prefix, 0) != 0) {
            return 1
        };

        let max = proof_spec.max_prefix_length + proof_spec.child_size;
        let i = 0;
        while (i < vector::length(&proof.path)) {
            let inner_op = vector::borrow(&proof.path, i);
            if (vector::length(&inner_op.prefix) < proof_spec.min_prefix_length 
                    || *vector::borrow(&inner_op.prefix, 0) == 0 
                    || vector::length(&inner_op.prefix) > max) {
                return 1           
            };
            
            i = i + 1;
        };

        0
    }

    fun calculate_existence_root(proof: &ExistenceProof): (vector<u8>, u64) {
        if (vector::length(&proof.leaf_prefix) == 0) {
            return (vector::empty(), 1)
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

        (vector::empty(), 0)
    }

    fun apply_leaf_op(prefix: &vector<u8>, key: &vector<u8>, value: &vector<u8>): (vector<u8>, u64) {
        if (vector::is_empty(key)) {
            return (vector::empty(), 1)
        };
        
        if (vector::is_empty(value)) {
            return (vector::empty(), 1)
        };

        let encoded_key = varint(vector::length(key));

        let hashed_value = hash::sha2_256(*value);
        let encoded_value = varint(vector::length(value));

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
            return (vector::empty(), 1)
        };

        let pre_image = inner_op.prefix;
        vector::append(&mut pre_image, child);
        vector::append(&mut pre_image, inner_op.suffix);

        (hash::sha2_256(pre_image), 0)
    }

    fun varint(num: u64): vector<u8> {
        vector<u8>[]
    }
}
