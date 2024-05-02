//! Encode and decode a list many times.
//!
//! Useful for `cargo flamegraph`.

use ssz::{
    types::{typenum::U8192, List},
    Ssz,
};

#[derive(Debug, Clone, Copy, Ssz)]
pub struct FixedLen {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

fn main() {
    let fixed_len = FixedLen {
        a: 42,
        b: 42,
        c: 42,
        d: 42,
    };

    let vec: List<FixedLen, U8192> = vec![fixed_len; 8192].try_into().unwrap();

    let output = (0..40_000)
        .map(|_| List::from_ssz_bytes(&vec.as_ssz_bytes()).unwrap())
        .collect::<Vec<List<FixedLen, U8192>>>();

    println!("{}", output.len());
}
