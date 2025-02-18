// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's aptos subdirectory
//                       The Licensed Work is (c) 2024 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
//

// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.

// Notice

// Business Source License 1.1

// Terms

// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.

// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.

// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.

// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.

// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.

// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.

// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).

// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

module ibc::aptos_verifier {
    use std::option::{Self, Option};
    use std::vector;
    use std::hash::sha3_256;

    use ibc::bcs_utils::{Self, BcsBuf};
    use ibc::bytes_bit_iterator;

    const E_INVALID_PROOF: u64 = 1000;
    const E_MUST_BE_MEMBERSHIP_PROOF: u64 = 1001;
    const E_INVALID_ROOT_HASH: u64 = 1002;
    const E_MAX_SIBLINGS_EXCEEDED: u64 = 1003;
    const E_ROOT_MISMATCH: u64 = 1004;

    struct SparseMerkleProof has drop {
        // the leaf to be proven, `None` for non-inclusion proof
        leaf: Option<SparseMerkleLeafNode>,
        // merkle hashes ordered from the root level to bottom level
        siblings: vector<vector<u8>>
    }

    struct SparseMerkleLeafNode has drop {
        // 32 bytes key
        key: vector<u8>,
        // 32 bytes value hash (sha3)
        value_hash: vector<u8>
    }

    public fun verify_membership(
        proof: vector<u8>, expected_root_hash: vector<u8>
    ): u64 {
        assert!(vector::length(&expected_root_hash) == 32, E_INVALID_ROOT_HASH);
        let proof = decode_proof(proof);
        if (option::is_none(&proof.leaf)) {
            return E_MUST_BE_MEMBERSHIP_PROOF
        };

        let leaf = option::extract(&mut proof.leaf);

        verify_existence_proof(
            proof.siblings,
            expected_root_hash,
            leaf.key,
            leaf.value_hash
        )
    }

    fun verify_existence_proof(
        siblings: vector<vector<u8>>,
        expected_root_hash: vector<u8>,
        element_key: vector<u8>,
        element_hash: vector<u8>
    ): u64 {
        let siblings_len = vector::length(&siblings);
        if (siblings_len > 256) {
            return E_MAX_SIBLINGS_EXCEEDED
        };

        let current_hash = hash_key_value(element_key, element_hash);

        vector::reverse(&mut siblings);

        let bits = bytes_bit_iterator::new_rev(element_key, 256 - (siblings_len as u32));

        let i = 0;

        while (i < siblings_len) {
            let (sibling_hash, bit) =
                (*vector::borrow(&siblings, i), bytes_bit_iterator::get_bit(&bits, i));
            current_hash = if (bit) {
                hash_sparse_merkle_internal_node(sibling_hash, current_hash)
            } else {
                hash_sparse_merkle_internal_node(current_hash, sibling_hash)
            };
            i = i + 1;
        };

        if (current_hash != expected_root_hash) {
            return E_ROOT_MISMATCH
        };

        0
    }

    fun hash_key_value(key: vector<u8>, value_hash: vector<u8>): vector<u8> {
        let buf: vector<u8> = vector::empty();
        vector::append(&mut buf, sha3_256(b"APTOS::SparseMerkleLeafNode"));
        vector::append(&mut buf, key);
        vector::append(&mut buf, value_hash);
        sha3_256(buf)
    }

    fun hash_sparse_merkle_internal_node(
        left_child: vector<u8>, right_child: vector<u8>
    ): vector<u8> {
        let buf: vector<u8> = vector::empty();
        vector::append(&mut buf, sha3_256(b"APTOS::SparseMerkleInternal"));
        vector::append(&mut buf, left_child);
        vector::append(&mut buf, right_child);
        sha3_256(buf)
    }

    fun decode_proof(proof: vector<u8>): SparseMerkleProof {
        let buf = bcs_utils::new(proof);

        SparseMerkleProof {
            leaf: bcs_utils::peel_option<SparseMerkleLeafNode>(
                &mut buf,
                |buf| peel_sparse_merkle_leaf_node(buf)
            ),
            siblings: bcs_utils::peel_vector<vector<u8>>(
                &mut buf,
                |buf| bcs_utils::peel_fixed_bytes(buf, 32)
            )
        }
    }

    fun peel_sparse_merkle_leaf_node(buf: &mut BcsBuf): SparseMerkleLeafNode {
        SparseMerkleLeafNode {
            key: bcs_utils::peel_fixed_bytes(buf, 32),
            value_hash: bcs_utils::peel_fixed_bytes(buf, 32)
        }
    }
}
