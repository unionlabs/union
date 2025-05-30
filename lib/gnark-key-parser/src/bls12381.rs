use ark_bls12_381::{G1Affine, G2Affine};

pub struct VerifyingKey {
    /// The `alpha * G`, where `G` is the generator of `E::G1`.
    pub alpha_g1: G1Affine,
    /// The `alpha * H`, where `H` is the generator of `E::G2`.
    pub beta_neg_g2: G2Affine,
    /// The `gamma * H`, where `H` is the generator of `E::G2`.
    pub gamma_neg_g2: G2Affine,
    /// The `delta * H`, where `H` is the generator of `E::G2`.
    pub delta_neg_g2: G2Affine,
    /// The `gamma^{-1} * (beta * a_i + alpha * b_i + c_i) * H`, where `H` is the generator of `E::G1`.
    pub gamma_abc_g1: Vec<G1Affine>,
    pub public_and_commitment_committed: Vec<Vec<u64>>,
    pub commitment_key: PedersenVerifyingKey,
}

pub struct PedersenVerifyingKey {
    pub g: G2Affine,
    pub g_root_sigma_neg: G2Affine,
}

#[cfg(test)]
mod tests {

    const G1_SIZE: usize = 48;
    const G2_SIZE: usize = G1_SIZE * 2;

