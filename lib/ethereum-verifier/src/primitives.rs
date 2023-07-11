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

// TODO: Derive RlpDecodable instead of implementing it manually
#[derive(Debug, Clone, Default)]
pub struct Account {
    pub nonce: u64,
    // REVIEW: Is this arbitrary bytes? Or a U256?
    pub balance: Vec<u8>,
    pub storage_root: Hash32,
    pub code_hash: Hash32,
}

impl Account {
    pub fn from_rlp_bytes(bz: &[u8]) -> Result<Account, Error> {
        let r = rlp::Rlp::new(bz);
        Ok(Account {
            nonce: r.val_at::<u64>(0).map_err(|_| Error::RlpDecode)?,
            balance: r.val_at::<Vec<u8>>(1).map_err(|_| Error::RlpDecode)?,
            storage_root: Hash32::try_from(
                r.val_at::<Vec<u8>>(2)
                    .map_err(|_| Error::RlpDecode)?
                    .as_slice(),
            )
            .map_err(|_| Error::InvalidHash)?,
            code_hash: Hash32::try_from(
                r.val_at::<Vec<u8>>(3)
                    .map_err(|_| Error::RlpDecode)?
                    .as_slice(),
            )
            .map_err(|_| Error::InvalidHash)?,
        })
    }
}
