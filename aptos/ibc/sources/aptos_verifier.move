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

        let bits = bytes_bit_iterator::new_rev(
            element_key, 256 - (siblings_len as u32), true
        );

        let i = 0;

        while (i < siblings_len) {
            let (sibling_hash, bit) =
                (*vector::borrow(&siblings, i), bytes_bit_iterator::next(&mut bits));
            std::debug::print(&bit);
            current_hash = if (bit) {
                hash_sparse_merkle_internal_node(sibling_hash, current_hash)
            } else {
                hash_sparse_merkle_internal_node(current_hash, sibling_hash)
            };
            i = i + 1;
        };
        std::debug::print(&current_hash);

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

    #[test]
    fun test_membership_verification_success() {
        let proof = SparseMerkleProof {
            leaf: option::some(
                SparseMerkleLeafNode {
                    key: x"6e4b28d40f98a106a65163530924c0dcb40c1349d3aa915d108b4d6cfc1ddb19",
                    value_hash: x"58e7bf0a6dd26e946946402cf50941bccba0a1724a2977bc2ec0986195e816a0"
                }
            ),
            // merkle hashes ordered from the root level to bottom level
            siblings: vector[
                x"1948fd458b370b4af82d68717cc05a54b4def266f6abae1e145529b23c842605", x"3cb1f03efda960b4acb5a119a933d6fd1a564f6c79a2e51efce337d3b90ee9fc", x"90082b6b09c6b2c74e4516fc7d16021986a02be5a5d9aa4636e7e85a55efea81", x"610db0a0b5237112b545bd4104208f58bd6b3344af7cb61e93f0be0483fbae44", x"93bd6d3814c8fa0c7e48df0be69010b00a5b22bce90f57ff77aa55b02a69d077", x"cf363af71ff591de4a6d67208f90bf2181699c6b2fc29e70169c5e5a4cb0fe16", x"20a067a6b1d387dd485781167453b44d62a82a45455252e942e7843a260ba751", x"bc8796ddc7b4dc70c84d94a7b4f7dc2ddcabfef7329ae7613ce06777b3652a14", x"9330e423338bf02370352eec1a859aaab9d547051441d6ce8d323458eb9d7e9d", x"1395f60e26649eae6b28c0598de6d822cfd41eef25e90b71949d2238aeebcf2a", x"f36442530dba881665d55e6990b49168df3b797d636df4cb664aed6f40f8458d", x"24201164119e44b45a577c817cc7c2ba99804978aae5f4e6918caa71465fc7fe", x"78add101849c73e0e48378821b1b477bfa7c2e61ef143369911cad820a4ab415", x"2616b21adb5de351fa04288b5c2170e71397904dd214e4cb8c60dc5741703841", x"1bbc1d74fdd5224e55615f87a3b36d2d39219aee13a07d3c0bd10bf29167370e", x"4044e478843cdf4a38b253218e7f538528b4a39c8655eee4ba6e55599203b4c0", x"235ec5b74305e0ade563df16eda641913d9ee5bb2026d75fdaf5a723f476d904", x"40a939dcef4e0ffd81ffc922cef404b475d5466c7bb0524cf55468d6f404a087", x"dd7a69bab2eccfd5707afa03375fe848fe26d2167bacd12bf81992ff42177478", x"883ff5974348b49852978d1482bccee6dec20ea2c4e391528a1580ca0c9a135e", x"20db621a20d61d5d8d8e4d36491d456a4bc99ebf2645af2f31eb843386b25bc9", x"321e76b23dd4ad9514bcbdeac367b9a3e1306c4d6f6f151ae8fbdbebf4c19568", x"334b56ad8afe3475932da9e8b7f9fae0ce7adc971b05d56dce1e37c54daada61", x"881bd464a4e77cbe3b32076451ee6942a783903ce256f214e565ff08f0cbb749", x"0b9ff7a37b22ce2a49dc2dd38a119d4a3e2b768c2e0777cb069b4c7b9c592631"
            ]
        };

        let expected_root =
            x"4dd91f1331754b8a01bd1f471170311ff50b7cfc9c83a471d47d4060f2ed01b2";

        let leaf = option::extract(&mut proof.leaf);

        std::debug::print(
            &verify_existence_proof(
                proof.siblings,
                expected_root,
                leaf.key,
                leaf.value_hash
            )
        );
    }
}
