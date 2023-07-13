use bytes::Bytes;
use primitive_types::U256;
use rlp::{Decodable, RlpDecodable};

use crate::error::Error;

pub type Root = [u8; 32];
pub type Slot = u64;
pub type Epoch = u64;
pub type Hash32 = [u8; 32];
pub type ExecutionAddress = [u8; 20];

pub const GENESIS_SLOT: Slot = 0;
pub const GENESIS_EPOCH: Epoch = 0;
pub const FAR_FUTURE_EPOCH: Epoch = Epoch::MAX;

pub const BLS_WITHDRAWAL_PREFIX: u8 = 0x00;
pub const ETH1_ADDRESS_WITHDRAWAL_PREFIX: u8 = 0x01;

#[derive(Debug, Clone, Default)]
pub struct Account {
    pub nonce: u64,
    pub balance: U256,
    pub storage_root: Hash32,
    pub code_hash: Hash32,
}

#[derive(Debug, Default, RlpDecodable)]
struct RawAccount {
    nonce: u64,
    balance: U256,
    storage_root: Bytes,
    code_hash: Bytes,
}

impl Account {
    pub fn from_rlp_bytes(bz: &[u8]) -> Result<Account, Error> {
        let r = rlp::Rlp::new(bz);
        let raw_account = RawAccount::decode(&r).map_err(|_| Error::RlpDecode)?;

        Ok(Account {
            nonce: raw_account.nonce,
            balance: raw_account.balance,
            storage_root: raw_account
                .storage_root
                .to_vec()
                .try_into()
                .map_err(|_| Error::RlpDecode)?,
            code_hash: raw_account
                .code_hash
                .to_vec()
                .try_into()
                .map_err(|_| Error::RlpDecode)?,
        })
    }
}