    use ark_bls12_381::{Fq, Fq2};
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, SerializationError, Valid};
    use num_bigint::BigUint;

    use super::*;

    #[test]
    fn dump_sui_proof() {
        let zkp = hex::decode("0A8FB8F6066E6956618158D1A61DD27C1B30C0CF6BA0E100E45E19E0F6CE48536406B028541DD1DB38B180615DFB7A050C55CCBE3D4C4ADCF4818E3A67BB186F67095D8BB8C60BC29F0C20B25421D3231260060DD63353B1D8BA96D140EAE02A1510808F24D1ABB6E491337614851D036B3D892953022BF81B185A7906455146338C7CAE98B4D3D213B64A6895780DFA065204820A3693C519AA2F556FDD8E538519A94164AC6C902DB630F07326565EE83C4ACA79C6C48AC9FA837BA22684C50886E74B93579D8270BD2918694FCBDF4FD834D30421E9FC8C81683BAC257F19A1D6D2B942B2CF7B5E2D30F1AD0A67AE0107171B1E46E25B1914A56446B4DCE9D4BFDC800CF688FD7FFA2B032AEFF6225BEC922740F8025ACBF949401BA586E11254BAEE94E80B8DE33C8E5382A94C4AFC84F3237966A93429E29AFC817D8369D552C53F3CFB5C7BF26452F5BC7B47AB157CC8E6B9CAC2B5FCA8C934B15960BA2AEAB017AB10E7D89C01B53B21B05AED12440C452EF69E0A2AAA5D10CE9D0C4509D68A72455778D02A30776150E97CC8A4C23FA3AD44ABE74699ECFA43108EB0951DA0778AEC9163FF0BA793C79A676C0F48884F44EB20D24E8E4599132EDB237C90E2DDFBB64CD9FBFB860F2226DED721117AA6AEF507C604314675F1160C780D3E9F262F2CBA70DD6EB812C524971B5C7379B510A6623DE8C2B10E2106CB167C15E45956FAF14F8539E47828A6A1C51123ED26609F6248370044143CCA0B322BEE55073828FB0B3ABA26393165A2F1A99A3AAFB3F47542F2FD1EDFA3CDCC731346E297892D74138BFEE2FCFEDBF421F5A000BD75CB87FECBCF5867DD715DC02CE84B6354099E089A6EE07A6A13922C37A4F08A8D28DE81FCBD3D970C926C10").unwrap();

        let mut buf = Vec::new();

        let serialize_g1 =
            |cursor: &mut usize, buf: &mut Vec<u8>, zkp: &[u8]| -> Result<(), SerializationError> {
                let proof = G1Affine::new_unchecked(
                    Fq::from(BigUint::from_bytes_be(&zkp[*cursor..*cursor + G1_SIZE])),
                    Fq::from(BigUint::from_bytes_be(
                        &zkp[*cursor + G1_SIZE..*cursor + (G1_SIZE * 2)],
                    )),
                );
                proof.check()?;
                *cursor += G1_SIZE * 2;
                proof.serialize_compressed(buf)?;
                Ok(())
            };

        let serialize_g2 =
            |cursor: &mut usize, buf: &mut Vec<u8>, zkp: &[u8]| -> Result<(), SerializationError> {
                let proof = G2Affine::new_unchecked(
                    Fq2::new(
                        Fq::from(BigUint::from_bytes_be(
                            &zkp[*cursor + G1_SIZE..*cursor + (2 * G1_SIZE)],
                        )),
                        Fq::from(BigUint::from_bytes_be(&zkp[*cursor..*cursor + G1_SIZE])),
                    ),
                    Fq2::new(
                        Fq::from(BigUint::from_bytes_be(
                            &zkp[*cursor + (G1_SIZE * 3)..*cursor + (G1_SIZE * 4)],
                        )),
                        Fq::from(BigUint::from_bytes_be(
                            &zkp[*cursor + (G1_SIZE * 2)..*cursor + (G1_SIZE * 3)],
                        )),
                    ),
                );
                proof.check()?;
                *cursor += G1_SIZE * 4;
                proof.serialize_compressed(buf)?;
                Ok(())
            };

        let mut cursor = 0;
        // zkp.proof.a
        serialize_g1(&mut cursor, &mut buf, &zkp).unwrap();
        // zkp.proof.b
        serialize_g2(&mut cursor, &mut buf, &zkp).unwrap();
        // zkp.proof.c
        serialize_g1(&mut cursor, &mut buf, &zkp).unwrap();
        // zkp.poc
        serialize_g1(&mut cursor, &mut buf, &zkp).unwrap();
        // zkp.pok
        serialize_g1(&mut cursor, &mut buf, &zkp).unwrap();

        buf.extend_from_slice(&zkp[cursor..]);

        println!("{}", hex::encode(&buf));
    }

    #[test]
    fn dump_sui() {
        let file = hex::decode("8086e08eb1cf3993e5ad8c6f563a64ce098028a481001d214d95a59a84e43d65c9f9ad2e9550b0f39a1d2ad6cbd4adb9a7b33f040abc2ba9b994e8a4a966a0994db3f84e9bd20f4308bbb5a835434c0510e39ac4193c4bdc6817912c028f1c638fd014d1d004ba199f0acd264c35afa7f21871996cc1d5124a8fe77d29a6adf713245f2ae42479b7b9f6a9c6215209b106f7dc4670698106c314c33bd4934aa904e7c8f1508ed6ee85bca883130450f03e20875d334afcbbf76648f5316bd0298f96ad63c4325a5fb3a3543ead06227ab9eaf3aabb2b0b81aa55e44dba938dee4b62b3e41ed39d96cce81569a4df88bd07ff886ab56ce74ddb88d799436c97fe6500f7957ae64b8cfa67faff2a842e9199f93d15a168e681751d3ed262855775940818295a72b5173f91f2ff44298efaf5bb793726c0c016602d11c47c5e51f7bea4bb49e7d9f70e88c3f67acc48e739883b49a682745799f90f65aa43aa219e4015a812631ecb4eb6cea58c4728509ddc670037f8470af2a4e570259f60749015ae641d3fdb0548b540c6526fcd67298480cf39fcb68db69dfc10c406738d3ffc11719fc6484b75dbdbda32979f19cb00000006ab8b14673eeba5331569419a8eef64d916761c7253de08eb2d9f1d180a4c65a87b22f85f1a84cb4f7e7b1b7bc07b5d5f8992ea01643a1cf3644ca96162e5c057264074e1837d0ef421db295c0502aaca1f3c7368270c5ba7ecbba2b06992ea4ab762895b5c5ac4c2b88b2b2b385af1963d666729d18552e58545883a4b78cf136840165c9b396ec40a7d6e8009c10cad8a4361ed926829a0bccc16df614bc4810fb37661f9b946173f6b45f339066037fce2e681cddcb2811a1f5728f133dc68ad226b5b860b5ff2a91e83c34add8b8ef087f719a413ddecb41bab41536c9b6d37ff0fa3c5b7c466c21962389eb2b2558afcbf691d60d25be2742fea58c80763964e5d537d6452e8403cafd851ca0177b22a8dc58016bb3e9090e6c17e3b06f3000000010000000096b833b796e17d0ae02eb662ebf8a73f1da45b3354b8f6d0660d43628328fc8c6a481dac073b1490dee2baec14f1baa1177d2390d8dd7751c0ed8b0bedbb620fc8b8dfe63a5e38f0fffa56e68e0e5d0b43ecb6ee5537d8cbb3ae3c5ca5d265b5999c10fdf65613a5e0c066a17222d25a95e61db7daad037a686f073dbccca19a3391bd11b68b0fa1878aa82f5c7439ab115f2692c0af6bb1f2b65c9db48f081ca0e306cd867c8351abd3e7fa9317170ea10ec9583a7b97a5803e9e0ea09c2299").unwrap();

        let mut cursor = 0;
        let alpha_g1 = G1Affine::deserialize_compressed(&file[cursor..cursor + G1_SIZE]).unwrap();
        cursor += G1_SIZE;
        let _beta_g1 = G1Affine::deserialize_compressed(&file[cursor..cursor + G1_SIZE]).unwrap();
        cursor += G1_SIZE;
        let beta_g2 = G2Affine::deserialize_compressed(&file[cursor..cursor + G2_SIZE]).unwrap();
        cursor += G2_SIZE;
        let gamma_g2 = G2Affine::deserialize_compressed(&file[cursor..cursor + G2_SIZE]).unwrap();
        cursor += G2_SIZE;
        let _delta_g1 = G1Affine::deserialize_compressed(&file[cursor..cursor + G1_SIZE]).unwrap();
        cursor += G1_SIZE;
        let delta_g2 = G2Affine::deserialize_compressed(&file[cursor..cursor + G2_SIZE]).unwrap();
        cursor += G2_SIZE;
        let (n_read, gamma_abc_g1) = parse_affine_g1_array(&file[cursor..]).unwrap();
        cursor += n_read;
        let (n_read, public_and_commitment_committed) =
            parse_uint64_slice_slice(&file[cursor..]).unwrap();
        cursor += n_read;
        let g = G2Affine::deserialize_compressed(&file[cursor..cursor + G2_SIZE]).unwrap();
        cursor += G2_SIZE;
        let g_root_sigma_neg =
            G2Affine::deserialize_compressed(&file[cursor..cursor + G2_SIZE]).unwrap();

        let parsed_key = VerifyingKey {
            alpha_g1,
            beta_neg_g2: -beta_g2,
            gamma_neg_g2: -gamma_g2,
            delta_neg_g2: -delta_g2,
            gamma_abc_g1,
            public_and_commitment_committed,
            commitment_key: PedersenVerifyingKey {
                g,
                g_root_sigma_neg,
            },
        };

        fn serialize<T: CanonicalSerialize>(point: &T) -> String {
            let mut out = Vec::new();
            CanonicalSerialize::serialize_compressed(point, &mut out).unwrap();
            hex::encode(out)
        }

        println!(
            "const ALPHA_G1: vector<u8> = x\"{}\";",
            serialize(&parsed_key.alpha_g1)
        );
        println!(
            "const BETA_G2: vector<u8> = x\"{}\";",
            serialize(&parsed_key.beta_neg_g2)
        );
        println!(
            "const GAMMA_G2: vector<u8> = x\"{}\";",
            serialize(&parsed_key.gamma_neg_g2)
        );
        println!(
            "const DELTA_G2: vector<u8> = x\"{}\";",
            serialize(&parsed_key.delta_neg_g2)
        );
        println!(
            "const PEDERSEN_G: vector<u8> = x\"{}\";",
            serialize(&parsed_key.commitment_key.g)
        );

        println!(
            "const PEDERSEN_G_ROOT_SIGMA_NEG: vector<u8> = x\"{}\";",
            serialize(&parsed_key.commitment_key.g_root_sigma_neg),
        );

        println!("const GAMMA_ABC_G1: vector<vector<u8>> = vector[");
        parsed_key.gamma_abc_g1.into_iter().for_each(|g1| {
            println!("\tx\"{}\",", serialize(&g1));
        });
        println!("];");
    }

    fn parse_affine_g1_array(buf: &[u8]) -> Result<(usize, Vec<G1Affine>), ()> {
        let size = u32::from_be_bytes((&buf[0..4]).try_into().expect("impossible"));
        let mut g1s = Vec::new();
        let mut cursor = 4;
        for _ in 0..size {
            let g1 = G1Affine::deserialize_compressed(&buf[cursor..cursor + G1_SIZE]).unwrap();
            cursor += G1_SIZE;
            g1s.push(g1);
        }

        Ok((G1_SIZE * (size as usize) + 4, g1s))
    }

    fn parse_uint64_slice(buf: &[u8]) -> Result<(usize, Vec<u64>), ()> {
        let size = u32::from_be_bytes((&buf[0..4]).try_into().expect("impossible"));
        let mut items = Vec::new();
        let mut cursor = 4;
        for _ in 0..size {
            items.push(u64::from_be_bytes(
                (&buf[cursor..cursor + 8]).try_into().expect("impossible"),
            ));
            cursor += 8;
        }
        Ok((cursor, items))
    }

    pub fn parse_uint64_slice_slice(buf: &[u8]) -> Result<(usize, Vec<Vec<u64>>), ()> {
        let size = u32::from_be_bytes((&buf[0..4]).try_into().expect("impossible"));
        let mut items = Vec::new();
        let mut cursor = 4;
        for _ in 0..size {
            let (cur_read, value) = parse_uint64_slice(&buf[cursor..])?;
            cursor += cur_read;
            items.push(value);
        }
        Ok((cursor, items))
    }
}
