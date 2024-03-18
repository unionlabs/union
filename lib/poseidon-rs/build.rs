#![allow(clippy::disallowed_types)]
// to avoid bringing in the entire unionlabs crate as a dependency of this build script
// TODO: benchmark to see if it makes a difference

use std::{env, fs, path::Path};

use primitive_types::U256;
use serde::Deserialize;

fn main() {
    println!("cargo:rerun-if-changed=constants.json");
    let constants = load_constants();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("constants.rs");
    let c = constants
        .c
        .into_iter()
        .map(|x| format!("&{x:?}"))
        .collect::<Vec<_>>()
        .join(",");
    let m = constants
        .m
        .into_iter()
        .map(|x| {
            format!(
                "&[{}]",
                x.into_iter()
                    .map(|x| format!("&{x:?}"))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    fs::write(
        dest_path,
        format!(
            "
            pub const C: &[&[[u64; 4]]] = &[{c}];
            pub const M: &[&[&[[u64; 4]]]] = &[{m}];
            ",
        ),
    )
    .unwrap();
}

pub fn load_constants() -> Constants {
    let RawConstants { c_str, m_str } =
        serde_json::from_str(&fs::read_to_string("./constants.json").unwrap()).unwrap();

    let mut c = Vec::new();
    for s in c_str {
        let mut cci = Vec::new();
        for c in s {
            let b = U256::from_str_radix(&c, 10).unwrap();
            cci.push(b.0);
        }
        c.push(cci);
    }
    let mut m = Vec::new();
    for i in m_str {
        let mut mi = Vec::new();
        for j in i {
            let mut mij = Vec::new();
            for k in j {
                let b: U256 = U256::from_str_radix(&k, 10).unwrap();
                mij.push(b.0);
            }
            mi.push(mij);
        }
        m.push(mi);
    }

    Constants {
        c,
        m,
        n_rounds_f: 8,
        n_rounds_p: vec![
            56, 57, 56, 60, 60, 63, 64, 63, 60, 66, 60, 65, 70, 60, 64, 68,
        ],
    }
}

#[derive(Debug)]
pub struct Constants {
    pub c: Vec<Vec<[u64; 4]>>,
    pub m: Vec<Vec<Vec<[u64; 4]>>>,
    pub n_rounds_f: usize,
    pub n_rounds_p: Vec<usize>,
}

#[derive(Deserialize)]
pub struct RawConstants {
    pub c_str: Vec<Vec<String>>,
    pub m_str: Vec<Vec<Vec<String>>>,
}
