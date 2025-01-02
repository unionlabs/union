#![deny(
    clippy::pedantic,
    clippy::std_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::alloc_instead_of_core
)]

use crate::encoding::HexPrefixed;

extern crate alloc;

pub mod encoding;

mod hash;

mod bytes;

mod compat {
    #[cfg(feature = "primitive-types-compat")]
    impl From<H256> for primitive_types::H256 {
        fn from(value: H256) -> Self {
            Self(*value.get())
        }
    }

    #[cfg(feature = "primitive-types-compat")]
    impl From<primitive_types::H256> for H256 {
        fn from(value: primitive_types::H256) -> Self {
            Self::new(value.0)
        }
    }

    #[cfg(feature = "primitive-types-compat")]
    impl From<H160> for primitive_types::H160 {
        fn from(value: H160) -> Self {
            Self(*value.get())
        }
    }

    #[cfg(feature = "primitive-types-compat")]
    impl From<primitive_types::H160> for H160 {
        fn from(value: primitive_types::H160) -> Self {
            Self::new(value.0)
        }
    }
}

pub use crate::{bytes::Bytes, hash::Hash};

pub type H64<E = HexPrefixed> = Hash<8, E>;
pub type H160<E = HexPrefixed> = Hash<20, E>;
pub type H256<E = HexPrefixed> = Hash<32, E>;
pub type H384<E = HexPrefixed> = Hash<48, E>;
pub type H512<E = HexPrefixed> = Hash<64, E>;
pub type H2048<E = HexPrefixed> = Hash<256, E>;
