use crate::{
    ethereum::H256, ibc::core::client::height::Height,
    tendermint::types::signed_header::SignedHeader,
};

#[derive(Clone, PartialEq)]
pub struct Header {
    pub signed_header: SignedHeader,
    pub untrusted_validator_set_root: H256,
    pub trusted_height: Height,
    pub zero_knowledge_proof: Vec<u8>,
}
