
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

#[allow(implicit_const_copy)]
module ibc::groth16_verifier {
    use sui::bls12381::{Self, G1, G2, g1_from_bytes, scalar_from_bytes, g1_mul, g1_add, g2_from_bytes, pairing, gt_add, g1_to_uncompressed_g1};
    use sui::group_ops::{Self, Element};
    use sui::hash::keccak256;
    use sui::bcs;
    
    use std::string::String;
    use std::hash::sha2_256;

    // begin auto-gen vk params
    const ALPHA_G1: vector<u8> = x"b9f7e77689d6c7266a897207825b267e93990de8b3986ab559b1a84482077c3db6dc3dcf209d999aaf648e6c7a010f8d";
    const BETA_G2: vector<u8> = x"a6102b906a85ee3318ee01a383ecde3813a12512980acdb10ae65fcf6707b4f815995705946313f62585eddebf9eb8421889d05dba98c06581e9c57173911c1a97b680aad76c86480c5fb60fa20f0ee18ff5eef48fcf5e4343d91d448b518d1c";
    const GAMMA_G2: vector<u8> = x"89c81c59618663032718533659a9da29f21da0e610bccc533752fc6659fa7962a17294146160f91e6b3e4492bfd3624a0096bfab558277c93c1bc056d5442692338293d25ddc75e78b88f3efa3f99a4d96a906b0f8f54b26d6a22240cd3c46c7";
    const DELTA_G2: vector<u8> = x"b619e27e840c20b9168ed0540f2c6ed626dddc034822b993ce3d3d850c73221463f0b363d78d819381ead058dd9c870e05f9fdd8a7817f05128e6ee751b96c1bac963c4d99a53647758e425b4f2faa722ee81d2d4135081441f78e92d4ab1aaf";
    const PEDERSEN_G: vector<u8> = x"8ae1a8808f22c833d734faba7c7557faf1b75e93d0ae4fcb8191b9ae0ec8bb18f86cdaf2ab9b085754afc2169f078b0a0dee291456886f03f4969c3f3092431f97de42dd3557a377b11babde9c5e830855cf38a57f661fcdfd1f522decca7b76";
    const PEDERSEN_G_ROOT_SIGMA_NEG: vector<u8> = x"acdaa84a7d14e824eca568c00f43d8138c6fae85e4e8a7be2ad6a94286260fb5e6b29b0ab579c2c821c6bc787887fdb0009204d8fd2d62b7614e381e4485a5257eade535dd2e8ca76aa25449b79d8be1c3efaadaec3a89d2896a7a7c112e7456";
    const GAMMA_ABC_G1: vector<vector<u8>> = vector[
            x"a0d2e164b255e053c2f5530edd2a4d6797c4ed65642d23dd52bb684af8941cd8d443bda3d18a75935b5924f63b07704d",
            x"b41a1a69a8935b94690674301a8b9ccc807dc29938fbea2764836fe6c55692b4824ea39d1bf57202101fd267d0bb8c4d",
            x"86a7e612533d4522a8b01eab2b4e0cf04a44c0f22ac6cfe8082d330244fd60290cf3600bccb28a9f423d9a913e22507a",
            x"a3f6b01756941d0ecf5ee248943cb88276c60af188b325b73f87f27328d3d0fa04c3bd3880e8aed86c49911104d8b24e",
            x"afa9ea7d690ac2097786b27a84c0e8d9f41db2cb914bd7291c694012408c7a2d4b58b4bed243c48ca790a800080bf782",
            x"87cb8b20abd7565ff4f6859a62e2c90c3a610e78499083fc648d87b881f3549fda97d12140529e9c210bab62850e6f8e",
    ];
    // end auto-gen vk params

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

    public(package) fun verify_zkp(
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
        inputs_hash.append(*chain_id.as_bytes());
        inputs_hash.append(light_header_hash);
        inputs_hash.append(*trusted_validators_hash);

        let mut inputs_hash = sha2_256(inputs_hash);
        let first_elem = inputs_hash.borrow_mut(0);
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

    fun hash_commitment_bytes(buffer: vector<u8>, prime: vector<u8>): u256 {
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
