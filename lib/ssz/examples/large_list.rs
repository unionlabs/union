//! Encode and decode a list many times.
//!
//! Useful for `cargo flamegraph`.

use ssz::{
    types::{typenum::U8192, VariableList},
    Decode, Encode,
};

fn main() {
    // let vec: VariableList<u64, U8192> = vec![4242; 8192].try_into().unwrap();
    let vec: VariableList<u64, U8192> = vec![4242; 8192].try_into().unwrap();

    let output = (0..40_000)
        .map(|_| <VariableList<u64, U8192> as Decode>::from_ssz_bytes(&vec.as_ssz_bytes()).unwrap())
        .collect::<Vec<VariableList<u64, U8192>>>();

    println!("{}", output.len());
}
