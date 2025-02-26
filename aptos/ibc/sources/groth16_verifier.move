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

module ibc::groth16_verifier {
    use std::bn254_algebra::{
        Fr,
        FormatFrMsb,
        FormatFrLsb,
        G1,
        FormatG1Uncompr,
        FormatG1Compr,
        G2,
        Gt,
        FormatG2Compr
    };
    use std::crypto_algebra::{
        deserialize,
        serialize,
        zero,
        add,
        scalar_mul,
        multi_pairing,
        Element,
        eq
    };
    use std::option;
    use std::vector;
    use std::from_bcs;
    use std::aptos_hash;
    use std::hash;
    use std::bcs;
    use std::string::{Self, String};

    friend ibc::cometbls_lc;

    const ALPHA_G1: vector<u8> = x"c7e253d6dbb0b365b15775ae9f8aa0ffcc1c8cde0bd7a4e8c0b376b0d92952240223184a278d794b2d6bc8c64a0b007ab47333fa5e4be9464eb8db8859c83ea5";
    const BETA_G2: vector<u8> = x"44d2615ebda233e141f4ca0a1270e1269680b20507d55f6872540af6c1bc2424dba1298a9727ff392b6f7f48b3e88e20cf925b7024be9992d3bbfae8820a0987";
    const GAMMA_G2: vector<u8> = x"edf692d95cbdde46ddda5ef7d422436779445c5e66006a42761e1f12efde0018c212f3aeb785e49712e7a9353349aaf1255dfb31b7bf60723a480d9293938e99";
    const DELTA_G2: vector<u8> = x"dc047186b12c9c677d34f3e2d5c826655e363239ee6eb2e4348d3fa7d2a5ac02e6972cd8563ce0d488b8063c4e51559172db66508e31265a07de0ba9efdbb887";
    const PEDERSEN_G: vector<u8> = x"edf692d95cbdde46ddda5ef7d422436779445c5e66006a42761e1f12efde0018c212f3aeb785e49712e7a9353349aaf1255dfb31b7bf60723a480d9293938e19";
    const PEDERSEN_G_ROOT_SIGMA_NEG: vector<u8> = x"dc047186b12c9c677d34f3e2d5c826655e363239ee6eb2e4348d3fa7d2a5ac02e6972cd8563ce0d488b8063c4e51559172db66508e31265a07de0ba9efdbb887";
    const GAMMA_ABC_G1: vector<vector<u8>> = vector[
        x"3e2753d1e56abd9bd5fbbdd7fa35f27f7bc43c0cc97335453e1df217388a5d2f32c4e7d7178d82fcdc848793025d76c8ea0f0a2387fc75ce86fdb14221a27f14",
        x"9a6bb65aef69c0c8f60f1a8c833e4904686ca4083a89201ad07b991c8eb9812a6ffb78639dadc1986e57987c20a67bcc94ec57f1a1d6d420ec5c07a8ad3869a7",
        x"b76ab396dfe5e21691ff953b5d5edafffd6e49fbe75e5ce39cf80d14ce969417fc3c9f640033aad5e454cdd85f8e8c93a5466e647d6b670359ce8846d4e72603"
    ];

    const HMAC_O: vector<u8> = x"1F333139281E100F5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C";
    const HMAC_I: vector<u8> = x"75595B5342747A653636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636";
    const PRIME_R_MINUS_ONE: vector<u8> = x"000000f093f5e1439170b97948e833285d588181b64550b829a031e1724e6430";
    const COMPR_G1_SIZE: u64 = 32;
    const COMPR_G2_SIZE: u64 = 64;

    struct Proof has drop {
        a: Element<G1>,
        b: Element<G2>,
        c: Element<G1>
    }

    struct ZKP has drop {
        proof: Proof,
        proof_commitment: Element<G1>,
        proof_commitment_pok: Element<G1>
    }

