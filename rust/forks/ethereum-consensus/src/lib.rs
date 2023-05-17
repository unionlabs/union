#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
extern crate alloc;

mod internal_prelude {
    pub use alloc::string::{String, ToString};
    pub use alloc::vec;
    pub use alloc::vec::Vec;
}

pub mod beacon;
pub mod bellatrix;
pub mod bls;
pub mod capella;
pub mod compute;
pub mod config;
pub mod context;
pub mod errors;
pub mod execution;
pub mod fork;
pub mod merkle;
pub mod preset;
pub mod sync_protocol;
pub mod types;
