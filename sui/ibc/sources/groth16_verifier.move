module ibc::groth16_verifier {
    use sui::bls12381::{Self, G1, G2, GT, scalar_one, g1_from_bytes, scalar_from_bytes, g1_multi_scalar_multiplication, g1_add, g2_from_bytes, pairing, gt_add, g1_to_uncompressed_g1};
    use sui::group_ops::{Self, Element};
    use sui::hash::keccak256;
    use sui::bcs;
    
    use std::string::String;
    use std::hash::sha2_256;

    const ALPHA_G1: vector<u8> = x"8086e08eb1cf3993e5ad8c6f563a64ce098028a481001d214d95a59a84e43d65c9f9ad2e9550b0f39a1d2ad6cbd4adb9";
    const BETA_G2: vector<u8> = x"afd014d1d004ba199f0acd264c35afa7f21871996cc1d5124a8fe77d29a6adf713245f2ae42479b7b9f6a9c6215209b106f7dc4670698106c314c33bd4934aa904e7c8f1508ed6ee85bca883130450f03e20875d334afcbbf76648f5316bd029";
    const GAMMA_G2: vector<u8> = x"af96ad63c4325a5fb3a3543ead06227ab9eaf3aabb2b0b81aa55e44dba938dee4b62b3e41ed39d96cce81569a4df88bd07ff886ab56ce74ddb88d799436c97fe6500f7957ae64b8cfa67faff2a842e9199f93d15a168e681751d3ed262855775";
    const DELTA_G2: vector<u8> = x"a83b49a682745799f90f65aa43aa219e4015a812631ecb4eb6cea58c4728509ddc670037f8470af2a4e570259f60749015ae641d3fdb0548b540c6526fcd67298480cf39fcb68db69dfc10c406738d3ffc11719fc6484b75dbdbda32979f19cb";
    const PEDERSEN_G: vector<u8> = x"96b833b796e17d0ae02eb662ebf8a73f1da45b3354b8f6d0660d43628328fc8c6a481dac073b1490dee2baec14f1baa1177d2390d8dd7751c0ed8b0bedbb620fc8b8dfe63a5e38f0fffa56e68e0e5d0b43ecb6ee5537d8cbb3ae3c5ca5d265b5";
    const PEDERSEN_G_ROOT_SIGMA_NEG: vector<u8> = x"999c10fdf65613a5e0c066a17222d25a95e61db7daad037a686f073dbccca19a3391bd11b68b0fa1878aa82f5c7439ab115f2692c0af6bb1f2b65c9db48f081ca0e306cd867c8351abd3e7fa9317170ea10ec9583a7b97a5803e9e0ea09c2299";
    const GAMMA_ABC_G1: vector<vector<u8>> = vector[
        x"ab8b14673eeba5331569419a8eef64d916761c7253de08eb2d9f1d180a4c65a87b22f85f1a84cb4f7e7b1b7bc07b5d5f",
        x"8992ea01643a1cf3644ca96162e5c057264074e1837d0ef421db295c0502aaca1f3c7368270c5ba7ecbba2b06992ea4a",
        x"b762895b5c5ac4c2b88b2b2b385af1963d666729d18552e58545883a4b78cf136840165c9b396ec40a7d6e8009c10cad",
        x"8a4361ed926829a0bccc16df614bc4810fb37661f9b946173f6b45f339066037fce2e681cddcb2811a1f5728f133dc68",
        x"ad226b5b860b5ff2a91e83c34add8b8ef087f719a413ddecb41bab41536c9b6d37ff0fa3c5b7c466c21962389eb2b255",
        x"8afcbf691d60d25be2742fea58c80763964e5d537d6452e8403cafd851ca0177b22a8dc58016bb3e9090e6c17e3b06f3",
    ];

    const PRIME_R_MINUS_ONE: vector<u8> = x"000000f093f5e1439170b97948e833285d588181b64550b829a031e1724e6430";
    const HMAC_O: vector<u8> = x"1F333139281E100F5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C";
    const HMAC_I: vector<u8> = x"75595B5342747A653636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636";
    const G1_SIZE: u64 = 48;
    const G2_SIZE: u64 = 96;

    public struct Proof has drop {
        a: Element<G1>,
        b: Element<G2>,
        c: Element<G1>
    }

    public struct ZKP has drop {
        proof: Proof,
        proof_commitment: Element<G1>,
        proof_commitment_pok: Element<G1>,
        inner_commitment: vector<u8>
    }

    public fun verify_zkp(
        // chain_id: &String,
        // trusted_validators_hash: &vector<u8>,
        // light_header_hash: vector<u8>,
        mut inputs_hash: vector<u8>,
        zkp: &ZKP
    ): bool {

        // let mut inputs_hash: vector<u8> = vector::empty();
        // let mut i = 0;
        // while (i < 32 - chain_id.length()) {
        //     inputs_hash.push_back(0);
        //     i = i + 1;
        // };
        // inputs_hash.append(*chain_id.bytes());
        // inputs_hash.append(light_header_hash);
        // inputs_hash.append(*trusted_validators_hash);

        // let mut inputs_hash = sha2_256(inputs_hash);
        // let mut first_elem = inputs_hash.borrow_mut(0);

        // *first_elem = 0;

        let inputs_hash = scalar_from_bytes(&inputs_hash);

        let mut inner_commitment_hash = bcs::to_bytes(&hash_commitment_bytes(zkp.inner_commitment));
        inner_commitment_hash.reverse();
        let inner_commitment_hash = scalar_from_bytes(&inner_commitment_hash);

        let inner_commitment_x = scalar_from_bytes(&vector_slice(&zkp.inner_commitment, 0, 32));
        let inner_commitment_y = scalar_from_bytes(&vector_slice(&zkp.inner_commitment, 32, 64));

        let alpha_g1 = g1_from_bytes(&ALPHA_G1);
        let beta_g2 = g2_from_bytes(&BETA_G2);
        let gamma_g2 = g2_from_bytes(&GAMMA_G2);
        let delta_g2 = g2_from_bytes(&DELTA_G2);

        let gamma_abc_1 = g1_from_bytes(&GAMMA_ABC_G1[0]);
        let gamma_abc_2 = g1_from_bytes(&GAMMA_ABC_G1[1]);
        let gamma_abc_3 = g1_from_bytes(&GAMMA_ABC_G1[2]);
        let gamma_abc_4 = g1_from_bytes(&GAMMA_ABC_G1[3]);
        let gamma_abc_5 = g1_from_bytes(&GAMMA_ABC_G1[4]);
        let gamma_abc_6 = g1_from_bytes(&GAMMA_ABC_G1[5]);

        let mut commitment_hash = bcs::to_bytes(&hash_commitment(&zkp.proof_commitment));
        commitment_hash.reverse();
        let commitment_hash = scalar_from_bytes(&commitment_hash);

        let gamma = g1_multi_scalar_multiplication(
            &vector[
                scalar_one(),
                inner_commitment_hash,
                inner_commitment_x,
                inner_commitment_y,
                inputs_hash,
                commitment_hash
            ],
            &vector[
                g1_add(&gamma_abc_1, &zkp.proof_commitment),
                gamma_abc_2,
                gamma_abc_3,
                gamma_abc_4,
                gamma_abc_5,
                gamma_abc_6
            ]
        );

        let res = gt_add(
            &pairing(&zkp.proof.a, &zkp.proof.b),
            &gt_add(
                &pairing(&gamma, &gamma_g2),
                &gt_add(
                    &pairing(&zkp.proof.c, &delta_g2),
                    &pairing(&alpha_g1, &beta_g2),
                )
            )
        );

        if (!group_ops::equal(&res, &bls12381::gt_identity())) {
            return false
        };

        let pedersen_g = g2_from_bytes(&PEDERSEN_G);
        let pedersen_g_root_sigma_neg = g2_from_bytes(&PEDERSEN_G_ROOT_SIGMA_NEG);
        
        let res = gt_add(
            &pairing(&zkp.proof_commitment, &pedersen_g),
            &pairing(&zkp.proof_commitment_pok, &pedersen_g_root_sigma_neg)
        );

        group_ops::equal(&res, &bls12381::gt_identity())
    }

    fun hmac_keccak(message: &vector<u8>): vector<u8> {
        let mut inner = HMAC_I;
        inner.append(*message);
        let mut outer = HMAC_O;
        outer.append(keccak256(&inner));

        keccak256(&outer)
    }

    fun hash_commitment_bytes(mut buffer: vector<u8>): u256 {
        let mut hmac = hmac_keccak(&buffer);
        hmac.reverse();

        let prime_r_minus_one = bcs::new(PRIME_R_MINUS_ONE).peel_u256();
        let hmac = bcs::new(hmac).peel_u256();

        (hmac % prime_r_minus_one) + 1
    }

    fun hash_commitment(commitment: &Element<G1>): u256 {
        let uncompr = g1_to_uncompressed_g1(commitment);
        let buffer = *uncompr.bytes(); // TODO(aeryz): check if this matches the uncompressed serialization in aptos

        hash_commitment_bytes(buffer)
    }

    fun vector_slice(v: &vector<u8>, start: u64, end: u64): vector<u8> {
        let mut ret = vector::empty();
        let mut i = start;
        while (i < end) {
            ret.push_back(v[i]);
            i = i + 1;
        };
        ret
    }

    #[test]
    fun test_proof() {
        let proof = x"8a8fb8f6066e6956618158d1a61dd27c1b30c0cf6ba0e100e45e19e0f6ce48536406b028541dd1db38b180615dfb7a059510808f24d1abb6e491337614851d036b3d892953022bf81b185a7906455146338c7cae98b4d3d213b64a6895780dfa065204820a3693c519aa2f556fdd8e538519a94164ac6c902db630f07326565ee83c4aca79c6c48ac9fa837ba22684c5b254baee94e80b8de33c8e5382a94c4afc84f3237966a93429e29afc817d8369d552c53f3cfb5c7bf26452f5bc7b47aba9d68a72455778d02a30776150e97cc8a4c23fa3ad44abe74699ecfa43108eb0951da0778aec9163ff0ba793c79a676cad3e9f262f2cba70dd6eb812c524971b5c7379b510a6623de8c2b10e2106cb167c15e45956faf14f8539e47828a6a1c51346e297892d74138bfee2fcfedbf421f5a000bd75cb87fecbcf5867dd715dc02ce84b6354099e089a6ee07a6a13922c37a4f08a8d28de81fcbd3d970c926c10";
        // let uncompr_proof = x"";
    
        let mut cursor = 0;

        let a = g1_from_bytes(&vector_slice(&proof, cursor, cursor + G1_SIZE));
        cursor = cursor + G1_SIZE;

        let b = g2_from_bytes(&vector_slice(&proof, cursor, cursor + G2_SIZE));
        cursor = cursor + G2_SIZE;

        let c = g1_from_bytes(&vector_slice(&proof, cursor, cursor + G1_SIZE));
        cursor = cursor + G1_SIZE;    

        let poc = g1_from_bytes(&vector_slice(&proof, cursor, cursor + G1_SIZE));
        cursor = cursor + G1_SIZE;

        let pok = g1_from_bytes(&vector_slice(&proof, cursor, cursor + G1_SIZE));
        cursor = cursor + G1_SIZE;

        let inner_commitment = vector_slice(&proof, cursor, cursor + 64);

        let zkp = ZKP {
            proof: Proof { a, b, c},
            proof_commitment: poc,
            proof_commitment_pok: pok,
            inner_commitment
        };

        let inputs_hash = x"001eb8378edf181e75b0c17186f0c36b5749e8698f4d892646c1079c29c2c6c7";

        assert!(verify_zkp(inputs_hash, &zkp), 1);

        
    }
    
}