    public(friend) fun verify_zkp(
        chain_id: &String,
        trusted_validators_hash: &vector<u8>,
        light_header_hash: vector<u8>,
        zkp: &ZKP
    ): bool {
        let inputs_hash: vector<u8> = vector::empty();
        let i = 0;
        while (i < 32 - string::length(chain_id)) {
            vector::push_back(&mut inputs_hash, 0);
            i = i + 1;
        };
        vector::append(&mut inputs_hash, *string::bytes(chain_id));
        vector::append(&mut inputs_hash, light_header_hash);
        vector::append(&mut inputs_hash, *trusted_validators_hash);

        let inputs_hash = hash::sha2_256(inputs_hash);

        let x = vector::borrow_mut(&mut inputs_hash, 0);
        *x = 0;

        let inputs_hash = option::extract(
            &mut deserialize<Fr, FormatFrMsb>(&inputs_hash)
        );
        let commitment_hash = hash_commitment(&zkp.proof_commitment);
        let hmac = bcs::to_bytes(&commitment_hash);
        let commitment_hash = option::extract(&mut deserialize<Fr, FormatFrLsb>(&hmac));

        let alpha_g1 = option::extract(&mut deserialize<G1, FormatG1Uncompr>(&ALPHA_G1));

        let beta_g2 = option::extract(&mut deserialize<G2, FormatG2Compr>(&BETA_G2));

        let gamma_g2 = option::extract(&mut deserialize<G2, FormatG2Compr>(&GAMMA_G2));
        let delta_g2 = option::extract(&mut deserialize<G2, FormatG2Compr>(&DELTA_G2));
        let pedersen_g = option::extract(
            &mut deserialize<G2, FormatG2Compr>(&PEDERSEN_G)
        );
        let pedersen_g_root_sigma_neg =
            option::extract(
                &mut deserialize<G2, FormatG2Compr>(&PEDERSEN_G_ROOT_SIGMA_NEG)
            );

        let gamma_abc_1 =
            option::extract(
                &mut deserialize<G1, FormatG1Uncompr>(
                    vector::borrow(&mut GAMMA_ABC_G1, 0)
                )
            );
        let gamma_abc_2 =
            option::extract(
                &mut deserialize<G1, FormatG1Uncompr>(
                    vector::borrow(&mut GAMMA_ABC_G1, 1)
                )
            );
        let gamma_abc_3 =
            option::extract(
                &mut deserialize<G1, FormatG1Uncompr>(
                    vector::borrow(&mut GAMMA_ABC_G1, 2)
                )
            );

        let msm_inner =
            add(
                &add<G1>(&gamma_abc_1, &zkp.proof_commitment),
                &scalar_mul<G1, Fr>(&gamma_abc_2, &inputs_hash)
            );
        let public_inputs_msm =
            add<G1>(
                &msm_inner,
                &scalar_mul<G1, Fr>(&gamma_abc_3, &commitment_hash)
            );

        let res =
            multi_pairing<G1, G2, Gt>(
                &vector[zkp.proof.a, public_inputs_msm, zkp.proof.c, alpha_g1],
                &vector[zkp.proof.b, gamma_g2, delta_g2, beta_g2]
            );

        if (!eq<Gt>(&res, &zero<Gt>())) {
            return false
        };

        let res =
            multi_pairing<G1, G2, Gt>(
                &vector[zkp.proof_commitment, zkp.proof_commitment_pok],
                &vector[pedersen_g, pedersen_g_root_sigma_neg]
            );

        eq<Gt>(&res, &zero<Gt>())
    }

