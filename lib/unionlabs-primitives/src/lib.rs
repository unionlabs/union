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

mod compat;

pub use crate::{
    bytes::Bytes,
    hash::{FixedBytesError, Hash},
};

pub type H64<E = HexPrefixed> = Hash<8, E>;
pub type H160<E = HexPrefixed> = Hash<20, E>;
pub type H256<E = HexPrefixed> = Hash<32, E>;
pub type H384<E = HexPrefixed> = Hash<48, E>;
pub type H512<E = HexPrefixed> = Hash<64, E>;
pub type H2048<E = HexPrefixed> = Hash<256, E>;
