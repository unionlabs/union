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
        let zkp = hex::decode("132726C4DDC6D39D064F620BFDFC8D0B1786818B9F3D150C59A3EAE294B78FFAB878AEE354E387278561A36BC5FC93FC17BAECEA01996CB591FD03F7913037FAC31BCFE94675EEE2C3690DCF1DB2D54911E6F27C10A893858944A2C71C3D15CC0C4E2A7683345FD4E01C804DA23097A4E18B098B47299C2566C37C8C22C4DEB05057947BCB90F24A7AF4B5E0E65B2ADC0FE6F413506CDC1FECF1E18CA23F4E356CB9D930BDBB0CD966101BCE3E6852BE2054C8CB92E371373FFEA6876BE171C018A6E641051F7E5F5039322DA99506401356B3515A91956EC15E11B8C4D5FDDF4F25265486C248933EFD959815E0B5970B9673FF2145608495144E150CFC1BA313D2BC4E3062D335FD53D6314E1D7C32450E976C8444563E180201FF818E640A18E9F3D16286303DDC387F47E435A4C378505BF3ABCD0D5C052B9CC007415A0E0BDDDA783D759625CEFABF7133FD41D8076A0EB0428C16A28AC89A150CDFEED543126B9EF0ACFD9895D38DD2C68506875513A8F12328BB2044A2A19E82074C841782ABFA90170A69DD6314A03388CF35F7A7DE483E08A457E2BC25AFEF9E2BA7AE257F7507EB4BCC3767281E5F63986E113824A4C8FB09F8E58A2864B38CB582E8C7B159082C06330707B75FCA255B79A6164E04369EA1532705512E0DA427790435E9828488F47AE8D60D1F1E88872187CFBC014878631B022233A9961A88DB7138C3AEA9878E101DC475F551FDB9E717C4EC2872C23A23F5F9B87C112C5A274E0FD875B53CE619590E356FD22FACF1F7A990B752548F73FAAFD8D94EFCFE6103B2697868231C45CED290C2C4000E368CB86A36C0649AA8575E0B423331F35103DCE7190414A8CE71CC6CFB15035E26BDA47FB60551B0AE44AC9A03EA992043").unwrap();

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
        let file = hex::decode("925c9a8d94f2a53ae6422956126ce095028ab0c69550a605d0b8f7abc90934ecd8f1ddd39744d3b6350bea3aca93ccdf859e3566688df58ea0349fd77fe2766233c3258169e927c4f48de38a78c9422a2cac68c6550524d61ab0a3e89f7d8d8aa70d09b7918654f7f886e976766961f41f58ae1513134cf99900e359fc2282de3db96afb32f3515d6b402c201dc0eb380aed7853a9a15fc2089c4d4d6220d1a9dccb1e1af709b9e6c7f1901873d8a2325337299f96180bc986c4b6b73bc943eb97184ef7a1cb6b29d15e5737fe21c1e8144d3f0ff0a4b111129c0e3eafed72f751776c7ea7b7121e2ef3cfbb6f326fb50944859bfb11c966d225132c24e9377d0649936ae40d23b05dc1fb04e38e12ffd23e086fa25bcda2c67ee05cf9e3525883595a722323eb32ea5fed585dbc6a885bba860493a741195294a506807b4084beafdc2d11ad73329cfeb528a3780b288cce5bd32c4400e142de4733f0ea331fa659757ba4776efda5d37a7d6bea6c0d272603fea15ede8993f2881b18904bfc17bf8ba06b37ef971120e37adacd6f79e0007775b433005a15a911902d0944d71b44811bb6afde00d65f7201ad33a8d30000000680b8092ccbfaa14079ceb76cd058e50e22ea80a22ba034123060552adb88c38749b0c72132c9771f58b48fbda359575f85263273f2a2158b2336f1445797bad8a05eb61e302c6f819c0bbcd329b9db510668de6894e0ae8340aaa217ed597acab89c669d8e6678ba4c2469bb04704b6b915b76657c21e86a35e17f666a710c9b32f3be287767edf142dfd483f8ec1685a79f14ce6ef6698143e2d6552ef86f8d9a17e792a658a4a4208a50deeece0c14097bfdf1e792cbea5efcb419b81b3885907fef8e452c6f6e5224dc1325eca7abbca4811cc5bfcde6fb5bba5722877b40584616564b1dd538e03bbc0c0b5343fc908f777c2fcc59ed481a56585f47c48b7776e92ef90a710bc7d6eeba3d8a0d5f3c8885924a2b5407b45718ec62f67982000000010000000099a4735f5ea5db8464326c674848d1075d5296f0d157f41a32833193b005b296ab6b8d56d98595afd587191ce0764b2f1380449dc6ed55558a7693429188d2d1ac5fdba824e28875f5c871695a8bd938cdd70fbbb83e462382981db8957d7ed4b5f6e5c6cfb88961a37ab43cdff8cf7df4e2427aefb232dfb083ea8e1437913c033734112aebcdd4243d5d88f21d708c07e139f82e1ff4f444adfe71db4695d5e2caec38815aaa202bc67637b052a0b340aa20e675638906d21d8493e27ea23d").unwrap();

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
