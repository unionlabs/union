#![recursion_limit = "512"]

mod ibc_client;
mod msg;
pub mod storage_utils;

pub use ibc_client::*;
pub use msg::*;

#[derive(Debug)]
pub enum Error {
    Decode(String),
    UnexpectedCallDataFromHostModule(String),
    ClientStateNotFound,
}
