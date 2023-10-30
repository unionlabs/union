//! Development helper functions.

use num_bigint::{BigUint, ToBigUint};
use num_traits::cast::ToPrimitive;

/// Converts a byte array (big-endian) to BigUint.
pub fn bytes_to_biguint(bytes: &[u8; 32]) -> BigUint {
    bytes
        .iter()
        .enumerate()
        .map(|(i, w)| w.to_biguint().unwrap() << ((31 - i) * 8))
        .sum()
}

/// Converts a BigUint to a byte array (big-endian).
pub fn biguint_to_bytes(x: &BigUint) -> [u8; 32] {
    let mask = BigUint::from(u8::MAX);
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        bytes[i] = ((x >> ((31 - i) * 8)) as BigUint & &mask).to_u8().unwrap();
    }
    bytes
}
