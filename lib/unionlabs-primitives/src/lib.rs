#![deny(
    clippy::pedantic,
    clippy::std_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::alloc_instead_of_core
)]

use crate::encoding::HexPrefixed;

extern crate alloc;

pub mod encoding;

mod fixed_bytes;

mod bytes;

mod compat;

pub use crate::{
    bytes::Bytes,
    fixed_bytes::{FixedBytes, FixedBytesError},
};

pub type H64<E = HexPrefixed> = FixedBytes<8, E>;
pub type H160<E = HexPrefixed> = FixedBytes<20, E>;
pub type H256<E = HexPrefixed> = FixedBytes<32, E>;
pub type H384<E = HexPrefixed> = FixedBytes<48, E>;
pub type H512<E = HexPrefixed> = FixedBytes<64, E>;
pub type H2048<E = HexPrefixed> = FixedBytes<256, E>;
