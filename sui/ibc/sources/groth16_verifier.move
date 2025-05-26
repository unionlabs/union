module ibc::groth16_verifier {
    use sui::bls12381::{Self, G1, G2, GT, g1_from_bytes, scalar_from_bytes, g1_multi_scalar_multiplication, g1_add, g2_from_bytes, pairing, gt_add, g1_to_uncompressed_g1};
    use sui::group_ops::Element;
    use sui::hash::keccak256;
    use sui::bcs;
    
    use std::string::String;
    use std::hash::sha2_256;

    const ALPHA_G1: vector<u8> = x"83f0a72972b0ca87eb501643b3f9fff3cc4cc7f6d5f9e8938c6d2917c20e363ebdd7af5d8c26b8eebe3f094c55e0c8fd";
    const BETA_G2: vector<u8> = x"b2bef6a85e656156574eae9d064f5d25b55f29bbb56843e156ca40e7d8b895a2f44f3790a3165a16663e716ccd783ece036581b7b630b120c3896b8b0557cf9abfbdf7cde9522f708f2e0194bb3caeddd19166bc05a5b0f4b4cf07b792e1771a";
    const GAMMA_G2: vector<u8> = x"a7e02035b5677de6874f1a1c7a91b11f78a9649022263b224ba4b40fc25448194b0bb7b1bafb56473c37a1d3a10369580e222aed4f5a6159737542f1e747dea28a6227d48a87f732277d41d2c28edbb3201a7d2563ba720f433e987accacf3a4";
    const DELTA_G2: vector<u8> = x"b04c99bc7505d2e74ea73439f1f01e135a93e2d7b9d7434b6fa0fc78d8dbf8e923af9b17fb6f6b3ca8e93e02de3d05410c8daf977ff30dfcd90a17e53444429a74572bf20fcd6fe06398692029f98fa84ae9b41d2d9fed7bdd7976609ed0339e";
    const PEDERSEN_G: vector<u8> = x"b3f9df9fc7270749e80778ee302627c8d14d9d5f36e120cc072d6149da382b87adde19386e1808eff14e20d5a506b3960f71412d1a20a5111f29d434f7c62bced572b6a3729757155f179448859ba5aca4003a85f1edba0e8e401055d2edf332";
    const PEDERSEN_G_ROOT_SIGMA_NEG: vector<u8> = x"af5339ce8765aae53e984ac66961647cdac360dca861126022f301d6284cdc916f02fa4e8d798e612da398500f87394e00676e5e302bc2a33885a3837d1cfaa6ab09a62af4b69a3c088a889f86f04df731d8468585500d7dfbe805fd2afe5ef7";
    const GAMMA_ABC_G1: vector<vector<u8>> = vector[
            x"a6f130ba9b6351a11d5f3867a3aa4e9578a8cf04fe7c92a01a2db435079d5dd03e490eab5fb2435d82e6631d77418c40",
            x"b2795395724ac078433e7e56ded961b7cfb60070de10e25ba68377b238836c892292c06150020fd4cf52b5779089f7d1",
            x"b6401e59059d00dd046c53099b1c3bdd40283a1ac40cafbc8a62b0d5fd3ac9e3a46785fb42a29d42be1e75401ee34e49",
            x"948d7e21eaed2f66457e3b9b4813748ccea380a274c64cce794c82d4e5051af3f0723c3114d5d77ab2ea566f665cb869",
            x"b7505858582e13bfca0b43e1c7453af1049b429fd06fd0817441ca5a4f17790f008fab611b2e7ae92314e86d5b326356",
            x"810bc91efb1d0b1dcc7cd95f1ffc847e8c5881c0ba10084a52e7f825414298eab27ea06b1fed631896af2b33c0da3745",
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

        let gamma = g1_add(
            &g1_add(&gamma_abc_1, &zkp.proof_commitment),
            &g1_multi_scalar_multiplication(&vector[inner_commitment_x], &vector[gamma_abc_2])
        );
            
        let gamma = g1_add(
            &gamma,
            &g1_multi_scalar_multiplication(&vector[inner_commitment_y], &vector[gamma_abc_3])
        );
        let gamma = g1_add(
            &gamma,
            &g1_multi_scalar_multiplication(&vector[inner_commitment_hash], &vector[gamma_abc_4])
        );
        let gamma = g1_add(
            &gamma,
            &g1_multi_scalar_multiplication(&vector[inputs_hash], &vector[gamma_abc_5])
        );
        let gamma = g1_add(
            &gamma,
            &g1_multi_scalar_multiplication(&vector[commitment_hash], &vector[gamma_abc_6])
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

        std::debug::print(&sui::group_ops::equal(&res, &bls12381::gt_identity()));

        let pedersen_g = g2_from_bytes(&PEDERSEN_G);
        let pedersen_g_root_sigma_neg = g2_from_bytes(&PEDERSEN_G_ROOT_SIGMA_NEG);
        
        let res = gt_add(
            &pairing(&zkp.proof_commitment, &pedersen_g),
            &pairing(&zkp.proof_commitment_pok, &pedersen_g_root_sigma_neg)
        );

        std::debug::print(&sui::group_ops::equal(&res, &bls12381::gt_identity()));

        // calculate inputs hash
        // calculate inner commitment hash
        // strip commitment X
        // strip commitment Y
        // 
        // sum inner commitment hash + commitment x + commitment Y + inner inputs hash + commitment
        // pair proof

        false
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
        let proof = x"b144aa93e381f277543dcfa70a76699e6364edce4753d94dc0c04459d5b9181e8a1ed877b6214f48fd702fa5883991a48affbd1498cbb108b6553eb2856059fb6d661b0e7995d26284124a270f242ba48bc0e9f8cfd405866f4db9dc8e8dc43513ec94ce81a28f200e87897ca481762126162865f0b3b8e753dd2d6eb2ae83e46f452296696c5ce24a757a606b45f46d81f4b955fdcc359bc0a7dcc058bab3f724d82ebcaafb486929a9d1522fb63a6723d3cc2f3f13bb8a373e15e413657409ae0d4bd8d1644c3746067bfea806d36276bd9e4ed327b66c9a175040ad99f2db84dee6c3dddd4287f4ba0d7c0daccdc4a6671308c58da252c183f56c67e1e4c0372d62fd1c8deec6390c6b9110b4b481c9c9cf94ded988965847fbd12482721f2256d03f891426c7b1b3e3592f37e255bf4a7f19a4a31873eacca2846ee0405d009f7443bbfdeb50bca4a90140951ae124afb082359270cd0d181e9c46f17ff8";
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

        let inputs_hash = x"0031653718e0758d702469a49c2e34e068ef363f778489a866d2b41887fe4ae6";

        verify_zkp(inputs_hash, &zkp);

        
    }
    
}
