//! Secret key tests

#![cfg(feature = "dev")]

use elliptic_curve::dev::SecretKey;

#[test]
fn from_slice_undersize() {
    assert!(SecretKey::from_slice(&[]).is_err());
}

#[test]
fn from_slice_expected_size() {
    let bytes = [1u8; 32];
    assert!(SecretKey::from_slice(&bytes).is_ok());
}

#[test]
fn from_slice_allowed_short() {
    let bytes = [1u8; 28];
    assert!(SecretKey::from_slice(&bytes).is_ok());
}

#[test]
fn from_slice_too_short() {
    let bytes = [1u8; 27];
    assert!(SecretKey::from_slice(&bytes).is_err());
}
