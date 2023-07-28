use crate::tendermint::types::{commit::Commit, header::Header};

#[derive(Clone, PartialEq)]
pub struct SignedHeader {
    pub header: Header,
    pub commit: Commit,
}
