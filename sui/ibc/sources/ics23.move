// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's sui subdirectory                      
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

module ibc::ics23 {
    use std::vector;
    use std::option::Option;
    use std::hash;

    use sui::bcs::{Self, BCS};

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

    public struct MembershipProof has drop {
        sub_proof: ExistenceProof,
        top_level_proof: ExistenceProof
    }

    public struct ExistenceProof has drop, copy {
        key: vector<u8>,
        value: vector<u8>,
        leaf_prefix: vector<u8>,
        path: vector<InnerOp>
    }

    public struct InnerOp has drop, copy {
        prefix: vector<u8>,
        suffix: vector<u8>
    }

    public struct NonExistenceProof has drop {
        key: vector<u8>,
        left: Option<ExistenceProof>,
        right: Option<ExistenceProof>
    }

    public struct ProofSpec has drop {
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

    public(package) fun verify_membership(
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
        let mut i = 0;
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

        let mut root = apply_leaf_op(&proof.leaf_prefix, &proof.key, &proof.value);

        let mut i = 0;
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

        let encoded_key = encode_varint(vector::length(key));

        let hashed_value = hash::sha2_256(*value);
        let encoded_value = encode_varint(32);

        let mut hash_data: vector<u8> = vector::empty();
        vector::append(&mut hash_data, *prefix);
        vector::append(&mut hash_data, encoded_key);
        vector::append(&mut hash_data, *key);
        vector::append(&mut hash_data, encoded_value);
        vector::append(&mut hash_data, hashed_value);

        hash::sha2_256(hash_data)
    }

    fun apply_inner_op(inner_op: InnerOp, child: vector<u8>): vector<u8> {
        assert!(!vector::is_empty(&child), E_EMPTY_CHILD);

        let mut pre_image = inner_op.prefix;
        vector::append(&mut pre_image, child);
        vector::append(&mut pre_image, inner_op.suffix);

        hash::sha2_256(pre_image)
    }

    public(package) fun decode_membership_proof(buf: vector<u8>): MembershipProof {
        let mut buf = bcs::new(buf);

        MembershipProof {
            sub_proof: decode_existence_proof(&mut buf),
            top_level_proof: decode_existence_proof(&mut buf)
        }
    }

    public(package) fun decode_existence_proof(buf: &mut BCS): ExistenceProof {
        let key = buf.peel_vec_u8();
        let value = buf.peel_vec_u8();
        let leaf_prefix = buf.peel_vec_u8();

        ExistenceProof {
            key,
            value,
            leaf_prefix,
            path: buf.peel_vec!<InnerOp>(
                |buf| InnerOp {
                    prefix: buf.peel_vec_u8(),
                    suffix: buf.peel_vec_u8()
                }
            )
        }
    }

