use std::{env, fs, marker::PhantomData, path::Path};

use byteorder::BigEndian;
use gnark_key_parser::VerifyingKey;
use hex_literal::hex;
use substrate_bn::{AffineG1, AffineG2};
use unionlabs::ByteArrayExt;

pub const FQ_SIZE: usize = 32;
pub const G1_SIZE: usize = 2 * FQ_SIZE;
pub const G2_SIZE: usize = 2 * G1_SIZE;
pub const COMMITMENT_HASH_SIZE: usize = 32;

fn parse_verifying_key(buf: &[u8]) -> String {
    let parsed_key = VerifyingKey::parse(buf).unwrap();
    let [alpha_g1_x0, alpha_g1_x1, alpha_g1_y0, alpha_g1_y1] =
        unsafe { std::mem::transmute::<AffineG1, [u128; 4]>(parsed_key.alpha_g1) };
    let [beta_g2_x00, beta_g2_x01, beta_g2_x10, beta_g2_x11, beta_g2_y00, beta_g2_y01, beta_g2_y10, beta_g2_y11] =
        unsafe { std::mem::transmute::<AffineG2, [u128; 8]>(parsed_key.beta_g2) };
    let [gamma_g2_x00, gamma_g2_x01, gamma_g2_x10, gamma_g2_x11, gamma_g2_y00, gamma_g2_y01, gamma_g2_y10, gamma_g2_y11] =
        unsafe { std::mem::transmute::<AffineG2, [u128; 8]>(parsed_key.gamma_g2) };
    let [delta_g2_x00, delta_g2_x01, delta_g2_x10, delta_g2_x11, delta_g2_y00, delta_g2_y01, delta_g2_y10, delta_g2_y11] =
        unsafe { std::mem::transmute::<AffineG2, [u128; 8]>(parsed_key.delta_g2) };

    let gamma_abc_size = parsed_key.gamma_abc_g1.len();
    let s: String = parsed_key
        .gamma_abc_g1
        .into_iter()
        .map(|g1| {
            let [g1_x0, g1_x1, g1_y0, g1_y1] =
                unsafe { std::mem::transmute::<AffineG1, [u128; 4]>(g1) };
            format!(
                r#"unsafe {{ core::mem::transmute::<[u128; 4], substrate_bn::AffineG1>
                        ([{g1_x0}, {g1_x1}, {g1_y0}, {g1_y1}]) }},
            "#
            )
        })
        .collect();

    format!(
        r#"
            pub const ALPHA_G1: substrate_bn::AffineG1 = unsafe {{ core::mem::transmute::<[u128; 4], substrate_bn::AffineG1>([{alpha_g1_x0}, {alpha_g1_x1}, {alpha_g1_y0}, {alpha_g1_y1}]) }};
            pub const BETA_G2: substrate_bn::AffineG2 = unsafe {{ core::mem::transmute::<[u128; 8], substrate_bn::AffineG2>([{beta_g2_x00}, {beta_g2_x01}, {beta_g2_x10}, {beta_g2_x11}, {beta_g2_y00}, {beta_g2_y01}, {beta_g2_y10}, {beta_g2_y11}]) }};
            pub const GAMMA_G2: substrate_bn::AffineG2 = unsafe {{ core::mem::transmute::<[u128; 8], substrate_bn::AffineG2>([{gamma_g2_x00}, {gamma_g2_x01}, {gamma_g2_x10}, {gamma_g2_x11}, {gamma_g2_y00}, {gamma_g2_y01}, {gamma_g2_y10}, {gamma_g2_y11}]) }};
            pub const DELTA_G2: substrate_bn::AffineG2 = unsafe {{ core::mem::transmute::<[u128; 8], substrate_bn::AffineG2>([{delta_g2_x00}, {delta_g2_x01}, {delta_g2_x10}, {delta_g2_x11}, {delta_g2_y00}, {delta_g2_y01}, {delta_g2_y10}, {delta_g2_y11}]) }};
            pub const GAMMA_ABC_G1: [substrate_bn::AffineG1; {gamma_abc_size}] = [{s}];
        "#
    )
}

pub struct G2Affine<FromOrder>(PhantomData<FromOrder>, substrate_bn::AffineG2);
pub type G2AffineBE = G2Affine<BigEndian>;

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidPublicInput,
    InvalidPoint,
    InvalidProof,
    InvalidVerifyingKey,
    InvalidCommitment,
    InvalidRawProof,
    InvalidChainId,
    InvalidHeight,
    InvalidTimestamp,
    InvalidSliceLength,
}

impl TryFrom<[u8; G2_SIZE]> for G2AffineBE {
    type Error = Error;
    fn try_from(value: [u8; G2_SIZE]) -> Result<Self, Self::Error> {
        Ok(G2Affine(
            PhantomData,
            substrate_bn::AffineG2::new(
                substrate_bn::Fq2::new(
                    substrate_bn::Fq::from_slice(&value.array_slice::<FQ_SIZE, FQ_SIZE>())
                        .map_err(|_| Error::InvalidPoint)?,
                    substrate_bn::Fq::from_slice(&value.array_slice::<0, FQ_SIZE>())
                        .map_err(|_| Error::InvalidPoint)?,
                ),
                substrate_bn::Fq2::new(
                    substrate_bn::Fq::from_slice(
                        &value.array_slice::<{ G1_SIZE + FQ_SIZE }, FQ_SIZE>(),
                    )
                    .map_err(|_| Error::InvalidPoint)?,
                    substrate_bn::Fq::from_slice(&value.array_slice::<G1_SIZE, FQ_SIZE>())
                        .map_err(|_| Error::InvalidPoint)?,
                ),
            )
            .map_err(|_| Error::InvalidPoint)?,
        ))
    }
}

