//! An example of implementing `serde::Serialize` and `serde::Deserialize`.
//! The `#[serde(transparent)]` attribute is recommended to serialize directly
//! to the underlying bits type without wrapping it in a `serde` newtype.

#[cfg(feature = "serde")]
fn main() {
    use serde_derive::*;

    bitflags::bitflags! {
        #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
        #[serde(transparent)]
        pub struct Flags: u32 {
            const A = 1;
            const B = 2;
            const C = 4;
            const D = 8;
        }
    }

    let flags = Flags::A | Flags::B;

    let serialized = serde_json::to_string(&flags).unwrap();

    println!("{:?} -> {}", flags, serialized);

    assert_eq!(serialized, r#""A | B""#);

    let deserialized: Flags = serde_json::from_str(&serialized).unwrap();

    println!("{} -> {:?}", serialized, flags);

    assert_eq!(deserialized, flags);
}

#[cfg(not(feature = "serde"))]
fn main() {}
