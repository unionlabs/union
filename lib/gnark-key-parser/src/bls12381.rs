use ark_bls12_381::{Fq, Fq2, G1Affine, G2Affine};
use ark_serialize::{CanonicalSerialize, SerializationError, Valid};
use num_bigint::BigUint;

pub const G1_SIZE: usize = 48;
pub const G2_SIZE: usize = G1_SIZE * 2;

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

pub fn reencode_evm_zkp_for_sui(zkp: &[u8]) -> Result<Vec<u8>, SerializationError> {
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
    serialize_g1(&mut cursor, &mut buf, zkp).unwrap();
    // zkp.proof.b
    serialize_g2(&mut cursor, &mut buf, zkp).unwrap();
    // zkp.proof.c
    serialize_g1(&mut cursor, &mut buf, zkp).unwrap();
    // zkp.poc
    serialize_g1(&mut cursor, &mut buf, zkp).unwrap();
    // zkp.pok
    serialize_g1(&mut cursor, &mut buf, zkp).unwrap();

    buf.extend_from_slice(&zkp[cursor..]);

    Ok(buf)
}

#[cfg(test)]
mod tests {

    use ark_serialize::CanonicalDeserialize;

    use super::*;

    #[test]
    fn dump_sui_proof() {
        let zkp = hex::decode("0E6F419184390EE9847CFF1AD98F8E29492391C536E99A2C95CB487AD22B9571931F9A9B33292FAE5567E5C0779F69E6055D4E5B48B834D71216AFCAEC185BECB919D325035A2F041F1480340DFEDFF7E67AD132B310CE1DA03F51188128ADAC0F75E49A4F4F93CA3655C0CEE5C585C2F49A266B8387DC3EA8740CB45FAC3A40B5FE2F004945FF5B1F9D4A1F49550E1509D2D919E35D3EC36D75428E950760225EC2FF49ACA3009786E67BD688E34E1B882BAAFC5317964E4A30F59ABF45B429063F586AD31A8F65F8898AEBA35973A35615DA7E82A3C167FB887D7C18C3602AF22C9ABEEC865038B544F756413AD8C409957FF28C5AA6131C3D3717B585A813B7BDECE396DE5A7C2476F8234D1DA3590B7A56A15B6CC6E9CB7C6A75D34AD4C20593B0B833D06F36E8A6447CCADE4B912E2FAC63B130D50F7A5F1CD3E04DD1AD280FE3DED32DDF1564CA4CDFB7EBA18E0F06D9D009E28B46A0F7C5460B5A6107DCD429604B508AFA61D9414E0F6F1915CE5787D871E90284E5B8FD559672D644170650667730FCF55085CE7AE91E0F271F648779C431C07A67C8925421DCFD8BD74E4AD3672BA0A57ACF1521BB653C6E0D24888CF809A128D8F3462BAE99400A645E1313CA535A9505832A87AA52644BD8B0A919FB11E5D3CF7DFED1A1FD638D04DD7BC5FE436681859040B7B77F9FA7E446B06D24369A9DA1C361AB2F9089C32A1614F35D8C0CE6D366C840F0866339156F0FF4C0E208AC203CF336BBDE9B6B5BD21898C35527B354A4E8D4ED77A2AA4F69AC7CDF9C70E2C4B7B2B9AB1913D22F448F3DD5351A07EC7BA74F77C4F232D4BA428F916D6809E7AD607926A5FDD41150FC7ACB376D2836365B2B98DDC97D5AE980398A13B9A3E43323D985943C2A").unwrap();
        let buf = reencode_evm_zkp_for_sui(&zkp).unwrap();
        println!("{}", hex::encode(&buf));
    }

    #[test]
    fn dump_sui() {
        let file = hex::decode("b706bf79c9b7e5d9d1fb703bf4f606777bfb2e2e1f724f91dd1d5193fbdd0d36fcceda04cf9a9fbf7d3718b3046eb4dd81cd76deaf2e2c5e5d3f0cc3c8e967571f449b597d2549bba89e5e5b2e5548ab857f75f8970abac5a2b02226043074dea01b5035865dccac3fadb033fbec5959e1b7264cdcdfc352c152d559de6283986b259212b2fd56daef7fd0a63e6803e50ab0c20188252f093aa54bbc737fc31f00c86e8ab4b272e49d2cd89370d28f5d24ad1887ba06ae5dca6b173bb5a5fd99ab55121e3a95a8f682f2e2086e005266dc944534e529ce4604ecaac49e69bf6e80734799553f6eb6772ad90939fb0d350480561eba24fad36ad28800c7d0167cebeda20eb3b4b8362df4bb05cb396c492e0756ef008af5bad190bd8dd4123cb7a1c2f414a5c8eb0388c252b8a54cba4b918515d4b604e51ca7bbc6e729643756bcc9bc17407c37c5e1da4f426747a18fa3eefe377f2900eade528966dc0dd84a919001b1f635c8176d9ad86c1acdf093061120e6be082c2a401ecfea67201d8c009301f18724caccb076284f878b22cbb747fa4e4340eec384d431f0966a08522e7aec2e5cc7a1626e5c395d08c20b39000000068b1094224b92d7f408b18aa15c9cccddb3542fa49ee6175f07e312129392a24f101812d9b27955470053b87d989b14d7867d9e0c16d70c2a828cb21326a0c8eab9680b806c713d07d39ce400d75572c6b7bcebd6ffb87f8cd50de25a1269acbd921ee09bec8e5ed7c7fadf2073bc1c06acc709c8e38b1e7818c0942f1e5dd1537f00eff80d8c99e8a385be17010c422993029375b005688fd07945e647df364e4ec3e5840345df4def65c19976864b23c05ca7a9a03befd01ecd3015b76e8989988fc72ebd9bb2c5e26906357ba75dab680e85df4d2e1e952bf487ccf1e032c010cd6c2966cbf7cdb0f7fa37969466e4abe2b500a5c63e0186a449adc97d65680a50db455ed90a4a05f57fadcd1a106d8af74e2feb318b8c4326fe88c1d82ca50000000100000000a0d19e09533d88ec21589d1e991ba17d388c5e70d83e40177da3d7637f62692d9c2fb8df9e189b140a84711675f8071808ba3f7ed662659073ac569ae2b890c3a5db05eabc14ee160ca96d884e8270bcbdb689f4d1254b0ecedea6ff1c8c1ec993fc5558cdd13eb91412343b294b640fe5de538bc3fbd707c5c7e4eb66c8eb4bc4d907e1dc469dd02ea4ce315f25e1df01c1a25d5dc5a35a790321f3c29a7d752501c579badf6d247aa24de1671501fcd86a2148d3facfac3ff69bb9578a223a").unwrap();

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
