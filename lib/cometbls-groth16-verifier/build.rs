use std::{env, fs, path::Path};

use gnark_key_parser::VerifyingKey;
use substrate_bn::{AffineG1, AffineG2};

pub const FQ_SIZE: usize = 32;
pub const G1_SIZE: usize = 2 * FQ_SIZE;
pub const G2_SIZE: usize = 2 * G1_SIZE;

fn parse_verifying_key(buf: &[u8]) -> String {
    let (n_read, parsed_key) = VerifyingKey::parse(buf).unwrap();
    // we expect the verifying key to be fully parsed
    assert_eq!(n_read, buf.len());
    let [alpha_g1_x0, alpha_g1_x1, alpha_g1_y0, alpha_g1_y1] =
        unsafe { std::mem::transmute::<AffineG1, [u128; 4]>(parsed_key.alpha_g1) };
    let [beta_g2_x00, beta_g2_x01, beta_g2_x10, beta_g2_x11, beta_g2_y00, beta_g2_y01, beta_g2_y10, beta_g2_y11] =
        unsafe { std::mem::transmute::<AffineG2, [u128; 8]>(parsed_key.beta_g2) };
    let [gamma_g2_x00, gamma_g2_x01, gamma_g2_x10, gamma_g2_x11, gamma_g2_y00, gamma_g2_y01, gamma_g2_y10, gamma_g2_y11] =
        unsafe { std::mem::transmute::<AffineG2, [u128; 8]>(parsed_key.gamma_g2) };
    let [delta_g2_x00, delta_g2_x01, delta_g2_x10, delta_g2_x11, delta_g2_y00, delta_g2_y01, delta_g2_y10, delta_g2_y11] =
        unsafe { std::mem::transmute::<AffineG2, [u128; 8]>(parsed_key.delta_g2) };
    let [pedersen_g_x00, pedersen_g_x01, pedersen_g_x10, pedersen_g_x11, pedersen_g_y00, pedersen_g_y01, pedersen_g_y10, pedersen_g_y11] =
        unsafe { std::mem::transmute::<AffineG2, [u128; 8]>(parsed_key.commitment_key.g) };
    let [pedersen_g_root_sigma_neg_x00, pedersen_g_root_sigma_neg_x01, pedersen_g_root_sigma_neg_x10, pedersen_g_root_sigma_neg_x11, pedersen_g_root_sigma_neg_y00, pedersen_g_root_sigma_neg_y01, pedersen_g_root_sigma_neg_y10, pedersen_g_root_sigma_neg_y11] = unsafe {
        std::mem::transmute::<AffineG2, [u128; 8]>(parsed_key.commitment_key.g_root_sigma_neg)
    };

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
            pub const PEDERSEN_G: substrate_bn::AffineG2 = unsafe {{ core::mem::transmute::<[u128; 8], substrate_bn::AffineG2>([{pedersen_g_x00}, {pedersen_g_x01}, {pedersen_g_x10}, {pedersen_g_x11}, {pedersen_g_y00}, {pedersen_g_y01}, {pedersen_g_y10}, {pedersen_g_y11}]) }};
            pub const PEDERSEN_G_ROOT_SIGMA_NEG: substrate_bn::AffineG2 = unsafe {{ core::mem::transmute::<[u128; 8], substrate_bn::AffineG2>([{pedersen_g_root_sigma_neg_x00}, {pedersen_g_root_sigma_neg_x01}, {pedersen_g_root_sigma_neg_x10}, {pedersen_g_root_sigma_neg_x11}, {pedersen_g_root_sigma_neg_y00}, {pedersen_g_root_sigma_neg_y01}, {pedersen_g_root_sigma_neg_y10}, {pedersen_g_root_sigma_neg_y11}]) }};
            pub const GAMMA_ABC_G1: [substrate_bn::AffineG1; {gamma_abc_size}] = [{s}];
        "#
    )
}

fn main() {
    println!("cargo:rerun-if-changed=verifying_key.bin");
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("constants.rs");

    let data = include_bytes!("./verifying_key.bin");
    let verifying_key = parse_verifying_key(data.as_slice());
    fs::write(dest_path, format!("{verifying_key}")).unwrap();
}
