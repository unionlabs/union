module ibc::groth16_verifier {
    use sui::bls12381::{Self, G1, G2, GT, scalar_one, g1_from_bytes, scalar_from_bytes, g1_multi_scalar_multiplication, g1_add, g2_from_bytes, pairing, gt_add, g1_to_uncompressed_g1};
    use sui::group_ops::{Self, Element};
    use sui::hash::keccak256;
    use sui::bcs;
    
    use std::string::String;
    use std::hash::sha2_256;

    const ALPHA_G1: vector<u8> = x"925c9a8d94f2a53ae6422956126ce095028ab0c69550a605d0b8f7abc90934ecd8f1ddd39744d3b6350bea3aca93ccdf";
    const BETA_G2: vector<u8> = x"870d09b7918654f7f886e976766961f41f58ae1513134cf99900e359fc2282de3db96afb32f3515d6b402c201dc0eb380aed7853a9a15fc2089c4d4d6220d1a9dccb1e1af709b9e6c7f1901873d8a2325337299f96180bc986c4b6b73bc943eb";
    const GAMMA_G2: vector<u8> = x"b7184ef7a1cb6b29d15e5737fe21c1e8144d3f0ff0a4b111129c0e3eafed72f751776c7ea7b7121e2ef3cfbb6f326fb50944859bfb11c966d225132c24e9377d0649936ae40d23b05dc1fb04e38e12ffd23e086fa25bcda2c67ee05cf9e35258";
    const DELTA_G2: vector<u8> = x"acce5bd32c4400e142de4733f0ea331fa659757ba4776efda5d37a7d6bea6c0d272603fea15ede8993f2881b18904bfc17bf8ba06b37ef971120e37adacd6f79e0007775b433005a15a911902d0944d71b44811bb6afde00d65f7201ad33a8d3";
    const PEDERSEN_G: vector<u8> = x"99a4735f5ea5db8464326c674848d1075d5296f0d157f41a32833193b005b296ab6b8d56d98595afd587191ce0764b2f1380449dc6ed55558a7693429188d2d1ac5fdba824e28875f5c871695a8bd938cdd70fbbb83e462382981db8957d7ed4";
    const PEDERSEN_G_ROOT_SIGMA_NEG: vector<u8> = x"b5f6e5c6cfb88961a37ab43cdff8cf7df4e2427aefb232dfb083ea8e1437913c033734112aebcdd4243d5d88f21d708c07e139f82e1ff4f444adfe71db4695d5e2caec38815aaa202bc67637b052a0b340aa20e675638906d21d8493e27ea23d";
    const GAMMA_ABC_G1: vector<vector<u8>> = vector[
            x"80b8092ccbfaa14079ceb76cd058e50e22ea80a22ba034123060552adb88c38749b0c72132c9771f58b48fbda359575f",
            x"85263273f2a2158b2336f1445797bad8a05eb61e302c6f819c0bbcd329b9db510668de6894e0ae8340aaa217ed597aca",
            x"b89c669d8e6678ba4c2469bb04704b6b915b76657c21e86a35e17f666a710c9b32f3be287767edf142dfd483f8ec1685",
            x"a79f14ce6ef6698143e2d6552ef86f8d9a17e792a658a4a4208a50deeece0c14097bfdf1e792cbea5efcb419b81b3885",
            x"907fef8e452c6f6e5224dc1325eca7abbca4811cc5bfcde6fb5bba5722877b40584616564b1dd538e03bbc0c0b5343fc",
            x"908f777c2fcc59ed481a56585f47c48b7776e92ef90a710bc7d6eeba3d8a0d5f3c8885924a2b5407b45718ec62f67982",
    ];
    const PRIME_R_MINUS_ONE: vector<u8> = x"00000000fffffffffe5bfeff02a4bd5305d8a10908d83933487d9d2953a7ed73";
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
        let proof = x"b32726c4ddc6d39d064f620bfdfc8d0b1786818b9f3d150c59a3eae294b78ffab878aee354e387278561a36bc5fc93fcac4e2a7683345fd4e01c804da23097a4e18b098b47299c2566c37c8c22c4deb05057947bcb90f24a7af4b5e0e65b2adc0fe6f413506cdc1fecf1e18ca23f4e356cb9d930bdbb0cd966101bce3e6852be2054c8cb92e371373ffea6876be171c098e9f3d16286303ddc387f47e435a4c378505bf3abcd0d5c052b9cc007415a0e0bddda783d759625cefabf7133fd41d8b782abfa90170a69dd6314a03388cf35f7a7de483e08a457e2bc25afef9e2ba7ae257f7507eb4bcc3767281e5f63986ea435e9828488f47ae8d60d1f1e88872187cfbc014878631b022233a9961a88db7138c3aea9878e101dc475f551fdb9e703b2697868231c45ced290c2c4000e368cb86a36c0649aa8575e0b423331f35103dce7190414a8ce71cc6cfb15035e26bda47fb60551b0ae44ac9a03ea992043";
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

        //
        let mut v = PRIME_R_MINUS_ONE;
        v.reverse();
        std::debug::print(&v);
        // assert!(verify_zkp(inputs_hash, &zkp), 1);
    }
    
}
