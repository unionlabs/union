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
        let zkp = hex::decode("0ABAA32CCF06FC56E41D26352BD618030C9D935F666CBEBE004225B89150D2CF63EE17ECF8A5D1B06C1789D85FF0E26009380699CC97F99A4E2CB17803BC231F6286F2192EC7D25C1960C754911B7BD7A060A0AE3061BEC04F4D6C586E605B6E16FE4FDA3ED397D42861EA88B35ADE85B17BA2D9A48B366D350EE15D7FCCB7BD6EBA66EEEB6386BAE0CC8D2B9B402287066DBED8B29E882F441F5A2EB1D97E9B10554166758E9E8EC4F3DFEAA81734634CF1C1A839411D51C7E2F237051848420625690BBA6AD16C33EE4081FD7200FA739D9AE1AA5EE08E707B77FE3E22566019AECDBB5F0C8773B284CC4A76DF7807159044C10952F04D131A541E4C30B54FF08151C492F16433CDC2AD14EAAB450BD68733AD80762FB34D682CFF7526CED416C240F87FB8D0DD347F747A62C691FE210CFADC157C2522D07966D22CC3B8C3846305054ACA0B1111CC17FE9786080716811320243E94D5A45E38D0C2FFF90D06FCC9DF9F1A06AC76A5E1765A4A1F3B89F9D63967344CA9EB0B893DBBA51BF8052EAE54D8B50EB8328EC6D36B644E7A444C9B05E4DEBAABEEF5792FD6E8ADDD61EE6A7A9B5A6606BBB8E9E111C8C75F08B615AB56E7209CBC352C9AC7EA0205D79B4AF84EAE80707983E20F69C8E9E300694AA9A240E9775B26BB76448E58AF0739CECE24438683AA3D8802BB60A481203823A5D324AFB9E7AEBFCCFB1A326F431F4D55A739B15C4E76C2AA89D0CAB0041DD466F09F7FA6111A51D5E61F4334EE89CBE4B9FF4CA1C40972B051473702D717D8F3E7A30D9A2A76228291C55F46").unwrap();

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

        println!("{}", hex::encode(&buf));
    }

    #[test]
    fn dump_sui() {
        let file = hex::decode("9550842cc4eccd283a3f7b666a97fd4f8b2503f3675416b41f93d9b31500ccf2827d9e9a533065b0dace3143d5eacf97b3cff6b813c6288c3de70b81c22f2c65bfc766e44b6df1fd2404070cd07814b237b64d80f8b76f3ec80e8b33438901c8afcb83ae46c8984f1aa5a929e4acd703123022c48c461996e864709092a617ed676be02dac6f153deb18c1b0b2934fce01ecfa883a35d24350d5e370278f01c8629d3baff42dc9e66972436bc125202d2adf27a3f38ec3c7ef7c33d369755c2fae18e6d5ce0b15090fd6fa2894fd16164156b8528b16d37990b1366266f68337356302e9d3bac34f6c8c3ec4028c177f062b45901802e28ef77442111777f0030947239066bdae0bf3a7b59da18fe032394eb2016709a930eb8eaf2cac34e69d888a8132bb97cf389daa3e29a232f49e876fd592767c541b073e73647a35cf0dad695a4df06378a72343179e1991720dae20df7a6c6b3e3689ae8507f3f3d480edd9b6934cb7993cfcd55d1f9a634baa965b0fd0ec6047180102ab1c7e150b2d182f80f8ee0f4eb58635a52af2035850307b6a57d094995e8d9e65f3de547a034da8890cea5cab9879d01d49e614aae20000000581e28d6aeac1d94bed921578f4cd727cfecc34b03a689ce1ee78656986cab73e0c6583c87d77a94423a8951cd1eaf3fea191abf1aa358b72efe748bac3406c117543401d945fa91285a8ff329b5f30188f2ce2e8f335b857be7aa101e90d72fcb726a5c9ee9cc1ce72de18090b128baf3fd23771070b5c619e088110b1043b075c3a52217c52304eabd3af697738253c8d7a503024d249fc24e947d424df71b8f2ed34f2d26985e7b8b226a168098bf4ce35effdf1e733c62dcd040504e4493694b5d76c6de68b70f34d6dd6abfe3fff7a9d90b6b51847b791cf78079d8f317b7f75a234f192fe368515685cf6fd108c00000001000000008d8bef59ac1cd2723d11e2c95029ad0ead271446343d408dd0cb49b9e091eb5770c9322fba788c65089c915d8c69be4d0d6838e6ef6f7e806879a31524069759b1a9306b1b4d0ff0fbf57509e7bff7ff14b6f45ebc1e2efb58338f885c1ac9d2969ef80fc24c1ea885059845feb1b518764f23cb08696511b7a90b1fba55c521b286a6a5ab16aaa14ea82d38afc133801339c86ec4a2831c7338ff26010c0b90c7e10cd95cd258178d3e6c5dbbba38dbc09039e888181e5ed35158373b47f4a5").unwrap();

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
