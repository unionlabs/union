#![cfg(feature="serde1")]

use rand_core::{RngCore, SeedableRng};
use rand_xoshiro::{SplitMix64, Xoroshiro64StarStar, Xoroshiro64Star,
    Xoroshiro128Plus, Xoroshiro128StarStar, Xoshiro128StarStar, Xoshiro128Plus,
    Xoshiro256StarStar, Xoshiro256Plus, Xoshiro512StarStar, Xoshiro512Plus};

macro_rules! serde_rng {
    ($rng:ident) => {
        use bincode;
        use std::io::{BufWriter, BufReader};

        let mut rng = $rng::seed_from_u64(0);

        let buf: Vec<u8> = Vec::new();
        let mut buf = BufWriter::new(buf);
        bincode::serialize_into(&mut buf, &rng).expect("Could not serialize");

        let buf = buf.into_inner().unwrap();
        let mut read = BufReader::new(&buf[..]);
        let mut deserialized: $rng = bincode::deserialize_from(&mut read)
            .expect("Could not deserialize");

        for _ in 0..16 {
            assert_eq!(rng.next_u64(), deserialized.next_u64());
        }
    }
}

#[test]
fn test_splitmix64() {
    serde_rng!(SplitMix64);
}

#[test]
fn test_xoroshiro64starstar() {
    serde_rng!(Xoroshiro64StarStar);
}

#[test]
fn test_xoroshiro64star() {
    serde_rng!(Xoroshiro64Star);
}

#[test]
fn test_xoroshiro128plus() {
    serde_rng!(Xoroshiro128Plus);
}

#[test]
fn test_xoroshiro128starstar() {
    serde_rng!(Xoroshiro128StarStar);
}

#[test]
fn test_xoshiro128starstar() {
    serde_rng!(Xoshiro128StarStar);
}

#[test]
fn test_xoshiro128plus() {
    serde_rng!(Xoshiro128Plus);
}

#[test]
fn test_xoshiro256starstar() {
    serde_rng!(Xoshiro256StarStar);
}

#[test]
fn test_xoshiro256plus() {
    serde_rng!(Xoshiro256Plus);
}

#[test]
fn test_xoshiro512starstar() {
    serde_rng!(Xoshiro512StarStar);
}

#[test]
fn test_xoshiro512plus() {
    serde_rng!(Xoshiro512Plus);
}
