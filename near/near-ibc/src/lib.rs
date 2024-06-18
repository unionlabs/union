#![allow(clippy::too_many_arguments)] // NOTE(aeryz): added because of `near_bindgen` producing too many args

mod contract;
mod error;
pub use contract::*;
