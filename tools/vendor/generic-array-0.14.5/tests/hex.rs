#[macro_use]
extern crate generic_array;
extern crate typenum;

use generic_array::GenericArray;
use std::str::from_utf8;
use typenum::U2048;

#[test]
fn short_lower_hex() {
    let ar = arr![u8; 10, 20, 30];
    assert_eq!(format!("{:x}", ar), "0a141e");
}

#[test]
fn short_upper_hex() {
    let ar = arr![u8; 30, 20, 10];
    assert_eq!(format!("{:X}", ar), "1E140A");
}

#[test]
fn long_lower_hex() {
    let ar = GenericArray::<u8, U2048>::default();
    assert_eq!(format!("{:x}", ar), from_utf8(&[b'0'; 4096]).unwrap());
}

#[test]
fn long_lower_hex_truncated() {
    let ar = GenericArray::<u8, U2048>::default();
    assert_eq!(format!("{:.3001x}", ar), from_utf8(&[b'0'; 3001]).unwrap());
}

#[test]
fn long_upper_hex() {
    let ar = GenericArray::<u8, U2048>::default();
    assert_eq!(format!("{:X}", ar), from_utf8(&[b'0'; 4096]).unwrap());
}

#[test]
fn long_upper_hex_truncated() {
    let ar = GenericArray::<u8, U2048>::default();
    assert_eq!(format!("{:.2777X}", ar), from_utf8(&[b'0'; 2777]).unwrap());
}

#[test]
fn truncated_lower_hex() {
    let ar = arr![u8; 10, 20, 30, 40, 50];
    assert_eq!(format!("{:.2x}", ar), "0a");
    assert_eq!(format!("{:.3x}", ar), "0a1");
    assert_eq!(format!("{:.4x}", ar), "0a14");
}

#[test]
fn truncated_upper_hex() {
    let ar = arr![u8; 30, 20, 10, 17, 0];
    assert_eq!(format!("{:.4X}", ar), "1E14");
    assert_eq!(format!("{:.5X}", ar), "1E140");
    assert_eq!(format!("{:.6X}", ar), "1E140A");
    assert_eq!(format!("{:.7X}", ar), "1E140A1");
    assert_eq!(format!("{:.8X}", ar), "1E140A11");
}
