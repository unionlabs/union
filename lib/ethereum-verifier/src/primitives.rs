use rlp::RlpDecodable;
use unionlabs::{hash::H256, uint::U256};

pub const GENESIS_SLOT: u64 = 0;
pub const GENESIS_EPOCH: u64 = 0;
pub const FAR_FUTURE_EPOCH: u64 = u64::MAX;

pub const BLS_WITHDRAWAL_PREFIX: u8 = 0x00;
pub const ETH1_ADDRESS_WITHDRAWAL_PREFIX: u8 = 0x01;

#[derive(Debug, Clone, RlpDecodable)]
pub struct Account {
    pub nonce: u64,
    pub balance: U256,
    pub storage_root: H256,
    pub code_hash: H256,
}
