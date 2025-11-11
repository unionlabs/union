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
        let file = hex::decode("adfb66157343861e3ef7f8157e008c5f9b56ab14c09d0bc5d564049bc1fc4b0f774b426b9f5d7a4fb18212eeb69feaf29420315a27f83de336886bd12582ee72ff163d518d3d6559d5683aa6796548805931e7ff8ea500432f6252785e192b10b59e1c22eecb4403a4869f1d649d1f61b8128398922af78a1925a5847e8fc1488252a45f63bd0ccee0da7c978cab9c7e02bffa98fd0e31ac4f945b47d525f2814b35476c97fe22e7ff5043243b3ea42376328c998b626d4b9e8f46191f5b4eb0ac0eae045e3810465990cfbf5a013dfc1d4cd628dc2d21ee97bc7fb3dea0774ce9ce52eee31946fb75589d6ff2b411bb16fd075eb7035b06a028fe39437dc9803da4065fb95e07aa241e006064f855e752835873f7344d10e164c36727c1455993704005e4b1409fc3a2272a1ec0fe523644c60eea964920d45c3184219350b928a18942ff4efe44fec9e05ad67d94dba5a4fbdf8b2c39507a586def717e01fe05fef1f5bac2ed7e40f56e031ae26de540e21e14b5a8e687d117dfca9b4751050204e742d28415c514645619b07173b82c79b08decfb72d74c06561582afff3d61cd788f1ad39bb2d22e173a60dd3c8f0000000a90a66f22fcbc9230eb1607ccf92d854418743e3ac39f4048d08bfe18a61f8b600244bc48c0d89639751b1f4cc6866e38981b27e2e75701208eef96ff00c15bfb3665f0216ed6368fe5e6c918191c8c13ed1ca16624a0727cb094ccd1aba02d1db2ad0013c80000d5e6edd620dab11e4d5ae4c837aebb21efe186cf2a2dec75091eeacd5458808d750d49fd50421d8e6692e7114153215cefbdf589eb597d96a48c34059dd21561ab36b5fd5ba7bb1ea209538e416717fd50f6e066dbc84d942a8856c7eaa48dca0a14cb5a71ca527f980172d6db3090826170751e2ea70b8acb85c29046048176479d088c02efeca78bb31b00805afecfd69679c738cd2c34b715c1832fa749a6fa03396fa5ab002c8b047cf84747368f5dbf600aa6fc590314a48063327a316636b163c0a70d44557cc9588193f0d736a07bad14845f3132e75559eb8abb06135b281e43229858e8a18f4029795fa9cf722da80313e708b43fb95e2a04031c4754cfd26548f07deb042573cadacd7d9c4372ba467e46e398ae8187b31dd0b944781ac36b3c373f17ca967a8c7ed8813f64e8649b9082986a9359fb70f25717010843bc713f822a4da594a4b72eac8904c918e602de9b195fc55da26516f8710c612afb58e3839682f48ae643f2e6698daae0c300eb322e38800000000100000004000000000000000100000000000000020000000000000003000000000000000484912efb383cc041b0326a7814fea6978fe8dde5cdef307814f2b9e77e27d2fb8426f6e0ea81ea1cb1a24103771f7a24145de139cf23599751235cd19a4667dfead22e110f0dece7a1081282014e67d386e98949f14e2cf2d22a2237488dde4885f1a0e0439e856ae2bb0113512fd3077a28257b750394bd802488452ea19c41596bf903f05f6485e8f1c1634bacc6a0089446e7909eccfcff578bc693fb91a49d721d9cae40f6e4667b380d4a32687c9298a4973a3a1bcb7dbb38bf46437064").unwrap();

        println!("{}", file.len());

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