    fun encode_varint(mut value: u64): vector<u8> {
        let mut buf: vector<u8> = vector::empty();
        let mut i = 0;
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
            path: vector[
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
    fun see_membership_proof() {
        let proof =
            x"4103f64b1e5af826603673cac212ceedb720fb845c66aa7ce71ca304905ad66b6e514ac83fca211703e3ddb90093cd219714e5e3715bf0b4fd15b0441390534a24e220bef2d492f3019fd09f1aa7422e9a6300851ac621eaec669fe158a5da1c2051b606000294b2d0030f07020494b2d003202120f9865013bdf054aac5730e5fcb7005975e4d6b8b1a607b43740bcb056e1b122728040894b2d00320d4d61930b7b5c18bc532fb7216192bee0dca56320bc4930d611956fb7e241e6f200028060e94b2d00320e52a8a42a2a27c155b28bc092a5823deae59c592848b52e25b23591b32e1ebba200007081c94b2d003202120727b975cddb576188f7e0e9638c80f11a2a969a4a7ab9885d2c9a8ee2e0acaea070a3694b2d00320212075004960e18d84ac54d6471955018b4ad9eee0cff1ebab2f705d424d9632aedb280c5894b2d00320b8ae8aa36acaba2a460002dc9adca14857feaccab8c0e08406dfbd437ff31f552000080eaa0194b2d0032021207cece7c6af5c582cee051db2d890207047d8430d7d0c08ea99d690eacd5814672910980294b2d003203ceaae1c5d9f8fc86455a689bfc91ee816784374d6b527acbd6f4ada0f3cdd9c20002912ea0394b2d00320b8cc6282c09fad2463570f41a6d98a3591eeb45a4bf9563b978647d595ccaa6e20000814900694b2d0032021201391826c62af4949e11426700ee245d65cda676cb1e52bbc4d30e449230f0b130818a00f94b2d0032021204c9d5ab287c3320bad305595e614b3583ad1ea04c76e399348335398c43e93df081aaa1b94b2d003202120d2deef0edf66a4abcd32decf3a8ad5aa1b0885a12d915c46ff5528870b5482a9081cf63694b2d003202120f24a985ce1ad6c9887ae80c3a341f68252d3e7b03f6bb5bf0952140418c45588081e8a6e94b2d003202120050a478bc75905e1f84feedbe9e3eb65990724552b8dc5057a879cc48c56e9982a22d09f0294b2d0032011a1de2970cc459fdd75950b625686fead65dd73fd6719b836cbd108e033ca062000047761736d2093f5a3747d541ff1bf2bb77b04c96ada845c3a2784c0bc1ce3cdb269d921abfb01000221015554509091978f539822c9bda7bd4b5b7ece565683ba0f35bc735a278678274d00210149f8c4eef21fe70e5e53a78738a255b248927d272a207247369c7189b0ea34d700";
        let proof = decode_membership_proof(proof);
        std::debug::print(&proof);
    }

    // #[test]
    // fun test_verify_chained_membership() {
    //     let mem_proof =
    //         decode_membership_proof(
    //             x"18636f6e6e656374696f6e732f636f6e6e656374696f6e2d30460a0930382d7761736d2d3012140a0131120f4f524445525f554e4f524445524544180222210a0a636f6d6574626c732d30120c636f6e6e656374696f6e2d301a050a03696263040002ca0104260204ca012067b76c7b82d60ebee7f41dd11a02534c1a16efa70c217310356230dfd5ad0c202000260406aa0220fe0560ee5685e1c214bcb958f761a467858478ed4a2ddcf77cc0f27258248f9c200005060eaa02202120140ee5ef0cddcc422e389954ff959f52c905a7211e62e3a14f67199ad81e032226081aaa02203d62d598ecb60b8721fb2ace147909fb3c61c54dc7b54e04d028cc21e10d505a20000369626320552a1b22544e343a046985a0ae8cc625adc18a18b7669a64ae9e4c9ba6754f460100050101202cd8b50700950546180ad979135a8708c2ea2098fff6ade31b7e40eb5dcf7c0521012cf3feea58fcdb48b73c2cdd1b018c90c4078f924385675a0e9457168cd47ff10021016bd19d4e1e3d1d96827c449152c4bedc0d5d306e9696d3ca78983d6866891f31002101a9788106a88704540fe0ead349d99096acaae60826863dd426a530b82570b75700010120a2fac4bcd28e2655f7985c9aad923140076c1764bd862ebfa999f8ed2bacfbf7"
    //         );
    //     verify_membership(
    //         mem_proof,
    //         x"88be092a61a8033111d4625bdbdc48c814b7258a2ec560e731b9fd17780e45ed",
    //         b"ibc",
    //         b"connections/connection-0",
    //         connection_end::encode_proto(
    //             connection_end::new(
    //                 0,
    //                 vector[
    //                     connection_end::new_version(
    //                         std::string::utf8(b"1"),
    //                         vector[std::string::utf8(b"ORDER_UNORDERED")]
    //                     )
    //                 ],
    //                 2,
    //                 0,
    //                 connection_end::new_counterparty(
    //                     1,
    //                     0,
    //                     b"ibc"
    //                 )
    //             )
    //         )
    //     );
    // }
}