    public(friend) fun parse_zkp(buf: vector<u8>): ZKP {
        let cursor = 0;

        let a =
            std::option::extract(
                &mut deserialize<G1, FormatG1Compr>(
                    &vector::slice(&buf, cursor, cursor + COMPR_G1_SIZE)
                )
            );
        cursor = cursor + COMPR_G1_SIZE;

        let b =
            std::option::extract(
                &mut deserialize<G2, FormatG2Compr>(
                    &vector::slice(&buf, cursor, cursor + COMPR_G2_SIZE)
                )
            );
        cursor = cursor + COMPR_G2_SIZE;

        let c =
            std::option::extract(
                &mut deserialize<G1, FormatG1Compr>(
                    &vector::slice(&buf, cursor, cursor + COMPR_G1_SIZE)
                )
            );
        cursor = cursor + COMPR_G1_SIZE;

        let proof_commitment =
            std::option::extract(
                &mut deserialize<G1, FormatG1Compr>(
                    &vector::slice(&buf, cursor, cursor + COMPR_G1_SIZE)
                )
            );
        cursor = cursor + COMPR_G1_SIZE;

        let proof_commitment_pok =
            std::option::extract(
                &mut deserialize<G1, FormatG1Compr>(
                    &vector::slice(&buf, cursor, cursor + COMPR_G1_SIZE)
                )
            );

        ZKP { proof: Proof { a, b, c }, proof_commitment, proof_commitment_pok }
    }

    fun hmac_keccak(message: &vector<u8>): vector<u8> {
        let inner = HMAC_I;
        vector::append(&mut inner, *message);
        let outer = HMAC_O;
        vector::append(&mut outer, aptos_hash::keccak256(inner));

        aptos_hash::keccak256(outer)
    }

    fun hash_commitment(proof_commitment: &Element<G1>): u256 {
        let buffer = serialize<G1, FormatG1Uncompr>(proof_commitment);
        let mask = vector::borrow_mut(&mut buffer, 63);
        *mask = *mask & 0x3f; // erase the mask (0x3f = 0b00111111)

        vector::reverse_slice(&mut buffer, 0, 32);
        vector::reverse_slice(&mut buffer, 32, 64);
        let hmac = hmac_keccak(&buffer);
        vector::reverse(&mut hmac);

        let prime_r_minus_one = from_bcs::to_u256(PRIME_R_MINUS_ONE);
        let hmac = from_bcs::to_u256(hmac);

        (hmac % prime_r_minus_one) + 1
    }

    #[test_only]
    public fun default(): ZKP {
        ZKP {
            proof: Proof {
                a: zero(),
                b: zero(),
                c: zero()
            },
            proof_commitment: zero(),
            proof_commitment_pok: zero()
        }
    }

    #[test]
    fun test_verify_zkp_ok() {
        let zkp =
            parse_zkp(
                x"21dd0bec9257ffaf9257d81e735ad91c0cf7ea0f10825a44d2031e2a1456cf03c734368377259eb336ef0f75ca08e6968c1840d53d73462566e17cdd901195025e949c5e386430974e0667a8c9d7523aec088df879bd2c8c76210493f1e7bc068ba4bf6e6fd6c6ba7e3405539a07042ee871009a1d4de53360a328a05b8a17ae39481dbf8678e72dbcea77beee754378cf31561cf23dd84bcadd83f62cdc8da6d2bde655171be094768d965d96cc1de0b6a1036506bfaaeed0b680f3369fbf89"
            );

        let res =
            verify_zkp(
                &std::string::utf8(b"union-devnet-1337"),
                &x"20DDFE7A0F75C65D876316091ECCD494A54A2BB324C872015F73E528D53CB9C4",
                x"00000000000000000000000000000000000000000000000000000000cafebabe00000000000000000000000000000000000000000000000000000000673f5ac3000000000000000000000000000000000000000000000000000000003b7e468e20ddfe7a0f75c65d876316091eccd494a54a2bb324c872015f73e528d53cb9c420ddfe7a0f75c65d876316091eccd494a54a2bb324c872015f73e528d53cb9c4ee7e3e58f98ac95d63ce93b270981df3ee54ca367f8d521ed1f444717595cd36",
                &zkp
            );

        assert!(res, 1);
    }
}
