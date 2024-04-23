use std::{env, fs, marker::PhantomData, path::Path};

use byteorder::BigEndian;
use gnark_key_parser::VerifyingKey;
use hex_literal::hex;
use substrate_bn::{AffineG1, AffineG2};
use unionlabs::ByteArrayExt;

pub const FQ_SIZE: usize = 32;
pub const G1_SIZE: usize = 2 * FQ_SIZE;
pub const G2_SIZE: usize = 2 * G1_SIZE;

pub struct G2Affine<FromOrder>(PhantomData<FromOrder>, substrate_bn::AffineG2);

impl<FromOrder> G2Affine<FromOrder> {
    fn parse(buf: [u8; G2_SIZE]) -> Self {
        G2Affine(
            PhantomData,
            substrate_bn::AffineG2::new(
                substrate_bn::Fq2::new(
                    substrate_bn::Fq::from_slice(&buf.array_slice::<FQ_SIZE, FQ_SIZE>()).unwrap(),
                    substrate_bn::Fq::from_slice(&buf.array_slice::<0, FQ_SIZE>()).unwrap(),
                ),
                substrate_bn::Fq2::new(
                    substrate_bn::Fq::from_slice(
                        &buf.array_slice::<{ G1_SIZE + FQ_SIZE }, FQ_SIZE>(),
                    )
                    .unwrap(),
                    substrate_bn::Fq::from_slice(&buf.array_slice::<G1_SIZE, FQ_SIZE>()).unwrap(),
                ),
            )
            .unwrap(),
        )
    }
}

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

fn pedersen_commitment_key() -> String {
    let g_raw = hex!("257DF6F8132CB0037F7DFDF1A29B04C1FF92BA082EDA513996BA2BFA9FBD198713F0D8D8879885CA567EF99298C30C397E6FBA584658F4127713A814C06DE55A1660EBCC60C7A3AC560EFCEA5993F528EE13685D3A39694ACD74FE67C80D798A15E80642C58DB4DBE0A87F92CE3C65E962F231278353783A691FD64078BA7F34");
    let g_root_sigma_neg_raw = hex!("2FBFE141A7555CF7E3E86B092660B81CFB68A025AD817E45CEC0B0F2E2CA636802A104DF1C015F2307FA2859627098CDF9FDB521D61D323943343A12304E5BAF27DA3F93ECF3BFD0B3A3354AE2162A6C230C0E539B6D9F82C0826E2B006A59222C0838551CB9E5CF67DB57DE7E2250BB97807F6687F135A6EB910359BA7BDB8D");
    let G2Affine(_, g) = G2Affine::<BigEndian>::parse(g_raw);
    let G2Affine(_, g_root_sigma_neg) = G2Affine::<BigEndian>::parse(g_root_sigma_neg_raw);
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
    println!("cargo:rerun-if-changed=vk.bin");
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("constants.rs");

    let data = include_bytes!("./vk.bin");
    let verifying_key = parse_verifying_key(data.as_slice());

    let pedersen_commitment = pedersen_commitment_key();

    fs::write(dest_path, format!("{verifying_key}\n{pedersen_commitment}")).unwrap();
}
