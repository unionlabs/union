use std::{
    env,
    fmt::{Display, Write},
    fs,
    mem::{align_of, size_of},
    path::Path,
};

use gnark_key_parser::VerifyingKey;
use substrate_bn::{G1, G2};

pub const FQ_SIZE: usize = 32;
pub const G1_SIZE: usize = 2 * FQ_SIZE;
pub const G2_SIZE: usize = 2 * G1_SIZE;

fn parse_verifying_key(buf: &[u8]) -> String {
    let (n_read, parsed_key) = VerifyingKey::parse(buf).unwrap();
    // we expect the verifying key to be fully parsed
    assert_eq!(n_read, buf.len());
    let alpha_g1 = G1Const(parsed_key.alpha_g1);
    let beta_neg_g2 = G2Const(parsed_key.beta_neg_g2);
    let gamma_neg_g2 = G2Const(parsed_key.gamma_neg_g2);
    let delta_neg_g2 = G2Const(parsed_key.delta_neg_g2);
    let pedersen_g = G2Const(parsed_key.commitment_key.g);
    let pedersen_g_root_sigma_neg = G2Const(parsed_key.commitment_key.g_root_sigma_neg);

    let gamma_abc_size = parsed_key.gamma_abc_g1.len();
    let s: String = parsed_key
        .gamma_abc_g1
        .into_iter()
        .fold(String::new(), |mut s, g1| {
            write!(&mut s, "{}, ", G1Const(g1)).unwrap();

            s
        });

    format!(
        r#"
            pub const ALPHA_G1: ::substrate_bn::G1 = {alpha_g1};
            pub const BETA_NEG_G2: ::substrate_bn::G2 = {beta_neg_g2};
            pub const GAMMA_NEG_G2: ::substrate_bn::G2 = {gamma_neg_g2};
            pub const DELTA_NEG_G2: ::substrate_bn::G2 = {delta_neg_g2};
            pub const PEDERSEN_G: ::substrate_bn::G2 = {pedersen_g};
            pub const PEDERSEN_G_ROOT_SIGMA_NEG: ::substrate_bn::G2 = {pedersen_g_root_sigma_neg};

            pub const GAMMA_ABC_G1: [substrate_bn::G1; {gamma_abc_size}] = [{s}];
        "#
    )
}

pub struct G1Const(G1);

impl Display for G1Const {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        pub const _: () = {
            assert!(align_of::<G1>() == align_of::<[u128; 6]>());
            assert!(size_of::<G1>() == size_of::<[u128; 6]>());
        };

        // SAFETY: See const assertions just above
        let [g1_x0, g1_x1, g1_y0, g1_y1, g1_z0, g1_z1] =
            unsafe { std::mem::transmute::<G1, [u128; 6]>(self.0) };

        write!(
            f,
            "
            unsafe {{
                ::core::mem::transmute::<[u128; 6], ::substrate_bn::G1>(
                    [{g1_x0}, {g1_x1}, {g1_y0}, {g1_y1}, {g1_z0}, {g1_z1}]
                )
            }}
            "
        )
    }
}

pub struct G2Const(G2);

impl Display for G2Const {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        pub const _: () = {
            assert!(align_of::<G2>() == align_of::<[u128; 12]>());
            assert!(size_of::<G2>() == size_of::<[u128; 12]>());
        };

        // SAFETY: See const assertions just above
        let [g2_x00, g2_x01, g2_x10, g2_x11, g2_y00, g2_y01, g2_y10, g2_y11, g2_z00, g2_z01, g2_z10, g2_z11] =
            unsafe { std::mem::transmute::<G2, [u128; 12]>(self.0) };

        write!(
            f,
            "
            unsafe {{
                ::core::mem::transmute::<[u128; 12], ::substrate_bn::G2>(
                    [{g2_x00}, {g2_x01}, {g2_x10}, {g2_x11}, {g2_y00}, {g2_y01}, {g2_y10}, {g2_y11}, {g2_z00}, {g2_z01}, {g2_z10}, {g2_z11}]
                )
            }}
            "
        )
    }
}

fn main() {
    println!("cargo:rerun-if-changed=verifying_key.bin");
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("constants.rs");

    let data = include_bytes!("./verifying_key.bin");
    let verifying_key = parse_verifying_key(data.as_slice());
    fs::write(dest_path, verifying_key).unwrap();
}
