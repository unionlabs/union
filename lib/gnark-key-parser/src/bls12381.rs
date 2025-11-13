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
        let file = hex::decode("b9f7e77689d6c7266a897207825b267e93990de8b3986ab559b1a84482077c3db6dc3dcf209d999aaf648e6c7a010f8d9061e5eaa17a364b146f36f4ea97a0d4e60aa9f423faa80409a5ecc33ad6fd9f56a8ae393a003b8cae853b731150bfcc86102b906a85ee3318ee01a383ecde3813a12512980acdb10ae65fcf6707b4f815995705946313f62585eddebf9eb8421889d05dba98c06581e9c57173911c1a97b680aad76c86480c5fb60fa20f0ee18ff5eef48fcf5e4343d91d448b518d1ca9c81c59618663032718533659a9da29f21da0e610bccc533752fc6659fa7962a17294146160f91e6b3e4492bfd3624a0096bfab558277c93c1bc056d5442692338293d25ddc75e78b88f3efa3f99a4d96a906b0f8f54b26d6a22240cd3c46c7ac3d6671282df3b33d9b8c31812f1a19bc2451fae8be0b4eca1d12de322a9f3a56f39b2128599d07784eedec907415cf9619e27e840c20b9168ed0540f2c6ed626dddc034822b993ce3d3d850c73221463f0b363d78d819381ead058dd9c870e05f9fdd8a7817f05128e6ee751b96c1bac963c4d99a53647758e425b4f2faa722ee81d2d4135081441f78e92d4ab1aaf00000006a0d2e164b255e053c2f5530edd2a4d6797c4ed65642d23dd52bb684af8941cd8d443bda3d18a75935b5924f63b07704db41a1a69a8935b94690674301a8b9ccc807dc29938fbea2764836fe6c55692b4824ea39d1bf57202101fd267d0bb8c4d86a7e612533d4522a8b01eab2b4e0cf04a44c0f22ac6cfe8082d330244fd60290cf3600bccb28a9f423d9a913e22507aa3f6b01756941d0ecf5ee248943cb88276c60af188b325b73f87f27328d3d0fa04c3bd3880e8aed86c49911104d8b24eafa9ea7d690ac2097786b27a84c0e8d9f41db2cb914bd7291c694012408c7a2d4b58b4bed243c48ca790a800080bf78287cb8b20abd7565ff4f6859a62e2c90c3a610e78499083fc648d87b881f3549fda97d12140529e9c210bab62850e6f8e00000001000000008ae1a8808f22c833d734faba7c7557faf1b75e93d0ae4fcb8191b9ae0ec8bb18f86cdaf2ab9b085754afc2169f078b0a0dee291456886f03f4969c3f3092431f97de42dd3557a377b11babde9c5e830855cf38a57f661fcdfd1f522decca7b76acdaa84a7d14e824eca568c00f43d8138c6fae85e4e8a7be2ad6a94286260fb5e6b29b0ab579c2c821c6bc787887fdb0009204d8fd2d62b7614e381e4485a5257eade535dd2e8ca76aa25449b79d8be1c3efaadaec3a89d2896a7a7c112e7456").unwrap();

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
