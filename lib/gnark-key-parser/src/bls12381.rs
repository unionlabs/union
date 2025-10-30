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
        let file = hex::decode("86efe8ebb6fb4db2efea81a6ca7ba5bf3b4f2ad21e3b005293fdbad767332109a6ac904d71f21884068a76f9445bf98697cbb52afc02648c66f26d2fe24acfb663d22a5791074f9b30f48836cb1ccfa2864f41a98ff42675f78fcf188e2bb5c18650b87220e4c4305b7a1dae122bfe4a209ee92dd422812806c903f14c9a8831283d6bdaa70d0eac432c9e09800e26c90011cb4439a00523899fc3816c6dfd0b9c954401ac62521446491a971376f90e8533404c6be57031ffcc5f4faf492ffba495b8ca8a6f5b0a2f818085e5eb8269cba621795d4fef61ca0a843a122a2f14acedb36ff5fe9a7c107ba79837998ba908327f8db8c09fa45ab0a90fc4dc4325894c642ba9957bbb42dfa14f290b05ec376c2513088d793256b7e7e626ce2e25b056858c7c2f92576c8d4989311757c34b3f76c2ebcb85d4a51a673d531a1ae8ad53d9680a50a58f463c27d5882ecfe9a927f9c5b3e446dfa5b0e7264744df3a14c24ac9ac860bf870f85f7b1bf14a1eaaf5abe1725b86993bf0fd14b045d0d816a3483103a8484efe6c008dfa3b7aa95eb8785e49eca0866a86f7a4a4ff622ebae2384d2021c524588299ec50abb0f100000006853c135761b32157dbe05e7930484ecb4759660b6bd2acc5b075f7ce1ff0592df644a5acefee0c36c5787bdd132299fc958885c6390db688ea119d8b428f96c4057fda38dba6d5ee4dc6bc849db6b0f154deb7601f9291ebb232607c014c705fa496781926ff946fccd2e3209d03735059fd0080da21a2f5dd5a74c43de6b2db007294f45c6f895767f8d30908d515a98b9f240722d30edb62fc60461908c8603615979701af2581f30956ef1e6d44e35f1a1b0fd6a74a7c793cd4ce850a861c937684fab29383af93b5a40cb39c729fd26d0fb97261c6a5884a38333a88b0bc784b0e588453ed83ceb358808be524aab8746a0d6cc210d8b2be3f6285b42bda96e269de1936c94070ff8708eaaa6ee52b51e2cdd64c27d855d6e1418535f31c00000001000000009497dbfa5b011295fa7ef0eb40c2a3e506d248acb6b2d3ac973ce943bbdf639d798449a3f5d25a5f3c2eba0e3643f75b02138c50e106b97d4a8412fdc31bfa94dafd8f1e0212bd919150fc6214cc8f863b82d722d89f5f8629d6b51118139b7ea2b4c167c35ba1d662c0c07077b7a451b273fab763634ab2acb7e324ad0d82f839504f61c26cd99a5f29070eb571b3db14d0b3bebfcf242fd6159ceb0e007b02416d523e39e399fd25601e396154ad896268865a472c845327cb408bf334cdfb").unwrap();

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
