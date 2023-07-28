use crate::{
    ethereum::Address, ibc::google::protobuf::timestamp::Timestamp,
    tendermint::types::block_id_flag::BlockIdFlag,
};

#[derive(Clone, PartialEq)]
pub struct CommitSig {
    pub block_id_flag: BlockIdFlag,
    pub validator_address: Address,
    pub timestamp: Timestamp,
    pub signature: Vec<u8>,
}
