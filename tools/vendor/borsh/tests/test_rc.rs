#![cfg(feature = "rc")]

use borsh::maybestd::rc::Rc;
use borsh::maybestd::sync::Arc;
use borsh::{BorshDeserialize, BorshSerialize};

#[test]
fn test_rc_roundtrip() {
    let value = Rc::new(8u8);
    let serialized = value.try_to_vec().unwrap();
    let deserialized = Rc::<u8>::try_from_slice(&serialized).unwrap();
    assert_eq!(value, deserialized);
}

#[test]
fn test_arc_roundtrip() {
    let value = Arc::new(8u8);
    let serialized = value.try_to_vec().unwrap();
    let deserialized = Arc::<u8>::try_from_slice(&serialized).unwrap();
    assert_eq!(value, deserialized);
}
