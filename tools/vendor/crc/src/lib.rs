//! # crc
//! Rust implementation of CRC(8, 16, 32, 64)
//!
//! ## Usage
//! ### Compute CRC16
//! ```rust
//! use crc::{Crc, Algorithm, CRC_16_IBM_SDLC, CRC_32_ISCSI};
//!
//! pub const X25: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_SDLC);
//! pub const CASTAGNOLI: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);
//!
//! assert_eq!(X25.checksum(b"123456789"), 0x906e);
//! assert_eq!(CASTAGNOLI.checksum(b"123456789"), 0xe3069283);
//!
//! // use custom algorithm
//! const CUSTOM_ALG: Algorithm<u16> = Algorithm {
//!     width: 16,
//!     poly: 0x8005,
//!     init: 0xffff,
//!     refin: false,
//!     refout: false,
//!     xorout: 0x0000,
//!     check: 0xaee7,
//!     residue: 0x0000
//! };
//! let crc = Crc::<u16>::new(&CUSTOM_ALG);
//! let mut digest = crc.digest();
//! digest.update(b"123456789");
//! assert_eq!(digest.finalize(), 0xaee7);
//! ```
#![no_std]
#![forbid(unsafe_code)]

pub use crc_catalog::*;

mod crc128;
mod crc16;
mod crc32;
mod crc64;
mod crc8;
mod table;
mod util;

pub struct Crc<W: Width> {
    pub algorithm: &'static Algorithm<W>,
    table: [W; 256],
}

#[derive(Clone)]
pub struct Digest<'a, W: Width> {
    crc: &'a Crc<W>,
    value: W,
}
