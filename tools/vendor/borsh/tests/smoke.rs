// Smoke tests that ensure that we don't accidentally remove top-level
// re-exports in a minor release.

use borsh::{self, BorshDeserialize};

#[test]
fn test_to_vec() {
    let value = 42u8;
    let serialized = borsh::to_vec(&value).unwrap();
    let deserialized = u8::try_from_slice(&serialized).unwrap();
    assert_eq!(value, deserialized);
}

#[test]
fn test_to_writer() {
    let value = 42u8;
    let mut serialized = vec![0; 1];
    borsh::to_writer(&mut serialized[..], &value).unwrap();
    let deserialized = u8::try_from_slice(&serialized).unwrap();
    assert_eq!(value, deserialized);
}
