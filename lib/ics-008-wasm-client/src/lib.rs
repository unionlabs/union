mod ibc_client;
mod msg;
pub mod storage_utils;

pub use ibc_client::*;
pub use msg::*;

#[derive(Debug)]
pub enum Error {
    Decode(String),
    NotSpecCompliant(String),
    ClientStateNotFound,
}

impl Error {
    pub fn decode<S: Into<String>>(msg: S) -> Error {
        Error::Decode(msg.into())
    }
}