fn pedersen_commitment_key() -> String {
    let g_raw = hex!("257DF6F8132CB0037F7DFDF1A29B04C1FF92BA082EDA513996BA2BFA9FBD198713F0D8D8879885CA567EF99298C30C397E6FBA584658F4127713A814C06DE55A1660EBCC60C7A3AC560EFCEA5993F528EE13685D3A39694ACD74FE67C80D798A15E80642C58DB4DBE0A87F92CE3C65E962F231278353783A691FD64078BA7F34");
    let g_root_sigma_neg_raw = hex!("2FBFE141A7555CF7E3E86B092660B81CFB68A025AD817E45CEC0B0F2E2CA636802A104DF1C015F2307FA2859627098CDF9FDB521D61D323943343A12304E5BAF27DA3F93ECF3BFD0B3A3354AE2162A6C230C0E539B6D9F82C0826E2B006A59222C0838551CB9E5CF67DB57DE7E2250BB97807F6687F135A6EB910359BA7BDB8D");
    let G2Affine(_, g) = G2Affine::<BigEndian>::try_from(g_raw).expect("impossible");
    let G2Affine(_, g_root_sigma_neg) =
        G2Affine::<BigEndian>::try_from(g_root_sigma_neg_raw).expect("impossible");
    let [g_g2_x00, g_g2_x01, g_g2_x10, g_g2_x11, g_g2_y00, g_g2_y01, g_g2_y10, g_g2_y11] =
        unsafe { std::mem::transmute::<AffineG2, [u128; 8]>(g) };
    let [g_root_sigma_neg_g2_x00, g_root_sigma_neg_g2_x01, g_root_sigma_neg_g2_x10, g_root_sigma_neg_g2_x11, g_root_sigma_neg_g2_y00, g_root_sigma_neg_g2_y01, g_root_sigma_neg_g2_y10, g_root_sigma_neg_g2_y11] =
        unsafe { std::mem::transmute::<AffineG2, [u128; 8]>(g_root_sigma_neg) };
    format!(
        r#"
            pub const PEDERSEN_G: substrate_bn::AffineG2 = unsafe {{ core::mem::transmute::<[u128; 8], substrate_bn::AffineG2>([{g_g2_x00}, {g_g2_x01}, {g_g2_x10}, {g_g2_x11}, {g_g2_y00}, {g_g2_y01}, {g_g2_y10}, {g_g2_y11}]) }};
            pub const PEDERSEN_G_ROOT_SIGMA_NEG: substrate_bn::AffineG2 = unsafe {{ core::mem::transmute::<[u128; 8], substrate_bn::AffineG2>([{g_root_sigma_neg_g2_x00}, {g_root_sigma_neg_g2_x01}, {g_root_sigma_neg_g2_x10}, {g_root_sigma_neg_g2_x11}, {g_root_sigma_neg_g2_y00}, {g_root_sigma_neg_g2_y01}, {g_root_sigma_neg_g2_y10}, {g_root_sigma_neg_g2_y11}]) }};
        "#,
    )
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("constants.rs");

    let data  = hex!("8967072901cc7ab63357f1ddc4196c7c1feda50540d8026d7f6f0167c118a899d923def15f75234f2a6d53b566a2528441e98050b38803673e9179b834fc39a499355fd270b7601d5d88408b7e9e53d260512e2180cd260017dc941f2fc96d65153f0344c6bf2d8a891b979bc61d39a98fb11155fcd57418f30ea018ea842874a0e76be91a3148e2f8ef644222b3ce5b939a73bd2e0a40814f7f92a79c483acf2216bbe0c289e07936b4d9653b91521a24c570c808fa46dfd12ec4429e71b61999fcfb245459d63a4923b8f8c488d1e6af7ca358867b88eb0cdefe896c221f09e95e4c18d1e0475de4549b2547611d8301e1afff1047a6f5a288c9314af0b9fc05d403c8c91820a385a72c18d6a4962cef41a3ab93daa7ed289b1e95db4d04eb00000003e71843e52743864f4bb67ce94a2ce8fe82c8f61042c4c1ced8531d94305392818b0dbe71f4d60e02e9160ec2b015cae3a09cbe4f437226e2c02e1a5e5d124bcac29e93d5f47c0c7671350398ed8c40f5bc5c2f5b00363c7e2eb18a91a1c490c70000000100000000a57df6f8132cb0037f7dfdf1a29b04c1ff92ba082eda513996ba2bfa9fbd198713f0d8d8879885ca567ef99298c30c397e6fba584658f4127713a814c06de55aefbfe141a7555cf7e3e86b092660b81cfb68a025ad817e45cec0b0f2e2ca636802a104df1c015f2307fa2859627098cdf9fdb521d61d323943343a12304e5baf");
    let verifying_key = parse_verifying_key(data.as_slice());

    let pedersen_commitment = pedersen_commitment_key();

    fs::write(dest_path, format!("{verifying_key}\n{pedersen_commitment}")).unwrap();
}
