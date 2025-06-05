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
