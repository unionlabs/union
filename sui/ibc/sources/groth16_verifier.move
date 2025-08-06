module ibc::groth16_verifier {
    use sui::bls12381::{Self, G1, G2, GT, scalar_one, g1_from_bytes, scalar_from_bytes, g1_mul, g1_multi_scalar_multiplication, g1_add, g2_from_bytes, pairing, gt_add, g1_to_uncompressed_g1};
    use sui::group_ops::{Self, Element};
    use sui::hash::keccak256;
    use sui::bcs;
    
    use std::string::String;
    use std::hash::sha2_256;

    const ALPHA_G1: vector<u8> = x"81fef9830153f4ee9f62e079e58c1e8a24a207c2a65dc7c85a021fa733188e2b3312456aedf5e9bfd200780cf5ddd1d2";
    const BETA_G2: vector<u8> = x"b872ad6df1ca3fca85a1e120aae72b3d7e9d51ab459254a028a6822f2349b5c6d66d0c53a660f0d4d769f02d043dfb2302b6d8540a3c6f2e9eb9724fe94b9bcf2f66c917670bdec553949ea34014d9715a42c9ce8c4df93268008cb86eb8b218";
    const GAMMA_G2: vector<u8> = x"aaa69ed56ddff8683e94181130ca466563edc0139aea58832c5ac60864e77a9fa88508544c94e76f3c21aed5f9e77e350240f74b767fbcdb9cf213657d075ee8ac6c3e37a2bff31ff2e02dd4b55903f54557a061def7d7c15496d13857a2335c";
    const DELTA_G2: vector<u8> = x"856a5cfecd0247613f9fea56e8f6c3d02921475e87456a81e8d3b834066b1d73b7eb53cc478dd4bfde682b1a9ca798700f1cdd3c4bf10a81bca6f2bee5e2d8eb9eece39ed1654de79be2cc31e329ce58afb2f7038777059fb8873bd1e0024b4e";
    const PEDERSEN_G: vector<u8> = x"aab7fa40d08381476be01ea8b9406f5bc6ea3643ec92b8928b5020a77d94156141097787a70102907c712d5bf4dc5dbe0d8c3ca56b5aaab0d727aa64094372e3f4d4bbba4c1f03800c694ecdc0818fe2604341b00959bd12349f3ee2ae2ab4bc";
    const PEDERSEN_G_ROOT_SIGMA_NEG: vector<u8> = x"903c7c703c4d11d22fe33b1954741830b5e78edd13faed8f70349df0e2ab0dfaf13345ab99e94830dce529cda56a5ad91672eae3bfa1eeca6dbf960e9b8e8418b2a0b47f97e8eaa8edf5b2851ddded7c74402c56ca0e2a3efcc60db939692d4d";
    const GAMMA_ABC_G1: vector<vector<u8>> = vector[
            x"8dbe9093ef308c557f92821b3426f56219b04885ca5057e24acccbea1ccdbb489defed38d18cf3e3146fced305fa036c",
            x"862eea1cad97fe194f2a54378bc04453e6f9c1f2d7b2b9dbc2cda38fb4127a50eaee28318baec7246069d0955fcfb75f",
            x"a4f52e0f1f72ec23242847b2967353947cdfe6fa5ebf8635961528de0b9367f3f6b4849478e132ddc6d2ee64d216ec0f",
            x"8bff361d3ec3d55d911f7b2f97b5cde31db92854fea0b270cb390c98df33273f926f205f166fe90316cf6446ad05aed2",
            x"b84f080a288b93d52e3122bcaeedff56c7f266afdda6ba869271f375676f25ec18a0ab9d318971c4eaf691ab11d8e484",
            x"9637a9173b39604bdeb0e09e38120dad655d9eee3cd98f257cdc5916d77428950c2e33828511155fa138b6eb3e803e71",
    ];

    const PRIME_R_MINUS_ONE: vector<u8> = x"00000000fffffffffe5bfeff02a4bd5305d8a10908d83933487d9d2953a7ed73";
    const PRIME_R_MINUS_ONE_BN254: vector<u8> = x"000000f093f5e1439170b97948e833285d588181b64550b829a031e1724e6430";
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
        chain_id: &String,
        trusted_validators_hash: &vector<u8>,
        light_header_hash: vector<u8>,
        zkp: &ZKP
    ): bool {

        let mut inputs_hash: vector<u8> = vector::empty();
        let mut i = 0;
        while (i < 32 - chain_id.length()) {
            inputs_hash.push_back(0);
            i = i + 1;
        };
        inputs_hash.append(*chain_id.bytes());
        inputs_hash.append(light_header_hash);
        inputs_hash.append(*trusted_validators_hash);

        let mut inputs_hash = sha2_256(inputs_hash);
        let mut first_elem = inputs_hash.borrow_mut(0);
        *first_elem = 0;

        let inputs_hash = scalar_from_bytes(&inputs_hash);

        let mut inner_commitment_hash = bcs::to_bytes(&hash_commitment_bytes(zkp.inner_commitment, PRIME_R_MINUS_ONE_BN254));
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

        let mut commitment_hash = bcs::to_bytes(&hash_commitment(&zkp.proof_commitment, PRIME_R_MINUS_ONE));
        commitment_hash.reverse();
        let commitment_hash = scalar_from_bytes(&commitment_hash);

        let gamma = g1_add(
            &g1_add(&gamma_abc_1, &zkp.proof_commitment),
            &g1_mul(
                &inner_commitment_hash,
                &gamma_abc_2
            )
        );

        let gamma = g1_add(
            &gamma,
            &g1_mul(
                &inner_commitment_x,
                &gamma_abc_3
            )
        );

        let gamma = g1_add(
            &gamma,
            &g1_mul(
                &inner_commitment_y,
                &gamma_abc_4
            )
        );

        let gamma = g1_add(
            &gamma,
            &g1_mul(
                &inputs_hash,
                &gamma_abc_5
            )
        );

        let gamma = g1_add(
            &gamma,
            &g1_mul(
                &commitment_hash,
                &gamma_abc_6
            )
        );

        // let gamma = g1_multi_scalar_multiplication(
        //     &vector[
        //         scalar_one(),
        //         inner_commitment_hash,
        //         inner_commitment_x,
        //         inner_commitment_y,
        //         inputs_hash,
        //         commitment_hash
        //     ],
        //     &vector[
        //         g1_add(&gamma_abc_1, &zkp.proof_commitment),
        //         gamma_abc_2,
        //         gamma_abc_3,
        //         gamma_abc_4,
        //         gamma_abc_5,
        //         gamma_abc_6
        //     ]
        // );

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

    fun hash_commitment_bytes(mut buffer: vector<u8>, prime: vector<u8>): u256 {
        let mut hmac = hmac_keccak(&buffer);
        hmac.reverse();

        let prime_r_minus_one = bcs::new(prime).peel_u256();
        let hmac = bcs::new(hmac).peel_u256();

        (hmac % prime_r_minus_one) + 1
    }

    fun hash_commitment(commitment: &Element<G1>, prime: vector<u8>): u256 {
        let uncompr = g1_to_uncompressed_g1(commitment);
        let buffer = *uncompr.bytes(); // TODO(aeryz): check if this matches the uncompressed serialization in aptos

        hash_commitment_bytes(buffer, prime)
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

    public(package) fun parse_zkp(proof: vector<u8>): ZKP {
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

        ZKP {
            proof: Proof { a, b, c},
            proof_commitment: poc,
            proof_commitment_pok: pok,
            inner_commitment
        }
    }

    #[test]
    fun test_proof() {
        let proof = x"8e6f419184390ee9847cff1ad98f8e29492391c536e99a2c95cb487ad22b9571931f9a9b33292fae5567e5c0779f69e68f75e49a4f4f93ca3655c0cee5c585c2f49a266b8387dc3ea8740cb45fac3a40b5fe2f004945ff5b1f9d4a1f49550e1509d2d919e35d3ec36d75428e950760225ec2ff49aca3009786e67bd688e34e1b882baafc5317964e4a30f59abf45b429a593b0b833d06f36e8a6447ccade4b912e2fac63b130d50f7a5f1cd3e04dd1ad280fe3ded32ddf1564ca4cdfb7eba18eb70650667730fcf55085ce7ae91e0f271f648779c431c07a67c8925421dcfd8bd74e4ad3672ba0a57acf1521bb653c6ea4dd7bc5fe436681859040b7b77f9fa7e446b06d24369a9da1c361ab2f9089c32a1614f35d8c0ce6d366c840f08663392f448f3dd5351a07ec7ba74f77c4f232d4ba428f916d6809e7ad607926a5fdd41150fc7acb376d2836365b2b98ddc97d5ae980398a13b9a3e43323d985943c2a";
        // let uncompr_proof = x"";
    
        let zkp = parse_zkp(proof);

        // let mut v = x"73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000000";
        // v.reverse();
        // std::debug::print(&v);
        
         std::debug::print(&verify_zkp(&std::string::utf8(b""), &vector::empty(), vector::empty(), &zkp));
        // assert!(verify_zkp(inputs_hash, &zkp), 1);
    }
    
}
